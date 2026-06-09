"""Run the Reachy Mini daemon with extra instrumentation around shutdown.

Wraps `reachy_mini.daemon.app.main` with:
  * Signal handlers (SIGINT/SIGTERM/SIGHUP/SIGQUIT) that log the signal and
    a Python stack trace before chaining to the default handler.
  * A monkey-patch on `uvicorn.Server.should_exit` so any code that sets it
    to True is logged with a stack trace.
  * A monkey-patch on `Daemon.stop` to log the caller.
  * Root log level forced to DEBUG.

Usage (same env vars the justfile uses):
    GI_TYPELIB_PATH=/opt/homebrew/lib/girepository-1.0 \
        DYLD_LIBRARY_PATH=/opt/homebrew/lib \
        uv run mjpython scripts/debug_daemon.py --sim
"""

from __future__ import annotations

import ctypes
import ctypes.util
import logging
import os
import signal
import sys
import threading
import traceback
from typing import Any

import psutil
import uvicorn

from reachy_mini.daemon.app.main import main as daemon_main
from reachy_mini.daemon.daemon import Daemon

log = logging.getLogger("debug_daemon")

# Globals populated by the SA_SIGINFO handler installed below.
_LAST_SIGNAL_SENDER_PID: int | None = None
_LAST_SIGNAL_SENDER_UID: int | None = None
_LAST_SIGNAL_NUM: int | None = None


def _format_stack() -> str:
    return "".join(traceback.format_stack())


# --- siginfo-aware signal capture via ctypes/sigaction --------------------
#
# Python's signal module does not expose `siginfo_t.si_pid` (the PID of the
# process that sent the signal). We install our own SA_SIGINFO handler in C
# via ctypes that stashes si_pid/si_uid into module-level globals, then chains
# to Python's signal machinery by re-raising with pthread_kill (so uvicorn's
# graceful-shutdown handling still runs).

# Layout of siginfo_t on macOS (Darwin). Only the leading fields are stable
# enough to read portably; we just need si_signo, si_errno, si_code, si_pid,
# si_uid. The struct is larger than this on Darwin but reading the first few
# fields by offset is safe — sigaction will write the whole struct, we just
# only access the prefix we declared.
class _Siginfo(ctypes.Structure):
    _fields_ = [
        ("si_signo", ctypes.c_int),
        ("si_errno", ctypes.c_int),
        ("si_code", ctypes.c_int),
        ("si_pid", ctypes.c_int),
        ("si_uid", ctypes.c_uint),
        # Padding to cover the rest of siginfo_t on Darwin (~104 bytes total).
        ("_pad", ctypes.c_byte * 128),
    ]


_SIGACTION_HANDLER = ctypes.CFUNCTYPE(
    None, ctypes.c_int, ctypes.POINTER(_Siginfo), ctypes.c_void_p
)


class _Sigaction(ctypes.Structure):
    # struct sigaction on Darwin: sa_handler/sa_sigaction is a union; we use
    # sa_sigaction. SA_SIGINFO must be set in sa_flags.
    _fields_ = [
        ("sa_sigaction", _SIGACTION_HANDLER),
        ("sa_mask", ctypes.c_uint32),
        ("sa_flags", ctypes.c_int),
    ]


_SA_SIGINFO = 0x0040  # Darwin

_libc = ctypes.CDLL(ctypes.util.find_library("c"), use_errno=True)
_libc.sigaction.argtypes = [
    ctypes.c_int,
    ctypes.POINTER(_Sigaction),
    ctypes.POINTER(_Sigaction),
]
_libc.sigaction.restype = ctypes.c_int


# Keep CFUNCTYPE instances alive (otherwise they get GC'd and the kernel
# jumps into freed memory next time the signal fires).
_HANDLER_REFS: list[Any] = []


def _make_handler(signal_name: str) -> Any:
    def handler(signum: int, info_ptr: Any, _ctx: Any) -> None:
        global _LAST_SIGNAL_SENDER_PID, _LAST_SIGNAL_SENDER_UID, _LAST_SIGNAL_NUM
        try:
            info = info_ptr.contents
            _LAST_SIGNAL_SENDER_PID = int(info.si_pid)
            _LAST_SIGNAL_SENDER_UID = int(info.si_uid)
            _LAST_SIGNAL_NUM = signum
            # async-signal-unsafe but we're already past the point of caring;
            # this script exists to debug, not to be production-safe.
            os.write(
                2,
                f"\n[debug_daemon] siginfo: signum={signum} ({signal_name}) "
                f"si_pid={info.si_pid} si_uid={info.si_uid} si_code={info.si_code}\n".encode(),
            )
        except Exception:
            pass
        # Re-raise via Python so uvicorn's own handler (and our chained Python
        # handler from _install_signal_loggers) still runs and shuts down
        # gracefully.
        signal.raise_signal(signum)

    cfunc = _SIGACTION_HANDLER(handler)
    _HANDLER_REFS.append(cfunc)
    return cfunc


def _install_sigaction_capture() -> None:
    for name in ["SIGINT", "SIGTERM", "SIGHUP", "SIGQUIT"]:
        sig = getattr(signal, name, None)
        if sig is None:
            continue
        act = _Sigaction()
        act.sa_sigaction = _make_handler(name)
        act.sa_mask = 0
        act.sa_flags = _SA_SIGINFO
        rc = _libc.sigaction(int(sig), ctypes.byref(act), None)
        if rc != 0:
            log.error("sigaction(%s) failed: errno=%d", name, ctypes.get_errno())
        else:
            log.info("Installed siginfo-capturing sigaction for %s", name)


def _install_signal_loggers() -> None:
    names = ["SIGINT", "SIGTERM", "SIGHUP", "SIGQUIT"]
    for name in names:
        sig = getattr(signal, name, None)
        if sig is None:
            continue
        previous = signal.getsignal(sig)

        def handler(signum: int, frame: Any, _prev: Any = previous, _name: str = name) -> None:
            log.error(
                "Received %s (signum=%d). Stack at time of signal:\n%s",
                _name,
                signum,
                "".join(traceback.format_stack(frame)) if frame else "<no frame>",
            )
            sys.stderr.flush()
            if callable(_prev):
                _prev(signum, frame)
            elif _prev == signal.SIG_DFL:
                signal.signal(signum, signal.SIG_DFL)
                signal.raise_signal(signum)

        signal.signal(sig, handler)
        log.info("Installed signal logger for %s (was %r)", name, previous)


def _patch_uvicorn_should_exit() -> None:
    original_init = uvicorn.Server.__init__

    def patched_init(self: uvicorn.Server, *args: Any, **kwargs: Any) -> None:
        original_init(self, *args, **kwargs)
        self.__dict__["_should_exit"] = False

    def get_should_exit(self: uvicorn.Server) -> bool:
        return bool(self.__dict__.get("_should_exit", False))

    def set_should_exit(self: uvicorn.Server, value: bool) -> None:
        prev = self.__dict__.get("_should_exit", False)
        self.__dict__["_should_exit"] = value
        if value and not prev:
            log.error(
                "uvicorn.Server.should_exit -> True. Stack:\n%s", _format_stack()
            )
            sys.stderr.flush()

    uvicorn.Server.__init__ = patched_init  # type: ignore[assignment]
    uvicorn.Server.should_exit = property(get_should_exit, set_should_exit)  # type: ignore[assignment]
    log.info("Patched uvicorn.Server.should_exit")


def _patch_daemon_stop() -> None:
    original_stop = Daemon.stop

    async def logged_stop(self: Daemon, *args: Any, **kwargs: Any) -> Any:
        log.error(
            "Daemon.stop called with args=%r kwargs=%r. Stack:\n%s",
            args,
            kwargs,
            _format_stack(),
        )
        sys.stderr.flush()
        return await original_stop(self, *args, **kwargs)

    Daemon.stop = logged_stop  # type: ignore[assignment]
    log.info("Patched Daemon.stop")


def _configure_logging() -> None:
    root = logging.getLogger()
    if not root.handlers:
        handler = logging.StreamHandler(sys.stderr)
        handler.setFormatter(
            logging.Formatter("%(asctime)s - %(name)s - %(levelname)s - %(message)s")
        )
        root.addHandler(handler)
    root.setLevel(logging.DEBUG)
    logging.getLogger("uvicorn").setLevel(logging.DEBUG)


def main() -> None:
    _configure_logging()
    log.info("debug_daemon starting; pid=%d", __import__("os").getpid())
    _install_signal_loggers()
    _patch_uvicorn_should_exit()
    _patch_daemon_stop()
    # Force --log-level DEBUG into argv if the user didn't pass one explicitly.
    if "--log-level" not in sys.argv:
        sys.argv.extend(["--log-level", "DEBUG"])
    daemon_main()


if __name__ == "__main__":
    main()
