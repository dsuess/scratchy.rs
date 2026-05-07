venv := ".venv"

default: setup

setup:
    brew install gstreamer
    uv sync --all-groups

sim *args="":
    GI_TYPELIB_PATH=/opt/homebrew/lib/girepository-1.0 DYLD_LIBRARY_PATH=/opt/homebrew/lib uv run mjpython -m reachy_mini.daemon.app.main --sim {{args}}

clean:
    rm -rf {{venv}}
