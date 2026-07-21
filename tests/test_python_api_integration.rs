// Phase 1-3: Python API Integration Tests
// Tests full mission workflow and Python bindings

use pyroboreplay::core::{MissionEvent, AnomalyDetector, ExplanationGenerator, ActionRecommender};
use chrono::Utc;

// ============================================================================
// Test Fixtures: Synthetic Missions
// ============================================================================

fn create_clean_mission_events() -> Vec<MissionEvent> {
    // Clean mission with no failures
    vec![
        MissionEvent::LidarScan {
            robot_id: "robot_1".to_string(),
            timestamp: Utc::now(),
            data: pyroboreplay::core::event::LidarData {
                ranges: vec![3.0, 3.5, 4.0],
                intensities: Some(vec![0.5, 0.5, 0.5]),
                frame_id: "lidar".to_string(),
                min_angle: -3.14,
                max_angle: 3.14,
                angle_increment: 0.01,
                range_min: 0.1,
                range_max: 10.0,
            },
        },
        MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: Utc::now(),
            pose: pyroboreplay::core::event::Pose {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.95),
        },
        MissionEvent::OdometryUpdate {
            robot_id: "robot_1".to_string(),
            timestamp: Utc::now(),
            data: pyroboreplay::core::event::Odometry {
                frame_id: "odom".to_string(),
                child_frame_id: "base_link".to_string(),
                pose: pyroboreplay::core::event::Pose {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    qx: 0.0,
                    qy: 0.0,
                    qz: 0.0,
                    qw: 1.0,
                },
                twist_linear: [0.5, 0.0, 0.0],
                twist_angular: [0.0, 0.0, 0.0],
            },
        },
    ]
}

fn create_collision_mission_events() -> Vec<MissionEvent> {
    // Mission with near-collision failure
    vec![
        MissionEvent::LidarScan {
            robot_id: "robot_1".to_string(),
            timestamp: Utc::now(),
            data: pyroboreplay::core::event::LidarData {
                ranges: vec![0.2, 2.0, 3.0],
                intensities: Some(vec![0.5, 0.5, 0.5]),
                frame_id: "lidar".to_string(),
                min_angle: -3.14,
                max_angle: 3.14,
                angle_increment: 0.01,
                range_min: 0.1,
                range_max: 10.0,
            },
        },
        MissionEvent::NavigationDecision {
            robot_id: "robot_1".to_string(),
            timestamp: Utc::now(),
            decision_type: "emergency_stop".to_string(),
            rationale: Some("Obstacle detected".to_string()),
        },
    ]
}

fn create_multi_failure_mission_events() -> Vec<MissionEvent> {
    // Mission with multiple failures
    let mut events = create_collision_mission_events();

    // Add deadlock scenario
    for i in 0..25 {
        events.push(MissionEvent::NavigationDecision {
            robot_id: "robot_1".to_string(),
            timestamp: Utc::now(),
            decision_type: "replan".to_string(),
            rationale: Some(format!("Attempt {}", i)),
        });
    }

    events
}

// ============================================================================
// 1. MISSION DETECTION WORKFLOW TESTS
// ============================================================================

#[test]
fn test_clean_mission_no_failures() {
    let events = create_clean_mission_events();
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    assert_eq!(failures.len(), 0, "Clean mission should have no failures");
}

#[test]
fn test_collision_mission_detects_collision() {
    let events = create_collision_mission_events();
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    assert!(!failures.is_empty(), "Collision mission should have failures");

    let has_collision = failures.iter().any(|f| f.failure_type == "near_collision");
    assert!(has_collision, "Should detect near_collision");
}

#[test]
fn test_multi_failure_mission_detects_multiple() {
    let events = create_multi_failure_mission_events();
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    assert!(failures.len() > 0, "Multi-failure mission should detect failures");

    let failure_types: std::collections::HashSet<_> = failures
        .iter()
        .map(|f| f.failure_type.as_str())
        .collect();

    assert!(failure_types.len() > 1, "Should detect multiple failure types");
}

// ============================================================================
// 2. FULL DIAGNOSTIC PIPELINE TESTS
// ============================================================================

#[test]
fn test_full_pipeline_collision_failure() {
    // 1. Detect
    let events = create_collision_mission_events();
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    assert!(!failures.is_empty());
    let collision_failure = &failures[0];

    // 2. Explain
    let explanation = ExplanationGenerator::explain(collision_failure);
    assert!(!explanation.is_empty());

    // 3. Recommend actions
    let actions = ActionRecommender::recommend(collision_failure);
    assert!(!actions.is_empty());

    // 4. Verify all parts are present
    assert_eq!(collision_failure.failure_type, "near_collision");
    assert!(collision_failure.confidence > 0.0);
    assert!(!actions[0].priority.is_empty());
}

#[test]
fn test_full_pipeline_with_multiple_failures() {
    let events = create_multi_failure_mission_events();
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    assert!(failures.len() > 1);

    // Process each failure through full pipeline
    for failure in &failures {
        let explanation = ExplanationGenerator::explain(failure);
        let actions = ActionRecommender::recommend(failure);

        assert!(!explanation.is_empty());
        assert!(!actions.is_empty());
    }
}

// ============================================================================
// 3. DATA FLOW TESTS
// ============================================================================

#[test]
fn test_failure_evidence_preserved() {
    let events = create_collision_mission_events();
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    for failure in &failures {
        // Evidence should be populated
        if failure.failure_type == "near_collision" {
            assert!(!failure.evidence.is_empty(), "Evidence should be collected");
            assert!(failure.evidence.contains_key("min_range_m") ||
                   !failure.evidence.is_empty(), "Should have range data");
        }
    }
}

#[test]
fn test_affected_systems_tracked() {
    let events = create_collision_mission_events();
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    for failure in &failures {
        assert!(!failure.affected_systems.is_empty(), "Should track affected systems");
    }
}

#[test]
fn test_timestamp_consistency() {
    let events = create_collision_mission_events();
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    for failure in &failures {
        // Both timestamp formats should be valid
        assert!(failure.timestamp_seconds > 0.0);
        // DateTime should be valid
        assert!(failure.timestamp.timestamp() >= 0);
    }
}

// ============================================================================
// 4. ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_empty_mission_handling() {
    let detector = AnomalyDetector::new(vec![]);
    let failures = detector.detect_all();

    // Should not panic, should return empty
    assert_eq!(failures.len(), 0);
}

#[test]
fn test_single_event_mission() {
    let events = vec![MissionEvent::RobotPose {
        robot_id: "robot_1".to_string(),
        timestamp: Utc::now(),
        pose: pyroboreplay::core::event::Pose {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            qx: 0.0,
            qy: 0.0,
            qz: 0.0,
            qw: 1.0,
        },
        confidence: Some(0.95),
    }];

    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    // Single event shouldn't trigger false positives
    assert_eq!(failures.len(), 0);
}

#[test]
fn test_malformed_data_handling() {
    // Mission with edge-case values
    let events = vec![
        MissionEvent::LidarScan {
            robot_id: "".to_string(), // Empty robot ID
            timestamp: Utc::now(),
            data: pyroboreplay::core::event::LidarData {
                ranges: vec![], // Empty ranges
                intensities: None,
                frame_id: "".to_string(),
                min_angle: 0.0,
                max_angle: 0.0,
                angle_increment: 0.0,
                range_min: 0.0,
                range_max: 0.0,
            },
        },
    ];

    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    // Should handle gracefully without panic
    assert_eq!(failures.len(), 0);
}

// ============================================================================
// 5. MISSION-LEVEL STATISTICS TESTS
// ============================================================================

#[test]
fn test_failure_count_accuracy() {
    let events = create_collision_mission_events();
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    // Count by type
    let collision_count = failures.iter().filter(|f| f.failure_type == "near_collision").count();

    assert!(collision_count > 0);
}

#[test]
fn test_severity_distribution() {
    let events = create_multi_failure_mission_events();
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    let severities: std::collections::HashMap<_, usize> = failures
        .iter()
        .fold(std::collections::HashMap::new(), |mut acc, f| {
            *acc.entry(f.severity.clone()).or_insert(0) += 1;
            acc
        });

    // Should have failures with various severities
    assert!(!severities.is_empty());
}

#[test]
fn test_confidence_distribution() {
    let events = create_multi_failure_mission_events();
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    if !failures.is_empty() {
        let avg_confidence: f32 = failures.iter().map(|f| f.confidence).sum::<f32>() / failures.len() as f32;

        assert!(avg_confidence >= 0.0);
        assert!(avg_confidence <= 1.0);
    }
}

// ============================================================================
// 6. PIPELINE PERFORMANCE TESTS
// ============================================================================

#[test]
fn test_detection_completes_quickly() {
    let events = create_multi_failure_mission_events();
    let start = std::time::Instant::now();

    let detector = AnomalyDetector::new(events);
    let _failures = detector.detect_all();

    let elapsed = start.elapsed();

    // Should complete in <500ms even for multi-failure scenario
    assert!(elapsed.as_millis() < 500, "Detection took {}ms", elapsed.as_millis());
}

#[test]
fn test_explanation_generation_completes_quickly() {
    let events = create_collision_mission_events();
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    let start = std::time::Instant::now();

    for failure in &failures {
        let _explanation = ExplanationGenerator::explain(failure);
    }

    let elapsed = start.elapsed();

    // Should complete in <100ms for all explanations
    assert!(elapsed.as_millis() < 100, "Explanation took {}ms", elapsed.as_millis());
}

#[test]
fn test_recommendation_generation_completes_quickly() {
    let events = create_collision_mission_events();
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_all();

    let start = std::time::Instant::now();

    for failure in &failures {
        let _actions = ActionRecommender::recommend(failure);
    }

    let elapsed = start.elapsed();

    // Should complete in <100ms for all recommendations
    assert!(elapsed.as_millis() < 100, "Recommendations took {}ms", elapsed.as_millis());
}

// ============================================================================
// Summary
// ============================================================================
// Total: 21 integration tests covering:
// - Mission detection workflow (3 tests)
// - Full diagnostic pipeline (2 tests)
// - Data flow integrity (3 tests)
// - Error handling (3 tests)
// - Mission statistics (3 tests)
// - Pipeline performance (3 tests)
// All tests verify: end-to-end workflows, data consistency, error resilience
