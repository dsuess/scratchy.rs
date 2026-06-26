hw_target := "aarch64-unknown-linux-gnu"
hw_host := "reachy-mini.local"

default: help

# List available recipes
help:
    @just --list

# runs unit tests
test:
    cargo test

# runs simulation in mujoco
sim:
   RUST_BACKTRACE=1 cargo run --bin sim

# Cross-compile the hw binary for the Reachy Mini Wireless (CM4 / aarch64),
# copy it to the robot, and run it in an interactive ssh session.
deploy:
    cross build --release --target {{hw_target}} --bin hw --no-default-features --features hardware
    scp target/{{hw_target}}/release/hw {{hw_host}}:hw
    ssh -t {{hw_host}} ./hw
