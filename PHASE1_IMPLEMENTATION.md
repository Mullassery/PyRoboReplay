# Phase 1: Failure Detection & Diagnosis APIs (Weeks 1-6)

**Status**: Week 1 Complete ✅  
**Goal**: Expose failure detection and root cause analysis as public Python APIs  
**Scope**: 8 detectable failure types + diagnosis engine + 30+ unit tests

## Progress

- [x] **Week 1 (Complete)**: Task 1.1 & 1.2 - Failure Detection API
  - ✅ AnomalyDetector with 8 failure types
  - ✅ Python `mission.detect_failures()` API
  - ✅ Failure data structure (Python-friendly)
  - ✅ Evidence collection for each failure
  - ✅ Code builds successfully

- **Week 2-3 (Next)**: Task 1.3 & 1.4 - Root Cause Analysis

---

## Architecture Overview

```
Mission (Python) 
  ↓
├─ detect_failures() → List[Failure]
│  ├─ LidarAnalyzer (min_range check)
│  ├─ LocalizationAnalyzer (covariance spikes)
│  ├─ NavigationAnalyzer (deadlock detection)
│  ├─ PerceptionAnalyzer (low confidence)
│  ├─ CommunicationAnalyzer (dropouts)
│  ├─ SensorAnalyzer (message rate)
│  ├─ TrajectoryAnalyzer (oscillation)
│  └─ CostmapAnalyzer (sudden changes)
│
└─ analyze_failure(timestamp) → RootCauseAnalysis
   ├─ Build causal graph
   ├─ Generate hypotheses
   ├─ Rank by confidence
   └─ Explain causality
```

---

## Phase 1 Tasks (6 Weeks)

### Week 1-2: Python API Exposure

**Task 1.1: Expose Failure Detection to Python**

File: `src/lib.rs`

```python
# Python API
mission = Mission.from_ros_bag("mission.bag")
failures = mission.detect_failures()  # → List[Failure]

# Each Failure contains:
# - failure_type: str (e.g., "near_collision")
# - timestamp: float
# - confidence: float (0.0-1.0)
# - severity: str ("critical", "high", "medium", "low")
# - description: str
# - affected_systems: List[str]
```

**Files to create/modify:**
- `src/core/anomaly_detector.rs` (NEW) - Unified anomaly detection engine
- `src/lib.rs` - Expose `detect_failures()` to Python
- `src/core/mod.rs` - Export new modules

**Acceptance Criteria:**
- [ ] `mission.detect_failures()` works
- [ ] Returns list of 8 failure types
- [ ] Each failure has timestamp, confidence, severity, description
- [ ] Unit tests: 15+ scenarios

---

**Task 1.2: Create Failure Data Structure**

File: `src/core/anomaly_detector.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Failure {
    pub id: String,
    pub failure_type: String,  // "near_collision", "localization_loss", etc.
    pub timestamp: f64,
    pub confidence: f32,  // 0.0-1.0
    pub severity: String,  // "critical", "high", "medium", "low"
    pub description: String,
    pub affected_systems: Vec<String>,  // ["lidar", "planner", "odometry"]
    pub evidence: HashMap<String, String>,  // Key evidence that triggered detection
}

pub struct AnomalyDetector {
    events: Vec<MissionEvent>,
    lidar_threshold: f32,
    covariance_threshold: f32,
    // ... other thresholds
}

impl AnomalyDetector {
    pub fn new(events: Vec<MissionEvent>) -> Self { ... }
    
    pub fn detect_all(&self) -> Vec<Failure> {
        let mut failures = Vec::new();
        failures.extend(self.detect_near_collision());
        failures.extend(self.detect_localization_loss());
        failures.extend(self.detect_navigation_deadlock());
        failures.extend(self.detect_perception_failure());
        failures.extend(self.detect_communication_loss());
        failures.extend(self.detect_sensor_dropout());
        failures.extend(self.detect_oscillation());
        failures.extend(self.detect_costmap_anomaly());
        failures
    }
    
    fn detect_near_collision(&self) -> Vec<Failure> { ... }
    fn detect_localization_loss(&self) -> Vec<Failure> { ... }
    // ... 6 more detectors
}
```

**Acceptance Criteria:**
- [ ] Failure struct serializable to JSON
- [ ] All 8 detectors implemented
- [ ] Each detector has configurable thresholds
- [ ] Detectors handle edge cases (empty event streams, etc.)

---

### Week 2-3: Root Cause Analysis Integration

**Task 1.3: Expose Root Cause Analysis to Python**

File: `src/lib.rs`

```python
# Python API
mission = Mission.from_ros_bag("mission.bag")
failure = mission.detect_failures()[0]  # Get first failure

# Analyze root cause
analysis = mission.analyze_failure(failure.timestamp)

# Each analysis contains:
# - primary_hypothesis: str (most likely cause)
# - hypotheses: List[Hypothesis] (ranked by confidence)
# - confidence: float (overall diagnostic confidence)
# - evidence: Dict[str, Any] (supporting data)
# - recommendation: str (suggested action)
```

**Files to modify:**
- `src/core/root_cause.rs` - Already exists, may need polish
- `src/lib.rs` - Add `analyze_failure()` method to Mission
- Create Python wrapper for RootCauseAnalysis

**Acceptance Criteria:**
- [ ] `mission.analyze_failure(timestamp)` works
- [ ] Returns RootCauseAnalysis with hypotheses
- [ ] Hypotheses ranked by confidence
- [ ] Evidence provided for each hypothesis
- [ ] Unit tests: 20+ scenarios

---

**Task 1.4: Create Analysis Data Structures**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub description: String,
    pub confidence: f32,  // 0.0-1.0
    pub evidence: Vec<Evidence>,
    pub counter_evidence: Vec<Evidence>,
    pub causal_chain: Vec<(String, f64)>,  // (event_type, timestamp)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub event_type: String,
    pub timestamp: f64,
    pub value: String,
    pub importance: f32,  // How critical to this hypothesis
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysisResult {
    pub failure_timestamp: f64,
    pub primary_hypothesis: String,
    pub hypotheses: Vec<Hypothesis>,
    pub diagnostic_confidence: f32,
    pub recommendation: String,
}
```

**Acceptance Criteria:**
- [ ] Structures serializable to JSON
- [ ] Evidence clearly linked to hypotheses
- [ ] Counter-evidence considered (not just supporting)
- [ ] Recommendations actionable

---

### Week 3-4: Failure Explanation API

**Task 1.5: Add Human-Readable Explanations**

File: `src/core/explanation.rs` (NEW)

```python
# Python API
failure = mission.detect_failures()[0]
explanation = mission.explain_failure(failure.timestamp)
print(explanation)
# Output: "At t=234.5, LiDAR detected obstacle at 2.5m (←→█░ intensity), 
#          causing planner to trigger collision avoidance. Velocity reduced 
#          from 1.0 m/s to 0.0 m/s. This was correct conservative behavior."
```

**Implementation:**
- Create explanation templates for each failure type
- Include sensor readings, thresholds, decisions in natural language
- Reference specific events with timestamps
- Explain why this was/wasn't handled correctly

**Acceptance Criteria:**
- [ ] Explanations for all 8 failure types
- [ ] 2-3 sentences per explanation
- [ ] Include specific measurements/thresholds
- [ ] Reference sensor data directly

---

### Week 4-5: Action Recommendations

**Task 1.6: Add Recommended Actions**

File: `src/core/recommendation.rs` (extends existing)

```python
# Python API
failure = mission.detect_failures()[0]
actions = mission.recommend_actions(failure)

for action in actions:
    print(f"{action.priority}: {action.description}")
    print(f"  Expected impact: {action.impact}")
    print(f"  Implementation: {action.implementation}")

# Output:
# P0: Improve LiDAR range detection threshold
#   Expected impact: Reduce false positives by ~30%
#   Implementation: Reduce threshold from 2.5m to 2.0m in config
#
# P1: Add sensor fusion with camera confidence
#   Expected impact: Better discrimination of false obstacles
#   Implementation: Cross-reference LiDAR + camera before triggering avoidance
```

**Implementation:**
- Link failure types to concrete mitigations
- Include estimated impact (high/medium/low)
- Implementation complexity (easy/medium/hard)
- Priority (P0/P1/P2)

**Files:**
- `src/core/recommendation.rs`

**Acceptance Criteria:**
- [ ] At least 2 actions per failure type
- [ ] Actions include priority, impact, complexity
- [ ] Actions are implementable (not vague)

---

### Week 5-6: Testing & Documentation

**Task 1.7: Comprehensive Unit Tests**

```rust
#[cfg(test)]
mod tests {
    // Failure detection tests (8 scenarios each)
    #[test]
    fn test_detect_near_collision() { ... }
    #[test]
    fn test_detect_localization_loss() { ... }
    #[test]
    fn test_detect_navigation_deadlock() { ... }
    // ... 5 more
    
    // Root cause analysis tests (4 scenarios each)
    #[test]
    fn test_analyze_collision_failure() { ... }
    #[test]
    fn test_analyze_localization_failure() { ... }
    // ... 2 more
    
    // Edge cases
    #[test]
    fn test_empty_mission() { ... }
    #[test]
    fn test_single_event_mission() { ... }
    #[test]
    fn test_malformed_events() { ... }
    
    // Python API tests
    #[test]
    fn test_python_detect_failures() { ... }
    #[test]
    fn test_python_analyze_failure() { ... }
}
```

**Target**: 30+ passing tests, 85%+ coverage

**Task 1.8: Python Examples & Documentation**

Create: `examples/failure_detection_demo.py`

```python
from pyroboreplay import Mission

# Example 1: Quick failure detection
mission = Mission.from_ros_bag("warehouse_incident.bag")
failures = mission.detect_failures()

print(f"Found {len(failures)} issues:")
for failure in failures:
    print(f"\n{failure.failure_type} @ t={failure.timestamp:.2f}")
    print(f"  Confidence: {failure.confidence:.0%}")
    print(f"  Severity: {failure.severity}")
    print(f"  {failure.description}")

# Example 2: Root cause analysis
first_failure = failures[0]
analysis = mission.analyze_failure(first_failure.timestamp)

print(f"\nAnalyzing {first_failure.failure_type}...")
print(f"Primary hypothesis: {analysis.primary_hypothesis}")
print(f"Confidence: {analysis.diagnostic_confidence:.0%}")
print(f"\nRecommended action: {analysis.recommendation}")

# Example 3: Batch analysis of fleet
from pathlib import Path
from pyroboreplay import Mission

mission_files = Path("warehouse_bags/").glob("*.bag")
for mission_file in mission_files:
    mission = Mission.from_ros_bag(str(mission_file))
    failures = mission.detect_failures()
    print(f"{mission_file.name}: {len(failures)} issues detected")
```

**Acceptance Criteria:**
- [ ] Example runs without errors
- [ ] Documentation clear and complete
- [ ] Covers all main APIs

---

## API Summary

**Public Python APIs (End of Phase 1):**

```python
# Failure Detection
mission.detect_failures() → List[Failure]

# Root Cause Analysis
mission.analyze_failure(timestamp) → RootCauseAnalysis

# Human-Readable Explanations
mission.explain_failure(timestamp) → str

# Recommended Actions
mission.recommend_actions(failure) → List[Action]

# Batch Analysis
mission.get_all_failures_with_analysis() → List[(Failure, Analysis)]
```

**Data Structures:**

```python
class Failure:
    failure_type: str
    timestamp: float
    confidence: float
    severity: str
    description: str
    affected_systems: List[str]

class RootCauseAnalysis:
    primary_hypothesis: str
    hypotheses: List[Hypothesis]
    diagnostic_confidence: float
    recommendation: str
    evidence: Dict[str, Any]

class Hypothesis:
    description: str
    confidence: float
    evidence: List[Evidence]
    counter_evidence: List[Evidence]
    causal_chain: List[Tuple[str, float]]

class Action:
    priority: str  # "P0", "P1", "P2"
    description: str
    impact: str    # "high", "medium", "low"
    implementation: str
    complexity: str  # "easy", "medium", "hard"
```

---

## Success Criteria

- ✅ 8 failure types detected correctly
- ✅ Root cause analysis working with >80% accuracy on known failures
- ✅ Explanations generated for all failure types
- ✅ At least 2 recommendations per failure type
- ✅ 30+ unit tests passing
- ✅ Python API fully functional
- ✅ Examples runnable end-to-end
- ✅ Documentation complete

---

## File Structure

```
src/
├── core/
│   ├── anomaly_detector.rs (NEW) - All 8 failure detectors
│   ├── explanation.rs (NEW) - Human-readable explanations
│   ├── root_cause.rs (MODIFY) - Expose to Python
│   ├── recommendation.rs (MODIFY) - Add action recommendations
│   └── mod.rs (MODIFY) - Export new modules
├── lib.rs (MODIFY) - Add Python methods
└── ...

examples/
└── failure_detection_demo.py (NEW)

tests/
├── test_anomaly_detection.rs (NEW)
├── test_root_cause_analysis.rs (NEW)
└── test_python_api.rs (NEW)
```

---

## Timeline

| Week | Tasks | Deliverable |
|------|-------|-------------|
| 1-2 | 1.1, 1.2 | Failure detection API |
| 2-3 | 1.3, 1.4 | Root cause analysis |
| 3-4 | 1.5 | Explanations |
| 4-5 | 1.6 | Recommendations |
| 5-6 | 1.7, 1.8 | Tests + docs |

---

**Next Steps**: Start Week 1 with Task 1.1 - Expose failure detection to Python
