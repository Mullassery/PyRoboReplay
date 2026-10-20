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

### Added
- `pyroboreplay.asl`: CPU-only AlpaSim ASL framing checks, decoded-entry
  mapping, and JSON/Markdown rollout reports. The upstream `alpasim_grpc`
  reader is imported lazily and remains an external dependency.
- Synthetic Alpamayo 1.5 and 2 contract fixtures with `model_version`
  provenance and 64-waypoint/10 Hz trajectories.

## [2.12.1] - 2026-10-20

### Changed
- Refreshed the README/PyPI description for 2.12.x: the wheel ships the
  `pyroboreplay` and `pyroboreplay-export` console scripts, declares `numpy`,
  and the wheel-coverage note now reflects what is actually published.
  Metadata-only release; no code changes.

## [2.12.0] - 2026-10-20

### Added
- CLI subcommands `terrain`, `forensics`, `nav2`, `session`, and `export`
  (`--format mcap`).
- Python functions `load_pyterrainmap_grid`, `analyze_detection_gaps`,
  `fuse_rgb_thermal`, `analyze_nav2_diagnostics`, and `build_navigation_session`.
- Real PyTerrainMap `pyterrainmap-gridmap` v1 JSON reader (`src/core/pyterrainmap.rs`).
- `pyroboreplay-export` console script for MCAP/Rerun export from a bag.
- `RetrospectiveDetectionEngine::load_dino_detections` for external DINO detections.
- Five public-surface integration tests (`tests/test_exposure_integration.rs`).
- Phase 16-20 JSON operation bridge: `ops()` and `run_op()` expose six real,
  allowlisted operations (causal graph, counterfactual query, fleet learning,
  temporal patterns, trend detection, decision tree). See
  `docs/ADVANCED_OPS.md`.
- Ten Rust unit tests and ten Python public-surface tests for the bridge,
  including unknown-op and malformed-JSON handling.
- Release wheel matrix for Linux x86_64/aarch64 (manylinux), macOS arm64/x86_64,
  and Windows x86_64 across Python 3.10–3.13. Existing Linux/macOS legs are
  retained; Linux aarch64 and macOS x86_64 use `PyO3/maturin-action` cross
  builds.
- Installed-wheel smoke tests (version import, `Mission` import, and `ops()`
  listing) on native runners before release assets or PyPI uploads proceed.

### Fixed
- Declared `numpy` as a runtime dependency so `import pyroboreplay` works from
  a clean install; previously it failed with `ModuleNotFoundError: numpy`.
- Added a `pyroboreplay` console script backed by the Rust CLI (via PyO3
  `run_cli`), and verified the built wheel ships both `pyroboreplay` and
  `pyroboreplay-export` in `entry_points.txt`.
- Replaced the `Operating System :: OS Independent` classifier with the actual
  Linux/macOS wheel coverage.
- `SessionBuilder` now keeps every selected modality, not just the first.
- `cargo clippy --all-targets --all-features -- -D warnings` is clean; the CI
  clippy job now gates merges instead of running with `continue-on-error`.

### Changed
- Repositioned package/repo metadata as offline mission forensics; removed
  fleet-monitoring/SLA keywords and the monitoring classifier.
- Tag publishing now runs only from the release workflow, after all wheel
  smoke tests pass. The CI workflow still builds and smokes native wheels but
  no longer publishes on tags.
- Documented that the expanded wheel matrix is configured but unpublished:
  PyPI 2.11.0 still has only Linux x86_64 and macOS arm64 wheels, and a new
  `v*` tag is required to produce the additional platforms.

## [2.11.0] - 2026-10-19

### Added
- Optional MCAP mission export with event/failure JSON messages and original timestamps.
- Optional Rerun mission playback for events and detected failures.
- Alpamayo forensic reporting for JSON/JSONL/NPZ/NPY artifacts, including
  minADE/minFDE, reasoning traces, mission-failure correlation, and JSON/Markdown output.

## [2.10.2] - 2026-10-18

### Security
- Upgrade `rustls` to `0.23.46`, resolving `RUSTSEC-2026-0346`, `0347`, `0348`, and `0349` (RECHECK-031).

### Added
- Native Windows terminal launcher (`wt`, `powershell`, `cmd`) for the stats dashboard (RECHECK-011).
- `DashboardStats::export_history_json` and `DashboardStats::export_history_csv` for offline telemetry analysis (RECHECK-012).
- Safe default `DEFAULT_MAX_STREAM_EVENTS` (`100_000`) on `ProcessorConfig::default()` to bound memory on multi-gigabyte bags unless explicitly opted out (RECHECK-013).

## [2.10.1] - 2026-10-12

### Fixed
- CI is green again: `cargo clippy --all-targets --all-features -- -D warnings`
  and `cargo fmt --check` now pass. Cleared all 47 pre-existing lib warnings
  and the denials in tests/examples — unused/dead fields now carry explicit
  `#[allow(dead_code)]` (intentional storage for future use), logic lints were
  genuinely fixed (collapsed redundant `if`/`else if` branches in
  `core/evidence_discovery.rs` and `phase17/counterfactual.rs`, replaced
  `sort_by` closures with `sort_by_key`, `vec![..]` instead of
  init-then-push, `matches!`, `checked_div`, iterator loops,
  `format!`-in-`format!`, `Default` impl for `CausalGraph`, `&mut [_]`
  over `&mut Vec`), and meaningless always-true assertions (`assert!(true)`,
  `len() >= 0`, `u8 >= 0`) in tests were removed.
- `PYRAMID`/`PI`/`TAU` literals in tests/examples now use
  `std::f32::consts::{PI,TAU}`.

### Changed
- Added an optional, isolated Graphify code knowledge-graph backend (`tools/graphify/`, `.graphifyignore`, optional workflow, README badge).

## [Unreleased]

### Fixed
- **CRITICAL: the ROS2 adapter never read real message content, for any
  sensor type** (found 2026-09-27, fixed 2026-09-29). Every parser in
  `src/adapters/ros2.rs` (`parse_lidar_message`, `parse_camera_message`,
  `parse_imu_message`, `parse_odometry_message`, `parse_pose_message`) took
  an unused `_msg: &RosMessage` and returned a hardcoded constant event
  regardless of the message's real CDR-encoded bytes. Added
  `src/adapters/cdr.rs`, a real CDR (Common Data Representation) decoder —
  the real OMG CDR encapsulation header, real little-endian primitive
  decoding, and the real alignment rules the format requires — and rewired
  all 5 parsers to decode the actual standard `sensor_msgs`/`nav_msgs`/
  `geometry_msgs` binary layouts (`LaserScan`, `Image`, `Imu`, `Odometry`,
  `PoseWithCovarianceStamped`). `parse_pose_message`'s `confidence` is now
  derived from the message's real position-covariance trace instead of a
  hardcoded `0.95`, so `detect_localization_loss`'s `confidence < 0.5` check
  can actually fire on real data — verified against a real 901-message
  benchmark bag with an injected covariance spike: 81/81 real
  localization-loss frames now correctly detected (previously 0/81). Added
  `tests/test_ros2_cdr_decoding.rs` (7 tests) against a real,
  independently-generated fixture (`tests/fixtures/ros2_cdr_fixture.db3`,
  built with the third-party `rosbags` Python library) as a permanent
  regression test. Full detail in `ROADMAP_HONEST.md` and README's "vs
  Rerun" section.
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
