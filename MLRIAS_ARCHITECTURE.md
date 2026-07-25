# Multi-Layer Robotics Incident Analysis System (MLRIAS)

## Executive Summary

The **Multi-Layer Robotics Incident Analysis System (MLRIAS)** is an extension of PyRoboReplay that ingests evidence from 4 hierarchical layers (ROS bags → Linux/kernel → resource metrics → configurations), correlates them on a unified timeline, detects failures across 5 failure domains, and generates confidence-backed recommendations. 

It enables forensic analysis of complex robot failures by providing a single source of truth for **what happened**, **why it happened**, **which subsystem was responsible**, and **how to fix it**.

---

## Core Philosophy

The objective is not merely to answer:
- **What happened?** (Which node failed, which error was logged)

Instead, the objective is to answer:
- **Why did it happen?** (Root cause across all layers)
- **What sequence of events led to the failure?** (Causal chain)
- **Which subsystem was the actual source?** (Cross-layer diagnosis)
- **What evidence supports the conclusion?** (Confidence-backed findings)
- **How can the issue be fixed?** (Actionable recommendations)
- **Can the issue be reproduced?** (Deterministic incident package)
- **How confident is the diagnosis?** (Confidence scoring model)

---

## 1. FOUR-LAYER EVIDENCE MODEL

### Layer Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│ Layer 4: Configuration Layer (Static)                          │
│ Nav2 YAML, SLAM parameters, launch files, tuning              │
│ ↑ Provides baseline/expected behavior context
└─────────────────────────────────────────────────────────────────┘
    ↑
┌─────────────────────────────────────────────────────────────────┐
│ Layer 3: Resource Metrics (Streaming)                          │
│ CPU/RAM/disk/temp, network I/O, DDS QoS metrics              │
│ ↑ Bridge between system health and robotics decisions
└─────────────────────────────────────────────────────────────────┘
    ↑
┌─────────────────────────────────────────────────────────────────┐
│ Layer 2: Linux/Kernel Layer (System Events)                    │
│ journalctl, dmesg, syslog, OOM kills, kernel panics           │
│ ↑ Explains system-level constraints on robot behavior
└─────────────────────────────────────────────────────────────────┘
    ↑
┌─────────────────────────────────────────────────────────────────┐
│ Layer 1: ROS Bags + Logs (Minimum Required)                    │
│ Topic messages, TF frames, per-node logs                       │
│ ↑ Robot-level decision making & sensor streams
└─────────────────────────────────────────────────────────────────┘
```

### Analysis Levels by Available Evidence

| Level | Available Evidence | Capabilities |
|-------|-------------------|--------------|
| **1** | ROS bags + logs | Topic analysis, TF inspection, message frequency, navigation failure detection, sensor timeout |
| **2** | + Linux/kernel logs | Hardware failures, driver failures, USB disconnects, kernel faults, filesystem issues, process crashes |
| **3** | + Resource metrics | Resource bottleneck analysis, thermal throttling, CPU starvation, memory pressure, network instability |
| **4** | + Configurations | Configuration validation, parameter anomaly detection, anti-pattern detection, recommendation generation |

---

## 2. INCIDENT BUNDLE FORMAT

### Standard ZIP Structure

```
incident_2024-07-25_robot1_nav_failure.zip
├── metadata.json                 # Bundle manifest
├── layer1/
│   ├── robot1.bag               # ROS 2 bag file
│   ├── robot1.log               # Node-level logs
│   └── tf_frames.log            # TF transforms
├── layer2/
│   ├── journalctl.log           # Linux system journal
│   ├── dmesg.log                # Kernel messages
│   ├── syslog.log               # System-wide logs
│   └── kernel_warnings.log      # Kernel panic logs, OOM kills
├── layer3/
│   ├── cpu_memory.csv           # Time, CPU%, RAM%, disk%, temp
│   ├── network_io.csv           # Eth0 RX/TX packets/bytes
│   ├── dds_metrics.json         # DDS discovery events, QoS violations
│   └── usb_events.log           # USB device attach/detach
├── layer4/
│   ├── nav2_params.yaml         # Navigation2 configuration
│   ├── slam_params.yaml         # SLAM algorithm parameters
│   ├── launch_files/            # All launch files used
│   └── hardware_config.yaml     # Robot kinematics, wheel radius
├── analysis_output.json         # Generated analysis
└── README.md                    # Human-readable summary
```

### Metadata.json Schema

Auto-discovered and auto-generated:

```json
{
  "bundle_id": "incident_2024-07-25_robot1_nav",
  "created_at": "2024-07-25T14:32:10Z",
  "robot_id": "robot1",
  "mission_type": "warehouse_navigation",
  "failure_type_suspected": "navigation_deadlock",
  "time_range": {
    "start": "2024-07-25T14:20:00Z",
    "end": "2024-07-25T14:35:00Z",
    "duration_seconds": 900
  },
  "layers_available": {
    "layer1_ros_bags": true,
    "layer2_linux_logs": true,
    "layer3_metrics": true,
    "layer4_configs": true
  },
  "detected_issues": [
    "planner_timeout_detected",
    "cpu_spike_observed",
    "dds_latency_spike",
    "usb_device_disconnection"
  ]
}
```

---

## 3. FAILURE DETECTION DOMAINS

The system detects and classifies failures across 5 domains:

### Navigation Failures
- **Planner timeout**: Path computation exceeds time limit
- **Controller oscillation**: Robot cycles through same states
- **Recovery loop**: Recovery behaviors triggered repeatedly
- **Goal failure**: Unable to reach target within constraints
- **Path deviation**: Actual path deviates significantly from planned

### Localization Failures
- **AMCL divergence**: Pose covariance grows unbounded
- **Map mismatch**: Scan inconsistent with loaded map
- **TF inconsistencies**: Transforms don't compose correctly
- **Pose instability**: Estimated pose jumps discontinuously
- **GPS dropout**: Absolute positioning unavailable

### Perception Failures
- **Sensor dropout**: Absence of frames for extended period
- **Camera frame loss**: Missing frames in video stream
- **LiDAR interruption**: Point cloud publishing stops
- **Synchronization issues**: Sensor data misaligned in time
- **Low confidence**: Detection confidence below threshold

### Middleware (DDS/ROS) Failures
- **DDS discovery timeout**: Nodes unable to find each other
- **QoS mismatches**: Incompatible pub/sub QoS settings
- **Topic starvation**: Subscription receiving no messages
- **Message latency spikes**: Unusual delays in communication
- **DDS buffer overflow**: Message queue exceeded capacity

### Linux/System Failures
- **OOM kills**: Out-of-memory process termination
- **Kernel panics**: Unrecoverable kernel errors
- **Driver failures**: Device driver crashes
- **USB resets**: USB device disconnection/reconnection
- **Filesystem errors**: Disk I/O failures
- **Network disconnects**: Link down/up events
- **Process crashes**: ROS node process death

---

## 4. TIMELINE CORRELATION ENGINE

### Core Responsibility

The Timeline Correlation Engine is the **heart** of MLRIAS. It:

1. **Normalizes timestamps** across all 4 layers
2. **Handles clock skew** between robot and host system
3. **Aligns multi-robot events** to shared time reference
4. **Constructs unified event timeline** with causal links

### Clock Synchronization Algorithm

**Input**: Events from Layer 1 (ROS timestamps), Layer 2 (host syslog timestamps)

**Approach**: PTP-like clock correction using reference events

```
1. Find anchor events: ROS clock_sync messages, system time messages
2. Compute offset: (host_timestamp - ros_timestamp)
3. Estimate skew: slope of (offset vs. time)
4. Apply inverse correction to all Layer 2+ events
5. Validate: Check that causal order is preserved
```

### Multi-Robot Time Alignment

For fleets, synchronize all robots to a **reference clock**:
- Choose reference robot (e.g., highest-quality clock source)
- Compute offsets using communication events
- Apply offsets to all non-reference robots

### Causal Chain Reconstruction

After timeline is normalized:
- Find temporal proximity between event pairs (default 2s window)
- Check semantic correlation (does event A plausibly cause event B?)
- Build causal graph with confidence scores
- Support cross-layer links (e.g., OOM kill → ROS node timeout)

---

## 5. CONFIDENCE SCORING MODEL

### Confidence Tiers

| Confidence | Description | Examples |
|------------|-------------|----------|
| **100%** | Facts | Explicitly logged events, sensor data, kernel entries |
| **80-90%** | High-Confidence Inferences | Temporal correlations, sensor dropout, explicit DDS failures |
| **60-80%** | Medium-Confidence Inferences | Pattern detection, cross-layer correlation, statistical anomalies |
| **40-60%** | Hypotheses | Counterfactual reasoning, speculative causal links |
| **0-40%** | Speculation | Not recommended for production diagnostics |

### Evidence Weighting

Each diagnosis is confidence-scored by aggregating supporting evidence:

```
Diagnosis Confidence = Σ(symptom_confidence × symptom_weight) / Σ(symptom_weights)
```

Example: **Navigation Planner Timeout**
- "plan_request_received" → weight 0.9
- "plan_timeout_logged" → weight 1.0 (direct evidence)
- "no_path_found" → weight 0.8
- "obstacle_nearby" → weight 0.3 (correlation, not causation)

---

## 6. RECOMMENDATIONS ENGINE

### Recommendation Structure

Each recommendation includes:

1. **Title**: Concise description ("Increase planner timeout")
2. **Description**: Detailed explanation with context
3. **Priority**: critical / high / medium / low
4. **Expected Impact**: 0.0-1.0 (how much does this fix help?)
5. **Implementation Effort**: 0.0-1.0 (how hard is this to implement?)
6. **ROI Score**: impact / effort (return on investment)
7. **Confidence**: 0.0-1.0 (confidence that this fixes the issue)
8. **Evidence Chain**: What evidence supports this recommendation?

### Example Recommendations

**For Navigation Planner Timeout**:
1. "Increase planner timeout from 5s to 7.5s"
   - Impact: 85% | Effort: 10% | ROI: 8.5 | Confidence: 90%
2. "Use faster planner algorithm (SmacPlannerLattice)"
   - Impact: 75% | Effort: 40% | ROI: 1.9 | Confidence: 65%

**For OOM Kill**:
1. "Reduce navigation stack memory footprint"
   - Impact: 95% | Effort: 30% | ROI: 3.2 | Confidence: 85%
2. "Increase available swap space"
   - Impact: 60% | Effort: 20% | ROI: 3.0 | Confidence: 70%

---

## 7. END-TO-END WORKFLOW EXAMPLE

### Scenario: Warehouse robot navigates, then crashes

**Timeline**:
- T=02:15: Navigation planner timeout (Layer 1)
- T=02:15: CPU spike to 95% (Layer 3)
- T=02:16: OOM kill detected in journalctl (Layer 2)
- T=02:17: Robot stops (Layer 1, TF frames freeze)

### Analysis Steps

```
1. Load incident bundle (incident_2024-07-25_robot1.zip)
   ↓
2. Auto-discover evidence (Layers 1-4)
   ↓
3. Load all adapters (ROS2, Linux, Metrics, Config)
   ↓
4. Ingest all evidence (50K + 20K + 15K + 5K = 90K events)
   ↓
5. Synchronize clocks (align Layer 2/3 with Layer 1 timestamps)
   ↓
6. Build causal chains (identify temporal dependencies)
   ↓
7. Detect failures (planner_timeout, cpu_saturation, oom_kill)
   ↓
8. Root cause analysis (CPU overload → memory growth → OOM)
   ↓
9. Generate recommendations (increase timeout, reduce memory footprint)
   ↓
10. Generate JSON report (with evidence chains and confidence)
```

### Example Output

```json
{
  "bundle_id": "incident_2024-07-25_robot1_nav",
  "analysis_timestamp": "2024-07-25T15:00:00Z",
  "detected_failures": [
    {
      "type": "planner_timeout",
      "confidence": 1.0,
      "timestamp": "2024-07-25T14:22:15Z",
      "description": "Planner timeout after 5.2s (threshold: 5.0s)"
    },
    {
      "type": "cpu_saturation",
      "confidence": 0.90,
      "timestamp": "2024-07-25T14:22:14Z",
      "description": "CPU >95% for 8.3s (sustained overload)"
    },
    {
      "type": "oom_kill",
      "confidence": 1.0,
      "timestamp": "2024-07-25T14:22:16Z",
      "description": "Out-of-memory kill: process nav_stack (pid 2341)"
    }
  ],
  "root_cause_analysis": {
    "primary_hypothesis": "CPU overload caused planner timeout, leading to OOM kill",
    "confidence": 0.925,
    "causal_chain": [
      {"event": "cpu_saturation (95%)", "time": "2024-07-25T14:22:14Z"},
      {"event": "node_memory_growth (+500MB)", "time": "2024-07-25T14:22:14.5Z"},
      {"event": "planner_timeout (5.2s)", "time": "2024-07-25T14:22:15Z"},
      {"event": "oom_kill (nav_stack)", "time": "2024-07-25T14:22:16Z"}
    ]
  },
  "recommended_actions": [
    {
      "title": "Increase planner timeout",
      "priority": "high",
      "impact": 0.85,
      "effort": 0.10,
      "roi": 8.5,
      "confidence": 0.90,
      "evidence": "Planner timeout detected with high confidence (1.0)"
    },
    {
      "title": "Reduce navigation stack memory footprint",
      "priority": "high",
      "impact": 0.95,
      "effort": 0.30,
      "roi": 3.2,
      "confidence": 0.85,
      "evidence": "OOM kill preceded by CPU saturation and memory growth"
    }
  ]
}
```

---

## 8. IMPLEMENTATION ROADMAP (24 WEEKS)

### Phase 1: Core Infrastructure (Weeks 1-4)

**Deliverables**:
- Evidence Discovery module (auto-detect Layers 1-4)
- Incident Bundle ZIP loader
- Unified MissionEvent extensions for Layer 2/3/4 events

**Files to Create**:
- `src/core/incident_bundle.rs`
- `src/adapters/evidence_discovery.rs`
- `src/core/event.rs` (extend existing)

### Phase 2: Layer Adapters (Weeks 5-8)

**Deliverables**:
- Linux/kernel log adapter (Layer 2)
- Resource metrics adapter (Layer 3)
- Configuration adapter (Layer 4)

**Files to Create**:
- `src/adapters/linux_log.rs`
- `src/adapters/metrics.rs`
- `src/adapters/configuration.rs`

### Phase 3: Timeline Correlation (Weeks 9-12)

**Deliverables**:
- Clock synchronization engine
- Multi-robot time alignment
- Causal chain reconstruction

**Files to Create**:
- `src/core/timeline_correlation.rs`
- `src/core/multi_robot_aligner.rs`
- `src/core/causality.rs` (extend existing)

### Phase 4: Failure Detection (Weeks 13-16)

**Deliverables**:
- Navigation, Localization, Perception, Middleware, System failure detectors
- Detection engines with evidence collection

**Files to Create**:
- `src/core/failure_detection/mod.rs`
- `src/core/failure_detection/navigation.rs`
- `src/core/failure_detection/localization.rs`
- `src/core/failure_detection/perception.rs`
- `src/core/failure_detection/middleware.rs`
- `src/core/failure_detection/system.rs`

### Phase 5: Confidence Scoring (Weeks 17-18)

**Deliverables**:
- Confidence scoring model
- Diagnosis confidence aggregation
- Evidence-backed confidence chains

**Files to Create**:
- `src/core/confidence_scoring.rs`

### Phase 6: Recommendations (Weeks 19-20)

**Deliverables**:
- Recommendation generation for each failure type
- ROI scoring (impact/effort)
- Evidence-backed explanations

**Files to Modify**:
- `src/core/recommendation.rs` (extend)

### Phase 7: Python API + CLI (Weeks 21-22)

**Deliverables**:
- Python bindings for incident analysis
- CLI commands: `pyroboreplay analyze-incident <bundle.zip>`
- JSON output for agent integration

**Files to Modify**:
- `src/lib.rs` (add PyO3 bindings)
- `src/cli/args.rs` (add commands)

**Files to Create**:
- `src/python/incident_analysis.py`

### Phase 8: Testing & Documentation (Weeks 23-24)

**Deliverables**:
- 50+ comprehensive test cases
- Integration tests for bundle ingestion
- Example incident analysis workflow

**Files to Create**:
- `tests/test_incident_bundle.rs`
- `tests/test_timeline_correlation.rs`
- `tests/test_failure_detection_*.rs`
- `tests/test_confidence_scoring.rs`
- Examples

---

## 9. SCALABILITY CONSIDERATIONS

### Storage & Memory

**Problem**: Large incidents can have millions of events across 4 layers.

**Solution - Tiered Storage**:
- In-memory: for incidents <1M events
- SQLite: for incidents <10M events
- PostgreSQL: for warehouse-scale analysis
- Parquet on S3: for long-term archival

### Indexing Strategy

Multi-level indices for O(log N) temporal queries:
- Block-wise index (1s blocks)
- Type index (events grouped by type)
- Robot index (events per robot)
- Composite index (robot + type + time_block)

### Parallel Processing

Use Rayon for parallel failure detection:
- Each failure domain (navigation, localization, perception, middleware, system) runs in parallel
- Significant speedup for large incidents (10M+ events)

### Streaming Ingestion

Support event stream API for real-time analysis:
- Continuously ingest events from running robots
- Buffer and batch for efficient processing
- Continuous analysis of active missions

---

## 10. CRITICAL ARCHITECTURAL DECISIONS

1. **Event-Centric Design**: All evidence (Layers 1-4) normalized to `MissionEvent` enum ensures unified processing pipeline

2. **Pluggable Adapters**: New layers add adapters, not core logic changes

3. **Confidence as First-Class**: Every diagnosis tagged with confidence tier; separates facts from inferences

4. **Storage-Agnostic**: Correlation engine operates on in-memory event streams; storage backend is pluggable for scalability

5. **Rust Core + Python Bindings**: Performance-critical paths (correlation, detection) in Rust; Python API for ease of use

6. **Forensic-Ready**: All analysis immutable and auditable; supports deterministic replay of incident diagnosis

7. **Semantic Correlation**: Causal links based on robotics domain knowledge, not just temporal proximity

8. **Cross-Layer Analysis**: Failures rarely have single-layer causes; system designed to find multi-layer root causes

---

## Next Steps

1. **Review this architecture** for gaps or misalignments
2. **Start Phase 1** (Evidence Discovery + Bundle Loader)
3. **Build reference incidents** for testing (warehouse navigation, SLAM failure, etc.)
4. **Integrate with existing PyRoboReplay replay infrastructure** (use existing MissionEvent types)
5. **Release v1.0 with Level 1-2 analysis** (ROS bags + Linux logs)
6. **Expand to Level 3-4** in subsequent releases

---

Generated: 2026-07-25
Author: Claude Code (Multi-Agent Planning)
