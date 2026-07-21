/// Integration tests for PyRoboReplay
/// Tests end-to-end pipeline: generation → parsing → replay → queries

use pyroboreplay::adapters::{MissionAdapter, ros2::Ros2Adapter};
use pyroboreplay::core::Timeline;
use std::path::Path;

/// Helper to ensure test bag file exists
fn ensure_test_bag() -> String {
    let bag_path = "warehouse_exploration_v1.db3";
    if !Path::new(bag_path).exists() {
        panic!(
            "Test bag file not found: {}. Run: cargo run --example generate_warehouse_mission --release",
            bag_path
        );
    }
    bag_path.to_string()
}

#[test]
fn test_warehouse_mission_parsing() {
    let bag_path = ensure_test_bag();
    let adapter = Ros2Adapter::new();
    let mission = adapter
        .read(&bag_path)
        .expect("Failed to parse warehouse mission");

    // Validate mission properties
    assert!(!mission.name.is_empty(), "Mission name should not be empty");
    assert_eq!(
        mission.event_count(),
        96000,
        "Expected 96000 events in warehouse mission"
    );

    // Check duration
    if let Some(duration) = mission.duration() {
        assert!(duration.num_seconds() > 0, "Duration should be positive");
        println!("✅ Mission duration: {}s", duration.num_seconds());
    }
}

#[test]
fn test_event_type_breakdown() {
    let bag_path = ensure_test_bag();
    let adapter = Ros2Adapter::new();
    let mission = adapter
        .read(&bag_path)
        .expect("Failed to parse warehouse mission");

    // Count events by type
    let mut event_counts: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();

    for event in &mission.events {
        *event_counts.entry(event.event_type()).or_insert(0) += 1;
    }

    // Verify expected event distribution (10 min at specified frequencies)
    // Lidar: 10 Hz × 600s = 6000
    // Camera: 30 Hz × 600s = 18000
    // IMU: 100 Hz × 600s = 60000
    // Odometry: 20 Hz × 600s = 12000

    assert_eq!(
        event_counts.get("lidar_scan").copied().unwrap_or(0),
        6000,
        "Expected 6000 lidar frames"
    );
    assert_eq!(
        event_counts.get("camera_frame").copied().unwrap_or(0),
        18000,
        "Expected 18000 camera frames"
    );
    assert_eq!(
        event_counts.get("imu_data").copied().unwrap_or(0),
        60000,
        "Expected 60000 IMU frames"
    );
    assert_eq!(
        event_counts.get("odometry_update").copied().unwrap_or(0),
        12000,
        "Expected 12000 odometry updates"
    );

    println!("✅ Event type breakdown verified:");
    for (event_type, count) in &event_counts {
        println!("  {}: {}", event_type, count);
    }
}

#[test]
fn test_timeline_sensor_filtering() {
    let bag_path = ensure_test_bag();
    let adapter = Ros2Adapter::new();
    let mission = adapter
        .read(&bag_path)
        .expect("Failed to parse warehouse mission");

    let mut timeline = Timeline::new();
    let mission_id = mission.id.to_string();
    timeline.add_mission(mission);

    // Test lidar filtering
    let lidar_frames = timeline
        .get_sensor_frames(&mission_id, "lidar")
        .expect("Failed to get lidar frames");
    assert_eq!(lidar_frames.len(), 6000, "Expected 6000 lidar frames");

    // Test camera filtering
    let camera_frames = timeline
        .get_sensor_frames(&mission_id, "camera")
        .expect("Failed to get camera frames");
    assert_eq!(camera_frames.len(), 18000, "Expected 18000 camera frames");

    // Test IMU filtering
    let imu_frames = timeline
        .get_sensor_frames(&mission_id, "imu")
        .expect("Failed to get imu frames");
    assert_eq!(imu_frames.len(), 60000, "Expected 60000 imu frames");

    // Test odometry filtering
    let odom_frames = timeline
        .get_sensor_frames(&mission_id, "odometry")
        .expect("Failed to get odometry frames");
    assert_eq!(odom_frames.len(), 12000, "Expected 12000 odometry frames");

    println!("✅ Sensor filtering verified");
}

#[test]
fn test_available_sensors() {
    let bag_path = ensure_test_bag();
    let adapter = Ros2Adapter::new();
    let mission = adapter
        .read(&bag_path)
        .expect("Failed to parse warehouse mission");

    let mut timeline = Timeline::new();
    let mission_id = mission.id.to_string();
    timeline.add_mission(mission);

    let sensors = timeline
        .get_available_sensors(&mission_id)
        .expect("Failed to get available sensors");

    // Should have 4 sensor types
    assert_eq!(sensors.len(), 4, "Expected 4 sensor types");

    let sensor_set: std::collections::HashSet<_> = sensors.iter().cloned().collect();
    assert!(sensor_set.contains("lidar"), "Should have lidar sensor");
    assert!(sensor_set.contains("camera"), "Should have camera sensor");
    assert!(sensor_set.contains("imu"), "Should have imu sensor");
    assert!(sensor_set.contains("odometry"), "Should have odometry sensor");

    println!("✅ Available sensors: {}", sensors.join(", "));
}

#[test]
fn test_multi_sensor_query() {
    let bag_path = ensure_test_bag();
    let adapter = Ros2Adapter::new();
    let mission = adapter
        .read(&bag_path)
        .expect("Failed to parse warehouse mission");

    let mut timeline = Timeline::new();
    let mission_id = mission.id.to_string();
    timeline.add_mission(mission);

    // Query lidar + camera
    let frames = timeline
        .get_multi_sensor_frames(&mission_id, &["lidar", "camera"])
        .expect("Failed to get multi-sensor frames");

    assert_eq!(
        frames.len(),
        6000 + 18000,
        "Expected 24000 frames (6k lidar + 18k camera)"
    );

    println!("✅ Multi-sensor query: {} frames", frames.len());
}

#[test]
fn test_event_ordering() {
    let bag_path = ensure_test_bag();
    let adapter = Ros2Adapter::new();
    let mission = adapter
        .read(&bag_path)
        .expect("Failed to parse warehouse mission");

    // Events should be sorted by timestamp
    for i in 0..mission.events.len() - 1 {
        let curr_ts = mission.events[i].timestamp();
        let next_ts = mission.events[i + 1].timestamp();
        assert!(
            curr_ts <= next_ts,
            "Events should be sorted by timestamp"
        );
    }

    println!("✅ Event ordering verified");
}

#[test]
fn test_event_robot_id() {
    let bag_path = ensure_test_bag();
    let adapter = Ros2Adapter::new();
    let mission = adapter
        .read(&bag_path)
        .expect("Failed to parse warehouse mission");

    // All sensor events should have robot_id
    for event in &mission.events {
        if let Some(sensor_type) = event.sensor_type() {
            assert!(
                event.robot_id().is_some(),
                "Sensor event {} should have robot_id",
                sensor_type
            );
        }
    }

    println!("✅ Robot ID validation passed");
}

#[test]
fn test_performance_query_latency() {
    let bag_path = ensure_test_bag();
    let adapter = Ros2Adapter::new();
    let mission = adapter
        .read(&bag_path)
        .expect("Failed to parse warehouse mission");

    let mut timeline = Timeline::new();
    let mission_id = mission.id.to_string();
    timeline.add_mission(mission);

    // Measure query latency
    let start = std::time::Instant::now();
    let _lidar_frames = timeline
        .get_sensor_frames(&mission_id, "lidar")
        .expect("Failed to query sensor frames");
    let elapsed = start.elapsed();

    // Should complete in < 10ms (fast in-memory query)
    assert!(
        elapsed.as_millis() < 10,
        "Sensor query took {}ms (expected <10ms)",
        elapsed.as_millis()
    );

    println!(
        "✅ Query latency: {:.2}ms (target: <10ms)",
        elapsed.as_secs_f64() * 1000.0
    );
}

#[test]
fn test_performance_parsing_latency() {
    let bag_path = ensure_test_bag();

    let start = std::time::Instant::now();
    let adapter = Ros2Adapter::new();
    let _mission = adapter.read(&bag_path).expect("Failed to parse bag");
    let elapsed = start.elapsed();

    // 96k events should parse in < 5s
    assert!(
        elapsed.as_secs() < 5,
        "Parsing took {}s (expected <5s)",
        elapsed.as_secs()
    );

    println!(
        "✅ Parsing latency: {:.2}s (target: <5s) for 96k events",
        elapsed.as_secs_f64()
    );
}

#[test]
fn test_empty_sensor_query() {
    let bag_path = ensure_test_bag();
    let adapter = Ros2Adapter::new();
    let mission = adapter
        .read(&bag_path)
        .expect("Failed to parse warehouse mission");

    let mut timeline = Timeline::new();
    let mission_id = mission.id.to_string();
    timeline.add_mission(mission);

    // Query non-existent sensor should return empty
    let frames = timeline
        .get_sensor_frames(&mission_id, "nonexistent")
        .expect("Failed to query non-existent sensor");
    assert_eq!(frames.len(), 0, "Non-existent sensor should return empty");

    println!("✅ Empty sensor query handled correctly");
}

#[test]
fn test_mission_metadata() {
    let bag_path = ensure_test_bag();
    let adapter = Ros2Adapter::new();
    let mission = adapter
        .read(&bag_path)
        .expect("Failed to parse warehouse mission");

    // Verify mission has valid metadata
    assert!(!mission.id.to_string().is_empty(), "Mission ID should not be empty");
    assert!(!mission.name.is_empty(), "Mission name should not be empty");

    // Created at should be recent (within last minute)
    let now = chrono::Utc::now();
    let time_diff = now.signed_duration_since(mission.created_at);
    assert!(
        time_diff.num_seconds() < 60,
        "Mission should have been created recently"
    );

    println!(
        "✅ Mission metadata validated (ID: {}, Name: {})",
        mission.id, mission.name
    );
}
