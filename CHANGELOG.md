# Changelog

All notable changes to this project are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

This file was started 2026-09-20, during an OSS-hygiene pass. It is not
backfilled with invented entries for prior releases — that history is real
but wasn't written down in this format as it happened, and reconstructing it
now from commit messages risks getting dates/scope wrong. For the actual
history of prior versions (v0.1 through v2.10.0), see `git log` and
`docs/archive/` (old release notes, phase-completion reports, etc.). Going
forward, new changes should be added here under `[Unreleased]` and moved to a
dated section when released.

## [Unreleased]

### Fixed
- `src/lib.rs:504` — `Hypothesis`'s `#[pyclass]` relied on PyO3's deprecated
  automatic `FromPyObject` derive for `Clone` types; migrated to the explicit
  opt-in `#[pyclass(from_py_object)]` (current recommended pattern for pyo3
  0.29, matching the pinned version in `Cargo.toml`). Preserves existing
  behavior; verified the deprecation warning is gone from `cargo build --lib`
  and `cargo clippy --all-targets --all-features`, and `cargo test --lib`
  still passes (834/834).
- 22 clippy `manual_clamp` findings across `src/analyzers/`, `src/cli/`,
  `src/core/`, `src/intelligence/`, `src/phase19/`, and `src/streaming/`
  (e.g. `src/analyzers/scoring.rs:91,108,125,144`,
  `src/analyzers/adaptive_recalibration.rs:121,162,204,260,266`) — replaced
  manual `.max(a).min(b)` / `.min(b).max(a)` chains with `.clamp(a, b)`, using
  clippy's own machine-generated replacement suggestions.
- 9 clippy `approx_constant` findings (imprecise `PI`/`TAU` float literals:
  `3.14`, `-3.14`, `6.28`) in test fixtures across `src/core/event.rs:392-393`,
  `src/core/timeline.rs:345-346,360-361,390-391`, `src/cli/causal_viz.rs:351`,
  and `tests/test_python_api_integration.rs` — replaced with
  `std::f32::consts::PI` / `std::f32::consts::TAU`.
- 3 clippy findings for `drop()` called on a `&mut` reference (a no-op) in
  `src/analyzers/robot_calibration.rs:339,358,375` — replaced with
  `let _ = profile;` per clippy's suggested fix.
- `.github/workflows/release.yml` used the archived/deprecated
  `actions/create-release@v1` and `actions/upload-release-asset@v1`; replaced
  both with `softprops/action-gh-release@v2` (used once to create the release,
  once per build matrix leg to attach wheel assets) and added an explicit
  `permissions: contents: write` block. Validated with `actionlint` (clean);
  not exercised against a real tag push (no network/GitHub access in this
  environment).
- `pyproject.toml`'s `[tool.maturin.sdist]` `include` list referenced a
  top-level `CLAUDE.md` that doesn't exist (the real file is at
  `docs/CLAUDE.md`); corrected the path.
- `.github/workflows/docs.yml`'s `readme` job ran `markdownlint README.md
  CLAUDE.md` against a nonexistent path; corrected to `docs/CLAUDE.md`.
- Outdated GitHub Actions versions flagged by `actionlint`: `actions/cache@v3`
  to `@v4`, `actions/setup-python@v4` to `@v5`, `codecov/codecov-action@v3`
  to `@v4`.
- `Cargo.lock` was gitignored despite this crate shipping a real binary
  (`pyroboreplay` CLI); now committed for reproducible builds.
- Removed stale `git.example.com` placeholder URLs from `CONTRIBUTING.md`,
  `docs/GITHUB_SETTINGS.md`, and `docs/QUICKSTART.md`, replaced with the real
  GitHub repo URL.
- `CONTRIBUTING.md` referenced a root-level `CLAUDE.md` and `CODE_OF_CONDUCT.md`
  that lived at different paths (or, for the latter, under `docs/`); fixed
  links and moved `docs/CODE_OF_CONDUCT.md` to the repository root (the
  conventional location).
- README version references (`2.9.2` in badges/text) were out of sync with
  `Cargo.toml`/`pyproject.toml` (`2.10.0`); updated, and the pinned
  `pip install pyroboreplay==2.9.2` install command was changed to an
  unpinned `pip install pyroboreplay` since this pass could not verify which
  version is actually live on PyPI (no network access in this environment).

### Changed
- `CONTRIBUTING.md`'s "Areas We're Looking For Help" and "Roadmap" sections
  contained a stale, fabricated-sounding quarterly roadmap (Q4 2026 / 2027
  dates) that pre-dated the actual v2.10.0 codebase and listed already-shipped
  work (Postgres/S3/BigQuery backends) as still-needed. Replaced with a
  pointer to `ROADMAP_HONEST.md`.
- Renamed `ROADMAP.md` to `ROADMAP_HONEST.md` and added a "Technical debt"
  section with concrete `file:line` findings (see that file).
- Moved 19 stale phase/status/strategy/testing snapshot docs (from both the
  repo root and `docs/`) into `docs/archive/` with an index explaining why
  each was archived, rather than deleting them.
- Consolidated three overlapping stats-dashboard docs into one canonical
  `docs/STATS_DASHBOARD.md` (kept current; the flag it documents,
  `--stats-dashboard`, is real and verified against `src/cli/args.rs`).

### Removed
- `pyroboreplay/scripts/__init__.py.bak` — an empty, tracked backup file with
  no purpose.

## Known issues as of this pass

See `ROADMAP_HONEST.md` for the full, current list. Highlights:
- The MCP tool integration (`pyroboreplay/_mcp_tools.py`, `_mcp_connector.py`)
  is unshippable dead code — it lives outside the directory maturin actually
  packages, so it's not importable from an installed wheel, despite
  `pyroboreplay.toml` claiming all 13 MCP tools are enabled. Not addressed in
  this pass (needs a real wire-up-or-delete decision).
- `cargo clippy --all-targets --all-features -- -D warnings` now fails with 61
  errors (down from 96 — the mechanical `manual_clamp`, `approx_constant`, and
  no-op-`drop` categories were fixed in this pass). The remainder is 3
  `>7-arg` function signatures and 15+ dead struct fields/methods that need a
  real refactor-or-delete decision, plus a handful of smaller style lints, none
  touched in this pass.
- `459` `.unwrap()` calls in `src/` (~208 estimated outside test modules) —
  not audited in this pass, too large a surface for a quick-fix pass.
- `src/adapters/ros2.rs` — every `parse_*_message` helper (lidar, camera, IMU,
  odometry, pose) ignores the actual message payload bytes (`msg.data`) and
  returns hardcoded zero/empty values; real per-topic CDR deserialization is
  unimplemented. Checked in this pass and confirmed to be real feature work,
  not a quick fix — left untouched.
