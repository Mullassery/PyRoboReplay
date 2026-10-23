use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const MISSION_DURATION_SEC: usize = 600;
const LIDAR_HZ: usize = 10;
const CAMERA_HZ: usize = 30;
const IMU_HZ: usize = 100;
const ODOM_HZ: usize = 20;

const TEST_BAG_NAME: &str = "warehouse_exploration_v2.db3";

struct CdrEncoder {
    buf: Vec<u8>,
}

impl CdrEncoder {
    fn new() -> Self {
        Self {
            buf: vec![0x00, 0x01, 0x00, 0x00],
        }
    }

    fn body_pos(&self) -> usize {
        self.buf.len() - 4
    }

    fn align(&mut self, size: usize) {
        let rem = self.body_pos() % size;
        if rem != 0 {
            self.buf.resize(self.buf.len() + size - rem, 0);
        }
    }

    fn u8(&mut self, value: u8) {
        self.buf.push(value);
    }

    fn u32(&mut self, value: u32) {
        self.align(4);
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    fn i32(&mut self, value: i32) {
        self.align(4);
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    fn f32(&mut self, value: f32) {
        self.align(4);
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    fn f64(&mut self, value: f64) {
        self.align(8);
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    fn string(&mut self, value: &str) {
        self.u32((value.len() + 1) as u32);
        self.buf.extend_from_slice(value.as_bytes());
        self.buf.push(0);
    }

    fn f32_seq(&mut self, values: &[f32]) {
        self.u32(values.len() as u32);
        for value in values {
            self.f32(*value);
        }
    }

    fn u8_seq(&mut self, values: &[u8]) {
        self.u32(values.len() as u32);
        self.buf.extend_from_slice(values);
    }

    fn f64_array(&mut self, values: &[f64]) {
        for value in values {
            self.f64(*value);
        }
    }

    fn header(&mut self, timestamp_ns: i64, frame_id: &str) {
        let (sec, nanosec) = split_timestamp(timestamp_ns);
        self.i32(sec);
        self.u32(nanosec);
        self.string(frame_id);
    }

    fn finish(self) -> Vec<u8> {
        self.buf
    }
}

fn split_timestamp(timestamp_ns: i64) -> (i32, u32) {
    let sec = timestamp_ns.div_euclid(1_000_000_000);
    let nanosec = timestamp_ns.rem_euclid(1_000_000_000);
    (sec as i32, nanosec as u32)
}

pub fn ensure_test_bag() -> String {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("test-data")
            .join(TEST_BAG_NAME);
        if !path.exists() {
            generate_bag(path.to_str().expect("test bag path must be valid UTF-8"))
                .expect("failed to generate integration test bag");
        }
        path
    })
    .to_str()
    .expect("test bag path must be valid UTF-8")
    .to_string()
}

pub fn generate_bag(output_path: &str) -> Result<()> {
    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
    }

    let mut conn = Connection::open(output_path)
        .with_context(|| format!("opening {output_path} for writing"))?;
    create_schema(&conn)?;

    let tx = conn.transaction()?;
    insert_metadata(&tx)?;
    let topic_ids = insert_topics(&tx)?;
    generate_mission(&tx, &topic_ids)?;
    tx.commit()?;

    Ok(())
}

fn create_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        DROP TABLE IF EXISTS messages;
        DROP TABLE IF EXISTS topics;
        DROP TABLE IF EXISTS metadata;

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

fn insert_metadata(conn: &Connection) -> Result<()> {
    let entries = [
        ("ros2_bagfile_version", "5"),
        ("compression_format", "cdr"),
        ("duration_sec", "600"),
        ("generator_version", "2"),
    ];
    for (key, value) in entries {
        conn.execute(
            "INSERT INTO metadata (key, value) VALUES (?, ?)",
            params![key, value],
        )?;
    }
    Ok(())
}

fn insert_topics(conn: &Connection) -> Result<HashMap<&'static str, i64>> {
    let topics = [
        ("/lidar/scan", "sensor_msgs/msg/LaserScan"),
        ("/camera/image_raw", "sensor_msgs/msg/Image"),
        ("/imu/data", "sensor_msgs/msg/Imu"),
        ("/odom", "nav_msgs/msg/Odometry"),
    ];

    let mut topic_ids = HashMap::new();
    for (name, msg_type) in topics {
        conn.execute(
            "INSERT INTO topics (name, type) VALUES (?, ?)",
            params![name, msg_type],
        )?;
        topic_ids.insert(name, conn.last_insert_rowid());
    }
    Ok(topic_ids)
}

fn generate_mission(conn: &Connection, topic_ids: &HashMap<&'static str, i64>) -> Result<()> {
    let start_ns = chrono::Utc::now()
        .timestamp_nanos_opt()
        .context("current time does not fit in nanoseconds")?;

    insert_topic_messages(
        conn,
        topic_ids["/lidar/scan"],
        LIDAR_HZ,
        start_ns,
        encode_lidar,
    )?;
    insert_topic_messages(
        conn,
        topic_ids["/camera/image_raw"],
        CAMERA_HZ,
        start_ns,
        encode_camera,
    )?;
    insert_topic_messages(conn, topic_ids["/imu/data"], IMU_HZ, start_ns, encode_imu)?;
    insert_topic_messages(conn, topic_ids["/odom"], ODOM_HZ, start_ns, encode_odom)?;

    Ok(())
}

fn insert_topic_messages(
    conn: &Connection,
    topic_id: i64,
    frequency_hz: usize,
    start_ns: i64,
    encode: fn(i64) -> Vec<u8>,
) -> Result<()> {
    let frame_count = MISSION_DURATION_SEC * frequency_hz;
    let interval_ns = 1_000_000_000_i64 / frequency_hz as i64;
    let mut stmt =
        conn.prepare("INSERT INTO messages (timestamp, topic_id, data) VALUES (?, ?, ?)")?;

    for frame_idx in 0..frame_count {
        let timestamp = start_ns + frame_idx as i64 * interval_ns;
        stmt.execute(params![timestamp, topic_id, encode(timestamp)])?;
    }

    Ok(())
}

fn encode_lidar(timestamp: i64) -> Vec<u8> {
    let mut enc = CdrEncoder::new();
    enc.header(timestamp, "lidar_link");
    enc.f32(-PI as f32);
    enc.f32(PI as f32);
    enc.f32((2.0 * PI / 360.0) as f32);
    enc.f32(0.0);
    enc.f32(0.1);
    enc.f32(0.1);
    enc.f32(100.0);

    let ranges: Vec<f32> = (0..360)
        .map(|ray| 5.0 + ((timestamp as f64 / 1e9 + ray as f64 / 36.0).sin() * 2.0) as f32)
        .collect();
    enc.f32_seq(&ranges);

    let intensities: Vec<f32> = (0..360).map(|ray| (ray % 100) as f32).collect();
    enc.f32_seq(&intensities);
    enc.finish()
}

fn encode_camera(timestamp: i64) -> Vec<u8> {
    let width = 8_u32;
    let height = 8_u32;
    let step = width * 3;
    let mut data = vec![0_u8; (width * height * 3) as usize];
    let shade = ((timestamp as f64 / 1e9).sin() * 127.0 + 128.0) as u8;
    data.fill(shade);

    let mut enc = CdrEncoder::new();
    enc.header(timestamp, "camera_link");
    enc.u32(height);
    enc.u32(width);
    enc.string("rgb8");
    enc.u8(0);
    enc.u32(step);
    enc.u8_seq(&data);
    enc.finish()
}

fn encode_imu(timestamp: i64) -> Vec<u8> {
    let t = timestamp as f64 / 1e9;
    let yaw = (t * 0.2).sin() * 0.3;
    let zero_covariance = [0.0_f64; 9];

    let mut enc = CdrEncoder::new();
    enc.header(timestamp, "imu_link");
    enc.f64(0.0);
    enc.f64(0.0);
    enc.f64((yaw / 2.0).sin());
    enc.f64((yaw / 2.0).cos());
    enc.f64_array(&zero_covariance);
    enc.f64((0.2 * t).sin() * 0.01);
    enc.f64((0.2 * t).cos() * 0.01);
    enc.f64(0.01);
    enc.f64_array(&zero_covariance);
    enc.f64((0.5 * t).sin() * 0.05);
    enc.f64((0.5 * t).cos() * 0.05);
    enc.f64(9.81);
    enc.f64_array(&zero_covariance);
    enc.finish()
}

fn encode_odom(timestamp: i64) -> Vec<u8> {
    let t = timestamp as f64 / 1e9;
    let distance = 0.5 * t;
    let yaw = (t * 0.2).sin() * 0.3;

    let mut pose_covariance = [0.0_f64; 36];
    for diagonal in [0, 7, 14, 21, 28, 35] {
        pose_covariance[diagonal] = 0.01;
    }

    let mut twist_covariance = [0.0_f64; 36];
    for diagonal in [0, 7, 14, 21, 28, 35] {
        twist_covariance[diagonal] = 0.001;
    }

    let mut enc = CdrEncoder::new();
    enc.header(timestamp, "odom");
    enc.string("base_link");
    enc.f64(distance);
    enc.f64(0.3 * (t * 0.1).sin());
    enc.f64(0.0);
    enc.f64(0.0);
    enc.f64(0.0);
    enc.f64((yaw / 2.0).sin());
    enc.f64((yaw / 2.0).cos());
    enc.f64_array(&pose_covariance);
    enc.f64(0.5);
    enc.f64(0.0);
    enc.f64(0.0);
    enc.f64(0.0);
    enc.f64(0.0);
    enc.f64(0.2 * (t * 0.2).cos() * 0.3);
    enc.f64_array(&twist_covariance);
    enc.finish()
}
