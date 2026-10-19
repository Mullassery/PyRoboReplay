# Project status

**PyPI version:** 2.11.0 · **Source version:** 2.11.0 (packaging fixes
unreleased) · **Updated:** 2026-10-19

For the long engineering snapshot, see
[`ROADMAP_HONEST.md`](../ROADMAP_HONEST.md),
[`QUEUE_STATUS.md`](../QUEUE_STATUS.md), and
[`IMPLEMENTATION_STATUS.md`](IMPLEMENTATION_STATUS.md).

## Verified working

- Rust core: 859 `cargo test --lib` tests pass; clippy gated in CI.
- Python public surface: `Mission`, failure detection/analysis, export helpers;
  5 public-surface integration tests pass.
- CLI (from source): `replay`, `compare`, `analyze`, `list`, `terrain`,
  `forensics`, `nav2`, `session`, `export`.
- ROS 2 CDR decoding, failure hypotheses, detection-gap analysis, RGB+thermal
  fusion, Nav2 heuristics, mission comparison.
- MCAP and Rerun export (optional extras).

## Fixed in the repository (unreleased)

These fixes are committed/pushed but are **not** in the 2.11.0 PyPI wheel:

| Fix | Detail |
|---|---|
| `pyroboreplay` console script | Wheel now ships `entry_points.txt` with `pyroboreplay` (Rust CLI via PyO3) and `pyroboreplay-export`. Verified by building and installing a wheel. |
| `numpy` dependency | Declared in `pyproject.toml`; clean install now imports the package without manual installs. |
| OS classifiers | Replaced "OS Independent" with Linux/macOS to match wheel coverage. |

## Known gaps

| Gap | Impact | Status |
|---|---|---|
| 2.11.0 wheel has no CLI and no `numpy` dependency | First-run failures on PyPI | Fixed in source; ships next release |
| Windows and macOS Intel wheels | Those users must build from source | Open |
| PyPI summary and GitHub description/topics are stale | Public positioning contradicts the forensics focus | Open (metadata refresh) |
| PyPI license metadata `None` | License scanners/trust signals | Open |
| Release workflow uses `PYPI_TOKEN` + twine | Not OIDC Trusted Publishing | Open |
| Docs CI uses `\|\| true` and lints only README + CLAUDE.md | Broken links/lint can merge | Open |
| Phases 16–20 library-only | Causal graphs/counterfactuals not exposed | Planned |
| No live/fleet monitoring | Offline recorded missions only | By design |
| Documentation sprawl (54 md files) | Discovery/onboarding cost | Mitigated by [docs index](README.md); consolidation ongoing |

## Rejected / deferred

- **Live fleet monitoring / hosted dashboards** — use a visualization or fleet
  platform (Foxglove, Rerun, etc.); see [Alternatives](ALTERNATIVES.md).
- **Benchmarks** — not published until reproducible methodology exists.

## How to help

See [`CONTRIBUTING.md`](../CONTRIBUTING.md). High-value areas: platform wheels,
metadata refresh, docs CI gates, and a runnable sample-bag quickstart.
