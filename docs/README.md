# PyRoboReplay documentation

Start here depending on what you need.

## Getting started

| Document | What it covers |
|---|---|
| [Install](INSTALL.md) | Python API vs CLI, platform/wheel matrix, build from source |
| [Quickstart](QUICKSTART.md) | First mission replay and analysis |
| [Project status](STATUS.md) | What is verified, current gaps, roadmap |
| [Alternatives](ALTERNATIVES.md) | How this differs from Foxglove and Rerun |

## Reference

| Document | What it covers |
|---|---|
| [API](API.md) | Python and Rust API reference |
| [Advanced operations](ADVANCED_OPS.md) | Phase 16-20 JSON operation bridge (`ops()` / `run_op()`) |
| [Architecture](ARCHITECTURE.md) | Phase-by-phase architecture and data flow |
| [Implementation status](IMPLEMENTATION_STATUS.md) | Phase-by-phase engineering status (historical detail) |

## Feature guides

- [Camera export](CAMERA_EXPORT.md)
- [IMU visualization](IMU_VISUALIZATION.md)
- [LiDAR visualization](LIDAR_VISUALIZATION.md)
- [Sensor metadata](SENSOR_METADATA.md)
- [Keyboard shortcuts](KEYBOARD_SHORTCUTS.md)
- [Stats dashboard](STATS_DASHBOARD.md)
- [Alpamayo and AlpaSim workflow](ALPASIM_WORKFLOW.md)

## Interop

- [MCAP export](../README.md#mcap-export)
- [Rerun playback](../README.md#rerun-mission-playback)
- [PyTerrainMap interop](../README.md)

## Internal / historical

These are maintainer or snapshot documents, not onboarding material:

- [CLAUDE.md](CLAUDE.md) — product vision and architecture notes
- [GITHUB_SETTINGS.md](GITHUB_SETTINGS.md) — repository settings
- [`archive/`](archive/) — old release notes and phase reports

## Status files

`ROADMAP_HONEST.md`, `QUEUE_STATUS.md`, and `IMPLEMENTATION_STATUS.md` in the
repository root are dated engineering snapshots. For the current user-facing
status, use [STATUS.md](STATUS.md).
