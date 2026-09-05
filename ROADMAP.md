# PyRoboReplay: Roadmap

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
  it on every push/PR. README states 826 passing unit tests, 0 failing,
  verified 2026-08-23; this roadmap doesn't re-verify that count but the CI
  wiring confirms the suite is real and enforced, not aspirational.
- **Real storage backend integrations** with dedicated Docker-backed integration
  test suites: Postgres, S3, BigQuery (`tests/test_postgres_backend_integration.rs`,
  `test_s3_backend_integration.rs`, `test_bigquery_backend_integration.rs`), plus
  Ollama LLM integration (`tests/test_ollama_integration.rs`).
- **A working CLI** with four real subcommands — `replay`, `analyze`, `compare`,
  `list` (`src/cli/`) — and a **working Python API** exposing `Mission`, `Event`,
  `Failure`, `Hypothesis`, `RootCauseAnalysis`, `Action`, `FleetStatistics`,
  `GeoHotspot` (`src/pyroboreplay/__init__.py`).
- **MCP tool schema definitions**: `pyroboreplay/_mcp_tools.py` defines 13 tool
  schemas (`load_replay`, `analyze_trajectory`, `detect_anomalies`,
  `build_causality_graph`, `reconstruct_decisions`, `analyze_sensor_reliability`,
  `compare_replays`, `extract_failure_root_cause`, `simulate_counterfactual`,
  `extract_skill_demonstrations`, `visualize_replay`, `export_replay_metadata`,
  `batch_analyze_replays`) with a connector (`_mcp_connector.py`) that optionally
  hooks into an external `statguardian` package if present, falling back to a
  local base class otherwise.

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
- **MCP tool handlers**: the 13 tool *schemas* are real and typed, but whether
  each handler performs real analysis end-to-end (versus returning
  placeholder/partial results) was not verified in this pass — nothing in
  README currently makes claims about MCP tool behavior one way or the other.
  Treat as unverified scaffolding until someone traces `_mcp_tools.py`'s
  handlers against the Rust core they're supposed to call.
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
2. **Verify and document the MCP tool handlers** — trace each of the 13 tool
   schemas in `_mcp_tools.py` to confirm it calls real Rust-core analysis
   rather than returning a stub, and record the result (real vs. partial) the
   same way README documents CLI/binding exposure today.
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
