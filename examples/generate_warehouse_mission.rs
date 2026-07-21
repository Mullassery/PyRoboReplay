/// Generate a synthetic ROS 2 bag file for testing PyRoboReplay
///
/// Creates a 10-minute warehouse exploration mission with:
/// - Lidar scans (10 Hz)
/// - Camera frames (30 Hz)
/// - IMU data (100 Hz)
/// - Odometry updates (20 Hz)
/// - Simulated obstacle detection

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result};
use std::f32::consts::PI;
use std::path::Path;

struct MissionGenerator {
    start_time: i64,
    robot_id: String,
}

impl MissionGenerator {
    fn new() -> Self {
        let start_time = Utc::now().timestamp_nanos_opt().unwrap();
        Self {
            start_time,
            robot_id: "warehouse_robot_1".to_string(),
        }
    }

    fn generate_bag(&self, output_path: &str) -> Result<()> {
        // Remove existing file
        if Path::new(output_path).exists() {
            std::fs::remove_file(output_path).ok();
        }

        let conn = Connection::open(output_path)?;

        // Create tables matching ROS 2 bag format
        self.create_schema(&conn)?;

        // Insert metadata
        self.insert_metadata(&conn)?;

        // Insert topics
        let topic_ids = self.insert_topics(&conn)?;

        // Generate and insert messages
        self.generate_mission(&conn, &topic_ids)?;

        println!("✅ Generated: {}", output_path);
        println!("   Duration: 10 minutes");
        println!("   Sensors: lidar (10Hz), camera (30Hz), imu (100Hz), odom (20Hz)");
        println!(
            "   Total events: {}",
            self.estimate_event_count()
        );

        Ok(())
    }

    fn create_schema(&self, conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "
            CREATE TABLE metadata (
                key TEXT PRIMARY KEY,
                value TEXT
            );

            CREATE TABLE topics (
                id INTEGER PRIMARY KEY,
                name TEXT UNIQUE,
                type TEXT
            );

            CREATE TABLE messages (
                timestamp INTEGER,
                topic_id INTEGER,
                data BLOB
            );

            CREATE INDEX idx_messages_timestamp ON messages(timestamp);
            CREATE INDEX idx_messages_topic ON messages(topic_id);
            ",
        )?;
        Ok(())
    }

    fn insert_metadata(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO metadata (key, value) VALUES (?, ?)",
            params!["ros2_bagfile_version", "5"],
        )?;
        conn.execute(
            "INSERT INTO metadata (key, value) VALUES (?, ?)",
            params!["compression_format", "cdr"],
        )?;
        conn.execute(
            "INSERT INTO metadata (key, value) VALUES (?, ?)",
            params!["duration_sec", "600"],
        )?;
        Ok(())
    }

    fn insert_topics(&self, conn: &Connection) -> Result<std::collections::HashMap<&'static str, i64>> {
        let topics = vec![
            ("/lidar/scan", "sensor_msgs/msg/LaserScan"),
            ("/camera/image_raw", "sensor_msgs/msg/Image"),
            ("/imu/data", "sensor_msgs/msg/Imu"),
            ("/odom", "nav_msgs/msg/Odometry"),
        ];

        let mut topic_ids = std::collections::HashMap::new();

        for (name, msg_type) in topics {
            conn.execute(
                "INSERT INTO topics (name, type) VALUES (?, ?)",
                params![name, msg_type],
            )?;
            let id = conn.last_insert_rowid();
            topic_ids.insert(name, id);
        }

        Ok(topic_ids)
    }

    fn generate_mission(
        &self,
        conn: &Connection,
        topic_ids: &std::collections::HashMap<&'static str, i64>,
    ) -> Result<()> {
        let mission_duration_sec = 600; // 10 minutes
        let start_ns = self.start_time;

        // Lidar: 10 Hz = 1 message every 100ms = 100,000,000 ns
        for i in 0..(mission_duration_sec * 10) {
            let timestamp = start_ns + (i as i64 * 100_000_000);
            let data = self.generate_lidar_data(i as f32);
            conn.execute(
                "INSERT INTO messages (timestamp, topic_id, data) VALUES (?, ?, ?)",
                params![timestamp, topic_ids["/lidar/scan"], data],
            )?;
        }

        // Camera: 30 Hz = 1 message every ~33ms = 33,333,333 ns
        for i in 0..(mission_duration_sec * 30) {
            let timestamp = start_ns + (i as i64 * 33_333_333);
            let data = self.generate_camera_data(i as f32);
            conn.execute(
                "INSERT INTO messages (timestamp, topic_id, data) VALUES (?, ?, ?)",
                params![timestamp, topic_ids["/camera/image_raw"], data],
            )?;
        }

        // IMU: 100 Hz = 1 message every 10ms = 10,000,000 ns
        for i in 0..(mission_duration_sec * 100) {
            let timestamp = start_ns + (i as i64 * 10_000_000);
            let data = self.generate_imu_data(i as f32);
            conn.execute(
                "INSERT INTO messages (timestamp, topic_id, data) VALUES (?, ?, ?)",
                params![timestamp, topic_ids["/imu/data"], data],
            )?;
        }

        // Odometry: 20 Hz = 1 message every 50ms = 50,000,000 ns
        for i in 0..(mission_duration_sec * 20) {
            let timestamp = start_ns + (i as i64 * 50_000_000);
            let data = self.generate_odom_data(i as f32);
            conn.execute(
                "INSERT INTO messages (timestamp, topic_id, data) VALUES (?, ?, ?)",
                params![timestamp, topic_ids["/odom"], data],
            )?;
        }

        Ok(())
    }

    fn generate_lidar_data(&self, frame_idx: f32) -> Vec<u8> {
        // Simplified: just return placeholder data
        // In real implementation, would serialize actual LaserScan message
        let mut data = vec![0u8; 1440]; // 360 rays * 4 bytes per range

        // Simulate some range data (simplified CDR encoding)
        for i in 0..360 {
            let range = 5.0 + (frame_idx / 100.0).sin() * 2.0; // 3-7m range, oscillating
            data[i * 4] = (range as u8);
        }

        data
    }

    fn generate_camera_data(&self, frame_idx: f32) -> Vec<u8> {
        // Placeholder: 640x480 RGB = 921,600 bytes
        // For testing, just create minimal valid data
        let mut data = vec![128u8; 100]; // Simplified: just 100 bytes of image header

        // Add some variation based on frame
        data[0] = ((frame_idx / 10.0).sin() * 255.0) as u8;

        data
    }

    fn generate_imu_data(&self, frame_idx: f32) -> Vec<u8> {
        // Placeholder IMU data
        let mut data = vec![0u8; 48]; // Simplified: ~48 bytes for accel + gyro

        // Simulate acceleration (0.1g noise)
        let accel_x = ((frame_idx / 50.0).sin() * 0.1) as i16;
        let accel_y = ((frame_idx / 47.0).cos() * 0.1) as i16;

        // Store in little-endian
        data[0..2].copy_from_slice(&accel_x.to_le_bytes());
        data[2..4].copy_from_slice(&accel_y.to_le_bytes());

        data
    }

    fn generate_odom_data(&self, frame_idx: f32) -> Vec<u8> {
        // Placeholder odometry data
        let mut data = vec![0u8; 128]; // Simplified odometry message

        // Simulate forward movement (0.5 m/s)
        let distance = frame_idx * 0.025; // 20Hz, 0.5m/s = 0.025m per message
        let x_pos = distance.cos();

        // Store position (simplified)
        data[0..4].copy_from_slice(&x_pos.to_le_bytes());

        data
    }

    fn estimate_event_count(&self) -> usize {
        // 10 minutes = 600 seconds
        let lidar_count = 600 * 10;
        let camera_count = 600 * 30;
        let imu_count = 600 * 100;
        let odom_count = 600 * 20;

        lidar_count + camera_count + imu_count + odom_count
    }
}

fn main() -> Result<()> {
    let generator = MissionGenerator::new();

    // Generate warehouse mission
    generator.generate_bag("warehouse_exploration_v1.db3")?;

    println!("\n📊 Warehouse Mission Generated");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Location: ./warehouse_exploration_v1.db3");
    println!("\n🎬 Try replaying:");
    println!("   cargo run --release -- replay warehouse_exploration_v1.db3");
    println!("\n📡 List sensors:");
    println!("   cargo run --release -- list warehouse_exploration_v1.db3");
    println!("\n📊 Analyze mission:");
    println!("   cargo run --release -- analyze warehouse_exploration_v1.db3");

    Ok(())
}
