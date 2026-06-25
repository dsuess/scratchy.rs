<div align="center">

<img src="assets/banner.svg" alt="Reachy Mini — from-scratch control stack" width="720">

<br>

![Rust](https://img.shields.io/badge/Rust-2024-ce422b?logo=rust&logoColor=white)
![MuJoCo](https://img.shields.io/badge/sim-MuJoCo-7ab7ff)
![Dynamixel](https://img.shields.io/badge/hardware-Dynamixel%20XL--330-9b6dff)
![Status](https://img.shields.io/badge/status-WIP%20%F0%9F%9A%A7-f5a623)
![Platform](https://img.shields.io/badge/robot-Reachy%20Mini%20Lite%20%2B%20Wireless-1b2233)
![License](https://img.shields.io/badge/license-MIT-green)

**A Reachy Mini control stack, built from scratch in Rust — no official SDK, no daemon, no Zenoh.**

</div>

---

## What is this?

[Reachy Mini](https://www.pollen-robotics.com/) is a 6-DOF robot head (Stewart platform) on a
rotating body, with two antennae. This repo is a **learning project**: instead of using the
official SDK, we rebuild the control stack one layer at a time — motor bus → kinematics →
real-time control loop → real hardware — to understand how production robot software is
actually structured.

The core idea is a single abstraction, `ActuatorIO`, that the control loop talks to. Two
backends implement it, so **the exact same loop code drives both the simulator and the real
robot**:

- 🖥️ **`sim`** — a MuJoCo physics model with a live interactive viewer ([`mujoco-rs`](https://crates.io/crates/mujoco-rs))
- 🤖 **`hardware`** — Dynamixel XL-330 servos over a serial bus ([`rustypot`](https://crates.io/crates/rustypot))

> 🚧 **Early-stage / WIP.** The backend abstraction, the MuJoCo sim, and a basic hardware
> bring-up all work today. Inverse/forward kinematics, the watchdog, and the full fixed-rate
> loop are still on the [roadmap](#roadmap).

## Highlights

- **One trait, two worlds.** [`ActuatorIO`](src/actuator_io.rs) (`set_angles` / `angles` /
  `step` / `alive`) is the only boundary the loop knows about — swapping sim for hardware is
  a one-line change.
- **Real-time discipline.** The [control loop](src/control.rs) is deadline-driven
  (`spin_sleep::sleep_until`, not naïve `sleep`) and asserts it stays within its timestep
  budget — 100 Hz on hardware, the model timestep in sim.
- **Threaded MuJoCo viewer.** The viewer renders on the main thread while the control loop
  runs in its own thread, sharing state through `ViewerSharedState`.
- **Runs on the real robot.** The hardware binary cross-compiles to `aarch64` and deploys to
  the Reachy Mini Wireless (CM4) with a single `just deploy`.
- **Testable from day one.** An in-memory `TestIO` backend lets the loop and (soon) kinematics
  be unit-tested with zero sim or hardware dependencies.

## Architecture

```mermaid
flowchart TD
    LOOP["control loop<br/>(fixed-rate, deadline-driven)"]
    IK["IK / FK<br/>(planned)"]
    IO{{"ActuatorIO trait<br/>set_angles · angles · step · alive"}}
    SIM["InteractiveSimulation<br/>MuJoCo + viewer"]
    HW["HardwareIO<br/>Dynamixel XL-330"]
    TEST["TestIO<br/>in-memory, for tests"]

    LOOP -. "pose → angles<br/>(planned)" .-> IK
    IK -. "" .-> LOOP
    LOOP --> IO
    IO --> SIM
    IO --> HW
    IO --> TEST

    SIM --> ROBOTSIM(["simulated robot"])
    HW --> ROBOTHW(["physical robot"])
```

## Quickstart

### Prerequisites

- **Rust** (edition 2024 toolchain) — install via [rustup](https://rustup.rs/)
- **[`just`](https://github.com/casey/just)** — command runner (`cargo install just`)
- **[`cross`](https://github.com/cross-rs/cross)** — only for deploying to the robot
- **[`uv`](https://github.com/astral-sh/uv)** — optional, for the Python helper scripts

### Run the simulator

```bash
just sim          # = cargo run --bin sim  (opens the MuJoCo viewer)
```

### Run the tests

```bash
just test
```

### Deploy to a real Reachy Mini Wireless

```bash
just deploy       # cross-compile hw → scp to the robot → ssh -t and run
```

This cross-builds the `hw` binary for `aarch64-unknown-linux-gnu`, copies it to the host
named `reachy` (configurable at the top of the [`justfile`](justfile)), and runs it over SSH.


## Repository layout

| Path | What's there |
|------|--------------|
| `src/actuator_io.rs` | The `ActuatorIO` trait + `sim`, `hardware`, and test backends |
| `src/control.rs` | The fixed-rate control loop and test motion pattern |
| `src/bin/sim.rs` | Simulator entry point (MuJoCo viewer + loop thread) |
| `src/bin/hw.rs` | Hardware entry point (Dynamixel over serial) |
| `vendor/` | Vendored MuJoCo model + Reachy Mini robot description |
| `assets/` | README banner / artwork |

## Roadmap

- [x] **1.** `ActuatorIO` motor-bus interface + joint-ordering convention
- [x] **2.** MuJoCo backend (`InteractiveSimulation`) + threaded viewer
- [x] **3.** Hardware backend (`HardwareIO`, Dynamixel XL-330) — basic bring-up working
- [ ] **4.** Stewart-platform **inverse kinematics** as a pure function
- [ ] **5.** IK round-trip tests (`fk(ik(pose)) == pose`, MuJoCo as FK oracle)
- [ ] **6.** Fixed-rate loop against a `FakeBus` (jitter, watchdog, comm-timeout)
- [ ] **7.** Wire loop + IK + MuJoCo together — first real "head goes" demo
- [ ] **8.** **Forward kinematics** from scratch; retire the MuJoCo FK oracle
- [ ] **9.** Real-robot smoke test with the full loop + IK
- [ ] **10.** Measure sim-vs-hardware jitter — the real-time lesson landing

See [`AGENTS.md`](AGENTS.md) for the robot/dev guide (components, safety limits, SDK notes).

## License

This project's own source code is licensed under the [MIT License](LICENSE).

The robot description and assets under `vendor/reachy_mini/` are **not** covered by MIT —
they are vendored from [pollen-robotics/reachy_mini](https://github.com/pollen-robotics/reachy_mini)
and retain their own terms (software Apache-2.0; hardware design files CC&nbsp;BY-SA-NC).
See [`vendor/reachy_mini/ATTRIBUTION.md`](vendor/reachy_mini/ATTRIBUTION.md).

---

<div align="center">
<sub>Reachy Mini is a robot by <a href="https://www.pollen-robotics.com/">Pollen Robotics</a>.
The MuJoCo model and assets under <code>vendor/reachy_mini/</code> are vendored from
<a href="https://github.com/pollen-robotics/reachy_mini">pollen-robotics/reachy_mini</a>
(software Apache-2.0; hardware design files CC&nbsp;BY-SA-NC) — see
<a href="vendor/reachy_mini/ATTRIBUTION.md">vendor/reachy_mini/ATTRIBUTION.md</a>.
This control stack is an independent, from-scratch reimplementation for learning.</sub>
</div>
