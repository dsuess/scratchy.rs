# Reachy Mini — Python Dev Guide

## Project Setup

This project uses **uv** for dependency management.

```bash
uv sync                  # install dependencies
uv run python main.py    # run a script
uv add <package>         # add a dependency
```

The `sim` dependency group includes MuJoCo simulation support:

```bash
uv sync --group sim
```

## Robot Basics

**Reachy Mini** components:

| Component | Description |
|-----------|-------------|
| Head | 6 DOF: roll, pitch, yaw + x, y, z (Stewart platform) |
| Body | Rotation around vertical axis |
| Antennas | 2 motors, also usable as physical buttons |

**Hardware variants:**
- **Lite**: USB connection to laptop (full compute)
- **Wireless**: Onboard CM4 via WiFi (limited compute)

## SDK Essentials

### Connection

```python
from reachy_mini import ReachyMini

with ReachyMini() as mini:
    # Your code here
    pass
```

### Motion Methods

| Method | Use when |
|--------|----------|
| `goto_target()` | Default — smooth interpolation for gestures ≥ 0.5s |
| `set_target()` | Real-time control loops at 10Hz+ (e.g. tracking) |

### Emotions Library

```python
from reachy_mini.motion.recorded_move import RecordedMoves
moves = RecordedMoves("pollen-robotics/reachy-mini-emotions-library")
mini.play_move(moves.get("happy"), initial_goto_duration=1.0)
```

### Before Writing Code

- Read `docs/source/SDK/python-sdk.md` for the API overview
- Skim `src/reachy_mini/reachy_mini.py` for method signatures
- Check `examples/` — start with `examples/minimal_demo.py`

## Safety Limits

| Joint | Range |
|-------|-------|
| Head pitch / roll | [-40, +40] degrees |
| Head yaw | [-180, +180] degrees |
| Body yaw | [-160, +160] degrees |
| Head–body yaw delta | Max 65° difference |

The SDK clamps values automatically. Gentle collisions with the body are safe.

## Skills

When you need deeper guidance, read the relevant file in `skills/`:

| Skill | When to use |
|-------|-------------|
| `control-loops.md` | Real-time reactive apps (tracking, games) |
| `motion-philosophy.md` | Choosing `goto_target` vs `set_target` |
| `safe-torque.md` | Enabling/disabling motors without jerky motion |
| `ai-integration.md` | LLM-powered apps |
| `debugging.md` | Connectivity issues, crashes, basic checks |
| `testing-apps.md` | Sim vs physical testing |

## Documentation

| Topic | File |
|-------|------|
| Quickstart | `docs/source/SDK/quickstart.md` |
| Python SDK | `docs/source/SDK/python-sdk.md` |
| Core concepts | `docs/source/SDK/core-concept.md` |
| AI integration | `docs/source/SDK/integration.md` |
| Troubleshooting | `docs/source/troubleshooting.md` |

## Planning & Workflow

- Write a `plan.md` before implementing anything non-trivial
- List approach, open questions, and expected behaviour
- Check in before coding; mark items done as you go
