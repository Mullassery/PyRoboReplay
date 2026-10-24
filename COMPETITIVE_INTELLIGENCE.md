# Competitive intelligence in roadmap and upgrade work

**Policy date:** 2026-10-19

Competitive findings are an input to product decisions, not a one-off report.
Every pending roadmap item, new feature, and routine dependency upgrade should
check the repository's [`AUDIT_FINDINGS.md`](AUDIT_FINDINGS.md) before work
starts, and record the decision in the PR or changelog.

## Required check before building

1. **Read `AUDIT_FINDINGS.md`.** Find the relevant competitor row and the
   ranked next actions. If the finding is stale, update it with a dated source.
2. **Name the competitor and the user problem.** A feature is not justified by
   "competitor has it". State the workflow it unlocks, who needs it, and how
   often. Mark unverified demand as a hypothesis.
3. **Prefer reuse over new code.** Check whether an existing Mullassery repo
   already provides the capability (see `BUNDLES.md` in the profile repo and
   the bundle role file in this repo, when present). Typical reuse:
   - crawling/fetching/robots/SSRF → **MudFish**
   - architecture maps / PR diffs → **CodeAtlas**
   - data-quality contracts / validation → **StatGuardian**
   - token counting / cost tables → **PyTokenCalc**
   - dependency graph / advisories / SBOM → **PyDeepMesh**
   - segmentation / CLV / churn → **ClusterAudienceKit**
   - activation / lineage / PII → **PyReverseETL**
   - fleet state sync → **SyncFleet** (only after conflict resolution)
4. **Choose one disposition:** build, integrate, defer, or reject. Write it
   down. A rejection with a reason is a successful outcome.
5. **For routine upgrades**, check whether the upgrade closes a known
   competitor gap or breaks a differentiated capability. Run the tests that
   cover the affected path; do not merge a green build that silently changes a
   user-visible contract.

## PR / changelog checklist

- [ ] Competitor and source URL/date recorded (or "no direct competitor").
- [ ] User impact and frequency stated; unverified demand labelled.
- [ ] Reuse check done against sibling repos/bundles.
- [ ] Failure mode and rollback considered (especially auth, PII, cost).
- [ ] Docs/roadmap status updated honestly.
- [ ] Tests or a manual verification command included.

## Review cadence

- Refresh the competitor row when a release changes the capability.
- Re-check the full `AUDIT_FINDINGS.md` at least quarterly and before a major
  release.
- If a finding is wrong or stale, correct it in the same PR that relies on it.

## Non-goals

- Copying features for parity.
- Adding a dependency just because a competitor uses it.
- Treating star counts or download counts as proof of user need.

## OSS gap analysis — 2026-10-24

**What matters most here:** MCAP import and visual evidence. Modern ROS 2/Foxglove logs cannot be opened, and terminal-only replay limits team incident reviews; wire MCAP import first, then decide whether to integrate Rerun rather than build 3D.

Scope: PyRoboReplay 2.13.1 (Rust/PyO3 + Python CLI, Apache-2.0). Implemented per README, ROADMAP_HONEST, AUDIT_FINDINGS, Cargo.toml and pyproject.toml: ROS 2 SQLite `.db3`/`.bag` ingest with real CDR decoding; terminal replay; failure detection, root-cause hypotheses and actions; reality-gap scoring; mission comparison and topic listing; MCAP export; optional Rerun `.rrd` export; external DINO detection-gap ingest; RGB+thermal fusion; Nav2 heuristics and navigation sessions; PyTerrainMap file interop; Postgres/S3/BigQuery backends; six Phase 16–20 operations via a JSON `ops()` bridge. Not shipped: MCAP import, typed Python APIs for most Phase 16–20 types, live streaming, in-process DINO/SAM, SANA-WM, and a packaged MCP server. Beta status.

### Comparable OSS projects
| Project | GitHub | License | Last activity | Why it is the comparison |
|---|---|---|---|---|
| Rerun | https://github.com/rerun-io/rerun | Apache-2.0 / MIT (dual) | Pushed 2026-10-23 | Multimodal robot-log viewer/data layer; ingests MCAP, rrd and LeRobot, renders multi-rate sensors in sync, supports dataframe/SQL queries and live streaming. The tool teams open first after a failure. |
| rosbag2 | https://github.com/ros2/rosbag2 | Apache-2.0 | Pushed 2026-10-22 (rolling) | The ROS 2 reference record/playback stack and `.db3` format owner; the ingestion/playback layer users already have installed and our ingest must interoperate with. |
| MCAP | https://github.com/foxglove/mcap | MIT | Pushed 2026-10-23 | Indexed robotics log format with C++/Go/Python/JS/Swift/Rust libraries and a CLI; the interchange format Foxglove/Rerun consume and the one we export to but cannot import. |

### Capability comparison
| Capability | Ours | Rerun | rosbag2 | MCAP | Gap |
|---|---|---|---|---|---|
| ROS 2 SQLite `.db3` ingest | Yes | Partial | Yes | No | Parity with rosbag2; Rerun needs conversion |
| MCAP import | No | Yes | Partial | Yes | Absent: export-only, adapter unwired |
| MCAP export | Yes | Partial | Partial | Yes | Parity: native CLI, but no import round-trip |
| Interactive 3D/sensor visualization | Partial | Yes | No | No | Weaker: terminal timeline only |
| Automated failure detection | Yes | No | No | No | Ahead: no comparable OSS tool |
| Root-cause hypotheses + actions | Yes | No | No | No | Ahead: unique |
| Reality-gap scoring | Yes | No | No | No | Ahead: unique |
| Detection-gap (RGB + thermal) | Partial | Partial | No | No | Ahead in analysis; ingests external detections only |
| Nav2 root-cause heuristics | Yes | No | No | No | Ahead: unique |
| Mission comparison | Yes | Partial | No | No | Ahead: automated diff vs visual side-by-side |
| Live streaming / record | No | Yes | Yes | No | Absent; deliberate offline-only scope |
| Language bindings | Partial | Yes | Yes | Yes | Weaker: Python + Rust, no C++ |
| Storage backends (Postgres/S3/BigQuery) | Yes | Partial | No | No | Ahead for archival analysis |
| Multi-platform wheels | Partial | Yes | Yes | Yes | Weaker: matrix configured, not fully published |

### Gaps that matter (ranked)
1. **MCAP import is absent.** README "At a glance" lists "MCAP **import** — Not wired"; the Phase 14 adapter exists but `Mission.from_ros_bag()` only accepts ROS 2 SQLite. Rerun and MCAP read MCAP natively and rosbag2 has an MCAP storage plugin, so users with modern Foxglove/CycloneDDS logs cannot open them at all. This is absent, not weaker. Effort: M.
2. **No 3D/sensor visualization.** Replay is a terminal timeline; README "What it is not" and docs/ALTERNATIVES.md concede Foxglove/Rerun own visual inspection. Incident reviews are team activities and reviewers expect point clouds, images and transforms side by side. Rerun also ingests MCAP/LeRobot and streams live. Effort: L to build, S to lean on the existing `.rrd` export.
3. **Causal analysis is partly trapped in Rust.** ROADMAP_HONEST and AUDIT_FINDINGS: Phase 16–20 are reachable only through the JSON `ops()`/`run_op()` bridge; decision reconstruction, clustering, scenario analysis and rule extraction remain Rust-library-only. The differentiator cannot be used as a typed Python API by the data-science users who need it. Effort: M.
4. **Platform coverage is configured but not delivered.** README says PyPI publishes a macOS arm64 wheel plus sdist; QUEUE_STATUS says the expanded Linux/Windows matrix needs a new `v*` tag and its first run is unverified. AUDIT_FINDINGS ranks this P1. Users on the dominant Linux robotics stack must build Rust from source. Effort: S-M.
5. **Advertised MCP support is dead code.** ROADMAP_HONEST lines 48-66: 13 MCP tool schemas live in an unpackaged top-level directory and are absent from the wheel, while `pyroboreplay.toml` claims `enabled = true`. Agent integrations fail with `ModuleNotFoundError`; either ship or delete. Effort: S.
6. **Detection analysis does not run models.** README and ROADMAP state Phase 12 ingests externally generated DINO detections and `run_dino_detection` is a test stub. "Detection-gap analysis" therefore requires a separate open-vocabulary pipeline; Rerun at least visualizes detections directly. Effort: L if in-process inference is pursued.

### Where we are differentiated
- **Automated causal forensics.** Failure detection, probabilistic root-cause hypotheses with confidence, prioritized actions, reality-gap scoring and Nav2 localization/planner/costmap analyzers have no equivalent in Rerun, rosbag2 or MCAP; those tools answer "what" and "where", not "why".
- **Mission diffing.** Side-by-side comparison of two recorded missions with findings is beyond a viewer's visual comparison.
- **Multispectral and detection-gap analysis.** RGB+thermal fusion and DINO-vs-YOLO gap ingestion target perception regressions that generic log tools do not model.
- **Archival storage integration.** First-class Postgres/S3/BigQuery backends with Docker-backed integration suites; the comparables are file- or viewer-centric.
- **NVIDIA artifact adapters.** Alpamayo/AlpaSim report ingestion preserves reasoning traces and provenance without requiring a GPU or the NVIDIA runtime.
- **Self-contained offline CLI.** No GUI, no cloud account, machine-readable JSON output for agent and CI use.

### Disposition
- **Build:** MCAP import wired into `Mission.from_ros_bag()`; typed Python APIs for the remaining Phase 16–20 operations; actually publish the Linux/Windows wheel matrix; ship or delete the MCP module.
- **Integrate/reuse:** keep `.rrd` export and let Rerun own 3D/sensor review rather than writing a viewer; use the `mcap` crate already in Cargo.toml for import; interoperate with rosbag2 for record/live capture; use Foxglove for hosted team review.
- **Defer/reject:** live fleet dashboard (explicitly rejected, recorded-mission forensics is the identity); in-process DINO/SAM execution (defer to external open-vocabulary pipelines); SANA-WM synthetic replay and a web UI until there is user evidence.

### Sources
- https://github.com/rerun-io/rerun (accessed 2026-10-24) — dual Apache-2.0/MIT, last commit 2026-10-23, MCAP/LeRobot ingest and multimodal viewer.
- https://github.com/ros2/rosbag2 (accessed 2026-10-24) — Apache-2.0, rolling branch last commit 2026-10-22, ROS 2 record/playback.
- https://github.com/foxglove/mcap (accessed 2026-10-24) — MIT, last commit 2026-10-23, multi-language log format and CLI.
- Repo files inspected: `README.md`, `COMPETITIVE_INTELLIGENCE.md`, `AUDIT_FINDINGS.md`, `ROADMAP_HONEST.md`, `QUEUE_STATUS.md`, `docs/ALTERNATIVES.md`, `docs/ARCHITECTURE.md`, `Cargo.toml`, `pyproject.toml`.
