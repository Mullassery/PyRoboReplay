use pyroboreplay::core::{
    MissionEvent, MissionRecord, Pose, Location,
    IncidentAnalysisOrchestrator, IncidentBundle, BundleManifest,
    ConfidenceScoringEngine, ConfidenceTier,
    Priority, TimeRange,
};
use chrono::Utc;
use std::path::PathBuf;
use std::collections::HashMap;

#[test]
fn test_mlrias_integration_basic_flow() {
    // Create a minimal incident bundle
    let manifest = BundleManifest {
        bundle_id: "test_incident_001".to_string(),
        created_at: Utc::now(),
        robot_ids: vec!["robot_1".to_string()],
        mission_type: Some("navigation".to_string()),
        failure_type_suspected: Some("timeout".to_string()),
        time_range: Some(TimeRange {
            start: Utc::now(),
            end: Utc::now(),
        }),
        layers_available: Default::default(),
        detected_issues: vec!["planner_timeout".to_string()],
        file_inventory: Default::default(),
        checksums: HashMap::new(),
    };

    let bundle = IncidentBundle {
        bundle_id: "test_incident_001".to_string(),
        bundle_path: PathBuf::from("/tmp/test_incident.zip"),
        manifest,
    };

    // Create a mission with test events
    let mut mission = MissionRecord::new("test_mission");
    let now = Utc::now();

    // Add some test events
    for i in 0..10 {
        let event = MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: now + chrono::Duration::milliseconds(i * 100),
            pose: Pose {
                x: i as f64,
                y: i as f64,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.95),
        };
        mission.add_event(event);
    }

    // Create orchestrator
    let mut orchestrator = IncidentAnalysisOrchestrator::new(bundle, mission.events);

    // Analyze the incident
    let report = orchestrator.analyze().expect("Analysis should succeed");

    // Verify report structure
    assert_eq!(report.bundle_id, "test_incident_001");
    assert_eq!(report.robots_involved.len(), 1);
    assert_eq!(report.robots_involved[0], "robot_1");
    assert!(report.analysis_summary.total_events_analyzed > 0);
}

#[test]
fn test_confidence_scoring_with_events() {
    // Test confidence tier classification
    assert_eq!(ConfidenceTier::classify(1.0), ConfidenceTier::Fact);
    assert_eq!(ConfidenceTier::classify(0.95), ConfidenceTier::Fact);
    assert_eq!(ConfidenceTier::classify(0.70), ConfidenceTier::HighInference);
    assert_eq!(ConfidenceTier::classify(0.50), ConfidenceTier::Hypothesis);
    assert_eq!(ConfidenceTier::classify(0.20), ConfidenceTier::Speculative);
}

#[test]
fn test_recommendation_priority_ordering() {
    // Verify priority ordering works correctly
    assert!(Priority::Critical > Priority::High);
    assert!(Priority::High > Priority::Medium);
    assert!(Priority::Medium > Priority::Low);

    assert_eq!(Priority::Critical.as_str(), "critical");
    assert_eq!(Priority::High.as_str(), "high");
}

#[test]
fn test_mission_event_with_timestamps() {
    let base_time = Utc::now();
    let mut mission = MissionRecord::new("timestamp_test");

    // Add events with different timestamps
    for i in 0..5 {
        let event = MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: base_time + chrono::Duration::seconds(i),
            pose: Pose {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.95),
        };
        mission.add_event(event);
    }

    // Verify event ordering
    mission.sort_by_timestamp();
    assert_eq!(mission.event_count(), 5);

    // Verify first and last timestamps
    let first_ts = mission.events.first().unwrap().timestamp();
    let last_ts = mission.events.last().unwrap().timestamp();
    assert_eq!(first_ts, base_time);
    assert_eq!(last_ts, base_time + chrono::Duration::seconds(4));
}

#[test]
fn test_multiple_robot_incident() {
    let manifest = BundleManifest {
        bundle_id: "multi_robot_incident".to_string(),
        created_at: Utc::now(),
        robot_ids: vec!["robot_1".to_string(), "robot_2".to_string(), "robot_3".to_string()],
        mission_type: Some("multi_robot_coordination".to_string()),
        failure_type_suspected: Some("coordination_failure".to_string()),
        time_range: Some(TimeRange {
            start: Utc::now(),
            end: Utc::now(),
        }),
        layers_available: Default::default(),
        detected_issues: vec!["communication_loss".to_string()],
        file_inventory: Default::default(),
        checksums: HashMap::new(),
    };

    let bundle = IncidentBundle {
        bundle_id: "multi_robot_incident".to_string(),
        bundle_path: PathBuf::from("/tmp/multi_robot.zip"),
        manifest,
    };

    let mut mission = MissionRecord::new("multi_robot_mission");

    // Add events from multiple robots
    let now = Utc::now();
    for robot_id in &["robot_1", "robot_2", "robot_3"] {
        for i in 0..5 {
            let event = MissionEvent::RobotPose {
                robot_id: robot_id.to_string(),
                timestamp: now + chrono::Duration::milliseconds(i * 100),
                pose: Pose {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                    qx: 0.0,
                    qy: 0.0,
                    qz: 0.0,
                    qw: 1.0,
                },
                confidence: Some(0.95),
            };
            mission.add_event(event);
        }
    }

    let mut orchestrator = IncidentAnalysisOrchestrator::new(bundle, mission.events);
    let report = orchestrator.analyze().expect("Analysis should succeed");

    // Verify multi-robot detection
    assert_eq!(report.robots_involved.len(), 3);
}

#[test]
fn test_incident_bundle_manifest() {
    let manifest = BundleManifest {
        bundle_id: "test_bundle".to_string(),
        created_at: Utc::now(),
        robot_ids: vec!["test_robot".to_string()],
        mission_type: Some("exploration".to_string()),
        failure_type_suspected: Some("sensor_failure".to_string()),
        time_range: Some(TimeRange {
            start: Utc::now(),
            end: Utc::now() + chrono::Duration::seconds(300),
        }),
        layers_available: Default::default(),
        detected_issues: vec!["lidar_dropout".to_string(), "high_cpu_load".to_string()],
        file_inventory: Default::default(),
        checksums: HashMap::new(),
    };

    assert_eq!(manifest.bundle_id, "test_bundle");
    assert_eq!(manifest.robot_ids[0], "test_robot");
    assert_eq!(manifest.detected_issues.len(), 2);

    // Test duration calculation
    if let Some(tr) = &manifest.time_range {
        assert_eq!(tr.duration_seconds(), 300);
    }
}

#[test]
fn test_sensor_event_creation() {
    // Test LiDAR event
    let lidar_event = MissionEvent::LidarScan {
        robot_id: "robot_1".to_string(),
        timestamp: Utc::now(),
        data: pyroboreplay::core::event::LidarData {
            ranges: vec![1.0, 1.5, 2.0],
            intensities: Some(vec![100.0, 150.0, 200.0]),
            frame_id: "laser_0".to_string(),
            min_angle: -3.14,
            max_angle: 3.14,
            angle_increment: 0.01,
            range_min: 0.0,
            range_max: 30.0,
        },
    };

    assert_eq!(lidar_event.event_type(), "lidar_scan");
    assert_eq!(lidar_event.sensor_type(), Some("lidar"));
    assert_eq!(lidar_event.robot_id(), Some("robot_1"));

    // Test Camera event
    let camera_event = MissionEvent::CameraFrame {
        robot_id: "robot_1".to_string(),
        timestamp: Utc::now(),
        data: pyroboreplay::core::event::CameraFrame {
            sensor_id: "camera_0".to_string(),
            frame_id: "camera_frame".to_string(),
            width: 640,
            height: 480,
            encoding: "rgb8".to_string(),
            image_data: vec![0u8; 640 * 480 * 3],
            camera_info: None,
        },
    };

    assert_eq!(camera_event.event_type(), "camera_frame");
    assert_eq!(camera_event.sensor_type(), Some("camera"));
}

#[test]
fn test_communication_event_creation() {
    let comm_event = MissionEvent::CommunicationEvent {
        timestamp: Utc::now(),
        from: "robot_1".to_string(),
        to: "robot_2".to_string(),
        event_type: "discovery_request".to_string(),
        data: Some(serde_json::json!({"attempt": 1})),
    };

    assert_eq!(comm_event.event_type(), "communication_event");
    assert_eq!(comm_event.robot_id(), None); // Communication events don't have single robot
}

#[test]
fn test_mission_duration_calculation() {
    let base_time = Utc::now();
    let mut mission = MissionRecord::new("duration_test");

    // Add events spanning 10 seconds
    for i in 0..11 {
        let event = MissionEvent::RobotPose {
            robot_id: "robot_1".to_string(),
            timestamp: base_time + chrono::Duration::seconds(i),
            pose: Pose {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            confidence: Some(0.95),
        };
        mission.add_event(event);
    }

    let duration = mission.duration().expect("Duration should be calculable");
    assert_eq!(duration, chrono::Duration::seconds(10));
}

#[test]
fn test_environmental_change_event() {
    let env_event = MissionEvent::EnvironmentalChange {
        timestamp: Utc::now(),
        location: Location {
            x: 10.0,
            y: 20.0,
            z: 0.0,
        },
        change_type: "obstacle_appeared".to_string(),
        description: Some("Dynamic obstacle detected at waypoint".to_string()),
    };

    assert_eq!(env_event.event_type(), "environmental_change");
    assert_eq!(env_event.robot_id(), None);
}

#[test]
fn test_mission_lifecycle_events() {
    let start_event = MissionEvent::MissionLifecycle {
        timestamp: Utc::now(),
        event_type: "start".to_string(),
        mission_id: "mission_001".to_string(),
    };

    let end_event = MissionEvent::MissionLifecycle {
        timestamp: Utc::now(),
        event_type: "end".to_string(),
        mission_id: "mission_001".to_string(),
    };

    assert_eq!(start_event.event_type(), "mission_lifecycle");
    assert_eq!(end_event.event_type(), "mission_lifecycle");
}

#[test]
fn test_obstacle_detected_event() {
    let obstacle_event = MissionEvent::ObstacleDetected {
        robot_id: "robot_1".to_string(),
        timestamp: Utc::now(),
        location: Location {
            x: 5.0,
            y: 10.0,
            z: 0.5,
        },
        obstacle_type: "person".to_string(),
        confidence: Some(0.85),
    };

    assert_eq!(obstacle_event.event_type(), "obstacle_detected");
    assert_eq!(obstacle_event.robot_id(), Some("robot_1"));
}

#[test]
fn test_navigation_decision_event() {
    let nav_event = MissionEvent::NavigationDecision {
        robot_id: "robot_1".to_string(),
        timestamp: Utc::now(),
        decision_type: "replan".to_string(),
        rationale: Some("Obstacle detected in planned path".to_string()),
    };

    assert_eq!(nav_event.event_type(), "navigation_decision");
    assert_eq!(nav_event.robot_id(), Some("robot_1"));
}
