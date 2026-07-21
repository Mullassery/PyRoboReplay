/// Demonstration of sensor metadata panel
/// Shows real-time statistics for all sensor types

use pyroboreplay::core::event::{MissionRecord, MissionEvent, LidarData, CameraFrame, IMUData, Odometry, Pose};
use pyroboreplay::cli::sensor_stats::SensorMetadataPanel;
use chrono::Utc;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║       PyRoboReplay Sensor Metadata Panel Demo                  ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Create a synthetic mission with multiple sensors
    println!("📹 Creating synthetic mission with 5 sensors...");
    let mission = create_sample_mission();

    println!("✅ Generated mission with {} events\n", mission.events.len());

    // Create metadata panel
    let panel = SensorMetadataPanel::from_mission(&mission);

    // Display full panel
    println!("{}", panel.render());

    // Display compact summary
    println!("\n{}", panel.render_compact());

    // Display overall summary
    println!("{}", panel.summary());

    println!("\n\n📊 Individual Sensor Statistics");
    println!("════════════════════════════════════════════════════════════════");

    for sensor_name in panel.sensor_names() {
        if let Some(stats) = panel.get_stats(sensor_name) {
            println!("\n{}:", sensor_name);
            println!("  Type: {}", stats.sensor_type);
            println!("  Frames: {}", stats.frame_count);
            println!("  FPS: {:.1}", stats.avg_fps);
            println!("  Quality: {:.0}%", stats.data_quality * 100.0);
            println!("  Status: {}", stats.quality_emoji());
        }
    }
}

fn create_sample_mission() -> MissionRecord {
    let mut mission = MissionRecord::new("demo_mission");
    let base_time = Utc::now();

    // Add 30 lidar scans
    for i in 0..30 {
        let timestamp = base_time + chrono::Duration::milliseconds(i * 33); // ~30 Hz
        let lidar_data = LidarData {
            ranges: vec![5.0; 360],
            intensities: Some(vec![0.5; 360]),
            frame_id: "lidar_0".to_string(),
            min_angle: 0.0,
            max_angle: 6.28,
            angle_increment: 0.0175,
            range_min: 0.1,
            range_max: 30.0,
        };

        mission.events.push(MissionEvent::LidarScan {
            robot_id: "robot_0".to_string(),
            timestamp,
            data: lidar_data,
        });
    }

    // Add 15 camera frames (15 Hz, half the lidar rate)
    for i in 0..15 {
        let timestamp = base_time + chrono::Duration::milliseconds(i * 66); // ~15 Hz
        let camera_data = CameraFrame {
            sensor_id: "camera_0".to_string(),
            frame_id: format!("frame_{}", i),
            width: 640,
            height: 480,
            encoding: "rgb8".to_string(),
            image_data: vec![0u8; 640 * 480 * 3],
            camera_info: None,
        };

        mission.events.push(MissionEvent::CameraFrame {
            robot_id: "robot_0".to_string(),
            timestamp,
            data: camera_data,
        });
    }

    // Add 30 IMU readings (30 Hz)
    for i in 0..30 {
        let timestamp = base_time + chrono::Duration::milliseconds(i * 33);
        let imu_data = IMUData {
            frame_id: "imu_0".to_string(),
            linear_acceleration: [0.1, 0.2, 9.8],
            angular_velocity: [0.01, 0.02, 0.03],
            magnetometer: Some([20.0, 15.0, 35.0]),
            orientation: None,
        };

        mission.events.push(MissionEvent::IMUData {
            robot_id: "robot_0".to_string(),
            timestamp,
            data: imu_data,
        });
    }

    // Add 10 odometry updates (10 Hz, sparse)
    for i in 0..10 {
        let timestamp = base_time + chrono::Duration::milliseconds(i * 100);
        let odometry_data = Odometry {
            frame_id: "odom".to_string(),
            child_frame_id: "base_link".to_string(),
            pose: Pose {
                x: (i as f64) * 0.5,
                y: 0.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            twist_linear: [0.5, 0.0, 0.0],
            twist_angular: [0.0, 0.0, 0.0],
        };

        mission.events.push(MissionEvent::OdometryUpdate {
            robot_id: "robot_0".to_string(),
            timestamp,
            data: odometry_data,
        });
    }

    // Add 5 costmap updates (5 Hz, very sparse)
    for i in 0..5 {
        let timestamp = base_time + chrono::Duration::milliseconds(i * 200);
        let costmap_data = pyroboreplay::core::event::Costmap {
            frame_id: "map".to_string(),
            resolution: 0.1,
            width: 100,
            height: 100,
            origin: Pose {
                x: -5.0,
                y: -5.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            data: vec![0u8; 10000],
        };

        mission.events.push(MissionEvent::CostmapUpdate {
            robot_id: "robot_0".to_string(),
            timestamp,
            data: costmap_data,
        });
    }

    mission
}
