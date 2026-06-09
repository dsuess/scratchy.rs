venv := ".venv"
mujoco_dir := justfile_directory() / "vendor/mujoco"

default: setup

setup:
    brew install gstreamer
    uv sync --all-groups --python /opt/homebrew/bin/python3.11

# Download and install MuJoCo 3.8.0 framework into vendor/mujoco/.
# mujoco-rs 4.0.x pins to 3.8.0; system Homebrew cask may be a different version.
mujoco-install:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p {{mujoco_dir}}
    cd {{mujoco_dir}}
    if [ -f libmujoco.dylib ]; then echo "MuJoCo already installed."; exit 0; fi
    curl -L -o mujoco-3.8.0.dmg https://github.com/google-deepmind/mujoco/releases/download/3.8.0/mujoco-3.8.0-macos-universal2.dmg
    hdiutil attach -nobrowse -mountpoint /tmp/mujoco380mnt mujoco-3.8.0.dmg
    cp -R /tmp/mujoco380mnt/mujoco.framework .
    hdiutil detach /tmp/mujoco380mnt
    ln -sf mujoco.framework/Versions/Current/libmujoco.3.8.0.dylib libmujoco.dylib
    ln -sf mujoco.framework/Versions/Current/libmujoco.3.8.0.dylib libmujoco.3.8.0.dylib
    rm mujoco-3.8.0.dmg

sim *args="":
    GI_TYPELIB_PATH=/opt/homebrew/lib/girepository-1.0 DYLD_LIBRARY_PATH=/opt/homebrew/lib uv run mjpython -m reachy_mini.daemon.app.main --sim {{args}}

sim-debug *args="":
    GI_TYPELIB_PATH=/opt/homebrew/lib/girepository-1.0 DYLD_LIBRARY_PATH=/opt/homebrew/lib uv run mjpython scripts/debug_daemon.py --sim {{args}}

clean:
    rm -rf {{venv}}
