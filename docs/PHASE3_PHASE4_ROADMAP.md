# PyRoboReplay Phase 3-4 Roadmap

## Phase 3: Learning & Integration Pipeline (Current)

**Goal**: Connect all components—aggregation → CLI output → historical database → learning loop

### Completed
- ✅ **Task 15**: CLI Finding Integration (ConsolidatedFormatter)
- ✅ **Task 16**: Feedback Recording Loop (FeedbackLoopManager)

### In Progress / Pending
- ⏳ **Task 17**: Gap Confidence Recalibration (Bayesian updating from feedback)
- ⏳ **Task 18**: Per-Robot Calibration (fleet-wide pattern learning)
- ⏳ **Task 19**: Phase 3 Validation (end-to-end test with feedback loop)

---

## Phase 4: StatGuardian Integration (Planned)

**Goal**: Leverage embedded StatGuardian for quality-driven severity classification and learned recalibration

### Rationale
- **Severity Classifier** (current): Hand-coded decision tree → replace with StatGuardian contracts
- **Reality Gap Scorer** (current): Hard-coded Bayesian priors → use StatGuardian drift detection + feedback learning
- **Historical Database** (current): Manual frequency queries → use StatGuardian's quality metadata + trending

### Tasks

#### Task 20: Severity Contracts
- Define quality contracts for gap severity (e.g., "response_time_degradation > 5% = High Severity")
- Wire contracts into SeverityClassifier
- Replace hard-coded thresholds with contract evaluation
- **Benefit**: Severity rules become auditable, versioned, composable

#### Task 21: Drift-Driven Detection
- Integrate StatGuardian's drift detector into scoring pipeline
- Use detected drift magnitude to boost gap_score (e.g., drift > 2σ → +0.15 to gap_score)
- Replace manual trend analysis with StatGuardian's statistical inference
- **Benefit**: Statistically rigorous gap detection; automatic outlier handling

#### Task 22: Quality-Aware Confidence
- Embed StatGuardian's quality scores into confidence calculation
- Formula: confidence = (evidence_strength × 0.4) + (quality_metadata × 0.6)
- Gaps detected during high-quality data periods get higher confidence
- **Benefit**: Confidence reflects data reliability; low-quality sensor data → lower gap confidence

#### Task 23: Feedback-Driven Recalibration
- Use StatGuardian's learning framework to update priors from feedback
- When human marks gap as VerifiedCorrect, boost category's base_probability
- When marked as Incorrect, reduce base_probability
- **Benefit**: Scorer learns from fleet feedback; improves over time

#### Task 24: Phase 4 Validation
- End-to-end test: detection → feedback → recalibration → improved scoring
- Verify confidence/severity predictions improve with feedback loop
- Measure: prior accuracy 0.70 → post-feedback accuracy 0.85+

---

## Architecture Integration

```
Phase 2 (Current):
  Raw Findings → Scorer (hard-coded priors) → Severity (decision tree) → Historical DB

Phase 4 (With StatGuardian):
  Raw Findings → Scorer (StatGuardian drift + learned priors)
                      ↓
                Severity (StatGuardian contracts + quality metadata)
                      ↓
                Historical DB (quality-aware trending)
                      ↓
                Feedback Loop (Bayesian prior updates)
```

---

## Timeline

| Phase | Tasks | LOC | Tests | Est. Hours |
|-------|-------|-----|-------|-----------|
| **3** | 17-19 | 500 | 15 | 24 |
| **4** | 20-24 | 400 | 20 | 32 |

**Total Phase 3+4**: 900 LOC, 35 tests, 56 hours (~2 weeks)

---

## Decision Gates

- **End of Phase 3**: Gap detection platform fully functional end-to-end
  - All domain detectors working ✅
  - Aggregation reduces noise ✅
  - CLI output production-ready ✅
  - Learning loop operational ✅
  - **Gate**: 342+ tests passing, <2% false positive rate on validation missions

- **End of Phase 4**: Quality-driven, learned platform
  - StatGuardian integration complete
  - Confidence/severity improve with feedback
  - Ready for fleet deployment
  - **Gate**: 380+ tests, accuracy >85%, zero regressions vs Phase 3
