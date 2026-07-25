use crate::core::confidence_scoring::ConfidenceChain;
use crate::core::failure_detection::{DetectedFailure, FailureDomain, FailureSeverity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Priority {
    Critical = 3,
    High = 2,
    Medium = 1,
    Low = 0,
}

impl Priority {
    pub fn as_str(&self) -> &str {
        match self {
            Priority::Critical => "critical",
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
        }
    }

    pub fn from_severity(severity: FailureSeverity) -> Self {
        match severity {
            FailureSeverity::Critical => Priority::Critical,
            FailureSeverity::High => Priority::High,
            FailureSeverity::Medium => Priority::Medium,
            FailureSeverity::Low => Priority::Low,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLRIASRecommendation {
    pub id: String,
    pub failure_id: String,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub impact: f32, // 0.0-1.0: how much does this fix help?
    pub effort: f32, // 0.0-1.0: how hard to implement?
    pub confidence: f32, // 0.0-1.0: confidence this fixes the issue
    pub roi_score: f32, // impact / effort (impact per unit effort)
    pub evidence_chain: Vec<String>, // evidence supporting this recommendation
    pub implementation_details: Option<String>,
    pub related_parameters: Vec<String>,
}

impl MLRIASRecommendation {
    pub fn new(
        failure_id: String,
        title: String,
        description: String,
        priority: Priority,
        impact: f32,
        effort: f32,
        confidence: f32,
    ) -> Self {
        let roi_score = if effort > 0.0 { impact / effort } else { 0.0 };

        Self {
            id: format!("rec_{}", uuid::Uuid::new_v4()),
            failure_id,
            title,
            description,
            priority,
            impact,
            effort,
            confidence,
            roi_score,
            evidence_chain: Vec::new(),
            implementation_details: None,
            related_parameters: Vec::new(),
        }
    }

    pub fn with_evidence(mut self, evidence: Vec<String>) -> Self {
        self.evidence_chain = evidence;
        self
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.implementation_details = Some(details);
        self
    }

    pub fn with_parameters(mut self, params: Vec<String>) -> Self {
        self.related_parameters = params;
        self
    }
}

pub struct MLRIASRecommendationsEngine {
    failures: Vec<DetectedFailure>,
    confidence_chains: Vec<ConfidenceChain>,
}

impl MLRIASRecommendationsEngine {
    pub fn new(
        failures: Vec<DetectedFailure>,
        confidence_chains: Vec<ConfidenceChain>,
    ) -> Self {
        Self {
            failures,
            confidence_chains,
        }
    }

    pub fn generate_recommendations(&self) -> Vec<MLRIASRecommendation> {
        let mut recommendations = Vec::new();

        for failure in &self.failures {
            let recs = self.generate_for_failure(failure);
            recommendations.extend(recs);
        }

        // Sort by ROI score (highest first)
        recommendations.sort_by(|a, b| {
            b.roi_score
                .partial_cmp(&a.roi_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        recommendations
    }

    fn generate_for_failure(&self, failure: &DetectedFailure) -> Vec<MLRIASRecommendation> {
        match failure.domain {
            FailureDomain::Navigation => self.nav_recommendations(failure),
            FailureDomain::Localization => self.loc_recommendations(failure),
            FailureDomain::Perception => self.perc_recommendations(failure),
            FailureDomain::Middleware => self.mw_recommendations(failure),
            FailureDomain::System => self.sys_recommendations(failure),
        }
    }

    fn nav_recommendations(&self, failure: &DetectedFailure) -> Vec<MLRIASRecommendation> {
        let mut recs = Vec::new();

        if failure.failure_type.contains("timeout") {
            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Increase planner timeout".to_string(),
                    "Increase navigation planner timeout from default 5.0s to 7.5s".to_string(),
                    Priority::High,
                    0.85, // impact
                    0.10, // effort
                    0.90, // confidence
                )
                .with_details(
                    "Edit nav2_params.yaml: planner_server.ros__parameters.planning_plugin_names timeout to 7.5".to_string()
                )
                .with_parameters(vec![
                    "planner_server.ros__parameters.planning_plugin_names".to_string(),
                    "planner_server.timeout".to_string(),
                ]),
            );

            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Use faster planner algorithm".to_string(),
                    "Switch to SmacPlannerLattice for faster computation".to_string(),
                    Priority::High,
                    0.75, // impact
                    0.40, // effort
                    0.65, // confidence
                )
                .with_details(
                    "Replace DWBLocalPlanner with SmacPlannerLattice in launch file".to_string()
                )
                .with_parameters(vec![
                    "planner_server.ros__parameters.planning_plugin_names".to_string(),
                ]),
            );
        }

        if failure.failure_type.contains("oscillation") {
            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Increase controller frequency".to_string(),
                    "Increase controller update rate for smoother path following".to_string(),
                    Priority::Medium,
                    0.70,
                    0.20,
                    0.75,
                )
                .with_details("Increase controller_frequency from 20Hz to 30Hz".to_string()),
            );
        }

        if failure.failure_type.contains("recovery") {
            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Adjust costmap inflation radius".to_string(),
                    "Increase inflation radius to provide safer margins".to_string(),
                    Priority::High,
                    0.80,
                    0.15,
                    0.80,
                )
                .with_parameters(vec!["costmap.inflation_radius".to_string()]),
            );
        }

        recs
    }

    fn loc_recommendations(&self, failure: &DetectedFailure) -> Vec<MLRIASRecommendation> {
        let mut recs = Vec::new();

        if failure.failure_type.contains("divergence") || failure.failure_type.contains("amcl") {
            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Increase AMCL particle count".to_string(),
                    "Add more particles for better pose estimation".to_string(),
                    Priority::High,
                    0.75,
                    0.25,
                    0.80,
                )
                .with_parameters(vec!["amcl.particle_count".to_string()]),
            );

            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Verify map-to-base_link transform".to_string(),
                    "Check TF transform chain for inconsistencies".to_string(),
                    Priority::Critical,
                    0.95,
                    0.30,
                    0.85,
                )
                .with_details("Run: ros2 run tf2_tools view_frames.py".to_string()),
            );
        }

        if failure.failure_type.contains("gps") {
            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Check GPS antenna and connection".to_string(),
                    "Verify GPS hardware and cable connections".to_string(),
                    Priority::Critical,
                    0.98,
                    0.50,
                    0.95,
                )
                .with_details("Physical inspection of GPS module required".to_string()),
            );
        }

        recs
    }

    fn perc_recommendations(&self, failure: &DetectedFailure) -> Vec<MLRIASRecommendation> {
        let mut recs = Vec::new();

        if failure.failure_type.contains("dropout") {
            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Increase sensor update rate".to_string(),
                    "Check sensor configuration for lower frame rate".to_string(),
                    Priority::High,
                    0.80,
                    0.20,
                    0.85,
                )
                .with_details("Increase sensor publishing frequency in launch file".to_string()),
            );

            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Check sensor cable/connection".to_string(),
                    "Verify sensor hardware connections for loose cables".to_string(),
                    Priority::Critical,
                    0.95,
                    0.40,
                    0.90,
                )
                .with_details("Physical inspection recommended".to_string()),
            );
        }

        if failure.failure_type.contains("sync") {
            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Adjust camera-lidar sync tolerance".to_string(),
                    "Increase time sync tolerance for multi-sensor fusion".to_string(),
                    Priority::Medium,
                    0.65,
                    0.15,
                    0.70,
                )
                .with_parameters(vec!["sensor_sync.time_tolerance_ms".to_string()]),
            );
        }

        recs
    }

    fn mw_recommendations(&self, failure: &DetectedFailure) -> Vec<MLRIASRecommendation> {
        let mut recs = Vec::new();

        if failure.failure_type.contains("discovery") {
            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Increase DDS discovery timeout".to_string(),
                    "Extend ROS discovery period for slower networks".to_string(),
                    Priority::High,
                    0.70,
                    0.10,
                    0.75,
                )
                .with_parameters(vec!["ROS_DISCOVERY_TIMEOUT_SEC".to_string()]),
            );
        }

        if failure.failure_type.contains("latency") {
            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Reduce message publishing frequency".to_string(),
                    "Lower QoS demands by publishing less frequently".to_string(),
                    Priority::Medium,
                    0.60,
                    0.20,
                    0.65,
                )
                .with_details("Reduce sensor publishing rates in launch configuration".to_string()),
            );
        }

        if failure.failure_type.contains("starvation") {
            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Check topic subscription QoS".to_string(),
                    "Verify QoS settings match publisher requirements".to_string(),
                    Priority::High,
                    0.85,
                    0.25,
                    0.80,
                )
                .with_details("Check rmw_qos_profile_t settings".to_string()),
            );
        }

        recs
    }

    fn sys_recommendations(&self, failure: &DetectedFailure) -> Vec<MLRIASRecommendation> {
        let mut recs = Vec::new();

        if failure.failure_type.contains("oom") {
            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Reduce navigation stack memory footprint".to_string(),
                    "Optimize memory usage in planner and controller".to_string(),
                    Priority::Critical,
                    0.95,
                    0.30,
                    0.85,
                )
                .with_details("Consider using costmap caching, particle filter optimization".to_string()),
            );

            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Increase available swap space".to_string(),
                    "Configure swap as safety valve for temporary memory spikes".to_string(),
                    Priority::High,
                    0.60,
                    0.20,
                    0.70,
                )
                .with_details("Add 2-4GB swap partition on root filesystem".to_string()),
            );
        }

        if failure.failure_type.contains("thermal") {
            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Reduce CPU usage".to_string(),
                    "Optimize compute-heavy algorithms".to_string(),
                    Priority::High,
                    0.75,
                    0.40,
                    0.70,
                )
                .with_details("Profile CPU usage with perf, optimize hot paths".to_string()),
            );

            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Improve thermal cooling".to_string(),
                    "Add heat sink, improve airflow, or passive cooling".to_string(),
                    Priority::High,
                    0.80,
                    0.50,
                    0.75,
                )
                .with_details("Hardware modification recommended".to_string()),
            );
        }

        if failure.failure_type.contains("cpu_saturation") {
            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Lower sensor publish rates".to_string(),
                    "Reduce data ingestion to match processing capacity".to_string(),
                    Priority::High,
                    0.70,
                    0.15,
                    0.80,
                )
                .with_details("Lower LiDAR/camera frame rates in launch files".to_string()),
            );
        }

        if failure.failure_type.contains("usb") {
            recs.push(
                MLRIASRecommendation::new(
                    failure.id.clone(),
                    "Inspect USB cable and connectors".to_string(),
                    "Check for loose or damaged USB connections".to_string(),
                    Priority::Critical,
                    0.98,
                    0.40,
                    0.95,
                )
                .with_details("Physical inspection of all USB devices required".to_string()),
            );
        }

        recs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Medium);
        assert!(Priority::Medium > Priority::Low);
    }

    #[test]
    fn test_priority_as_str() {
        assert_eq!(Priority::Critical.as_str(), "critical");
        assert_eq!(Priority::High.as_str(), "high");
        assert_eq!(Priority::Medium.as_str(), "medium");
        assert_eq!(Priority::Low.as_str(), "low");
    }

    #[test]
    fn test_mlrias_recommendation_creation() {
        let rec = MLRIASRecommendation::new(
            "failure_1".to_string(),
            "Test recommendation".to_string(),
            "This is a test".to_string(),
            Priority::High,
            0.8,
            0.2,
            0.85,
        );

        assert_eq!(rec.title, "Test recommendation");
        assert_eq!(rec.impact, 0.8);
        assert_eq!(rec.effort, 0.2);
        assert_eq!(rec.confidence, 0.85);
        assert_eq!(rec.roi_score, 4.0); // 0.8 / 0.2
    }

    #[test]
    fn test_mlrias_recommendation_roi_calculation() {
        let rec = MLRIASRecommendation::new(
            "failure_1".to_string(),
            "High ROI recommendation".to_string(),
            "Quick win".to_string(),
            Priority::High,
            0.9,
            0.1,
            0.9,
        );

        assert_eq!(rec.roi_score, 9.0); // 0.9 / 0.1
    }

    #[test]
    fn test_mlrias_recommendation_builder_pattern() {
        let rec = MLRIASRecommendation::new(
            "failure_1".to_string(),
            "Test".to_string(),
            "Description".to_string(),
            Priority::Medium,
            0.5,
            0.5,
            0.75,
        )
        .with_evidence(vec!["evidence_1".to_string()])
        .with_details("Implementation steps".to_string())
        .with_parameters(vec!["param_1".to_string()]);

        assert_eq!(rec.evidence_chain.len(), 1);
        assert_eq!(rec.implementation_details, Some("Implementation steps".to_string()));
        assert_eq!(rec.related_parameters.len(), 1);
    }

    #[test]
    fn test_mlrias_recommendations_engine_creation() {
        let engine = MLRIASRecommendationsEngine::new(Vec::new(), Vec::new());
        let recs = engine.generate_recommendations();
        assert_eq!(recs.len(), 0);
    }

    #[test]
    fn test_priority_from_severity() {
        assert_eq!(Priority::from_severity(FailureSeverity::Critical), Priority::Critical);
        assert_eq!(Priority::from_severity(FailureSeverity::High), Priority::High);
        assert_eq!(Priority::from_severity(FailureSeverity::Medium), Priority::Medium);
        assert_eq!(Priority::from_severity(FailureSeverity::Low), Priority::Low);
    }
}
