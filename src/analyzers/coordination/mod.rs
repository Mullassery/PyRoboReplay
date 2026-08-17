//! Coordination Domain Gap Analyzer
//!
//! Detects gaps in multi-robot systems:
//! - Deadlocks and circular wait conditions
//! - Swarm congestion and flow breakdown
//! - Communication partitions and loss
//! - State divergence between robots
//!
//! IMPORTANT SCOPE NOTE: real multi-robot infrastructure already exists in
//! this codebase — `core::multi_robot::MultiRobotCoordinationAnalyzer`
//! (CoordinationEvent, FleetSnapshot, RobotState, InterRobotCausalLink,
//! pattern detection, pairwise distance, fleet centroid — all real,
//! already-tested code). The gap is that it isn't wired into this
//! `GapDetector` trait: `GapDetector::analyze()` takes only a single
//! mission's `MissionAnalysisData`, with no fleet-wide input, so this
//! analyzer has no way to reach `core::multi_robot`'s data even though that
//! data model is exactly what "multi-robot deadlocks/congestion/partitions"
//! needs. Threading fleet data through here means changing the `GapDetector`
//! trait signature itself (affecting every domain analyzer, not just this
//! one) — a real but larger architectural change, out of scope for filling
//! in this one file.
//!
//! What IS implementable against `MissionAnalysisData` as it stands today:
//! a single robot's own control loop coordination — stalls/gaps in its
//! command stream (a real, if narrower, instance of the same general
//! "coordination gap" concept this domain covers). That's what
//! `ControlLoopStallDetector` does below.

pub mod control_loop_stall;

use crate::analyzers::{GapDetector, MissionAnalysisData, RealityDomain, RealityGapFinding};
use control_loop_stall::ControlLoopStallDetector;

pub struct CoordinationDomainAnalyzer {
    stall_detector: ControlLoopStallDetector,
}

impl CoordinationDomainAnalyzer {
    pub fn new() -> Self {
        CoordinationDomainAnalyzer { stall_detector: ControlLoopStallDetector::new() }
    }
}

impl Default for CoordinationDomainAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl GapDetector for CoordinationDomainAnalyzer {
    fn analyze(&self, mission_data: &MissionAnalysisData) -> Vec<RealityGapFinding> {
        self.stall_detector.analyze(&mission_data.control_messages, mission_data.duration_sec)
    }

    fn domain(&self) -> RealityDomain {
        RealityDomain::Coordination
    }
}
