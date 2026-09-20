# Causality & Decision Intelligence Engine — Vision

## Objective

Transform PyRoboReplay from a single-mission forensic tool into an enterprise-scale platform that reconstructs not just *what happened*, but *why*, *what alternatives existed*, and *what could have been*.

## Core Domains

The engine must work across:

- **Robots** — Single robots and multi-robot fleets
- **AI Agents** — Autonomous agents and multi-agent systems
- **Workflow Orchestration** — Task orchestration, DAG execution
- **Industrial Automation** — Manufacturing, logistics, process automation
- **Autonomous Vehicles** — Self-driving cars, delivery robots
- **Digital Twins** — Simulated + real execution analysis
- **Human-in-the-Loop Systems** — Systems with human oversight/intervention

## Nine Strategic Layers

### 1. Causality Graph Construction

Automatically construct directed acyclic causal graphs from recorded execution data.

**Represents relationships such as:**

```
Sensor Drift
  → Localization Error
  → Path Replanning
  → Increased Travel Distance
  → Battery Depletion
  → Mission Failure

or

Ambiguous Prompt
  → Context Retrieval Failure
  → Incorrect Tool Selection
  → Hallucinated Output
  → User Escalation
```

**Graph Properties:**
- One-to-one, one-to-many, many-to-one causality
- Cascading failures
- Feedback loops & cyclic dependencies
- Temporal causality windows
- Confidence scoring per edge

### 2. Decision Reconstruction Engine

Reconstruct every significant decision made during execution.

**For each decision, store:**

- **Context**: Current state, inputs available, environmental conditions, historical context, constraints
- **Alternatives**: All possible alternatives identified at decision time with feasibility scores
- **Selected Path**: Which alternative was chosen and why
- **Confidence**: Quality score of the decision (0.5 = uncertain, 0.95 = certain)
- **Outcome**: Actual result, delay, safety margin change

**Example:**

```
Obstacle Encountered

Available Options:
  Option A: Wait
  Option B: Replan
  Option C: Request Human Assistance

Selected: Replan (84% confidence)
Outcome: Mission Continued, 12s delay
```

### 3. Multi-Layer Decision Modeling

Capture decisions at multiple levels:

**Strategic Decisions**
- Mission assignment
- Route planning
- Goal selection
- Task prioritization

**Tactical Decisions**
- Obstacle avoidance
- Planner switching
- Recovery behavior selection

**Operational Decisions**
- Speed reduction
- Tool invocation
- Sensor selection
- Resource allocation

Allow correlation across layers to understand how strategic decisions influence tactical outcomes.

### 4. Root Cause Analysis Engine (Enhanced Phase 15)

Automatically identify:

**Direct Causes**
- Immediate causes of an outcome
- Example: Collision caused by incorrect obstacle classification

**Contributing Factors**
- Secondary causes
- Example: Sensor occlusion, low lighting, outdated map

**Systemic Causes**
- Long-term causes
- Example: Insufficient training data, configuration drift, maintenance delays

### 5. Counterfactual Analysis Engine

Generate alternative histories. Allow users to ask:

- "What if another planner was used?"
- "What if battery level was higher?"
- "What if weather conditions differed?"
- "What if another model was selected?"
- "What if a human intervened?"

**Generate:**
- Actual timeline (what happened)
- Alternative timeline (what could have happened)
- Highlight divergence points (where alternatives branch)

### 6. Outcome Influence Scoring

Calculate impact scores for each factor:

```
Factor A contributed 42%
Factor B contributed 27%
Factor C contributed 18%
...
```

Rank all influences affecting the final outcome. Understand which factors mattered most.

### 7. Failure Chain Mining

Analyze large execution datasets to automatically identify:

- **Most Common Failure Chains**: Localization Error → Planner Recovery → Mission Timeout
- **Most Expensive Failure Chains**: Docking Failure → Manual Intervention → Production Delay
- **Most Frequent Recovery Paths**: Obstacle Detection → Replan → Continue Mission

Cluster similar chains across thousands of missions.

### 8. Decision Pattern Discovery

Discover recurring behaviors and decision patterns:

**Example Pattern:**

```
Pattern: Low Battery Detected

Typical Response: Speed Reduction
Outcome: Mission Delay (20-40%)
Success Rate: 92%

vs.

Pattern: Low Battery Detected (in known high-traffic areas)
Typical Response: Seek Charger
Outcome: Mission Delay (5-15%)
Success Rate: 100%
```

Automatically cluster similar decision patterns and correlate with outcomes.

### 9. Knowledge Extraction & Recommendation Engine

Convert raw executions into reusable organizational knowledge:

**Insights Generated:**
- "Route B performs better during peak hours"
- "Planner X fails more often in narrow corridors"
- "Agent Configuration Y increases success rate by 15%"

**Proactive Recommendations:**
- "Switch to Planner V3 in dense environments"
- "Increase localization frequency by 2x in GPS-denied areas"
- "Enable secondary obstacle verification when human detected"

Recommendations must be evidence-backed using historical execution data.

## Integration Scope

### With Existing PyRoboReplay Phases

**Phase 14: Universal Temporal Fusion**
- Input: Unified 5D timeline with multi-modal sensor data
- Query interface: `timeline.query_range()`, `timeline.filter_by_type()`
- Rich multi-modal access (camera, lidar, imu, logs, environment)

**Phase 15: Root Cause Inference**
- Input: `FailurePattern`, `RootCauseFinding`, `Recommendation`
- Enhanced by: Counterfactual layer, outcome influence scoring
- Extended to: Link to decision reconstruction, causal graphs

### With Companion Projects

**StatGuardian**
- Integration: Use quality gates for decision confidence scoring
- Exchange: Share lineage data for causal edge construction
- Synergy: Data quality impacts causal graph reliability

**PyStreamMCP**
- Two-stage selective intelligence applied to causal graphs
- Selective edge expansion for large-scale analysis
- Metadata filtering for fast pattern discovery

**PyTerrainMap**
- Spatial causality: "Terrain type A → Navigation difficulty B"
- Temporal causality: "Lighting change → Localization shift"
- 5D causal coordinates: (x, y, z, time, quality)

## Enterprise-Scale Analytics

**Scale Targets:**
- Millions of missions
- Millions of agent runs
- Billions of individual decisions

**Outputs:**
- Failure heatmaps (spatial, temporal, causal)
- Causality heatmaps (edge frequency, impact magnitude)
- Decision quality scores (across fleets)
- Recovery effectiveness scores
- Planner/agent performance rankings

## Explainability Layer

Generate human-readable explanations for:

**Engineers**: Technical detail, algorithm decisions, data lineage  
**Operators**: Actionable guidance, recovery recommendations, next steps  
**Auditors**: Compliance trail, decision audit, event chain  
**Business Stakeholders**: ROI, risk, reliability metrics, trend analysis

**Example Explanation:**

```
Mission Failed

Primary Cause: Localization Drift

Contributing Factors:
  - Temporary Sensor Occlusion (camera obscured for 3s)
  - Map Inconsistency (feature moved since last update)

Decision Made: Route Replanning
Alternative: Stop and Wait
Predicted Outcome of Alternative: Mission Delay but No Failure

Recommendation: Increase localization update frequency by 2x
```

## Long-Term Goal

Create an engine capable of transforming raw execution histories into a **living map of causality, decisions, outcomes, risks, and optimization opportunities**.

The system should continuously learn from every execution and become the authoritative source for understanding:

- **Why** autonomous systems succeed, fail, adapt, and evolve over time
- **Which** decision patterns lead to success or failure
- **How** to optimize outcomes across millions of executions
- **What** architectural or tuning changes will improve performance

## Success Criteria

1. **Causal Reconstruction**: Automatically reconstruct causal chains from raw execution data
2. **Root Cause Identification**: Identify >95% of root causes with confidence scoring
3. **Counterfactual Speed**: Generate counterfactual timelines in <1 second per query
4. **Pattern Clustering**: Cluster 10,000+ missions into decision patterns in minutes
5. **Query Capability**: Support queries like "Show all missions failing due to sensor drift"
6. **Recommendation Impact**: Generate evidence-backed recommendations improving outcome rates by 15%+
7. **Explainability**: Generate human-readable explanations for all layers of analysis
8. **Scale**: Support analysis of billions of decisions across enterprise deployments

## Implementation Phases

**Phase 16** (8-12 weeks): Causal Graph Construction & Decision Reconstruction  
**Phase 17** (6-8 weeks): Counterfactual Analysis & Outcome Influence Scoring  
**Phase 18** (8-10 weeks): Pattern Discovery & Similarity Clustering  
**Phase 19** (6-8 weeks): Knowledge Extraction & Recommendation Engine  
**Phase 20** (12-16 weeks): Enterprise Analytics & Multi-Domain Support  

Total: ~40-54 weeks from Phase 16 to full feature set.
