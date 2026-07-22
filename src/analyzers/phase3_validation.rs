//! Phase 3 End-to-End Validation
//!
//! Tests complete pipeline: detection → aggregation → CLI output → feedback → recalibration → learning

#[cfg(test)]
mod tests {
    use crate::analyzers::{
        RealityGapFinding, Severity, RealityDomain, Evidence, MissionAnalysisData,
    };
    use crate::analyzers::aggregation::EvidenceAggregator;
    use crate::cli::consolidated_output::ConsolidatedFormatter;
    use crate::analyzers::feedback_loop::{FeedbackLoopManager, FindingFeedback, FeedbackEvent};
    use crate::analyzers::recalibration::RecalibrationEngine;
    use crate::analyzers::robot_calibration::RobotCalibrationManager;
    use crate::analyzers::scoring::RealityGapScorer;
    use std::collections::HashMap;

    fn create_test_mission(robot_type: &str) -> MissionAnalysisData {
        MissionAnalysisData {
            mission_id: format!("mission_{}", robot_type),
            duration_sec: 600.0,
            robot_type: robot_type.to_string(),
            control_messages: vec![],
            joint_states: vec![],
            odometry_messages: vec![],
            camera_frames: vec![],
            lidar_scans: vec![],
            imu_measurements: vec![],
            encoder_data: vec![],
            motor_currents: vec![],
            thermal_readings: vec![],
            battery_data: vec![],
            detection_results: vec![],
            perception_errors: vec![],
            message_timestamps: vec![],
        }
    }

    fn create_test_finding(
        category: &str,
        confidence: f32,
        gap_score: f32,
    ) -> RealityGapFinding {
        RealityGapFinding {
            domain: RealityDomain::Physical,
            category: category.to_string(),
            finding_type: format!("Test {}", category),
            severity: Severity::Medium,
            confidence,
            reality_gap_score: gap_score,
            description: "Test finding".to_string(),
            evidence: vec![Evidence {
                signal: "test_signal".to_string(),
                value: 0.5,
                timestamp: 100.0,
                confidence: 0.85,
            }],
            metrics: HashMap::new(),
            sim_recreation_suggestion: "Test sim".to_string(),
            remediation: "Test fix".to_string(),
            detection_time_sec: None,
        }
    }

    #[test]
    fn test_end_to_end_detection_to_feedback() {
        // Phase 1: Detect gaps
        let mission = create_test_mission("mobile_robot");
        let findings = vec![
            create_test_finding("Mechanical Degradation", 0.8, 0.7),
            create_test_finding("Mechanical Degradation", 0.75, 0.75),
            create_test_finding("Thermal Effects", 0.7, 0.6),
        ];

        // Phase 2: Aggregate findings
        let consolidated = EvidenceAggregator::aggregate(findings.clone());
        assert_eq!(consolidated.len(), 2); // 2 root causes

        // Phase 3: Format for output
        let formatter = ConsolidatedFormatter::new(true);
        let text_output = formatter.format_text(&consolidated);
        assert!(text_output.contains("Consolidated Reality Gap Analysis"));

        let json_output = formatter.format_json(&consolidated);
        assert_eq!(json_output["summary"]["total_consolidated"], 2);

        // Phase 4: Record findings and feedback
        let mut feedback_mgr = FeedbackLoopManager::new();
        let finding_ids = feedback_mgr.record_findings(&findings, &mission);
        assert_eq!(finding_ids.len(), 3);

        feedback_mgr.record_mission(&mission, false); // Failed mission

        // Submit feedback
        for (idx, id) in finding_ids.iter().enumerate() {
            let feedback = FindingFeedback {
                finding_id: *id,
                mission_id: mission.mission_id.clone(),
                feedback_event: if idx == 0 {
                    FeedbackEvent::VerifiedCorrect("Bearing Wear".to_string())
                } else if idx == 1 {
                    FeedbackEvent::PartiallyCorrect("Wear".to_string())
                } else {
                    FeedbackEvent::Inconclusive
                },
                feedback_timestamp: 100.0,
                additional_notes: "Verified in visual inspection".to_string(),
            };
            feedback_mgr.submit_feedback(feedback);
        }

        let accuracy = feedback_mgr.feedback_accuracy();
        assert!(accuracy >= 0.33); // 1 correct + 0.5 partial + 0 incorrect = 1.5 / 3 ≈ 0.5
    }

    #[test]
    fn test_end_to_end_recalibration() {
        // Record feedback to recalibrate priors
        let mut recal_engine = RecalibrationEngine::new();
        let scorer = RealityGapScorer::new();
        recal_engine.initialize_from_scorer(&scorer);
        recal_engine.set_min_samples(3);

        // Simulate 5 instances of Mechanical Degradation detection
        for _ in 0..5 {
            recal_engine.record_feedback("Mechanical Degradation", "correct");
        }

        // Ready to recalibrate
        assert!(recal_engine.is_ready_to_recalibrate("Mechanical Degradation"));

        let (before, after) = recal_engine
            .recalibrate_category("Mechanical Degradation")
            .unwrap();

        // Prior should increase (all correct feedback)
        assert!(after > before);
        assert!(after > 0.75);
    }

    #[test]
    fn test_end_to_end_robot_calibration() {
        let mut robot_mgr = RobotCalibrationManager::new();

        // Record missions for wheel_robot
        for _ in 0..8 {
            robot_mgr.record_mission("wheel_robot", true);
        }
        robot_mgr.record_mission("wheel_robot", false); // 1 failure
        robot_mgr.record_mission("wheel_robot", false); // 2 failures

        // Record gaps
        robot_mgr.record_gap("wheel_robot", "Mechanical Degradation", 0.7);
        robot_mgr.record_gap("wheel_robot", "Mechanical Degradation", 0.75);
        robot_mgr.record_gap("wheel_robot", "Thermal Effects", 0.6);

        robot_mgr.calibrate_sensitivities();
        robot_mgr.learn_severity_threshold("wheel_robot");

        let profile = robot_mgr.get_profile("wheel_robot").unwrap();
        assert_eq!(profile.mission_count, 10);
        assert_eq!(profile.failure_count, 2);
        assert!((profile.failure_rate() - 0.2).abs() < 0.01);

        // Severity threshold should reflect failure rate: 0.5 + 0.4*(1-0.2) = 0.82
        // With 20% failure rate, threshold should be relatively high
        assert!(profile.learned_severity_threshold > 0.8);

        // Mechanical sensitivity should be slightly boosted (2 gaps recorded)
        // frequency = 0.02, sensitivity = 1.0 + 0.02*0.5 ≈ 1.01
        assert!(profile.mechanical_sensitivity > 1.0);
    }

    #[test]
    fn test_end_to_end_fleet_learning() {
        let mut robot_mgr = RobotCalibrationManager::new();
        let mut recal_engine = RecalibrationEngine::new();
        let scorer = RealityGapScorer::new();
        recal_engine.initialize_from_scorer(&scorer);

        // Simulate 2 robot types with different gap patterns
        // mobile_robot: high mechanical issues
        for _ in 0..7 {
            robot_mgr.record_mission("mobile_robot", true);
        }
        robot_mgr.record_mission("mobile_robot", false);

        for _ in 0..6 {
            robot_mgr.record_gap("mobile_robot", "Mechanical Degradation", 0.8);
        }

        // drone: high thermal issues
        for _ in 0..8 {
            robot_mgr.record_mission("drone", true);
        }

        for _ in 0..5 {
            robot_mgr.record_gap("drone", "Thermal Effects", 0.75);
        }

        robot_mgr.calibrate_sensitivities();

        // Check fleet statistics
        let stats = robot_mgr.fleet_statistics();
        assert_eq!(stats.robot_type_count, 2);
        assert_eq!(stats.total_missions, 16);

        // Each robot type should have different sensitivity profiles
        let mobile = robot_mgr.get_profile("mobile_robot").unwrap();
        let drone = robot_mgr.get_profile("drone").unwrap();

        // mobile_robot should have mechanical sensitivity > drone
        assert!(mobile.mechanical_sensitivity > drone.mechanical_sensitivity);

        // drone should have thermal sensitivity > mobile
        assert!(drone.thermal_sensitivity > mobile.thermal_sensitivity);
    }

    #[test]
    fn test_feedback_improves_recalibration() {
        let mut recal_engine = RecalibrationEngine::new();
        let scorer = RealityGapScorer::new();
        recal_engine.initialize_from_scorer(&scorer);
        recal_engine.set_min_samples(5);
        recal_engine.set_learning_rate(0.15);

        // Initial confidence in category
        let initial_conf = recal_engine.recalibration_confidence("Optical Contamination");

        // Record 5 feedback items
        for _ in 0..5 {
            recal_engine.record_feedback("Optical Contamination", "correct");
        }

        // Confidence should increase with more data
        let post_feedback_conf =
            recal_engine.recalibration_confidence("Optical Contamination");
        assert!(post_feedback_conf > initial_conf);

        // Recalibrate
        let (before, after) = recal_engine
            .recalibrate_category("Optical Contamination")
            .unwrap();

        // All correct → prior should increase significantly
        assert!(after > before);
    }

    #[test]
    fn test_severity_adjustment_by_robot_type() {
        let mut robot_mgr = RobotCalibrationManager::new();

        // Create two robot types with different failure patterns
        // reliable_robot: low failures
        for _ in 0..19 {
            robot_mgr.record_mission("reliable_robot", true);
        }
        robot_mgr.record_mission("reliable_robot", false);

        // unreliable_robot: high failures
        for _ in 0..5 {
            robot_mgr.record_mission("unreliable_robot", true);
        }
        for _ in 0..5 {
            robot_mgr.record_mission("unreliable_robot", false);
        }

        robot_mgr.learn_severity_threshold("reliable_robot");
        robot_mgr.learn_severity_threshold("unreliable_robot");

        let reliable = robot_mgr.get_profile("reliable_robot").unwrap();
        let unreliable = robot_mgr.get_profile("unreliable_robot").unwrap();

        // Reliable robot should have higher threshold (less strict, avoids false positives)
        assert!(reliable.learned_severity_threshold > unreliable.learned_severity_threshold);

        // Verify thresholds reflect failure rates
        // reliable: 5% failure -> high threshold ≈ 0.85
        // unreliable: 50% failure -> low threshold ≈ 0.7
        assert!(reliable.learned_severity_threshold > 0.82);
        assert!(unreliable.learned_severity_threshold < 0.72);
    }

    #[test]
    fn test_full_pipeline_accuracy_improvement() {
        // Simulate running 3 mission cycles
        let mut feedback_mgr = FeedbackLoopManager::new();
        let mut recal_engine = RecalibrationEngine::new();
        let scorer = RealityGapScorer::new();
        recal_engine.initialize_from_scorer(&scorer);
        recal_engine.set_min_samples(3);

        // Cycle 1: 3 missions, collect feedback
        for cycle in 0..3 {
            let mission = create_test_mission("mobile_robot");
            let findings = vec![create_test_finding("Mechanical Degradation", 0.8, 0.7)];

            let ids = feedback_mgr.record_findings(&findings, &mission);
            feedback_mgr.record_mission(&mission, cycle == 2); // Last one succeeds

            // All feedback is correct
            for id in ids {
                let fb = FindingFeedback {
                    finding_id: id,
                    mission_id: mission.mission_id.clone(),
                    feedback_event: FeedbackEvent::VerifiedCorrect("Root".to_string()),
                    feedback_timestamp: 0.0,
                    additional_notes: String::new(),
                };
                feedback_mgr.submit_feedback(fb);
                recal_engine.record_feedback("Mechanical Degradation", "correct");
            }
        }

        // Check accuracy
        let accuracy = feedback_mgr.feedback_accuracy();
        assert!(accuracy > 0.95); // All correct

        // Recalibration ready?
        assert!(recal_engine.is_ready_to_recalibrate("Mechanical Degradation"));

        // Recalibrate
        let (before, after) = recal_engine
            .recalibrate_category("Mechanical Degradation")
            .unwrap();

        assert!(after > before);
    }
}
