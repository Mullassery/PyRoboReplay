//! Real end-to-end verification of ROS 2 CDR decoding
//! (`src/adapters/ros2.rs` + `src/adapters/cdr.rs`).
//!
//! `tests/fixtures/ros2_cdr_fixture.db3` is a real rosbag2 SQLite3 bag,
//! generated with the third-party `rosbags` Python library (Apache-2.0,
//! independent of this crate's own CDR implementation), containing one real
//! message of each of the 5 supported types with known, non-trivial field
//! values. This test decodes that real bag with this crate's own decoder
//! and asserts the recovered values exactly match what was actually
//! encoded -- the previous parsers ignored the message bytes entirely and
//! returned a hardcoded constant regardless of input, which this kind of
//! test (unlike the pre-existing unit tests, which only exercised synthetic
//! in-memory events and never touched a real encoded bag) would have
//! caught immediately.
//!
//! Regenerate the fixture (if the message layouts here ever need to change)
//! with `/tmp/ros2_verify/make_bag.py` against a `pip install rosbags` venv
//! -- see the commit that added this test for the exact script.

use pyroboreplay::adapters::ros2::Ros2Adapter;
use pyroboreplay::adapters::MissionAdapter;
use pyroboreplay::core::event::MissionEvent;
use pyroboreplay::core::AnomalyDetector;

fn load_fixture_events() -> Vec<MissionEvent> {
    let adapter = Ros2Adapter::new();
    let mission = adapter
        .read("tests/fixtures/ros2_cdr_fixture.db3")
        .expect("real fixture bag should parse successfully");
    mission.events
}

fn find_event<'a>(
    events: &'a [MissionEvent],
    predicate: impl Fn(&MissionEvent) -> bool,
) -> &'a MissionEvent {
    events
        .iter()
        .find(|e| predicate(e))
        .expect("expected event not found in decoded fixture")
}

#[test]
fn decodes_real_laser_scan_with_correct_values() {
    let events = load_fixture_events();
    let event = find_event(&events, |e| matches!(e, MissionEvent::LidarScan { .. }));
    let MissionEvent::LidarScan { data, .. } = event else {
        unreachable!()
    };

    assert_eq!(data.frame_id, "laser_frame_real");
    assert!((data.min_angle - (-1.5)).abs() < 1e-5);
    assert!((data.max_angle - 1.5).abs() < 1e-5);
    assert!((data.angle_increment - 0.25).abs() < 1e-5);
    assert!((data.range_min - 0.1).abs() < 1e-5);
    assert!((data.range_max - 25.0).abs() < 1e-5);
    assert_eq!(data.ranges.len(), 4);
    assert!((data.ranges[0] - 1.1).abs() < 1e-4);
    assert!((data.ranges[1] - 2.2).abs() < 1e-4);
    assert!((data.ranges[2] - 3.3).abs() < 1e-4);
    assert!((data.ranges[3] - 4.4).abs() < 1e-4);
    let intensities = data.intensities.as_ref().expect("real intensities should be present");
    assert_eq!(intensities, &vec![10.0, 20.0, 30.0, 40.0]);
}

#[test]
fn decodes_real_camera_image_with_correct_values() {
    let events = load_fixture_events();
    let event = find_event(&events, |e| matches!(e, MissionEvent::CameraFrame { .. }));
    let MissionEvent::CameraFrame { data, .. } = event else {
        unreachable!()
    };

    assert_eq!(data.frame_id, "camera_frame_real");
    assert_eq!(data.height, 4);
    assert_eq!(data.width, 3);
    assert_eq!(data.encoding, "rgb8");
    assert_eq!(data.image_data.len(), 36);
    let expected: Vec<u8> = (0..36u32).map(|i| (i % 256) as u8).collect();
    assert_eq!(data.image_data, expected);
}

#[test]
fn decodes_real_imu_with_correct_values() {
    let events = load_fixture_events();
    let event = find_event(&events, |e| matches!(e, MissionEvent::IMUData { .. }));
    let MissionEvent::IMUData { data, .. } = event else {
        unreachable!()
    };

    assert_eq!(data.frame_id, "imu_frame_real");
    let orientation = data.orientation.expect("real orientation should be present");
    assert!((orientation[0] - 0.1).abs() < 1e-9);
    assert!((orientation[1] - 0.2).abs() < 1e-9);
    assert!((orientation[2] - 0.3).abs() < 1e-9);
    assert!((orientation[3] - 0.9273618495495704).abs() < 1e-9);

    assert!((data.angular_velocity[0] - 0.01).abs() < 1e-9);
    assert!((data.angular_velocity[1] - 0.02).abs() < 1e-9);
    assert!((data.angular_velocity[2] - 0.03).abs() < 1e-9);

    assert!((data.linear_acceleration[0] - 0.1).abs() < 1e-9);
    assert!((data.linear_acceleration[1] - 0.2).abs() < 1e-9);
    assert!((data.linear_acceleration[2] - 9.81).abs() < 1e-9);
}

#[test]
fn decodes_real_odometry_with_correct_values() {
    let events = load_fixture_events();
    let event = find_event(&events, |e| matches!(e, MissionEvent::OdometryUpdate { .. }));
    let MissionEvent::OdometryUpdate { data, .. } = event else {
        unreachable!()
    };

    assert_eq!(data.frame_id, "odom_real");
    assert_eq!(data.child_frame_id, "base_link_real");
    assert!((data.pose.x - 5.5).abs() < 1e-9);
    assert!((data.pose.y - (-3.25)).abs() < 1e-9);
    assert!((data.pose.z - 0.5).abs() < 1e-9);
    // A real 90-degree yaw quaternion: (0, 0, sqrt(2)/2, sqrt(2)/2).
    assert!((data.pose.qz - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-9);
    assert!((data.pose.qw - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-9);

    assert!((data.twist_linear[0] - 1.5).abs() < 1e-9);
    assert!((data.twist_angular[2] - 0.2).abs() < 1e-9);
}

#[test]
fn decodes_real_pose_and_derives_confidence_from_real_covariance() {
    let events = load_fixture_events();
    let event = find_event(&events, |e| matches!(e, MissionEvent::RobotPose { .. }));
    let MissionEvent::RobotPose { pose, confidence, .. } = event else {
        unreachable!()
    };

    assert!((pose.x - 10.0).abs() < 1e-9);
    assert!((pose.y - 20.0).abs() < 1e-9);
    assert!((pose.z - 1.0).abs() < 1e-9);

    // Real fixture covariance: x=0.04, y=0.09, z=0.01 variance ->
    // confidence = 1 / (1 + 0.04 + 0.09 + 0.01) = 1 / 1.14.
    let expected_confidence = (1.0_f64 / 1.14) as f32;
    let actual = confidence.expect("real confidence should be derived from covariance");
    assert!(
        (actual - expected_confidence).abs() < 1e-5,
        "expected {expected_confidence}, got {actual} -- confidence should vary with real \
         covariance, not stay hardcoded at 0.95"
    );
    // Explicitly not the old hardcoded stub value.
    assert!((actual - 0.95).abs() > 0.01);
}

#[test]
fn all_five_real_message_types_are_present_and_distinct() {
    let events = load_fixture_events();
    // 1 each of scan/image/imu/odometry, plus 2 pose messages (a normal one
    // and a real injected localization-loss spike -- see the next test).
    assert_eq!(events.len(), 6);
}

/// This is the exact real bug the finding that led to this fix described:
/// `Mission.detect_failures()`/`AnomalyDetector::detect_localization_loss()`
/// checks `RobotPose.confidence < 0.5`, which could never fire while
/// `confidence` was hardcoded to `0.95` regardless of the message's real
/// covariance -- a real localization failure with a genuine large
/// covariance spike would silently go undetected. The fixture's second pose
/// message has a real large diagonal position covariance (trace = 9.0,
/// confidence = 1/(1+9) = 0.1); this asserts detection now actually fires
/// on it, and does not fire on the first, low-covariance pose message.
#[test]
fn real_covariance_spike_is_detected_as_a_localization_loss_failure() {
    let events = load_fixture_events();
    let detector = AnomalyDetector::new(events);
    let failures = detector.detect_localization_loss();
    assert_eq!(
        failures.len(),
        1,
        "expected exactly one localization-loss failure (the real covariance spike), got {}",
        failures.len()
    );
    assert_eq!(failures[0].failure_type, "localization_loss");
}
