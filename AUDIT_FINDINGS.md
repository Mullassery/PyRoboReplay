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

No code changes in this pass.
