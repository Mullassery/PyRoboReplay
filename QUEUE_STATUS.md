# Queue status — 2026-10-19

This file records what was shipped, what is blocked/failed, and what is rejected for this repository. It complements `AUDIT_FINDINGS.md`.

## Shipped (2026-10-19 implementation pass)
- Exposed phases 12–15 via CLI (`forensics`, `session`, `nav2`) and Python
  (`analyze_detection_gaps`, `fuse_rgb_thermal`, `build_navigation_session`,
  `analyze_nav2_diagnostics`).
- Native MCAP export (`export --format mcap`) plus `pyroboreplay-export` for
  Rerun/MCAP through the Python API.
- Real PyTerrainMap `pyterrainmap-gridmap` v1 JSON reader (`terrain` CLI,
  `load_pyterrainmap_grid`).
- Clippy gate fixed; `-D warnings` is clean and gates CI.
- Earlier in the day: audit/bundle docs (no code changes).

## Blocked / deferred
- Phases 16–20 CLI/Python exposure.
- Full PyTerrainMap runtime integration (H2 store, ROS bridge).
- Linux/Windows wheel builds.

## Rejected
- Live fleet dashboard.

## Policy

Any future feature or upgrade must check `AUDIT_FINDINGS.md` and this file before implementation. A blocked item is retried only when its external dependency is available; a rejected item needs new user evidence or a design change to reopen.
