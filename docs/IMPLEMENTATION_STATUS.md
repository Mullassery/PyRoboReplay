# PyRoboReplay Implementation Status

**Last Updated**: 2026-08-12
**Current Phase**: Phase 15+ (Root Cause Inference Engine and beyond); this document
reflects the actual, verified state of the codebase as of this update, not a
point-in-time snapshot from early development.

---

## How to read this document

This is a single, current status report — not a phase-by-phase changelog. Earlier
versions of this file kept an unmaintained "Phase 1 / Phase 2 / Phase 3" section
below a newer "Post-Remediation Status" header, which made the file self-contradictory
(a "Phase 15+" header sitting on top of "Phase 2: 100% complete... Task #12 pending"
prose from 2026-07-21, describing a ~3,800-LOC codebase that has since grown to 157
files / ~50k LOC). That structure has been replaced with this single, dated snapshot.
If you need the historical phase-by-phase task lists, they're in git history for this
file prior to this rewrite.

---

## Verified Current State (2026-08-12)

**Scale**: 157+ Rust source files, ~50k+ LOC, organized into 15+ phases
(`src/core`, `src/adapters`, `src/analyzers`, `src/phase14`, `src/phase15`,
`src/storage`, `src/streaming`, `src/perception`, `src/reasoning`, `src/knowledge`,
`src/fusion`, `src/intelligence`, `src/cli`).

**Tests**: `cargo test --lib` → **722 passing, 0 failing**. This includes a live
spot-check of the original Phase 1-3 functionality (event model, timeline engine,
causal analysis engine, lidar/IMU/JSON-output CLI visualizations — 51 unit tests) plus
the 11 original Phase 1 integration tests (`tests/integration_tests.rs`, all passing
once the synthetic fixture bag is regenerated via
`cargo run --example generate_warehouse_mission --release`, which is a pre-existing,
documented, working-directory-dependent requirement of that test file, not a
regression). See "Phase 1-3 spot-check findings" below for one real gap this
spot-check surfaced.

### Storage backends — all three now real, all three verified against live local services

- **`PostgresBackend`** (`src/storage/backends.rs`): real `tokio-postgres`
  implementation. Verified with 6 integration tests
  (`tests/test_postgres_backend_integration.rs`) against a live local PostgreSQL 16
  container.
- **`S3Backend`**: real `aws-sdk-s3` implementation. Verified with 6 integration tests
  (`tests/test_s3_backend_integration.rs`) against a live local MinIO container.
- **`BigQueryBackend`**: **now a real implementation** (this was previously an honest
  stub — no local emulator, no GCP credentials available). Built on
  `gcp-bigquery-client`, driven through the same dedicated-runtime + `block_on` pattern
  as the other two backends. Connects to either real BigQuery (Application Default
  Credentials) or `ghcr.io/goccy/bigquery-emulator` (via a `?endpoint=` query param on
  the `bigquery://project/dataset` connection string) for local testing. Upserts are
  implemented as delete-then-insert (BigQuery has no `ON CONFLICT`, and `MERGE`
  against a `SELECT` subquery is rejected by the emulator — both verified against a
  real running instance, documented in the source). Verified with 7 integration tests
  (`tests/test_bigquery_backend_integration.rs`) against a live local
  `ghcr.io/goccy/bigquery-emulator` container.

All three backend integration test suites are `#[ignore]`d by default (so plain
`cargo test` never requires Docker) and documented with the exact `docker run` command
needed to stand up their respective service in each test file's module doc comment.

### Phase 14 multi-modal adapters (`src/phase14/modality_adapters.rs`)

All originally-outstanding adapter TODOs are now real, tested implementations:

- **ROS 2 bags**: both real on-disk formats are parsed — `.mcap` (via the `mcap`
  crate) and `.db3` / rosbag2-sqlite (via direct `rusqlite` queries against the
  `topics`/`messages` schema). Format is auto-detected from the file extension.
- **Linux logs**: syslog RFC 3164 and RFC 5424 line formats, and dmesg's kernel
  ring-buffer format (`[timestamp] msg`) including both the raw monotonic-time form
  and the wall-clock `-T`/`--ctime` form (monotonic-time lines are explicitly flagged
  `boot_relative: true` since they have no fixed relationship to wall-clock time
  without an external boot-time anchor).
- **Nav2 exports**: costmap/map YAML+PGM pairs (`nav2_map_server`/`map_saver_cli`
  output) and `diagnostic_msgs/DiagnosticArray` records (the message type Nav2's
  diagnostic updaters publish), exported as JSON Lines or a JSON array — the practical
  text form obtainable via `ros2 topic echo -f json` without a live ROS 2 install.
- **Video metadata**: duration, resolution, codec, frame count, fps via the same
  `ffprobe` shell-out pattern already used by `src/phase14/video_processing.rs`.

12 tests in `modality_adapters::tests` exercise all of the above against real files
(real `.mcap`/`.db3` bags, real syslog/dmesg lines, a real Nav2 export directory, a
real video probed with a real `ffprobe`).

**Known scope boundary, left as explicit follow-up work**: these adapters are real and
independently tested, but nothing in the codebase yet drives their `extract_stream()`
output into `timeline_indexing::Timeline`/`TimelineEvent` end-to-end. This was
evaluated and deliberately not attempted in this pass: `TimelineEvent` has 6 variants
(`RosEvent`, `VideoFrame`, `LogEntry`, `SensorReading`, `Annotation`, `SystemMetric`)
that don't map 1:1 onto the 9 `DataSourceType`s these adapters cover, and each mapping
needs adapter-specific deserialization of the adapter's own `TimeSeriesPoint.value`
payload (e.g. `LinuxLogsAdapter` serializes a `LogRecord` as JSON that would need to be
decoded back out to populate `TimelineEvent::LogEntry`'s `level`/`message` fields) —
this is a real, scoped, but non-trivial second pass, not a one-line wire-up. Doing it
hastily risked either a shallow/wrong mapping or scope creep into a larger
refactor under time pressure; it is left as clearly-flagged follow-up work instead.

### Architectural boundary decision: 3D reconstruction stays out of this repo

`src/phase14/navigation_session.rs` previously had `// TODO: Implement 3D
reconstruction from point clouds` with a stub returning `Ok(Vec::new())`. This repo's
own `docs/CLAUDE.md` is explicit that PyRoboReplay is "mapping-independent" by design
("Does not generate maps. Consumes maps from PyTerrainMap, SLAM systems, GIS
platforms.") and that 3D reconstruction, real-time SLAM, and traversability analysis
belong to **PyTerrainMap**, a sibling system in the SHER/robotics stack.

Verified: `PyTerrainMap` is **not** a declared dependency of this crate — no path or
git dependency in `Cargo.toml` or `pyproject.toml`. Building a real 3D-reconstruction
implementation here would therefore mean duplicating a whole subsystem another
component already owns, using no real integration point, and it would need to be
thrown away once actual PyTerrainMap integration lands.

Instead, `NavigationSession::reconstruct_3d()` now delegates to an injectable
`SceneReconstructor` trait (`reconstruct_3d_with(Some(&reconstructor))`) — the seam a
future PyTerrainMap adapter would implement — and returns an explicit
`SessionError::Unsupported` (not a silently-empty `Vec`) when none is configured. This
is a deliberate scope decision per this repo's own architecture principles, not
unfinished work.

### Legacy Ros2 adapter — real limitation, not new drift

Spot-checking Phase 1's "COMPLETE" claim surfaced one genuine, pre-existing gap:
`src/adapters/ros2.rs` (the original Phase 1 `.bag`/`.db3` adapter, still live — wired
into both the CLI (`src/cli/mod.rs`) and the PyO3 Python bindings (`src/lib.rs`), not
dead code) correctly parses real topic/message/timestamp structure from a `.db3`
SQLite bag, but each `parse_*_message()` function (lidar/camera/imu/odometry/pose)
returns hardcoded default field values (empty ranges, all-zero acceleration, identity
pose, etc.) rather than actually decoding the message payload bytes — each is
literally commented `// Stub: return minimal ... event`. This was already noted in
this file's old "Known Issues" section ("ROS 2 CDR deserialization: Currently stubs")
and remains true today; it is called out explicitly here rather than silently dropped
during this rewrite. Note this is a different code path from the new, real
`RosBagAdapter` in `src/phase14/modality_adapters.rs` described above — that adapter
extracts raw per-topic message bytes (format-correct, doesn't need message-type
semantics) rather than claiming to populate typed sensor fields it doesn't decode.

### What's still known-incomplete or deferred (by design, not oversight)

Per this remediation's explicit scope bar ("core functionality complete, defer
speculative/enterprise features"), the following remain out of scope and were not
attempted:
- Gazebo / Isaac Sim adapters
- Cross-mission ML pattern learning
- Real-time streaming ingestion
- Cryptographic audit-trail signatures
- Deterministic, bit-perfect replay
- Mission-critical failover
- Compliance / ISO-3691-4 reporting

Additionally:
- `cargo fmt --check` and `cargo clippy -- -D warnings` still fail across the wider
  codebase (pre-existing, not touched by this pass or the prior remediation pass — see
  history below). Files touched in remediation passes are kept clippy-reasonable; the
  rest of the codebase's formatting/lint debt is unchanged and would need its own
  dedicated pass (a prior attempt at an automated `cargo fix --tests` sweep broke
  ~50 files' `use super::*` test imports and was reverted).
- Of the `.unwrap()`/`panic!()` calls across `src/`, only the ones on the actual
  untrusted-input attack surface (`src/adapters/`, `src/core/` bundle/evidence-discovery
  code) were audited in the prior remediation pass; the rest were not exhaustively
  swept.
- Marketing language in `README.md` (e.g. "Debug 10x faster") is aspirational framing,
  not a measured benchmark.
- **Python packaging gap, now fixed** (two compounding bugs, both fixed):
  1. As of v2.9.1, the published wheel contained *only* the compiled CLI binary
     (`pyroboreplay-2.9.1.data/scripts/pyroboreplay`) — `import pyroboreplay` raised
     `ModuleNotFoundError`. Root cause: `Cargo.toml` had no `[lib]` section, so
     `cargo`/`maturin` never produced a `cdylib` at all; maturin's bindings
     auto-detection fell back to packaging `src/main.rs`'s CLI binary as a
     "bin"-bindings wheel instead of a PyO3 extension module. Fixed by adding
     `[lib] crate-type = ["cdylib", "rlib"]` to `Cargo.toml` (`rlib` kept so
     `cargo build --lib`/`cargo test --lib` keep working normally for development) plus
     a small `build.rs` that emits the macOS `-undefined dynamic_lookup` cdylib-linker
     workaround PyO3's `extension-module` feature needs but that a plain (non-maturin)
     `cargo build` doesn't otherwise supply on this platform (maturin injects it itself
     at wheel-build time, which is why `maturin build` alone had been masking this).
  2. Fixing (1) alone was not sufficient: it produced a wheel with a real compiled
     `.so`, but `import pyroboreplay` returned an *empty PEP 420 namespace package*
     (`dir(pyroboreplay) == []`) — none of `Mission`/`Event`/etc. were reachable.
     Root cause: `src/pyroboreplay/__init__.py` existed (added in an earlier pass to
     hold `__version__`) but `pyproject.toml`'s `[tool.maturin]` had no
     `python-source`, so maturin never found or packaged it; separately, that
     `__init__.py`'s own `from ._core import *` / `__all__ = ['Replay', 'Fusion']`
     referred to a `_core` submodule and class names that never existed anywhere in
     `src/lib.rs` (dead, never-verified content from whenever that file was added).
     Fixed by wiring the two together for real: `pyproject.toml` now sets
     `python-source = "src"` and `module-name = "pyroboreplay._core"`; `src/lib.rs`'s
     `#[pymodule] fn pyroboreplay` was renamed to `fn _core` to match (PyO3 requires
     this name match the compiled module's `PyInit_<name>` symbol); and
     `src/pyroboreplay/__init__.py` now imports the classes the module actually
     registers (`Mission`, `Event`, `Failure`, `Hypothesis`, `RootCauseAnalysis`,
     `Action`, `FleetStatistics`, `GeoHotspot`) instead of the nonexistent
     `Replay`/`Fusion`.
  - **Verified end-to-end**: built the wheel with `maturin build --release`, confirmed
    the wheel contains `pyroboreplay/__init__.py` + `pyroboreplay/_core.cpython-311-darwin.so`,
    installed it into a clean virtualenv, and confirmed `import pyroboreplay`,
    `pyroboreplay.__version__`, `pyroboreplay.Mission`, and `pyroboreplay.__all__`
    all work and return the real PyO3 classes — not just that the import statement
    doesn't raise.

---

## Architecture reference

For the product vision, core principles (input-agnostic, mapping-independent,
explainability-first), and full features-by-version roadmap, see `docs/CLAUDE.md` and
`docs/ROADMAP.md`. This file intentionally does not duplicate that content — it exists
to report verified current state, not to restate the product plan.

---

## Revision history (of this document)

- **2026-08-12**: Rewritten from scratch to remove the self-contradictory "Phase 15+
  header over Phase 1-3 in-progress prose" structure; added BigQuery backend,
  Phase 14 adapter completion, PyTerrainMap boundary decision, legacy Ros2Adapter
  stub finding, and the Python packaging fix.
- **2026-08-12 (earlier same day)**: "Post-Remediation Status" section added
  documenting the security fix, 3 failing test fixes, license badge fix, and real
  Postgres/S3/Ollama integrations (v2.9.1). Superseded by this rewrite, which
  incorporates and updates that content rather than stacking another section on top
  of it.
- **2026-07-21**: Original Phase 1-3 task-tracking content (now historical; see git
  history for this file if needed).
