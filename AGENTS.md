# Scratchy — Dev Guide (for agents & humans)

**Scratchy** is a **from-scratch Rust control stack** for the Reachy Mini robot. We are *not*
using the official Python SDK or its daemon/Zenoh/Rust crates — we rebuild each layer
(motor bus → kinematics → real-time control loop → hardware) by hand to learn how robot
software is structured. See [`README.md`](README.md) for the pitch and
[`TODO.md`](TODO.md) for the full build plan and test strategy.

> The official Python SDK is installed only as a **reference and comparison** point (and for
> the helper scripts under `scripts/`). Read it for ideas; don't depend on it in the Rust code.

## Build & run

This is a Cargo project (Rust **edition 2024**). Commands are wrapped in a
[`justfile`](justfile):

```bash
just sim            # run the MuJoCo simulator + viewer   (= cargo run --bin sim)
cargo test         # run the unit tests
cargo build        # build both binaries (default features = sim + hardware)
just deploy        # cross-compile `hw` for aarch64, scp to the robot, run over ssh
```

### Cargo features & binaries

| Feature | Pulls in | Binary |
|---------|----------|--------|
| `sim` (default) | `mujoco-rs` (with `viewer`) | `src/bin/sim.rs` → `sim` |
| `hardware` (default) | `rustypot`, `serialport`, `ctrlc` | `src/bin/hw.rs` → `hw` |

Build a single backend with e.g. `cargo run --no-default-features --features sim --bin sim`.

> **macOS caveat:** `Cargo.toml` pins a patched `glutin` fork to work around a MuJoCo viewer
> OpenGL bug. Keep the `[patch.crates-io]` block until the upstream fix ships.

## Architecture

Everything hangs off one trait, [`ActuatorIO`](src/actuator_io.rs). The control loop talks
only to this boundary, so the same loop drives sim and hardware:

```
control loop (src/control.rs)
        │  set_angles / angles / step / alive
        ▼
   ActuatorIO  ──┬── InteractiveSimulation   (sim)      MuJoCo model + threaded viewer
                 ├── HardwareIO              (hardware)  Dynamixel XL-330 over serial
                 └── TestIO                  (#[cfg(test)]) in-memory echo, for unit tests
```

- **`src/actuator_io.rs`** — the trait and all three backends.
- **`src/control.rs`** — the fixed-rate loop (`run_test`) and named joint indices.
- **`src/bin/{sim,hw}.rs`** — thin entry points that build a backend and call the loop.

### The control loop

`run_test(model, timestep)` is deadline-driven: it does work, then `spin_sleep::sleep_until`s
to the next deadline, and **asserts** the iteration fit inside `timestep` (a missed deadline
panics — that's the real-time contract made visible). Rates today: **100 Hz** (10 ms) on
hardware, the **MuJoCo model timestep** in sim. No allocation belongs in the loop body.

## Joint ordering convention (important — every layer assumes it)

Actuator angles are a fixed `[f64; NUM_ACTUATORS]` with `NUM_ACTUATORS = 9`. Named indices
live in `src/control.rs`:

| Index | Joint | Servo ID |
|------:|-------|---------:|
| `0` | `YAW_BODY` — body rotation | 10 |
| `1`–`6` | head (Stewart-platform) motors | 11–16 |
| `7` | `RIGHT_ANTENNA` | 17 |
| `8` | `LEFT_ANTENNA` | 18 |

Servo IDs `[10..=18]` map one-to-one to indices `[0..=8]` (`SERVO_IDS` in
`actuator_io::hardware`). The MuJoCo model's actuator count is asserted to equal
`NUM_ACTUATORS` at startup. **Lock this convention before touching IK** — sign and axis
conventions are the most common source of bugs.

## Hardware notes

- Servos: **Dynamixel XL-330**, Protocol v2, on `/dev/ttyAMA3` at **1 Mbaud**.
- `HardwareIO::new()` **disables all torques by default**; `enable_servos(&[..indices..])`
  turns on only the joints you ask for. The `hw` binary currently enables body yaw + both
  antennas only.
- `Ctrl-C` is trapped (`ctrlc`) and flips `alive()` to false so the loop exits cleanly.
- Deploy target host is `reachy` (see top of `justfile`); cross target is
  `aarch64-unknown-linux-gnu` (Reachy Mini Wireless / CM4).

## Robot basics

| Component | Description |
|-----------|-------------|
| Head | 6 DOF: roll, pitch, yaw + x, y, z (Stewart platform) |
| Body | Rotation around the vertical axis |
| Antennas | 2 motors, also usable as physical buttons |

**Variants:** *Lite* — USB to a laptop (full compute). *Wireless* — onboard CM4 over WiFi
(limited compute; this is the `just deploy` target).

## Safety limits

We are responsible for clamping — there is no SDK doing it for us. Respect these ranges when
writing IK and motion code:

| Joint | Range |
|-------|-------|
| Head pitch / roll | [-40, +40]° |
| Head yaw | [-180, +180]° |
| Body yaw | [-160, +160]° |
| Head–body yaw delta | ≤ 65° |

Gentle collisions with the body are safe. When in doubt, move slowly and enable the fewest
servos needed.

## Python helper scripts (optional)

The `scripts/` directory holds small programs that drive the robot through the **official
SDK** — useful as a behavioral reference to compare our stack against, not part of the build.

```bash
uv sync --group sim                       # installs reachy-mini[mujoco] (+ deps)
uv run python scripts/nod_head.py         # SDK-driven motion example
```

- `scripts/nod_head.py` — `goto_target` head/antenna/body demo via `reachy_mini`.
- `scripts/debug_daemon.py` — daemon debugging helpers.

## Reference: the official SDK

When you want to see how Pollen solved something (control-loop shape, analytical kinematics,
Dynamixel packet construction), read the installed package — **for reference, not reuse**:

- `daemon/backend/robot/backend.py` — control-loop structure
- `kinematics/analytical_kinematics.py` — Stewart-platform IK pattern
- `tools/scan_motors.py` — Dynamixel packet usage

## Planning & workflow

- Write/extend a plan (`plan.md` or update `TODO.md`) before anything non-trivial: approach,
  open questions, expected behaviour. Check in before coding; mark items done as you go.
- Keep changes minimal and root-cause focused. Match the surrounding Rust style.
- New layers ship with the test tier from `TODO.md` that covers them (pure unit → `FakeBus`
  → MuJoCo → real robot).
