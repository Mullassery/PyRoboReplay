# Queue status — 2026-10-20

This file records what was shipped, what is blocked/failed, and what is rejected for this repository. It complements `AUDIT_FINDINGS.md`.

## Shipped
- Exposed phases 12–15 via CLI (`forensics`, `session`, `nav2`) and Python
  (`analyze_detection_gaps`, `fuse_rgb_thermal`, `build_navigation_session`,
  `analyze_nav2_diagnostics`) on 2026-10-19.
- Exposed six Phase 16–20 operations through the JSON `ops()` / `run_op()`
  bridge on 2026-10-20: causal graph, counterfactual query, fleet learning,
  temporal patterns, trend detection, decision tree. See
  `docs/ADVANCED_OPS.md`.
- Native MCAP export (`export --format mcap`) plus `pyroboreplay-export` for
  Rerun/MCAP through the Python API.
- Real PyTerrainMap `pyterrainmap-gridmap` v1 JSON reader (`terrain` CLI,
  `load_pyterrainmap_grid`).
- Clippy gate fixed; `-D warnings` is clean and gates CI.
- Earlier in the day: audit/bundle docs (no code changes).

## Blocked / deferred
- Typed Python APIs and CLI commands for the remaining Phase 16–20 library
  types (decision reconstruction, clustering, scenario analysis, rule
  extraction, window analysis).
- Full PyTerrainMap runtime integration (H2 store, ROS bridge).
- Linux/Windows wheel builds.

## Rejected
- Live fleet dashboard.

## Policy

Any future feature or upgrade must check `AUDIT_FINDINGS.md` and this file before implementation. A blocked item is retried only when its external dependency is available; a rejected item needs new user evidence or a design change to reopen.
