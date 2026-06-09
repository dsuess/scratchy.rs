venv := ".venv"

default: setup

setup:
    brew install gstreamer
    uv sync --all-groups --python /opt/homebrew/bin/python3.11

sim *args="":
    GI_TYPELIB_PATH=/opt/homebrew/lib/girepository-1.0 DYLD_LIBRARY_PATH=/opt/homebrew/lib uv run mjpython -m reachy_mini.daemon.app.main --sim {{args}}

sim-debug *args="":
    GI_TYPELIB_PATH=/opt/homebrew/lib/girepository-1.0 DYLD_LIBRARY_PATH=/opt/homebrew/lib uv run mjpython scripts/debug_daemon.py --sim {{args}}

clean:
    rm -rf {{venv}}
