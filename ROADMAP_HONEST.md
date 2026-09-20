# PyRoboReplay: Honest Roadmap & Status

Last verified: 2026-09-20, against commit `10a17bf` on `master`. Commands run
directly in this pass: `cargo build --lib`, `cargo test --lib`, `cargo fmt
--check`, `cargo clippy --all-targets --all-features -- -D warnings`,
`maturin sdist`. Results are reported exactly, including failures — see
"Technical debt" below.

## Honest status (read this first)

The previous version of this file was generic platform boilerplate — "MCP 2.0
Platform member," "19 platform projects," "Product Team," "Enterprise features,"
quarterly phase timelines with week-by-week estimates — that had nothing to do
with this repo's actual codebase (a Rust forensic-analysis/causal-inference
engine for robot mission replay). It was last edited alongside commit `f47a912`
(2026-08-06), before Phases 14 through 20 (temporal fusion, root-cause inference,
causal graphs, counterfactual analysis, decision trees — all real, see below)
shipped. It never described this project at any point in its life. This rewrite
replaces it with a roadmap grounded in what's actually in `src/` today, matching
the audit standard already applied to README.md's per-phase notes (see, e.g.,
the Phase 11 and CLI-exposure caveats there).

## What's real and working today

- **A real Rust core** implementing 20 numbered phases of causal/forensic
  analysis logic (`src/analyzers/`, `src/core/`, `src/perception/`,
  `src/fusion/`, `src/knowledge/`, `src/phase14/` through `src/phase20/`) —
  reality-gap detection, root-cause inference, causal graph construction,
  counterfactual scenario analysis, and decision-tree/rule extraction
  (`src/phase20/decision_tree.rs`, `counterfactual_scenarios.rs`,
  `rule_extraction.rs`).
- **A real, CI-enforced test suite**: `cargo test --lib` runs in
  `.github/workflows/ci.yml` on both `ubuntu-latest` and `macos-latest`, on
  stable and beta Rust — this isn't just a local claim, CI actually exercises
  it on every push/PR. Re-run locally in this pass on macOS/arm64
  (`RUSTFLAGS="-C link-args=-undefined -C link-args=dynamic_lookup" cargo test
  --lib`): **834 passed, 0 failed, 0 ignored** (up from the 826 recorded
  2026-08-23 — the suite has grown, not shrunk). `cargo build --lib` and
  `cargo fmt --check` are also clean, verified directly.
- **Real storage backend integrations** with dedicated Docker-backed integration
  test suites: Postgres, S3, BigQuery (`tests/test_postgres_backend_integration.rs`,
  `test_s3_backend_integration.rs`, `test_bigquery_backend_integration.rs`), plus
  Ollama LLM integration (`tests/test_ollama_integration.rs`).
- **A working CLI** with four real subcommands — `replay`, `analyze`, `compare`,
  `list` (`src/cli/`) — and a **working Python API** exposing `Mission`, `Event`,
  `Failure`, `Hypothesis`, `RootCauseAnalysis`, `Action`, `FleetStatistics`,
  `GeoHotspot` (`src/pyroboreplay/__init__.py`).
- **MCP tool schema definitions exist as source code but are not shipped or
  importable** (re-verified this pass, corrects the previous "unverified
  scaffolding" note below — it's worse than unverified, it's dead): `pyroboreplay/_mcp_tools.py` and
  `pyroboreplay/_mcp_connector.py` define 13 tool schemas (`load_replay`,
  `analyze_trajectory`, `detect_anomalies`, `build_causality_graph`,
  `reconstruct_decisions`, `analyze_sensor_reliability`, `compare_replays`,
  `extract_failure_root_cause`, `simulate_counterfactual`,
  `extract_skill_demonstrations`, `visualize_replay`, `export_replay_metadata`,
  `batch_analyze_replays`), but they live in a top-level `pyroboreplay/`
  directory that is **a different directory from the one actually packaged**.
  `pyproject.toml` sets `python-source = "src"`, so maturin only ever packages
  `src/pyroboreplay/` (which contains only `__init__.py`) into the wheel. The
  top-level `pyroboreplay/_mcp_tools.py` / `_mcp_connector.py` are never
  included in `pip install pyroboreplay` and are not importable from an
  installed package — `from pyroboreplay import _mcp_tools` fails with
  `ModuleNotFoundError` on a real install. `pyroboreplay.toml`'s `[mcp]`
  section claims all 13 tools are `enabled = true`, which is misleading: the
  config exists, the code implementing it is orphaned. See "Technical debt"
  below.

## What's partially built or scaffolding

- **Phases 12–15 (retrospective DINO/SAM detection, RGB+thermal fusion,
  universal temporal fusion, Nav2 root-cause inference) exist as internal Rust
  library modules with real unit tests, but are not wired up to the CLI or
  Python bindings** — confirmed by README's own notes under each phase and in
  "Your First Forensic Analysis." The same gap likely extends to Phases 16–20
  (causal graphs, counterfactual analysis, decision trees), which are newer and
  not yet mentioned in the CLI/Python-exposure notes at all — treat them as
  Rust-internal-only until confirmed otherwise.
- **Phase 11's "PyTerrainMap Integration" naming is misleading**: despite the
  module name (`pyterrain_bridge.rs`) and doc comments referencing PyTerrainMap,
  it is self-contained terrain-modeling logic with no actual dependency on or
  data compatibility with the separate PyTerrainMap repo (verified: no
  Cargo/pip dependency between them, incompatible data shapes). This is already
  corrected in README; flagging it here so the roadmap doesn't imply real
  interop work remains to "finish" that integration — there is no integration
  to finish, only a naming cleanup that could still be done.
- **No crates.io publication** — the Rust crate is source-build-only
  (`cargo build --release`); only the Python wheel is published to PyPI. This
  is stated plainly in README and isn't a gap so much as a scope decision, but
  it means "build from source" is the only path to the raw Rust API today.

## Realistic near-term roadmap

Given what's actually shipped versus exposed, the highest-leverage next steps
are closing the CLI/binding gap on work that already exists, not adding new
analysis phases:

1. **Expose Phases 12–15 (and audit 16–20) through the CLI or Python bindings.**
   The analysis logic and its unit tests already exist; the gap is purely
   plumbing. This directly resolves the caveat repeated four times in README's
   "Quick Start" and "Real-World Use Cases" sections.
2. **Fix or remove the orphaned MCP module.** Either move `pyroboreplay/_mcp_tools.py`
   and `_mcp_connector.py` into `src/pyroboreplay/` so they actually ship in the
   wheel (then verify each of the 13 handlers calls real Rust-core analysis
   rather than returning a stub), or delete them and the misleading `[mcp]`
   block in `pyroboreplay.toml` if MCP support isn't actually a near-term
   priority. Leaving unshippable code and an "enabled = true" config next to
   each other is actively misleading to a reader who doesn't check maturin's
   packaging rules.
3. **Correct or remove the `pyterrain_bridge` naming/doc-comment mismatch**
   (Phase 11) so a future reader doesn't need to re-derive that it's not real
   PyTerrainMap interop.
4. **Replace this file's old quarterly/enterprise fantasy items** (SaaS,
   multi-tenancy, ">5 teams in production," 2027 predictive modeling) — none
   of that reflects a committed plan; if any of it is still wanted, it should
   be re-added later as a deliberate, dated decision, not carried forward by
   default from a template.

No dates or week-by-week estimates are given above because nothing in this
repo's commit history supports a specific pace claim beyond "phases keep
shipping" — the last thing this file said about timelines was wrong for over
a month before this rewrite, and a wrong date is worse than no date.

## Technical debt (verified this pass, 2026-09-20)

### Broken / not built
- **MCP integration is dead code, not "scaffolding."** See above — `pyroboreplay/_mcp_tools.py`
  and `pyroboreplay/_mcp_connector.py` are outside `src/`, so they are never
  packaged by maturin (`python-source = "src"` in `pyproject.toml`) and are not
  importable from an installed wheel. `pyroboreplay.toml`'s `[mcp.tools]` block
  claims all 13 tools are `enabled = true`; none of them are reachable.
- **`pyroboreplay/scripts/__init__.py.bak`** — an empty, tracked `.bak` file
  with no purpose (removed in this pass).
- **`docs/RELEASES.md` (now archived) and the old `docs/ROADMAP.md`** contained
  a fantasy "v0.9/v1.0/v1.1+ 2027" roadmap and release notes stopping at v0.8.0,
  years behind the actual v2.10.0 codebase. `CONTRIBUTING.md` had the same
  stale fantasy roadmap (Q4 2026 "AI-Driven Remediation," 2027 "Autonomous
  Systems") — removed in this pass, replaced with a pointer to this file.

### CI / build gaps
- **`Cargo.lock` is gitignored** (`.gitignore` had a bare `Cargo.lock` line) even
  though this crate ships a real binary (`src/main.rs`, `[[bin]] pyroboreplay`)
  in addition to the library — per this org's established pattern, binary-shipping
  crates should commit `Cargo.lock` for reproducible builds. Fixed in this pass:
  removed from `.gitignore` and committed `Cargo.lock`.
- **Outdated GitHub Actions versions** (found via `actionlint`): `actions/cache@v3`
  (`.github/workflows/ci.yml:29,35,41`), `actions/setup-python@v4`
  (`ci.yml:83,111`, `release.yml:43`), `codecov/codecov-action@v3`
  (`security.yml:68`). Fixed in this pass — bumped to `@v4`/`@v5` as appropriate.
- **`docs.yml`'s `readme` job lints a nonexistent path**: `markdownlint README.md
  CLAUDE.md` (`.github/workflows/docs.yml`) — there is no top-level `CLAUDE.md`
  (it lives at `docs/CLAUDE.md`), so this step has always either errored or been
  silently swallowed by the trailing `|| true`. Fixed in this pass: path updated
  to `docs/CLAUDE.md`.
- **`release.yml` uses archived/deprecated GitHub Actions**: `actions/create-release@v1`
  and `actions/upload-release-asset@v1` are both archived by GitHub (unmaintained
  since 2021); the maintained replacement is `softprops/action-gh-release`. Not
  changed in this pass — swapping the release mechanism touches how tags publish
  real releases and can't be verified without actually cutting a release in this
  sandbox (no network/GitHub access), so it's flagged here for a dedicated
  follow-up rather than risked blind.
- **`release.yml`/`ci.yml`'s `publish` jobs depend on `secrets.PYPI_TOKEN`**;
  this pass has no way to confirm that secret is actually configured on the
  GitHub repo (no network access, can't query GitHub). If it's missing or
  expired, tag-triggered publishes silently fail at the `twine upload` step.
  Verify manually in repo Settings → Secrets.
- **`cargo audit` / `cargo outdated` could not be run** in this sandbox (no
  network access to crates.io). `security.yml`'s existing `--ignore` list for
  RUSTSEC-2026-0258/-0104/-0098/-0099 (all from `aws-smithy-http-client`'s
  hyper-0.14 compat path) is documented in-workflow as still current as of the
  last edit; not re-verified here.

### Code quality (large surface, needs a dedicated pass, not touched here)
- **`cargo clippy --all-targets --all-features -- -D warnings` fails with 96
  errors** as of this commit (re-run 2026-09-20; CI already runs this with
  `continue-on-error: true`, so it isn't silently regressing, but README's
  documented "Quality Checks" command (`cargo clippy --all-targets -- -D
  warnings`) will fail if a contributor actually runs it as written). Breakdown
  by category:
  - 22× "clamp-like pattern without using clamp function" (mechanical, safe to
    fix in bulk with `.clamp()`).
  - 8× imprecise `std::f32/f64::consts::PI` literal usage instead of the
    constant.
  - 3× functions with too many arguments (>7): `src/knowledge/world_model.rs:272`,
    `src/phase14/video_processing.rs:448`, `src/phase14/video_processing.rs:479`.
  - 3× `drop()` called on a `&mut` reference, doing nothing:
    `src/analyzers/robot_calibration.rs:339,358,375`.
  - 2× `assert!(x.len() >= 0)` — always-true assertions on an unsigned length:
    `src/phase19/temporal_patterns.rs:301`, `src/phase19/trend_detector.rs:300`.
  - 2× `field_reassign_with_default` (construct-then-mutate instead of struct
    update syntax), e.g. `src/streaming/fleet_monitor.rs:391-392`.
  - **1× deprecated PyO3 API usage that will break on a future PyO3 upgrade**:
    `src/lib.rs:504` — `#[pyclass]` on a `Clone` type relies on the
    now-deprecated automatic `FromPyObject` derive; PyO3 is moving this to
    opt-in via `#[pyclass(from_py_object)]`. Worth fixing proactively before
    the next PyO3 bump breaks the build.
  - ~15+ dead-code fields/methods/structs never read or constructed, spread
    across nearly every phase module, e.g.: `src/adapters/ros2.rs:20`
    (`RosMessage.data`/`.topic_id` parsed from the ROS2 bag but never used —
    this is the "legacy Ros2Adapter stub" noted in prior status docs),
    `src/core/root_cause.rs:85` (`hypotheses_cache`), `src/core/counterfactual.rs:89`
    (`events`), `src/phase16/causal_builder.rs:274` (`fleet_data`),
    `src/phase17/counterfactual.rs:68` (`events`), `src/phase18/decision_clustering.rs:44`
    (`DecisionSignature` struct never constructed), `src/cli/stats_dashboard.rs:229`
    (`is_running`). This pattern (fields written, never read) across so many
    modules suggests either incomplete feature wiring or write-only
    bookkeeping left over from earlier iterations — worth a real audit, not a
    blanket `#[allow(dead_code)]`.
  - Full list saved during this pass; re-run `cargo clippy --all-targets
    --all-features -- -D warnings` for the current complete output (96 errors,
    2026-09-20).
- **459 `.unwrap()` calls in `src/`** (counted via `grep -rn '\.unwrap()'
  --include='*.rs' src/`); a heuristic pass (checking for a `#[cfg(test)]`
  marker within 200 lines above each call) estimates roughly **208 are outside
  test modules** — i.e. potential panic points in library code that a caller
  (CLI or Python binding) could trigger with bad input. Not verified
  individually; flagged for a dedicated error-handling audit rather than a
  blind global rewrite.
- **`Mission.from_ros_bag` / `Ros2Adapter` reads message payload and topic id
  but never uses them** (`src/adapters/ros2.rs:17-20`) — the struct fields
  exist and are populated but clippy confirms they're dead, meaning the actual
  ROS2 bag-to-event conversion for those fields may be incomplete. Needs
  tracing through `parse_bag_file` to confirm what data actually reaches
  `MissionRecord` versus what's silently dropped.

### Duplication
- Five near-duplicate docs describing the same stats-dashboard feature existed
  at once (`STATS_DASHBOARD.md`, `STATS_DASHBOARD_QUICKSTART.md`,
  `CLI_STATS_DASHBOARD_FEATURE.md`, `IMPLEMENTATION_SUMMARY.md`, plus mentions
  in `docs/BUILD_SUMMARY.md`). Consolidated in this pass: `docs/STATS_DASHBOARD.md`
  kept as the canonical doc (verified against the real `--stats-dashboard` CLI
  flag in `src/cli/args.rs:45-47`); the other three archived to `docs/archive/`.
- 32 files and ~12,000 lines under `docs/` before this pass, most of them
  point-in-time phase/status/strategy snapshots that were never updated after
  being written (some referencing `git.example.com` placeholder URLs, e.g. the
  now-archived `docs/RELEASES.md` and `docs/PUBLISH.md`). 19 files moved to
  `docs/archive/` in this pass with an index; the rest were current reference
  docs and were left in place.
