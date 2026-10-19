# Audit findings — 2026-10-19

This file records the repository-specific findings from the portfolio audit
conducted on 2026-10-19. It is documentation only; no code or release artifacts
were changed in this pass unless explicitly noted under "Implementation status".

Sources: the repository's own README/source/workflows, GitHub metadata and
Actions runs, PyPI metadata, and official competitor documentation. Claims are
marked as verified, inferred, or open questions. "Absent" means not found in the
inspected repository/docs, not proof that it cannot exist.

## What it is

Mission replay and forensic root-cause analysis for autonomous robots: ROS 2 CDR decoding, reality-gap scoring, failure hypotheses, thermal/RGB forensics.

## Verified strengths

- 70k LOC; real ROS 2 CDR decoding; replay; reality-gap scoring; failure hypotheses; mission compare; RGB/thermal; causal graph/narrative/evidence quality in Rust; 843 lib tests + 7 CDR integration tests; 2 releases.
- Forensic causal analysis + thermal fusion is a genuine niche.

## Verified gaps and user-facing risks

- Phases 12–15 are not reachable from CLI/Python; clippy reports ~96 errors and CI uses `continue-on-error`.
- No crates.io; PyTerrainMap 'integration' is not real interop; no live fleet dashboard.
- Advanced features are library-only and untested through public APIs.

## Direct competitors

- Rerun (multimodal viewer/data layer), Foxglove (visualization), RViz/rosbag2, PlotJuggler.

## Ranked next actions

- P1: fix clippy gate (M).
- P1: publish Linux/Windows wheels (S/M).
- P2: expose phases 12–15 + rerun/MCAP export (L).

## Do not build

- Live fleet dashboard; recorded-mission forensics is the identity.

## Evidence gaps / validation needed

- No real customer missions; advanced features untested through public APIs.

## Recommended disposition

Invest; reliability gates first, then expose the forensic differentiators.

## Implementation status

Addressed in the 2026-10-19 implementation pass:

- **P1 clippy gate — done.** `cargo clippy --all-targets --all-features -- -D
  warnings` exits 0 and the CI clippy job gates merges.
- **P2 expose phases 12–15 + rerun/MCAP export — done.**
  - Phase 12/13: `forensics` CLI + `analyze_detection_gaps` / `fuse_rgb_thermal`.
  - Phase 14: `session` CLI + `build_navigation_session` (real bag/logs/Nav2).
  - Phase 15: `nav2` CLI + `analyze_nav2_diagnostics`.
  - MCAP export in the native CLI (`export --format mcap`); Rerun export via
    `pyroboreplay-export` / `export_mission`.
  - Covered by 5 new public-surface integration tests plus 859 lib tests at the
    time (869 including the later Phase 16-20 bridge tests).
- **PyTerrainMap interop — partial, now real at the file level.**
  `src/core/pyterrainmap.rs` reads the documented `pyterrainmap-gridmap` v1 JSON
  and is exposed as `terrain` / `load_pyterrainmap_grid`. Still no runtime
  dependency, H3 store access, or ROS bridge.
- **Phases 16–20 — JSON operation bridge done (2026-10-20).** `ops()` and
  `run_op()` expose six real, allowlisted Rust operations (causal graph,
  counterfactual query, fleet learning, temporal patterns, trend detection,
  decision tree). They are not typed Python APIs, and the rest of the Phase
  16–20 library types remain Rust-only.
- **Still open:** typed Python APIs / CLI commands for the remaining Phase
  16–20 types; Linux/Windows wheel builds; real-customer validation. Live fleet
  dashboard remains explicitly out of scope.
