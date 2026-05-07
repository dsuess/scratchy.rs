# Reachy Mini — Experiments

Personal experiments and scripts for [Reachy Mini](https://github.com/pollen-robotics/reachy_mini), a small expressive robot by Pollen Robotics.

## Getting Started

Needs: `uv`, `homebrew`, `just`

```bash
just setup           # install all dependencies (uv sync --all-groups)
```

## Running the Simulator

Start the MuJoCo simulator daemon (keeps running until you press Ctrl+C):

```bash
just sim
```

To connect via the Reachy Mini app from another device on the same network:

```bash
just sim --no-localhost-only
```