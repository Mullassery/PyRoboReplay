# Revised Roadmap: Bridging Causal Reasoning & Gap Detection

**Context**: PyRoboReplay has two independent systems:
1. **Causal Reasoning Engine** (src/core/, ~9,000 LOC) - Explains *why* failures happen
2. **Reality Gap Detection** (src/analyzers/, 407 tests, ~2,500 LOC) - Detects *what* gaps exist

**Gap in the architecture**: They don't talk to each other.

**Opportunity**: Integrate them to create a complete understanding system.

---

## Current State Analysis

### System 1: Causal Reasoning (Existing)

```
src/core/causality.rs (710 LOC)
  → CausalGraph: Represents causal relationships between events
  → CausalLink: Individual cause → effect links with confidence

src/core/root_cause.rs (449 LOC)
  → RootCauseAnalyzer: Identifies upstream causes of failures
  → RootCauseHypothesis: Ranked explanations with alternatives

src/core/correlation.rs (463 LOC)
  → TemporalCorrelationAnalyzer: Detects time-based relationships

src/core/explanation.rs
  → ExplanationGenerator: Converts failures → natural language

src/core/recommendation.rs (508 LOC)
  → Generates corrective actions from root causes

src/core/counterfactual.rs (521 LOC)
  → Explores "what if" scenarios
  → Identifies critical path events

src/core/multi_robot.rs (524 LOC)
  → Fleet-level causal analysis
  → Cross-robot pattern discovery

src/core/diagnostic_report.rs (451 LOC)
  → Structured incident reports
```

**Capability**: Given a failure event, explain why it happened with ranked confidence.

**Limitation**: Only sees mission events, doesn't understand gap semantics (sim-real phenomena, sensor degradation, etc.)

### System 2: Reality Gap Detection (New - Phases 1-4)

```
src/analyzers/mod.rs
  → RealityGapFinding: Structured gap representation

Phase 1-2:
  → 5 domain detectors (Physical/Sensor/Environmental/System/Coordination)
  → Bayesian scorer (distinguishes sim-gaps from algorithm bugs)
  → Severity classifier (4-factor decision tree)
  → Historical database (tracks gap frequency per robot type)

Phase 3:
  → Evidence aggregation (multi-detector fusion)
  → Feedback recording (human verification)
  → Bayesian recalibration (learn from feedback)
  → Per-robot calibration (type-specific thresholds)

Phase 4:
  → Severity contracts (auditable, versioned rules)
  → Drift detection (statistical anomalies)
  → Quality confidence (data reliability scoring)
```

**Capability**: Detect sim-real phenomena, score probability, track frequency, adapt per-robot.

**Limitation**: Operates independently of causal reasoning. Doesn't explain *why* gaps matter or how they chain.

---

## The Integration Gap

### What's Missing: Cross-System Understanding

**Scenario**: Warehouse robot collision

**Causal Engine sees**:
```
14:32:10 → Obstacle detection confidence drops 0.92 → 0.45
  (but doesn't know WHY)
14:32:11 → Planning chooses "pass obstacle" (seems safe based on confidence)
14:32:12 → Reality: confidence was wrong
14:32:13 → Collision
```

**Gap Detection Engine knows**:
```
Optical Contamination detected with 0.78 confidence
  → Water droplets on camera lens
  → Detected from LiDAR intensity correlation
  → Common in rainy environments
  → This robot type: occurs 3x/week in wet weather
  → Recommended: add optical flow failover
```

**But they never meet!**

---

## New Integration Architecture

### Phase 5: Evidence Bridge (Tasks 1-3)

**Goal**: Connect RealityGapFinding events → CausalGraph

**Task 5.1: Gap Event Adapter**
```rust
pub struct GapToCausalAdapter {
    gap_finding: RealityGapFinding,
    mission_events: Vec<MissionEvent>,
}

impl GapToCausalAdapter {
    /// Convert a detected gap into causal events
    pub fn gap_to_events(&self) -> Vec<MissionEvent> {
        // Example: Optical Contamination gap with:
        //   - detection_time: 14:32:10
        //   - severity: High
        //   - confidence: 0.78
        //   - evidence: [LiDAR_intensity_drop, detection_confidence_drop]
        //
        // Becomes causal events:
        // 1. "optical_contamination_detected" @ 14:32:10
        // 2. "detection_confidence_degraded" @ 14:32:11
        // 3. "planner_unaware_of_gap" @ 14:32:12
        // 4. "collision_due_to_stale_perception" @ 14:32:13
    }
    
    /// Add gap evidence to existing causal graph
    pub fn enrich_causal_graph(&self, graph: &mut CausalGraph) {
        // Add edges:
        //   weather_condition (rain) → optical_contamination
        //   optical_contamination → detection_failure
        //   detection_failure → planning_error
        //   planning_error → collision
    }
}
```

**Task 5.2: Multi-Factor Causal Inference**
```rust
pub struct MultiFactorCausalInference {
    gaps: Vec<RealityGapFinding>,
    drift_stats: Vec<DriftStats>,
    quality_metadata: HashMap<String, QualityMetadata>,
    causal_graph: CausalGraph,
}

impl MultiFactorCausalInference {
    /// Identify causal chains involving gaps + drift + quality factors
    pub fn infer_combined_causality(&self) -> Vec<CausalChain> {
        // Example chain:
        // rain_detected
        //   ↓ (0.87 confidence)
        // optical_contamination_forms
        //   ↓ (0.78 confidence)
        // detection_confidence_drops
        //   ↓ (0.92 confidence) [with drift: +0.05]
        // planner_chooses_unsafe_path
        //   ↓ (0.65 confidence)
        // collision
    }
}
```

**Task 5.3: Incident Narrative from Gaps**
```rust
pub struct GapNarrator {
    gap_findings: Vec<RealityGapFinding>,
    causal_chains: Vec<CausalChain>,
    quality_context: QualityContext,
}

impl GapNarrator {
    pub fn generate_gap_narrative(&self) -> IncidentNarrative {
        "At 14:30, rain began. Water accumulated on the camera lens,
         causing optical contamination (detected: 0.78 confidence).
         Object detection confidence dropped 40% in wet regions.
         The planner, unaware of image degradation, trusted the detection
         confidence scores and chose a path that would pass the obstacle.
         However, at 14:32, the true obstacle position differed by 0.8m
         from the detection, resulting in collision.
         
         Root cause: Lack of weather-aware perception confidence
         normalization. The system should discount detection confidence
         when optical contamination is detected."
    }
}
```

**Tests**: 15 tests validating gap-to-event conversion, causal graph enrichment, narrative generation

---

### Phase 6: Causal Gap Analysis (Tasks 4-6)

**Goal**: Use causal reasoning to explain gap root causes

**Task 6.1: Root Cause of Gaps**
```rust
pub struct GapRootCauseAnalyzer {
    gap_analyzer: RootCauseAnalyzer,
    gap_detector: RealityGapDetector,
}

impl GapRootCauseAnalyzer {
    /// Why is this gap occurring frequently?
    pub fn explain_gap_frequency(
        &self, 
        gap_category: &str,
        occurrences: &[RealityGapFinding],
    ) -> GapExplanation {
        // For "Optical Contamination" happening 15x in last week:
        // 1. Trace back to environmental conditions
        // 2. Identify correlations: rain ↔ outdoor operations
        // 3. Identify system responses: detection failures ↔ rover type
        // 4. Generate explanation:
        //    "Outdoor rovers deployed in wet season
        //     → water on optics inevitable
        //     → detection confidence unreliable without compensation
        //     → collision risk 3x higher than indoor fleet"
    }
}
```

**Task 6.2: Fleet Causal Patterns**
```rust
pub struct FleetCausalPatternDetector {
    gap_history: HistoricalDatabase,
    causal_graph_per_mission: HashMap<String, CausalGraph>,
    robot_type_profiles: HashMap<String, RobotTypeProfile>,
}

impl FleetCausalPatternDetector {
    /// Identify causal patterns recurring across fleet
    pub fn discover_fleet_patterns(&self) -> Vec<RecurringCausalPattern> {
        // Pattern 1: "Thermal throttling → Latency spike → Detection delay"
        //   Frequency: 47 missions (18% of fleet)
        //   Robot types: All wheel robots in summer
        //   Causal confidence: 0.91
        //   Recommendation: Thermal management or algorithmic optimization
        
        // Pattern 2: "GPS multipath → Localization uncertainty → Planner error"
        //   Frequency: 23 missions (9% of fleet)
        //   Robot types: Urban-deployed vehicles
        //   Causal confidence: 0.87
        //   Recommendation: IMU fusion or HD map fallback
    }
}
```

**Task 6.3: Causal Explanations for Agents**
```rust
pub struct AgentCausalQueryEngine {
    causal_graphs: HashMap<String, CausalGraph>,
    gap_findings: Vec<RealityGapFinding>,
    explanations: Vec<IncidentNarrative>,
}

impl AgentCausalQueryEngine {
    /// Answer: "Why did this failure happen?"
    pub async fn explain_failure(
        &self,
        mission_id: &str,
        failure_type: &str,
    ) -> ExplanationResponse {
        // Query processor:
        // 1. Find failure event in mission
        // 2. Trace causal chain backward
        // 3. Identify gap findings involved
        // 4. Synthesize explanation for LLM consumption
        
        ExplanationResponse {
            root_cause_event: "optical_contamination",
            causal_chain: [...],
            gap_evidence: [...],
            confidence: 0.91,
            alternative_explanations: [...],
            recommended_actions: [...],
        }
    }
}
```

**Tests**: 18 tests validating gap root cause analysis, fleet patterns, agent queries

---

### Phase 7: LLM Agent Reasoning (Tasks 7-9)

**Goal**: Enable AI agents to reason about causality + gaps

**Task 7.1: Structured Causal Context for LLMs**
```rust
pub struct LLMCausalContext {
    causal_chain: Vec<CausalLink>,
    gap_evidence: Vec<RealityGapFinding>,
    quality_metrics: QualityMetadata,
    fleet_patterns: Vec<RecurringCausalPattern>,
}

impl LLMCausalContext {
    /// Format for LLM processing
    pub fn to_prompt_context(&self) -> String {
        r#"
        Mission Failure Analysis:
        
        Timeline:
        14:30:00 - Rain detected
        14:30:05 - Optical contamination: 0.78 confidence
        14:30:11 - Detection confidence: 0.92 → 0.45 (-49%)
        14:30:12 - Planner unaware of perception issue
        14:30:13 - Collision
        
        Causal Chain (confidence 0.91):
        1. Weather change → Optical contamination (0.78 conf)
        2. Contamination → Detection degradation (0.87 conf)
        3. Degradation → Planning error (0.65 conf)
        4. Error → Collision (0.99 conf)
        
        Gap Evidence:
        - Optical Contamination: water droplets detected
        - Detection Robustness: confidence drop correlated with image quality
        
        Fleet Pattern Match:
        Similar to "GPS multipath → localization failure" (23 prior cases)
        
        Root Cause: System trust in detection confidence without weather
        compensation. Recommendation: Add optical flow failover for rain.
        "#.to_string()
    }
}
```

**Task 7.2: Causal Query Language**
```rust
pub struct CausalQueryEngine {
    causal_graphs: HashMap<String, CausalGraph>,
    gap_index: GapFindingIndex,
}

impl CausalQueryEngine {
    /// Enable agents to ask causal questions
    pub fn query(&self, question: CausalQuery) -> QueryResult {
        match question {
            // "Why did the robot fail?"
            CausalQuery::WhyDidFailure { mission_id, failure_time } => {
                self.explain_failure_cascade(mission_id, failure_time)
            },
            
            // "What evidence supports this root cause?"
            CausalQuery::Evidence { hypothesis } => {
                self.gather_evidence_for_hypothesis(&hypothesis)
            },
            
            // "Is this gap related to thermal effects?"
            CausalQuery::GapCorrelation { gap_type, factor } => {
                self.find_correlations(&gap_type, &factor)
            },
            
            // "How often does this chain occur across the fleet?"
            CausalQuery::FleetFrequency { causal_chain } => {
                self.measure_fleet_frequency(&causal_chain)
            },
        }
    }
}
```

**Task 7.3: Agent Decision Explanation**
```rust
pub struct AgentDecisionExplainer {
    causal_engine: CausalAnalyzer,
    gap_detector: RealityGapDetector,
}

impl AgentDecisionExplainer {
    /// Explain agent's decision: "Why did you avoid that region?"
    pub fn explain_decision(
        &self,
        agent_action: &AgentAction,
        context: &MissionContext,
    ) -> DecisionExplanation {
        // Agent action: "Reduce speed by 40% in region X"
        // Explanation:
        // 1. Causal analysis detected: "thermal throttling → latency"
        // 2. Gap detection showed: "frequency: 12x in hot weather"
        // 3. Combined inference: "Region X predicted to trigger chain"
        // 4. Decision: "Slow down to give planner more time"
        
        DecisionExplanation {
            action: agent_action.clone(),
            causal_reasoning: "Thermal + latency risk detected",
            gap_evidence: "12 prior thermal-induced failures",
            confidence: 0.89,
            alternatives_considered: ["Wait for cooler time", "Use alternate path"],
        }
    }
}
```

**Tests**: 12 tests validating LLM context generation, causal queries, decision explanations

---

### Phase 8: Fleet-Wide Causal Learning (Tasks 10-12)

**Goal**: Aggregate causal patterns across entire fleet for predictive insights

**Task 8.1: Causal Pattern Mining**
```rust
pub struct FleetCausalPatternMiner {
    all_causal_graphs: HashMap<String, CausalGraph>,
    historical_gaps: HistoricalDatabase,
}

impl FleetCausalPatternMiner {
    /// Mine recurring causal patterns from fleet data
    pub fn mine_patterns(&self) -> Vec<FleetCausalPattern> {
        // Pattern:
        // "CPU throttle (frequency: 3.2%) → Latency spike → Detection delay → Collision"
        // Appears in: 47 missions across 8 robot types
        // Environmental trigger: Temperature > 45°C
        // Time to collision: 45±12 seconds after CPU throttle
        // Impact: 18% fleet failure rate increase
        
        // Pattern becomes predictive model:
        // IF temp > 45°C THEN expect CPU throttle chain in next 10 min
        // IF CPU throttle detected THEN pre-emptively reduce speed
    }
    
    /// Predict failures by detecting causal chains forming
    pub fn predict_failure(
        &self,
        current_state: &SystemState,
        patterns: &[FleetCausalPattern],
    ) -> Vec<PredictedFailure> {
        // Current state shows: temperature rising toward 45°C
        // CPU frequency: 3.2 GHz (normal)
        // Detection latency: 250ms (normal)
        //
        // Prediction: "CPU throttle chain likely to form in 3-5 minutes"
        // Recommendation: "Reduce load now or move to cooler location"
        // Confidence: 0.76
    }
}
```

**Task 8.2: Cross-Robot Causal Alignment**
```rust
pub struct CrossRobotCausalAlignment {
    robot_profiles: HashMap<String, RobotCausalProfile>,
    fleet_patterns: Vec<FleetCausalPattern>,
}

impl CrossRobotCausalAlignment {
    /// Identify how robot-specific causal chains differ
    pub fn align_causal_profiles(&self) -> RobotCausalDifferences {
        // Robot A (outdoor, wheel, large):
        //   Primary chain: thermal → latency
        //   Frequency: 0.18 (18%)
        //   Weather: hot/sunny
        
        // Robot B (indoor, tracked, small):
        //   Primary chain: memory → GC pause
        //   Frequency: 0.06 (6%)
        //   Environment: static, crowded
        
        // Robot C (drone, electric):
        //   Primary chain: battery → power reduction
        //   Frequency: 0.14 (14%)
        //   Usage: continuous flight
        
        // Insight: Deploy recommendations per robot type
        //   A: Thermal management, route planning
        //   B: Memory optimization, task scheduling
        //   C: Battery monitoring, charging strategy
    }
}
```

**Task 8.3: Predictive Causal Alerts**
```rust
pub struct PredictiveCausalAlerts {
    causal_patterns: Vec<FleetCausalPattern>,
    current_fleet_state: FleetState,
}

impl PredictiveCausalAlerts {
    /// Generate alerts when causal chains are forming
    pub fn generate_alerts(&self) -> Vec<Alert> {
        // Alert 1 (HIGH):
        //   "Thermal chain forming: 8 robots now >43°C, CPU throttle imminent"
        //   "Action: Reduce speed or relocate to shaded area"
        //   "Time to critical: ~5 minutes"
        //   "Confidence: 0.84"
        
        // Alert 2 (MEDIUM):
        //   "Memory pressure detected on 3 indoor robots"
        //   "Correlates with prior GC pause chain (confidence: 0.71)"
        //   "Recommendation: Restart perception services"
        
        // Alert 3 (LOW):
        //   "Battery voltage declining on 1 drone"
        //   "Within normal range but trending toward power-reduction chain"
        //   "No action needed yet; monitor next 15 min"
    }
}
```

**Tests**: 14 tests validating pattern mining, cross-robot alignment, predictive alerts

---

### Phase 9: Causal Digital Twin (Tasks 13-15)

**Goal**: Simulate causal chains to forecast system behavior

**Task 9.1: Causal Simulation**
```rust
pub struct CausalSimulator {
    causal_graph_template: CausalGraph,
    fleet_patterns: Vec<FleetCausalPattern>,
    system_parameters: HashMap<String, f32>,
}

impl CausalSimulator {
    /// Simulate how causal chains unfold under different conditions
    pub fn simulate_scenario(&self, scenario: SimulationScenario) -> SimulationResult {
        // Scenario: "Deploy robot in 38°C environment for 2 hours"
        // Simulation:
        // t=0: Temp 38°C, CPU freq 3.2 GHz
        // t=30min: Temp 42°C, thermal accumulation begins
        // t=45min: Thermal throttle threshold reached
        // t=46min: CPU freq reduced 15%, latency +1.2s
        // t=48min: Detection latency >400ms, planning confidence drops
        // t=50min: First collision risk event
        // 
        // Prediction: Mission failure with 67% probability
        // Recommendation: Shorten mission to 45 minutes
    }
}
```

**Task 9.2: What-If Causal Analysis**
```rust
pub struct CausalWhatIfAnalyzer {
    causal_graph: CausalGraph,
    counterfactuals: CounterfactualEngine,
}

impl CausalWhatIfAnalyzer {
    /// Explore counterfactual causal chains
    pub fn what_if_intervention(
        &self,
        original_chain: &CausalChain,
        intervention: &Intervention,
    ) -> CounterfactualOutcome {
        // Original chain: thermal → throttle → latency → collision
        // 
        // What if: "We add optical flow as backup to detection?"
        // Result: Even with detection failure, optical flow prevents collision
        //         Chain is broken at "detection failure → planning error"
        //
        // Impact: Collision probability drops from 67% → 12%
        // Cost-benefit: Very high ROI
    }
}
```

**Task 9.3: Causal Audit Trail**
```rust
pub struct CausalAuditTrail {
    decision_history: Vec<AgentDecision>,
    causal_reasoning: CausalAnalyzer,
}

impl CausalAuditTrail {
    /// Generate explainable audit of agent decisions via causality
    pub fn audit_decision_chain(
        &self,
        mission: &Mission,
    ) -> AuditReport {
        // Every agent decision gets causal explanation:
        // Decision 1: "Reduce speed to 0.3 m/s"
        //   Reason: "Detected thermal chain forming (conf 0.84)"
        //   Evidence: "8 robots throttled, latency spiking"
        //   Causal link: "thermal → throttle → latency → collision"
        //   Alternative considered: "Wait for cooler time" (rejected: time-critical)
        //   Outcome: "Completed safely, avoided collision"
        //   Verdict: ✓ CORRECT DECISION
        
        // This enables:
        // - Regulatory compliance (explain every decision)
        // - Learning (did the causal reasoning work?)
        // - Debugging (where did reasoning fail?)
    }
}
```

**Tests**: 10 tests validating simulation, counterfactual analysis, audit trails

---

## Summary: New Roadmap

### Phase 5: Evidence Bridge (3 tasks, 15 tests)
- Connect gap findings → causal graph
- Multi-factor causal inference
- Generate narratives from gaps

### Phase 6: Causal Gap Analysis (3 tasks, 18 tests)
- Root cause of gap frequencies
- Fleet causal patterns
- Causal explanations for agents

### Phase 7: LLM Agent Reasoning (3 tasks, 12 tests)
- Structured context for LLMs
- Causal query language
- Agent decision explanations

### Phase 8: Fleet Learning (3 tasks, 14 tests)
- Causal pattern mining
- Cross-robot alignment
- Predictive alerts

### Phase 9: Digital Twin (3 tasks, 10 tests)
- Causal simulation
- What-if analysis
- Audit trails

**Total**: 15 tasks, 69 tests, ~1,500 LOC bridging causal + gap systems

---

## Architecture Diagram

```
PHASE 1-4: Reality Gap Detection
├─ Detectors (5 domains)
├─ Bayesian Scoring
├─ Severity Classification
├─ Historical Database
├─ Learning Loop
└─ Fleet Calibration
    ↓ (NEW BRIDGE)

PHASE 5-9: Causal Integration
├─ Evidence Bridge (gap → causal events)
├─ Causal Gap Analysis (why gaps happen?)
├─ LLM Agent Reasoning (query & explain)
├─ Fleet Causal Learning (predict failures)
└─ Digital Twin (simulate & audit)
    ↓ (OUTPUT)

Combined System:
├─ "This gap is happening because X"
├─ "Similar pattern in Y robots"
├─ "Predict failure in 5 minutes"
├─ "Here's why I made that decision"
└─ "Audit trail showing all reasoning"
```

---

## Why This Matters

**Before (Disconnected Systems)**:
- Gap detection: "Optical contamination detected, confidence 0.78"
- Causal reasoning: "Collision caused by detection failure (confidence 0.65)"
- Agent: "I slowed down" (no reasoning)

**After (Integrated Systems)**:
- Gap detection: "Optical contamination, confidence 0.78"
- Causal reasoning: "Optical contamination → detection failure → collision (chain confidence 0.91)"
- Agent: "I slowed down because optical contamination chains to collision in 47 prior fleet events"
- System: "Prediction: This scenario has 67% failure probability; recommendation: change route"

---

## Timeline

- **Phase 5**: 1 week (bridge systems)
- **Phase 6**: 1.5 weeks (gap causal analysis)
- **Phase 7**: 1.5 weeks (LLM integration)
- **Phase 8**: 2 weeks (fleet learning)
- **Phase 9**: 1.5 weeks (digital twin)

**Total**: 7.5 weeks for complete system

---

## Success Metrics

- ✅ All gap findings flow into causal graph
- ✅ Fleet patterns automatically discovered
- ✅ Predictive alerts with >80% accuracy
- ✅ Agent decisions explainable to humans
- ✅ Audit trail complete and verifiable
- ✅ Simulation matches real failures >85%

**Result**: An observability system that transforms terabytes of data into actionable causal narratives.
