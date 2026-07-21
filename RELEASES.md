# PyRoboReplay Release Notes

## [v0.8.0](https://github.com/mullassery/pyroboreplay/releases/tag/v0.8.0) - Extended Observability (2026-07-22)

**Production-Ready Extended Observability Layer**

### 🎉 Major Features

#### Real-Time Fleet Monitoring (Phase 8.1)
- **Multi-robot health tracking** with 6 status types (Active, Idle, Degraded, Failed, Offline, Charging)
- **Dynamic health scoring** (1.0 clean → 0.0 degraded) with trend detection (Improving/Stable/Degrading)
- **Automatic offline detection** based on configurable timeout thresholds
- **Alert aggregation** by severity (Critical/High/Medium/Info)
- **Historical dashboard** with time-windowed snapshots for trend analysis
- **Per-robot diagnostics** integration for detailed failure tracking

**Use Case:** Monitor 10+ robot fleet, spot degradation patterns, prevent failures before they cascade

#### Cross-Mission Learning (Phase 8.2)
- **Pattern extraction** from mission root cause analyses
- **Failure prediction** using learned patterns with confidence scoring
- **Anomaly detection** across mission histories
- **SQLite persistence** for pattern library storage and retrieval
- **Automatic improvement recommendations** based on past failures

**Use Case:** Learn that "deadlock + high CPU → navigation failure", predict it in next mission

#### SLA Enforcement (Phase 8.3)
- **Service Level Agreements** with customizable thresholds per mission type
- **Automated compliance scoring** (0.0-1.0) with penalty system
- **Violation tracking** for navigation deadlock, sensor dropout, coverage, emergency stops, speed
- **Severity tiers** (Medium/High/Critical) with graduated penalties
- **Audit trails** for compliance reporting and regulatory requirements

**Use Case:** Enforce "Max 60s deadlock, max 30s sensor dropout, min 80% coverage", get compliance report

#### EventStream Bug Fix
- Fixed critical subscription channel bug where subscribers never received events
- Now properly maintains multiple concurrent subscribers with Arc<Mutex> pattern
- All subscribers receive events as published

### 📊 Quality & Testing

- **267 passing tests** (up from 221 in v0.7)
  - 13 fleet monitoring tests
  - 13 cross-mission learning tests
  - 15 SLA enforcement tests
  - Plus 2 EventStream subscription tests
- **Zero unsafe code** across all new modules
- **Thread-safe design** using Arc<Mutex> patterns
- **Comprehensive error handling** with thiserror

### 🔧 Technical Details

#### Phase 8.1: Fleet Monitoring (~350 LOC)
- `FleetMonitor` — Active mission tracking with health scoring
- `FleetDashboard` — Historical window with trend analysis
- `RobotStatus` — Per-robot state (last_seen, status, alert_count)
- `FleetHealthSummary` — Fleet-wide aggregated metrics

#### Phase 8.2: Cross-Mission Learning (~450 LOC)
- `PatternLibrary` — In-memory pattern storage with type/frequency queries
- `CrossMissionAnalyzer` — Learns from RootCauseAnalysis, predicts failures
- `MissionPattern` — Captures pattern_type, occurrences, frequency, confidence
- SQLite patterns table for persistence

#### Phase 8.3: SLA Enforcement (~380 LOC)
- `SlaMonitor` — Active mission tracking with threshold enforcement
- `SlaContract` — Customizable limits per mission type
- `SlaEnforcementReport` — Compliance snapshot with score + violations
- 4-tier severity system (Medium/High/Critical) with graduated penalties

### 📦 Package Updates

- **PyPI:** Now on PyPI as `pyroboreplay==0.8.0`
- **Python Support:** 3.10, 3.11, 3.12, 3.13
- **Wheels:** Published for macOS (arm64), Linux (x86_64, aarch64)
- **Installation:** `pip install pyroboreplay==0.8.0`

### 📚 Documentation

- **README.md** — Completely rewritten with v0.8 features, use cases, architecture
- **CONTRIBUTING.md** — First-time contributor guide with setup, workflow, commit style
- **CODE_OF_CONDUCT.md** — Contributor Covenant v2.0 community standards
- **GITHUB_SETTINGS.md** — Repository configuration guide for discoverability
- **9 Working Examples** — Fleet monitoring, cross-mission learning, SLA enforcement, compliance, failover

### 🚀 Breaking Changes

None. v0.8 is fully backward compatible with v0.7.

### 📋 Upgrade Path

```bash
# From v0.7.0
pip install --upgrade pyroboreplay==0.8.0

# Or fresh install
pip install pyroboreplay==0.8.0
```

All existing APIs remain unchanged. New features are purely additive.

---

## [v0.7.0](https://github.com/mullassery/pyroboreplay/releases/tag/v0.7.0) - Advanced Forensics (2026-07-20)

**Production-Ready Forensic Grade Replay & Failover**

### Major Features

#### Bit-Perfect Deterministic Replay (Phase 7.1)
- SHA-256 event hashing with canonical JSON serialization
- Tamper-proof audit trails with chain integrity verification
- Replay manifests capturing event identity and integrity proofs
- Forensic-grade reproducibility for regulatory compliance

#### Mission-Critical Failover & Redundancy (Phase 7.2)
- Primary + standby backend redundancy with automatic promotion
- Write-ahead logging ensures zero data loss during failover
- Heartbeat-based health checking with automatic backend promotion
- Complete failover audit trail with timestamps and decision history

#### ISO 3691-4 Regulatory Compliance (Phase 7.3)
- Proximity zone violation detection (safety, warning, protective zones)
- Emergency stop monitoring and recovery time tracking
- Speed compliance checking with configurable thresholds
- Operator presence verification during motion
- Compliance reporting with violation aggregation

### Quality

- **221 passing tests** (up from 171 in v0.6)
- Implements all Phase 7 features as specified
- Thread-safe design with Arc<Mutex> patterns
- Zero data loss guarantees during failover

### Package

- **PyPI:** Published as `pyroboreplay==0.7.0`
- **Wheels:** macOS (arm64), Linux (x86_64, aarch64)
- **Python Support:** 3.10, 3.11, 3.12, 3.13

---

## [v0.6.0](https://github.com/mullassery/pyroboreplay/releases/tag/v0.6.0) - Production Scale (2026-07-15)

Production-ready storage backends, streaming, and diagnostics.

### Features

- Pluggable storage backends (PostgreSQL, BigQuery, S3, SQLite, In-Memory)
- Real-time event streaming with live diagnostics
- Live alert detection (navigation deadlock, sensor dropout, obstacle storms)
- Data tiering and query federation

---

## Earlier Versions

See [commit history](https://github.com/mullassery/pyroboreplay/commits/main) for details on v0.1-v0.5.

---

## Changelog Format

We follow [Keep a Changelog](https://keepachangelog.com/) format:
- **Added** for new features
- **Changed** for changes in existing functionality
- **Deprecated** for soon-to-be removed features
- **Removed** for now removed features
- **Fixed** for any bug fixes
- **Security** for any security fixes

---

## Version Numbering

We follow [Semantic Versioning](https://semver.org/):
- **MAJOR** version (0.X.0) — Breaking changes, new major features
- **MINOR** version (0.7.X) — New features (backward compatible)
- **PATCH** version (0.7.1) — Bug fixes (backward compatible)

Pre-release versions use: `0.8.0-alpha`, `0.8.0-beta`

---

## Contributing

Found a bug? Want a feature? See [CONTRIBUTING.md](CONTRIBUTING.md) for how to help!

---

## Support

- **Documentation:** https://github.com/mullassery/pyroboreplay
- **Issues:** https://github.com/mullassery/pyroboreplay/issues
- **Discussions:** https://github.com/mullassery/pyroboreplay/discussions
- **Email:** mullassery@gmail.com
