# Phase 16: Causal Graph Construction & Decision Reconstruction

**Status**: Roadmap (Ready for Implementation)  
**Duration**: 8-12 weeks  
**Effort**: ~2,800 LoC + 50+ tests  

## Entry Condition

- ✅ Phase 14: Universal Temporal Fusion (complete)
- ✅ Phase 15: Root Cause Inference (complete)

## Exit Condition

- Automated causal graph generation from mission timelines
- Decision reconstruction for 80%+ of recorded decisions
- Full context capture (strategic/tactical/operational levels)
- Causal graph validation framework
- 50+ unit and integration tests
- <2s graph construction for 30-minute missions

## Problem Statement

### Current State (Phase 15)

✅ We can identify *what* failed and *why* (root cause + contributing factors)  
✅ We have temporal fusion (unified 5D timeline with multi-modal data)  
✅ We generate recommendations (tuning, capability, architecture tiers)

### Gap (What Phase 16 Solves)

❌ We cannot yet answer: "What *alternatives* existed at decision points?"  
❌ We cannot yet visualize the *chain of causality* (A → B → C → Failure)  
❌ We cannot yet reconstruct *why* a decision was made  
❌ We lack structured *decision metadata* (context, confidence, outcome)  

### Outcome (What Phase 16 Delivers)

✅ Automated causal DAG construction from `NavigationSession`  
✅ Decision reconstruction with full context and alternatives  
✅ Foundation for counterfactual analysis (Phase 17)  
✅ Causal graph validator ensuring quality & interpretability  

## Architecture: Four Subsystems

### 1. Causal Graph Builder (40% effort)

**Purpose**: Construct causal DAGs from mission timelines  
**Input**: `NavigationSession` from Phase 14  
**Output**: `CausalGraph { vertices, edges, confidence_scores }`

#### Vertex Types

```rust
enum VertexType {
    SensorReading,      // lidar_scan, camera_frame, imu_update
    ComputedState,      // localization_state, costmap_state, planner_state
    DecisionNode,       // replan_trigger, recovery_activated, speed_changed
    OutcomeNode,        // mission_success, mission_failure, battery_depleted
    Environmental,      // obstacle_detected, human_detected, map_inconsistency
}
```

#### Edge Types

```rust
enum EdgeType {
    // Causal (X causes Y)
    DirectCausal,       // A → B (immediate, high confidence)
    LatentCausal,       // A ~~~> B (delayed, temporal window)
    
    // Correlation (X correlates with Y)
    CoOccurrence,       // A | B (happen together)
    TemporalProximity,  // A ~~ B (close in time)
    
    // Dependency (Y depends on X)
    Prerequisite,       // X ⟹ Y (X must precede Y)
    Information,        // X ⊢ Y (X provides info for Y)
}
```

#### Confidence Scoring

```
confidence = α * temporal_proximity
           + β * magnitude_change_at_boundary
           + γ * historical_frequency
           + δ * fleet_validation

where:
  temporal_proximity = 1 - (time_gap / temporal_window)
  magnitude_change = (|y_after - y_before| / y_baseline)
  historical_frequency = count(edge_in_fleet) / total_edges_in_fleet
  fleet_validation = correlation_coefficient_across_50plus_similar_missions
```

#### Algorithm

```rust
struct CausalGraphBuilder {
    timeline: NavigationTimeline,
    detectors: Vec<Box<dyn EdgeDetector>>,
    config: BuilderConfig,
}

impl CausalGraphBuilder {
    fn build() -> Result<CausalGraph> {
        // 1. Extract all vertices from timeline
        let vertices = self.extract_vertices()?;
        
        // 2. For each vertex pair, check causal edge existence
        let candidate_edges = self.detect_edges(&vertices)?;
        
        // 3. Score each edge by multiple heuristics
        let scored_edges = self.score_edges(&candidate_edges)?;
        
        // 4. Prune low-confidence edges, enforce DAG property
        let pruned = self.enforce_dag(&scored_edges)?;
        
        Ok(CausalGraph { vertices, edges: pruned })
    }
}
```

#### Built-in Edge Detectors

**1. Temporal Proximity Detector**
```
If X ends at t1 and Y starts at t2, and (t2 - t1) < threshold:
  confidence = decay(t2 - t1, decay_rate)
  edge: X → Y
```
*Use Case*: "Replanning was triggered shortly after obstacle detected"

**2. Magnitude Change Detector**
```
If X is sensor S1 at t1 with value v1,
   and Y is state S2 at t2 with derivative dY/dt,
   and dY/dt correlates with ΔS1:
  confidence = correlation_coefficient(dY/dt, ΔS1)
  edge: S1 → S2
```
*Use Case*: "Battery sensor drop caused speed reduction"

**3. Decision Trigger Detector**
```
If X is threshold violation (e.g., obstacle_distance < safety_margin),
   and Y is decision (e.g., replan_triggered),
   and X precedes Y in timeline:
  confidence = high (conditional causality)
  edge: X → Y
```
*Use Case*: "Obstacle triggered replanning decision"

**4. Multi-Modal Alignment Detector**
```
If X (camera: person detected), Y (lidar: obstacle), Z (planner: replan),
   all occur within 200ms window,
   and any pair's correlation validates across 50+ similar missions:
  confidence = fleet_validation_score
  edges: (X→Z), (Y→Z) or other combinations
```
*Use Case*: "Multiple sensors triggered same recovery behavior"

**5. Historical Validation Detector**
```
If edge X → Y appears in this mission,
   and X → Y appears in 40+ other fleet missions,
   and success_rate_with_edge ≠ success_rate_without_edge:
  
  Δ_success = |success_with_edge - success_without_edge|
  confidence = historical_frequency * effect_size(Δ_success)
  edge: X → Y
```
*Use Case*: "This causal pattern is known and validated across fleet"

### 2. Decision Reconstruction Engine (35% effort)

**Purpose**: Recover every significant decision with full context  
**Input**: `CausalGraph`, `NavigationSession`  
**Output**: `Vec<Decision>` with context, alternatives, selected path, outcome

#### Decision Structure

```rust
struct Decision {
    id: String,
    timestamp: i64,
    category: DecisionCategory,  // Strategic/Tactical/Operational
    trigger: String,             // what caused this decision?
    
    context: DecisionContext {
        current_state: Map<String, Value>,           // pose, battery, etc.
        recent_inputs: Vec<SensorInput>,             // visible data
        environment: EnvironmentState,               // obstacles, people, etc.
        constraints: Vec<Constraint>,                // time, safety, etc.
        historical: Vec<SimilarPastDecision>,        // what happened before
    },
    
    alternatives: Vec<Alternative> {
        id: String,
        action: String,
        predicted_outcome: String,
        feasibility: f32,           // 0-1: can this execute?
        compatibility: f32,         // 0-1: aligns with constraints?
    },
    
    selected: Alternative,           // which was chosen?
    confidence: f32,                 // 0-1: quality of decision
    
    outcome: DecisionOutcome {
        result: String,
        delay_ms: i32,
        safety_margin_change: f32,
        success: bool,
    },
}

enum DecisionCategory {
    Strategic,    // mission assignment, route planning, goal selection
    Tactical,     // obstacle avoidance, planner switching, recovery
    Operational,  // speed reduction, tool selection, sensor config
}
```

#### Decision Point Identification

A decision occurs when:

1. **Discrete choice** — System selects between alternatives (A vs B vs C)
2. **Recovery activation** — System invokes recovery behavior
3. **Mode switching** — System changes operational mode (planning → execution)
4. **Adaptation** — System responds to environment change
5. **Objective conflict** — System prioritizes between competing goals

#### Examples

- **Planner generates 3 routes** → robot selects one → *Decision point*
- **Obstacle detected** → robot decides: wait/replan/request_help → *Decision point*
- **Battery drops** → robot decides: increase_speed/reduce_speed/seek_charger → *Decision point*
- **Ambiguous query** → agent decides: web_search/internal_kb/ask_human → *Decision point*

#### Reconstruction Algorithm

```rust
impl DecisionReconstructor {
    fn reconstruct(&self) -> Result<Vec<Decision>> {
        // 1. Identify decision points in timeline
        let decision_points = self.identify_decision_points()?;
        
        for point in decision_points {
            // 2. Snapshot current state at decision time
            let state = self.snapshot_state_at(point.timestamp)?;
            
            // 3. Collect recent sensor inputs (past 5 seconds)
            let inputs = self.get_recent_inputs(
                point.timestamp - 5000,
                point.timestamp
            )?;
            
            // 4. Extract constraints (mission deadline, safety, etc.)
            let constraints = self.extract_constraints()?;
            
            // 5. Find similar historical decisions
            let history = self.find_similar_decisions(&state, 10)?;
            
            // 6. Enumerate alternatives for this decision type
            let alternatives = self.enumerate_alternatives(&point)?;
            
            // 7. Score each alternative
            for alt in &alternatives {
                alt.feasibility = self.score_feasibility(&alt)?;
                alt.compatibility = self.score_compatibility(&alt, &constraints)?;
            }
            
            // 8. Determine which was selected
            let selected = self.identify_selected(&point)?;
            
            // 9. Score confidence of decision
            let confidence = self.score_decision_quality(&selected, &alternatives)?;
            
            // 10. Retrieve outcome
            let outcome = self.get_decision_outcome(&point)?;
            
            decisions.push(Decision {
                context: DecisionContext { state, inputs, constraints, history },
                alternatives,
                selected,
                confidence,
                outcome,
            });
        }
        
        Ok(decisions)
    }
}
```

### 3. Causal Graph Validator (15% effort)

**Purpose**: Ensure quality and interpretability of causal graphs  
**Validation Rules**:
- No cycles (preserve DAG property)
- Confidence scoring is calibrated
- Graph explains >80% outcome variance
- Edges validated on fleet data

#### Validation Tests

```rust
#[test]
fn test_single_mission_dag() {
    let session = load_test_mission("collision_avoided.bag");
    let graph = CausalGraphBuilder::new(&session).build()?;
    assert!(graph.is_dag(), "Graph contains cycles!");
}

#[test]
fn test_edge_confidence_calibration() {
    // If edge has 0.92 confidence, 92% of similar missions
    // should show the same causal pattern
    let edges_high_conf = graph.edges_with_confidence(0.90..);
    let validation_rate = validate_across_fleet(&edges_high_conf, &fleet_data);
    assert!(validation_rate > 0.85, "Confidence not calibrated");
}

#[test]
fn test_graph_explains_outcome() {
    // Removing a high-confidence edge should degrade outcome prediction
    let outcome_with_edge = predict_outcome(&graph);
    let graph_without = graph.without_edge(&high_conf_edge);
    let outcome_without = predict_outcome(&graph_without);
    
    let degradation = (outcome_with_edge - outcome_without).abs();
    assert!(degradation > 0.15, "Edge not important to outcome");
}

#[test]
fn test_conflict_resolution() {
    // When edges conflict (A→B 0.8, A→¬B 0.75), reduce both
    let conflicts = graph.find_conflicting_edges();
    for conflict in conflicts {
        assert!(graph[conflict.edge1].confidence < 0.75);
        assert!(graph[conflict.edge2].confidence < 0.75);
    }
}
```

### 4. Decision Pattern Matcher (10% effort)

**Purpose**: Accelerate decision reconstruction via templates  
**Benefit**: Reduce 500ms reconstruction to <50ms per decision

#### Pattern Definition

```rust
struct DecisionPattern {
    id: String,
    trigger_template: String,       // "obstacle_distance < threshold"
    context_template: Map<String, String>,
    
    alternatives: Vec<Alternative>,
    typical_selected: String,
    typical_outcome: String,
    historical_success_rate: f32,
}

// Example:
DecisionPattern {
    id: "sudden_obstacle",
    trigger_template: "obstacle_detected(distance < safety_margin)",
    context_template: {
        "moving": "forward",
        "localization_confidence": "> 0.8",
    },
    alternatives: vec![
        Alternative { action: "wait", feasibility: 1.0, success_rate: 0.72 },
        Alternative { action: "replan", feasibility: 0.9, success_rate: 0.85 },
        Alternative { action: "request_help", feasibility: 1.0, success_rate: 0.95 },
    ],
    typical_selected: "replan",
    typical_outcome: "delay 2-8s, mission continues",
    historical_success_rate: 0.85,
}
```

#### Usage

When a decision matches a pattern:
1. Reuse pattern's alternative list
2. Fill in mission-specific values
3. Reduce reconstruction time from 500ms → <50ms
4. Improve accuracy via historical data

## Data Model

### Protobuf Definitions

```protobuf
syntax = "proto3";

package pyroboreplay.v1;

message CausalGraph {
    repeated Vertex vertices = 1;
    repeated Edge edges = 2;
    int64 timestamp_ns = 3;
}

message Vertex {
    string id = 1;
    string type = 2;                // sensor_reading, decision, outcome
    int64 timestamp_ns = 3;
    map<string, string> attributes = 4;
    float confidence = 5;
}

message Edge {
    string source_id = 1;
    string target_id = 2;
    string edge_type = 3;            // causal, correlation, dependency
    float confidence = 4;
    int32 time_gap_ms = 5;
    repeated string evidence = 6;
}

message Decision {
    string id = 1;
    int64 timestamp_ns = 2;
    string category = 3;
    string trigger = 4;
    
    DecisionContext context = 5;
    repeated Alternative alternatives = 6;
    Alternative selected = 7;
    float confidence = 8;
    
    DecisionOutcome outcome = 9;
}

message DecisionContext {
    map<string, float> state = 1;
    repeated SensorInput recent_inputs = 2;
    repeated Constraint constraints = 3;
    repeated SimilarHistoricalDecision history = 4;
}

message Alternative {
    string id = 1;
    string action = 2;
    float feasibility = 3;
    float compatibility = 4;
    string predicted_outcome = 5;
}

message DecisionOutcome {
    string result = 1;
    int32 delay_ms = 2;
    float safety_margin_change = 3;
    bool success = 4;
}
```

## Testing Strategy

### Unit Tests (30)
- Edge detector accuracy (temporal, magnitude, decision trigger, multi-modal, historical)
- Graph DAG property preservation
- Decision reconstruction completeness
- Context extraction correctness
- Pattern matcher accuracy

### Integration Tests (20)
- End-to-end graph construction on 10 synthetic missions
- Decision reconstruction on Phase 15 test cases
- Causal graph explains >80% of root causes from Phase 15

### Fleet Validation (Planned Phase 17)
- Compare reconstructed causal edges against 50+ real missions
- Validate that high-confidence edges correlate with repeated outcomes

## Deliverables

1. **CausalGraphBuilder** — Automated graph construction (1,200 LoC)
   - 5 edge detectors
   - Confidence scoring
   - DAG enforcement

2. **DecisionReconstructor** — Decision recovery (900 LoC)
   - Alternative enumeration
   - Feasibility scoring
   - Context capture

3. **CausalGraphValidator** — Quality assurance (400 LoC)
   - DAG validation
   - Confidence calibration
   - Outcome explanation

4. **DecisionPatternMatcher** — Template-based optimization (300 LoC)
   - Pattern definition
   - Matching logic
   - Reconstruction speedup

5. **Tests** — Comprehensive validation (800 LoC)
   - Unit tests (30)
   - Integration tests (20)

6. **Protobuf Definitions** — Data models

7. **Documentation** — API guide, examples, design rationale

## Success Metrics

| Metric | Target |
|--------|--------|
| Causal Edge Precision | >85% on synthetic missions |
| Decision Reconstruction Completeness | >80% of decisions captured |
| Graph Quality (DAG property) | <5% cycle formation |
| Confidence Calibration | 0.9 confidence → 88%+ fleet validation |
| Performance | <2s for 30-min mission |

## Phase 16 → Phase 17 Handoff

**Phase 17 (Counterfactual Analysis)** will:
- Query causal graphs: "What if we removed this edge?"
- Simulate outcomes: "What if we chose Alternative B instead of C?"
- Generate alternative timelines with probability distributions

**Phase 16 provides foundation**: Accurate causal graphs, decision reconstruction, confidence scoring.

## Implementation Plan

### Week 1-2: Setup & Edge Detectors
- [ ] Define protobuf messages
- [ ] Implement temporal proximity detector
- [ ] Implement magnitude change detector
- [ ] Tests for both detectors

### Week 3-4: Decision Triggers & Multi-Modal
- [ ] Decision trigger detector
- [ ] Multi-modal alignment detector
- [ ] Historical validation detector
- [ ] Detector integration & tests

### Week 5-6: Decision Reconstruction
- [ ] Decision point identification
- [ ] Context capture (state, inputs, constraints)
- [ ] Alternative enumeration
- [ ] Decision reconstruction end-to-end

### Week 7-8: Validation & Optimization
- [ ] Causal graph validator
- [ ] Decision pattern matcher
- [ ] DAG enforcement
- [ ] Confidence calibration

### Week 9-10: Integration & Testing
- [ ] End-to-end tests
- [ ] Fleet validation framework
- [ ] Performance optimization
- [ ] Documentation

### Week 11-12: Polish & Delivery
- [ ] Code review & cleanup
- [ ] Final testing
- [ ] Release preparation
- [ ] Phase 17 handoff

## Related

- [[phase14_universal_temporal_fusion.md]] — Input data source
- [[phase15_root_cause_inference.md]] — Root causes integrated with causal graphs
- [[causality_decision_engine_vision.md]] — Strategic vision (Phases 16-20)
