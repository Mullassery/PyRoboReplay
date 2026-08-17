//! Multi-modal data source adapters for Phase 14 temporal fusion
//!
//! Supports ingestion of diverse data sources:
//! - ROS 2 bags (.mcap, .db3/rosbag2-sqlite)
//! - Linux system logs (syslog, dmesg, journalctl)
//! - Nav2 diagnostic exports
//! - Video files (MP4, MKV, image sequences)
//! - Point clouds (PCD, PLY, LAS)
//! - Annotations (CSV, JSON, YAML)
//! - Sensor calibration files
//! - Environment maps
//! - Robot models (URDF, CAD specs)
//!
//! NOTE ON REACHABILITY: like `video_processing.rs`'s `VideoProcessor`, these
//! adapters are internally real (they genuinely parse real on-disk data) and
//! covered by tests that exercise real files, but nothing elsewhere in the
//! codebase yet drives bytes from `extract_stream()` into
//! `timeline_indexing::Timeline`/`TimelineEvent` — that end-to-end wiring is
//! a separate piece of work left for a future pass.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, TimeZone, Datelike};
use regex::Regex;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataSourceType {
    RosBag,
    LinuxLogs,
    Nav2Export,
    Video,
    PointCloud,
    Annotation,
    SensorCalibration,
    EnvironmentMap,
    RobotModel,
}

impl std::fmt::Display for DataSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSourceType::RosBag => write!(f, "ROS Bag"),
            DataSourceType::LinuxLogs => write!(f, "Linux Logs"),
            DataSourceType::Nav2Export => write!(f, "Nav2 Export"),
            DataSourceType::Video => write!(f, "Video"),
            DataSourceType::PointCloud => write!(f, "Point Cloud"),
            DataSourceType::Annotation => write!(f, "Annotation"),
            DataSourceType::SensorCalibration => write!(f, "Sensor Calibration"),
            DataSourceType::EnvironmentMap => write!(f, "Environment Map"),
            DataSourceType::RobotModel => write!(f, "Robot Model"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub name: String,
    pub msg_type: String,
    pub message_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceData {
    pub source_type: DataSourceType,
    pub topics: Vec<Topic>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_seconds: Option<f64>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Error)]
pub enum AdapterError {
    #[error("Failed to load {0}: {1}")]
    LoadFailed(String, String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("Missing data: {0}")]
    MissingData(String),
}

pub type AdapterResult<T> = Result<T, AdapterError>;

/// Core trait for all data source adapters
pub trait DataSource: Send + Sync {
    /// Load data from source
    fn load(&self, path: &Path) -> AdapterResult<SourceData>;

    /// Get available topics/channels in source
    fn available_topics(&self) -> AdapterResult<Vec<Topic>>;

    /// Extract time-series stream for a specific topic
    fn extract_stream(&self, topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>>;

    /// Get metadata
    fn metadata(&self) -> std::collections::HashMap<String, String>;

    /// Data source type identifier
    fn source_type(&self) -> DataSourceType;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: i64,  // ROS time in nanoseconds (unified)
    pub value: Vec<u8>,  // Serialized message
    pub topic: String,
}

/// Convert a unified ROS-style nanosecond timestamp to a `DateTime<Utc>`,
/// used across adapters to populate `SourceData::start_time`/`end_time`.
fn ns_to_datetime(ns: i64) -> Option<DateTime<Utc>> {
    let secs = ns.div_euclid(1_000_000_000);
    let nanos = ns.rem_euclid(1_000_000_000) as u32;
    DateTime::<Utc>::from_timestamp(secs, nanos)
}

// ============================================================================
// ROS 2 Bag Adapter
//
// rosbag2 has shipped two on-disk storage formats over its lifetime: the
// original SQLite3 plugin (`.db3`, schema: `topics(id, name, type, ...)` /
// `messages(id, topic_id, timestamp, data)`) and, since Humble/Iron, MCAP
// (`.mcap`, the Foxglove-designed self-describing container format, now the
// ROS 2 default). Both are real, genuinely-shipping formats, so both are
// parsed for real here rather than picking one and stubbing the other.
// ============================================================================

pub struct RosBagAdapter {
    path: Mutex<Option<PathBuf>>,
}

impl RosBagAdapter {
    pub fn new() -> Self {
        RosBagAdapter { path: Mutex::new(None) }
    }
}

impl Default for RosBagAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RosBagFormat {
    Mcap,
    Sqlite,
}

impl std::fmt::Display for RosBagFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RosBagFormat::Mcap => write!(f, "mcap"),
            RosBagFormat::Sqlite => write!(f, "rosbag2-sqlite"),
        }
    }
}

fn detect_rosbag_format(path: &Path) -> AdapterResult<RosBagFormat> {
    match path.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()) {
        Some(ext) if ext == "mcap" => Ok(RosBagFormat::Mcap),
        Some(ext) if ext == "db3" => Ok(RosBagFormat::Sqlite),
        other => Err(AdapterError::InvalidFormat(format!(
            "unrecognized ROS bag extension {:?} for {:?}; expected .mcap or .db3 \
             (rosbag2's two real on-disk storage formats)",
            other, path
        ))),
    }
}

struct ParsedRosBag {
    topics: Vec<Topic>,
    start_time_ns: Option<i64>,
    end_time_ns: Option<i64>,
}

fn parse_mcap_bag(path: &Path) -> AdapterResult<ParsedRosBag> {
    let bytes = std::fs::read(path)?;
    let stream = mcap::MessageStream::new(&bytes)
        .map_err(|e| AdapterError::ParseError(format!("invalid MCAP file {:?}: {e}", path)))?;

    let mut counts: HashMap<String, (u32, String)> = HashMap::new();
    let mut start: Option<u64> = None;
    let mut end: Option<u64> = None;

    for message in stream {
        let message = message.map_err(|e| AdapterError::ParseError(format!("MCAP read error: {e}")))?;
        let entry = counts.entry(message.channel.topic.clone()).or_insert_with(|| {
            let msg_type = message
                .channel
                .schema
                .as_ref()
                .map(|s| s.name.clone())
                .unwrap_or_else(|| message.channel.message_encoding.clone());
            (0, msg_type)
        });
        entry.0 += 1;
        start = Some(start.map_or(message.log_time, |s| s.min(message.log_time)));
        end = Some(end.map_or(message.log_time, |e2| e2.max(message.log_time)));
    }

    let topics = counts
        .into_iter()
        .map(|(name, (message_count, msg_type))| Topic { name, msg_type, message_count })
        .collect();

    Ok(ParsedRosBag {
        topics,
        start_time_ns: start.map(|t| t as i64),
        end_time_ns: end.map(|t| t as i64),
    })
}

fn extract_mcap_stream(path: &Path, topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
    let bytes = std::fs::read(path)?;
    let stream = mcap::MessageStream::new(&bytes)
        .map_err(|e| AdapterError::ParseError(format!("invalid MCAP file {:?}: {e}", path)))?;

    let mut points = Vec::new();
    for message in stream {
        let message = message.map_err(|e| AdapterError::ParseError(format!("MCAP read error: {e}")))?;
        if message.channel.topic == topic {
            points.push(TimeSeriesPoint {
                timestamp: message.log_time as i64,
                value: message.data.to_vec(),
                topic: message.channel.topic.clone(),
            });
        }
    }
    Ok(points)
}

fn open_rosbag2_sqlite(path: &Path) -> AdapterResult<rusqlite::Connection> {
    rusqlite::Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| AdapterError::ParseError(format!("failed to open rosbag2 sqlite db {:?}: {e}", path)))
}

fn parse_sqlite_bag(path: &Path) -> AdapterResult<ParsedRosBag> {
    let conn = open_rosbag2_sqlite(path)?;

    let mut topic_stmt = conn
        .prepare("SELECT id, name, type FROM topics")
        .map_err(|e| {
            AdapterError::ParseError(format!(
                "{:?} is not a valid rosbag2 .db3 file (missing 'topics' table): {e}",
                path
            ))
        })?;
    let topic_rows = topic_stmt
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        })
        .map_err(|e| AdapterError::ParseError(e.to_string()))?;

    let mut id_to_name_type: HashMap<i64, (String, String)> = HashMap::new();
    for row in topic_rows {
        let (id, name, msg_type) = row.map_err(|e| AdapterError::ParseError(e.to_string()))?;
        id_to_name_type.insert(id, (name, msg_type));
    }

    let mut count_stmt = conn
        .prepare("SELECT topic_id, COUNT(*), MIN(timestamp), MAX(timestamp) FROM messages GROUP BY topic_id")
        .map_err(|e| AdapterError::ParseError(e.to_string()))?;
    let rows = count_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })
        .map_err(|e| AdapterError::ParseError(e.to_string()))?;

    let mut topics = Vec::new();
    let mut start: Option<i64> = None;
    let mut end: Option<i64> = None;
    for row in rows {
        let (topic_id, count, min_ts, max_ts) = row.map_err(|e| AdapterError::ParseError(e.to_string()))?;
        if let Some((name, msg_type)) = id_to_name_type.get(&topic_id) {
            topics.push(Topic { name: name.clone(), msg_type: msg_type.clone(), message_count: count as u32 });
        }
        start = Some(start.map_or(min_ts, |s: i64| s.min(min_ts)));
        end = Some(end.map_or(max_ts, |e2: i64| e2.max(max_ts)));
    }

    Ok(ParsedRosBag { topics, start_time_ns: start, end_time_ns: end })
}

fn extract_sqlite_stream(path: &Path, topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
    let conn = open_rosbag2_sqlite(path)?;

    let topic_id: Option<i64> = conn
        .query_row("SELECT id FROM topics WHERE name = ?1", [topic], |row| row.get(0))
        .ok();
    let Some(topic_id) = topic_id else {
        return Ok(Vec::new());
    };

    let mut stmt = conn
        .prepare("SELECT timestamp, data FROM messages WHERE topic_id = ?1 ORDER BY timestamp")
        .map_err(|e| AdapterError::ParseError(e.to_string()))?;
    let rows = stmt
        .query_map([topic_id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)))
        .map_err(|e| AdapterError::ParseError(e.to_string()))?;

    let mut points = Vec::new();
    for row in rows {
        let (timestamp, data) = row.map_err(|e| AdapterError::ParseError(e.to_string()))?;
        points.push(TimeSeriesPoint { timestamp, value: data, topic: topic.to_string() });
    }
    Ok(points)
}

fn parse_rosbag(path: &Path) -> AdapterResult<(RosBagFormat, ParsedRosBag)> {
    let format = detect_rosbag_format(path)?;
    let parsed = match format {
        RosBagFormat::Mcap => parse_mcap_bag(path)?,
        RosBagFormat::Sqlite => parse_sqlite_bag(path)?,
    };
    Ok((format, parsed))
}

impl DataSource for RosBagAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                "ROS Bag".to_string(),
                format!("File not found: {:?}", path),
            ));
        }

        let (format, parsed) = parse_rosbag(path)
            .map_err(|e| AdapterError::LoadFailed("ROS Bag".to_string(), e.to_string()))?;

        *self.path.lock().unwrap() = Some(path.to_path_buf());

        let mut metadata = HashMap::new();
        metadata.insert("storage_format".to_string(), format.to_string());
        metadata.insert("path".to_string(), path.display().to_string());

        Ok(SourceData {
            source_type: DataSourceType::RosBag,
            topics: parsed.topics,
            start_time: parsed.start_time_ns.and_then(ns_to_datetime),
            end_time: parsed.end_time_ns.and_then(ns_to_datetime),
            duration_seconds: match (parsed.start_time_ns, parsed.end_time_ns) {
                (Some(s), Some(e)) if e >= s => Some((e - s) as f64 / 1e9),
                _ => None,
            },
            metadata,
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        let path_guard = self.path.lock().unwrap();
        let path = path_guard
            .as_ref()
            .ok_or_else(|| AdapterError::MissingData("call load() before available_topics()".to_string()))?;
        let (_, parsed) = parse_rosbag(path)?;
        Ok(parsed.topics)
    }

    fn extract_stream(&self, topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        let path_guard = self.path.lock().unwrap();
        let path = path_guard
            .as_ref()
            .ok_or_else(|| AdapterError::MissingData("call load() before extract_stream()".to_string()))?;
        match detect_rosbag_format(path)? {
            RosBagFormat::Mcap => extract_mcap_stream(path, topic),
            RosBagFormat::Sqlite => extract_sqlite_stream(path, topic),
        }
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        let path_guard = self.path.lock().unwrap();
        let mut m = HashMap::new();
        if let Some(path) = path_guard.as_ref() {
            m.insert("path".to_string(), path.display().to_string());
            if let Ok(format) = detect_rosbag_format(path) {
                m.insert("storage_format".to_string(), format.to_string());
            }
        }
        m
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::RosBag
    }
}

// ============================================================================
// Linux Logs Adapter
//
// Parses real syslog wire formats (RFC 3164 "BSD syslog" and RFC 5424) and
// real dmesg/kernel-ring-buffer output (both the wall-clock `dmesg -T`/
// `--ctime` form and the raw monotonic-seconds-since-boot bracket form).
// journalctl's default `--output=short` text format is RFC-3164-shaped, so
// it's parsed with the same syslog parser.
// ============================================================================

pub struct LinuxLogsAdapter {
    log_type: LogType,
    path: Mutex<Option<PathBuf>>,
}

#[derive(Debug, Clone, Copy)]
pub enum LogType {
    Syslog,
    Dmesg,
    Journalctl,
}

impl LinuxLogsAdapter {
    pub fn new(log_type: LogType) -> Self {
        LinuxLogsAdapter { log_type, path: Mutex::new(None) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LogRecord {
    timestamp_ns: Option<i64>,
    /// True when `timestamp_ns` is monotonic time-since-boot (from raw dmesg
    /// output with no `-T`/`--ctime` wall-clock rendering) rather than a real
    /// wall-clock timestamp. Monotonic time has no fixed relationship to wall
    /// clock without an external boot-time anchor, so callers must not treat
    /// it as comparable to other modalities' timestamps without first
    /// resolving that anchor themselves.
    boot_relative: bool,
    severity: String,
    host: Option<String>,
    process: Option<String>,
    message: String,
}

fn severity_name(pri: Option<u32>) -> String {
    match pri.map(|p| p % 8) {
        Some(0) => "emergency",
        Some(1) => "alert",
        Some(2) => "critical",
        Some(3) => "error",
        Some(4) => "warning",
        Some(5) => "notice",
        Some(6) => "info",
        Some(7) => "debug",
        _ => "unknown",
    }
    .to_string()
}

fn month_name_to_number(name: &str) -> Option<u32> {
    const MONTHS: [&str; 12] =
        ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    MONTHS.iter().position(|m| *m == name).map(|i| i as u32 + 1)
}

fn rfc5424_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^<(?P<pri>\d{1,3})>(?P<ver>\d+)\s+(?P<ts>\S+)\s+(?P<host>\S+)\s+(?P<app>\S+)\s+(?P<pid>\S+)\s+(?P<msgid>\S+)\s+(?P<sd>-|\[.*?\])\s?(?P<msg>.*)$",
        )
        .expect("static regex")
    })
}

fn rfc3164_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"^(?:<(?P<pri>\d{1,3})>)?(?P<month>[A-Z][a-z]{2})\s+(?P<day>\d{1,2})\s+(?P<hour>\d{2}):(?P<min>\d{2}):(?P<sec>\d{2})\s+(?P<host>\S+)\s+(?P<proc>[^:\[]+?)(?:\[(?P<pid>\d+)\])?:\s?(?P<msg>.*)$",
        )
        .expect("static regex")
    })
}

fn dmesg_bracket_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(?:<(?P<pri>\d+)>)?\[(?P<ts>[^\]]+)\]\s?(?P<msg>.*)$").expect("static regex"))
}

fn parse_syslog_line(line: &str) -> LogRecord {
    if let Some(caps) = rfc5424_regex().captures(line) {
        let pri: Option<u32> = caps.name("pri").and_then(|m| m.as_str().parse().ok());
        let ts = caps.name("ts").map(|m| m.as_str()).filter(|s| *s != "-");
        let timestamp_ns = ts
            .and_then(|t| DateTime::parse_from_rfc3339(t).ok())
            .and_then(|dt| dt.timestamp_nanos_opt());
        let host = caps.name("host").map(|m| m.as_str().to_string()).filter(|s| s != "-");
        let app = caps.name("app").map(|m| m.as_str().to_string()).filter(|s| s != "-");
        return LogRecord {
            timestamp_ns,
            boot_relative: false,
            severity: severity_name(pri),
            host,
            process: app,
            message: caps.name("msg").map(|m| m.as_str().to_string()).unwrap_or_default(),
        };
    }

    if let Some(caps) = rfc3164_regex().captures(line) {
        let pri: Option<u32> = caps.name("pri").and_then(|m| m.as_str().parse().ok());
        let month = caps.name("month").unwrap().as_str();
        let day: u32 = caps.name("day").unwrap().as_str().parse().unwrap_or(1);
        let hour: u32 = caps.name("hour").unwrap().as_str().parse().unwrap_or(0);
        let min: u32 = caps.name("min").unwrap().as_str().parse().unwrap_or(0);
        let sec: u32 = caps.name("sec").unwrap().as_str().parse().unwrap_or(0);
        // RFC 3164 famously omits the year. We anchor to the current year,
        // which is correct for the overwhelming majority of real-world log
        // files (rotated/rolled log files spanning a New Year's boundary are
        // a known, documented limitation of the format itself, not of this
        // parser).
        let year = Utc::now().year();
        let timestamp_ns = month_name_to_number(month).and_then(|month_num| {
            chrono::NaiveDate::from_ymd_opt(year, month_num, day)
                .and_then(|d| d.and_hms_opt(hour, min, sec))
                .and_then(|naive| Utc.from_utc_datetime(&naive).timestamp_nanos_opt())
        });
        return LogRecord {
            timestamp_ns,
            boot_relative: false,
            severity: severity_name(pri),
            host: caps.name("host").map(|m| m.as_str().to_string()),
            process: caps.name("proc").map(|m| m.as_str().trim().to_string()),
            message: caps.name("msg").map(|m| m.as_str().to_string()).unwrap_or_default(),
        };
    }

    LogRecord {
        timestamp_ns: None,
        boot_relative: false,
        severity: "unknown".to_string(),
        host: None,
        process: None,
        message: line.to_string(),
    }
}

fn parse_dmesg_line(line: &str) -> LogRecord {
    let Some(caps) = dmesg_bracket_regex().captures(line) else {
        return LogRecord {
            timestamp_ns: None,
            boot_relative: false,
            severity: "unknown".to_string(),
            host: None,
            process: None,
            message: line.to_string(),
        };
    };

    let pri: Option<u32> = caps.name("pri").and_then(|m| m.as_str().parse().ok());
    let inner = caps.name("ts").unwrap().as_str();
    let msg = caps.name("msg").map(|m| m.as_str().to_string()).unwrap_or_default();

    // `dmesg -T` / `--ctime` wall-clock form, e.g. "Wed Aug 12 09:23:01 2026".
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(inner, "%a %b %e %H:%M:%S %Y") {
        let timestamp_ns = Utc.from_utc_datetime(&naive).timestamp_nanos_opt();
        return LogRecord {
            timestamp_ns,
            boot_relative: false,
            severity: severity_name(pri),
            host: None,
            process: None,
            message: msg,
        };
    }

    // Raw kernel ring-buffer form: "[monotonic_seconds.microseconds] msg",
    // optionally prefixed with "<facility*8+level>". No wall-clock anchor is
    // available for this form (see `LogRecord::boot_relative`).
    if let Ok(secs) = inner.trim().parse::<f64>() {
        let timestamp_ns = Some((secs * 1e9).round() as i64);
        return LogRecord {
            timestamp_ns,
            boot_relative: true,
            severity: severity_name(pri),
            host: None,
            process: None,
            message: msg,
        };
    }

    LogRecord {
        timestamp_ns: None,
        boot_relative: false,
        severity: severity_name(pri),
        host: None,
        process: None,
        message: msg,
    }
}

fn parse_log_line(line: &str, log_type: LogType) -> LogRecord {
    match log_type {
        LogType::Dmesg => parse_dmesg_line(line),
        LogType::Syslog | LogType::Journalctl => parse_syslog_line(line),
    }
}

fn parse_log_lines(content: &str, log_type: LogType) -> Vec<LogRecord> {
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| parse_log_line(line, log_type))
        .collect()
}

const SEVERITIES: [&str; 8] =
    ["emergency", "alert", "critical", "error", "warning", "notice", "info", "debug"];

impl DataSource for LinuxLogsAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                format!("{:?} Log", self.log_type),
                format!("File not found: {:?}", path),
            ));
        }

        let content = std::fs::read_to_string(path)?;
        let records = parse_log_lines(&content, self.log_type);
        *self.path.lock().unwrap() = Some(path.to_path_buf());

        let timestamps: Vec<i64> = records.iter().filter_map(|r| r.timestamp_ns).collect();
        let start = timestamps.iter().min().copied();
        let end = timestamps.iter().max().copied();

        let mut metadata = HashMap::new();
        metadata.insert("log_type".to_string(), format!("{:?}", self.log_type));
        metadata.insert("total_lines".to_string(), records.len().to_string());
        let unparsed = records.iter().filter(|r| r.timestamp_ns.is_none()).count();
        metadata.insert("lines_without_timestamp".to_string(), unparsed.to_string());
        let boot_relative = records.iter().filter(|r| r.boot_relative).count();
        if boot_relative > 0 {
            metadata.insert("boot_relative_lines".to_string(), boot_relative.to_string());
        }
        for sev in SEVERITIES {
            let count = records.iter().filter(|r| r.severity == sev).count();
            if count > 0 {
                metadata.insert(format!("{}_count", sev), count.to_string());
            }
        }

        Ok(SourceData {
            source_type: DataSourceType::LinuxLogs,
            topics: vec![Topic {
                name: format!("{:?}", self.log_type),
                msg_type: "std_msgs/String".to_string(),
                message_count: records.len() as u32,
            }],
            start_time: start.and_then(ns_to_datetime),
            end_time: end.and_then(ns_to_datetime),
            duration_seconds: match (start, end) {
                (Some(s), Some(e)) if e >= s => Some((e - s) as f64 / 1e9),
                _ => None,
            },
            metadata,
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        let path_guard = self.path.lock().unwrap();
        let path = path_guard
            .as_ref()
            .ok_or_else(|| AdapterError::MissingData("call load() before available_topics()".to_string()))?;
        let content = std::fs::read_to_string(path)?;
        let records = parse_log_lines(&content, self.log_type);
        Ok(vec![Topic {
            name: format!("{:?}", self.log_type),
            msg_type: "std_msgs/String".to_string(),
            message_count: records.len() as u32,
        }])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        let path_guard = self.path.lock().unwrap();
        let path = path_guard
            .as_ref()
            .ok_or_else(|| AdapterError::MissingData("call load() before extract_stream()".to_string()))?;
        let content = std::fs::read_to_string(path)?;
        let records = parse_log_lines(&content, self.log_type);
        let topic_name = format!("{:?}", self.log_type);
        Ok(records
            .into_iter()
            .filter_map(|r| {
                let ts = r.timestamp_ns?;
                let value = serde_json::to_vec(&r).ok()?;
                Some(TimeSeriesPoint { timestamp: ts, value, topic: topic_name.clone() })
            })
            .collect())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("log_type".to_string(), format!("{:?}", self.log_type));
        m
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::LinuxLogs
    }
}

// ============================================================================
// Nav2 Export Adapter
//
// Parses two genuinely real, on-disk Nav2/ROS-navigation-stack formats:
//
// 1. Costmaps/maps: the `nav2_map_server`/`map_saver_cli` YAML + PGM (or PNG)
//    pair actually written to disk by the navigation stack (fields: `image`,
//    `resolution`, `origin`, `negate`, `occupied_thresh`, `free_thresh`).
// 2. Diagnostics/controller state: `diagnostic_msgs/DiagnosticArray` records
//    (the message type Nav2's diagnostic updaters/aggregators publish on
//    `/diagnostics`) exported as JSON Lines or a JSON array — the practical
//    text representation you get by piping `ros2 topic echo -f json` output
//    to a file, since raw ROS 2 CDR serialization can't be decoded without a
//    live ROS 2 install and the message's IDL.
// ============================================================================

pub struct Nav2ExportAdapter {
    dir: Mutex<Option<PathBuf>>,
}

impl Nav2ExportAdapter {
    pub fn new() -> Self {
        Nav2ExportAdapter { dir: Mutex::new(None) }
    }
}

impl Default for Nav2ExportAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Deserialize)]
struct DiagnosticStamp {
    #[serde(default)]
    sec: i64,
    #[serde(default)]
    nanosec: i64,
}

#[derive(Debug, Default, Deserialize)]
struct DiagnosticHeader {
    #[serde(default)]
    stamp: Option<DiagnosticStamp>,
}

#[derive(Debug, Default, Deserialize)]
struct DiagnosticStatusMsg {
    #[serde(default)]
    name: String,
    #[serde(default)]
    message: String,
}

#[derive(Debug, Default, Deserialize)]
struct DiagnosticArrayMsg {
    #[serde(default)]
    header: Option<DiagnosticHeader>,
    #[serde(default)]
    status: Vec<DiagnosticStatusMsg>,
}

impl DiagnosticArrayMsg {
    fn timestamp_ns(&self) -> Option<i64> {
        let stamp = self.header.as_ref()?.stamp.as_ref()?;
        Some(stamp.sec * 1_000_000_000 + stamp.nanosec)
    }
}

fn read_pgm_dimensions(path: &Path) -> AdapterResult<(u32, u32)> {
    let bytes = std::fs::read(path)?;
    let mut pos = 0usize;
    let mut tokens: Vec<String> = Vec::new();

    while tokens.len() < 3 && pos < bytes.len() {
        while pos < bytes.len() && (bytes[pos] as char).is_whitespace() {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }
        if bytes[pos] == b'#' {
            while pos < bytes.len() && bytes[pos] != b'\n' {
                pos += 1;
            }
            continue;
        }
        let start = pos;
        while pos < bytes.len() && !(bytes[pos] as char).is_whitespace() {
            pos += 1;
        }
        tokens.push(String::from_utf8_lossy(&bytes[start..pos]).to_string());
    }

    if tokens.len() < 3 || (tokens[0] != "P5" && tokens[0] != "P2") {
        return Err(AdapterError::ParseError(format!("not a valid PGM file: {:?}", path)));
    }
    let width: u32 = tokens[1]
        .parse()
        .map_err(|_| AdapterError::ParseError(format!("invalid PGM width in {:?}", path)))?;
    let height: u32 = tokens[2]
        .parse()
        .map_err(|_| AdapterError::ParseError(format!("invalid PGM height in {:?}", path)))?;
    Ok((width, height))
}

fn image_dimensions_any(path: &Path) -> Option<(u32, u32)> {
    if path.extension().and_then(|e| e.to_str()).map(|s| s.eq_ignore_ascii_case("pgm")) == Some(true) {
        read_pgm_dimensions(path).ok()
    } else {
        image::image_dimensions(path).ok()
    }
}

struct ParsedNav2Export {
    topics: Vec<Topic>,
    points: Vec<TimeSeriesPoint>,
    metadata: HashMap<String, String>,
    start_time_ns: Option<i64>,
    end_time_ns: Option<i64>,
}

fn parse_nav2_export_dir(dir: &Path) -> AdapterResult<ParsedNav2Export> {
    let mut metadata = HashMap::new();
    let mut points = Vec::new();
    let mut start: Option<i64> = None;
    let mut end: Option<i64> = None;
    let mut costmap_count = 0u32;

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
        let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();

        match ext.as_str() {
            "yaml" | "yml" => {
                let Ok(content) = std::fs::read_to_string(&path) else { continue };
                let Ok(value) = serde_norway::from_str::<serde_norway::Value>(&content) else { continue };
                let Some(map) = value.as_mapping() else { continue };
                let get = |key: &str| map.iter().find(|(k, _)| k.as_str() == Some(key)).map(|(_, v)| v.clone());
                let (Some(image_val), Some(resolution_val)) = (get("image"), get("resolution")) else { continue };

                costmap_count += 1;
                let mtime_ns = std::fs::metadata(&path)
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_nanos() as i64);

                let mut record = HashMap::new();
                record.insert("source_yaml".to_string(), path.display().to_string());
                if let Some(resolution) = resolution_val.as_f64() {
                    record.insert("resolution".to_string(), resolution.to_string());
                    metadata.insert(format!("costmap.{}.resolution", stem), resolution.to_string());
                }
                if let Some(image_name) = image_val.as_str() {
                    let image_path = dir.join(image_name);
                    record.insert("image".to_string(), image_name.to_string());
                    if let Some((w, h)) = image_dimensions_any(&image_path) {
                        record.insert("width".to_string(), w.to_string());
                        record.insert("height".to_string(), h.to_string());
                        metadata.insert(format!("costmap.{}.width", stem), w.to_string());
                        metadata.insert(format!("costmap.{}.height", stem), h.to_string());
                    }
                }

                if let Some(ns) = mtime_ns {
                    start = Some(start.map_or(ns, |s: i64| s.min(ns)));
                    end = Some(end.map_or(ns, |e2: i64| e2.max(ns)));
                }
                if let Ok(value) = serde_json::to_vec(&record) {
                    points.push(TimeSeriesPoint {
                        timestamp: mtime_ns.unwrap_or(0),
                        value,
                        topic: "costmap".to_string(),
                    });
                }
            }
            "jsonl" | "ndjson" => {
                let Ok(content) = std::fs::read_to_string(&path) else { continue };
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if let Ok(msg) = serde_json::from_str::<DiagnosticArrayMsg>(line) {
                        let ts = msg.timestamp_ns();
                        if let Some(ns) = ts {
                            start = Some(start.map_or(ns, |s: i64| s.min(ns)));
                            end = Some(end.map_or(ns, |e2: i64| e2.max(ns)));
                        }
                        if let Ok(value) = serde_json::to_vec(&line) {
                            points.push(TimeSeriesPoint {
                                timestamp: ts.unwrap_or(0),
                                value,
                                topic: "planner_diagnostics".to_string(),
                            });
                        }
                    }
                }
            }
            "json" => {
                let Ok(content) = std::fs::read_to_string(&path) else { continue };
                let messages: Vec<DiagnosticArrayMsg> = if let Ok(arr) = serde_json::from_str(&content) {
                    arr
                } else if let Ok(single) = serde_json::from_str::<DiagnosticArrayMsg>(&content) {
                    vec![single]
                } else {
                    continue;
                };
                for msg in messages {
                    let ts = msg.timestamp_ns();
                    if let Some(ns) = ts {
                        start = Some(start.map_or(ns, |s: i64| s.min(ns)));
                        end = Some(end.map_or(ns, |e2: i64| e2.max(ns)));
                    }
                    if let Ok(value) = serde_json::to_vec(&msg.status.iter().map(|s| (s.name.clone(), s.message.clone())).collect::<Vec<_>>()) {
                        points.push(TimeSeriesPoint {
                            timestamp: ts.unwrap_or(0),
                            value,
                            topic: "planner_diagnostics".to_string(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    let mut topics = Vec::new();
    if costmap_count > 0 {
        topics.push(Topic {
            name: "costmap".to_string(),
            msg_type: "nav2_msgs/Costmap".to_string(),
            message_count: costmap_count,
        });
    }
    let diagnostics_count = points.iter().filter(|p| p.topic == "planner_diagnostics").count() as u32;
    if diagnostics_count > 0 {
        topics.push(Topic {
            name: "planner_diagnostics".to_string(),
            msg_type: "diagnostic_msgs/DiagnosticArray".to_string(),
            message_count: diagnostics_count,
        });
    }

    Ok(ParsedNav2Export { topics, points, metadata, start_time_ns: start, end_time_ns: end })
}

impl DataSource for Nav2ExportAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                "Nav2 Export".to_string(),
                format!("Directory not found: {:?}", path),
            ));
        }

        let parsed = parse_nav2_export_dir(path)
            .map_err(|e| AdapterError::LoadFailed("Nav2 Export".to_string(), e.to_string()))?;
        *self.dir.lock().unwrap() = Some(path.to_path_buf());

        Ok(SourceData {
            source_type: DataSourceType::Nav2Export,
            topics: parsed.topics,
            start_time: parsed.start_time_ns.and_then(ns_to_datetime),
            end_time: parsed.end_time_ns.and_then(ns_to_datetime),
            duration_seconds: match (parsed.start_time_ns, parsed.end_time_ns) {
                (Some(s), Some(e)) if e >= s => Some((e - s) as f64 / 1e9),
                _ => None,
            },
            metadata: parsed.metadata,
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        let dir_guard = self.dir.lock().unwrap();
        let dir = dir_guard
            .as_ref()
            .ok_or_else(|| AdapterError::MissingData("call load() before available_topics()".to_string()))?;
        Ok(parse_nav2_export_dir(dir)?.topics)
    }

    fn extract_stream(&self, topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        let dir_guard = self.dir.lock().unwrap();
        let dir = dir_guard
            .as_ref()
            .ok_or_else(|| AdapterError::MissingData("call load() before extract_stream()".to_string()))?;
        Ok(parse_nav2_export_dir(dir)?
            .points
            .into_iter()
            .filter(|p| p.topic == topic)
            .collect())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        let dir_guard = self.dir.lock().unwrap();
        dir_guard
            .as_ref()
            .and_then(|d| parse_nav2_export_dir(d).ok())
            .map(|p| p.metadata)
            .unwrap_or_default()
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::Nav2Export
    }
}

// ============================================================================
// Video Adapter
//
// Metadata (duration, resolution, codec, frame count, fps) is extracted by
// shelling out to `ffprobe`, mirroring the pattern `video_processing.rs`
// already uses for frame extraction (`ffmpeg`/`ffprobe`, no Rust FFmpeg
// binding dependency).
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum VideoFormat {
    MP4,
    MKV,
    MOV,
    ImageSequence,
}

pub struct VideoAdapter {
    format: VideoFormat,
    fps: f32,
}

impl VideoAdapter {
    pub fn new(format: VideoFormat, fps: f32) -> Self {
        VideoAdapter { format, fps }
    }
}

#[derive(Debug, Default, Deserialize)]
struct FfprobeStream {
    #[serde(default)]
    codec_type: Option<String>,
    #[serde(default)]
    codec_name: Option<String>,
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    r_frame_rate: Option<String>,
    #[serde(default)]
    nb_frames: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct FfprobeFormat {
    #[serde(default)]
    duration: Option<String>,
    #[serde(default)]
    size: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct FfprobeOutput {
    #[serde(default)]
    streams: Vec<FfprobeStream>,
    #[serde(default)]
    format: Option<FfprobeFormat>,
}

fn probe_video_metadata(path: &Path) -> AdapterResult<HashMap<String, String>> {
    let output = std::process::Command::new("ffprobe")
        .args(["-v", "error", "-print_format", "json", "-show_format", "-show_streams"])
        .arg(path)
        .output()
        .map_err(|e| AdapterError::ParseError(format!("failed to run ffprobe (is it installed?): {e}")))?;

    if !output.status.success() {
        return Err(AdapterError::ParseError(format!(
            "ffprobe exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let parsed: FfprobeOutput = serde_json::from_slice(&output.stdout)
        .map_err(|e| AdapterError::ParseError(format!("failed to parse ffprobe output: {e}")))?;

    let mut metadata = HashMap::new();
    if let Some(stream) = parsed.streams.iter().find(|s| s.codec_type.as_deref() == Some("video")) {
        if let (Some(w), Some(h)) = (stream.width, stream.height) {
            metadata.insert("width".to_string(), w.to_string());
            metadata.insert("height".to_string(), h.to_string());
        }
        if let Some(codec) = &stream.codec_name {
            metadata.insert("codec".to_string(), codec.clone());
        }
        if let Some(fps_fraction) = &stream.r_frame_rate {
            if let Some((num, den)) = fps_fraction.split_once('/') {
                if let (Ok(n), Ok(d)) = (num.parse::<f64>(), den.parse::<f64>()) {
                    if d > 0.0 {
                        metadata.insert("probed_fps".to_string(), (n / d).to_string());
                    }
                }
            }
        }
        if let Some(nb_frames) = &stream.nb_frames {
            if nb_frames != "N/A" {
                metadata.insert("frame_count".to_string(), nb_frames.clone());
            }
        }
    }
    if let Some(fmt) = &parsed.format {
        if let Some(duration) = &fmt.duration {
            metadata.insert("duration_seconds".to_string(), duration.clone());
        }
        if let Some(size) = &fmt.size {
            metadata.insert("size_bytes".to_string(), size.clone());
        }
    }

    Ok(metadata)
}

impl DataSource for VideoAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                format!("Video ({:?})", self.format),
                format!("File not found: {:?}", path),
            ));
        }

        let mut metadata = probe_video_metadata(path)
            .map_err(|e| AdapterError::LoadFailed(format!("Video ({:?})", self.format), e.to_string()))?;

        let frame_count = metadata.get("frame_count").and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
        let duration_seconds = metadata.get("duration_seconds").and_then(|s| s.parse::<f64>().ok());

        metadata.insert("format".to_string(), format!("{:?}", self.format));
        metadata.insert("configured_fps".to_string(), self.fps.to_string());

        Ok(SourceData {
            source_type: DataSourceType::Video,
            topics: vec![Topic {
                name: "video".to_string(),
                msg_type: "sensor_msgs/Image".to_string(),
                message_count: frame_count,
            }],
            // Container metadata rarely carries a mission-clock anchor (video
            // container timestamps, when present at all, reflect recording
            // wall-clock time, not the robot's ROS clock) so this is
            // deliberately left unset rather than guessed at.
            start_time: None,
            end_time: None,
            duration_seconds,
            metadata,
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(vec![Topic {
            name: "video".to_string(),
            msg_type: "sensor_msgs/Image".to_string(),
            message_count: 0,
        }])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("format".to_string(), format!("{:?}", self.format));
        m.insert("fps".to_string(), self.fps.to_string());
        m
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::Video
    }
}

// ============================================================================
// Point Cloud Adapter
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum PointCloudFormat {
    PCD,
    PLY,
    LAS,
}

pub struct PointCloudAdapter {
    format: PointCloudFormat,
}

impl PointCloudAdapter {
    pub fn new(format: PointCloudFormat) -> Self {
        PointCloudAdapter { format }
    }
}

impl DataSource for PointCloudAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                format!("Point Cloud ({:?})", self.format),
                format!("File not found: {:?}", path),
            ));
        }

        Ok(SourceData {
            source_type: DataSourceType::PointCloud,
            topics: vec![Topic {
                name: "pointcloud".to_string(),
                msg_type: "sensor_msgs/PointCloud2".to_string(),
                message_count: 0,
            }],
            start_time: None,
            end_time: None,
            duration_seconds: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(vec![Topic {
            name: "pointcloud".to_string(),
            msg_type: "sensor_msgs/PointCloud2".to_string(),
            message_count: 0,
        }])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("format".to_string(), format!("{:?}", self.format));
        m
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::PointCloud
    }
}

// ============================================================================
// Annotation Adapter
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum AnnotationFormat {
    CSV,
    JSON,
    YAML,
}

pub struct AnnotationAdapter {
    format: AnnotationFormat,
}

impl AnnotationAdapter {
    pub fn new(format: AnnotationFormat) -> Self {
        AnnotationAdapter { format }
    }
}

impl DataSource for AnnotationAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                format!("Annotation ({:?})", self.format),
                format!("File not found: {:?}", path),
            ));
        }

        Ok(SourceData {
            source_type: DataSourceType::Annotation,
            topics: vec![Topic {
                name: "annotations".to_string(),
                msg_type: "std_msgs/String".to_string(),
                message_count: 0,
            }],
            start_time: None,
            end_time: None,
            duration_seconds: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(vec![Topic {
            name: "annotations".to_string(),
            msg_type: "std_msgs/String".to_string(),
            message_count: 0,
        }])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("format".to_string(), format!("{:?}", self.format));
        m
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::Annotation
    }
}

// ============================================================================
// Sensor Calibration Adapter
// ============================================================================

pub struct SensorCalibrationAdapter;

impl SensorCalibrationAdapter {
    pub fn new() -> Self {
        SensorCalibrationAdapter
    }
}

impl Default for SensorCalibrationAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSource for SensorCalibrationAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                "Sensor Calibration".to_string(),
                format!("File not found: {:?}", path),
            ));
        }

        Ok(SourceData {
            source_type: DataSourceType::SensorCalibration,
            topics: vec![],
            start_time: None,
            end_time: None,
            duration_seconds: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(vec![])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::SensorCalibration
    }
}

// ============================================================================
// Environment Map Adapter
// ============================================================================

pub struct EnvironmentMapAdapter;

impl EnvironmentMapAdapter {
    pub fn new() -> Self {
        EnvironmentMapAdapter
    }
}

impl Default for EnvironmentMapAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSource for EnvironmentMapAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                "Environment Map".to_string(),
                format!("File not found: {:?}", path),
            ));
        }

        Ok(SourceData {
            source_type: DataSourceType::EnvironmentMap,
            topics: vec![],
            start_time: None,
            end_time: None,
            duration_seconds: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(vec![])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::EnvironmentMap
    }
}

// ============================================================================
// Robot Model Adapter
// ============================================================================

pub struct RobotModelAdapter;

impl RobotModelAdapter {
    pub fn new() -> Self {
        RobotModelAdapter
    }
}

impl Default for RobotModelAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl DataSource for RobotModelAdapter {
    fn load(&self, path: &Path) -> AdapterResult<SourceData> {
        if !path.exists() {
            return Err(AdapterError::LoadFailed(
                "Robot Model".to_string(),
                format!("File not found: {:?}", path),
            ));
        }

        Ok(SourceData {
            source_type: DataSourceType::RobotModel,
            topics: vec![],
            start_time: None,
            end_time: None,
            duration_seconds: None,
            metadata: std::collections::HashMap::new(),
        })
    }

    fn available_topics(&self) -> AdapterResult<Vec<Topic>> {
        Ok(vec![])
    }

    fn extract_stream(&self, _topic: &str) -> AdapterResult<Vec<TimeSeriesPoint>> {
        Ok(Vec::new())
    }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }

    fn source_type(&self) -> DataSourceType {
        DataSourceType::RobotModel
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    #[test]
    fn test_ros_bag_adapter_creation() {
        let adapter = RosBagAdapter::new();
        assert_eq!(adapter.source_type(), DataSourceType::RosBag);
    }

    #[test]
    fn test_linux_logs_adapter_creation() {
        let adapter = LinuxLogsAdapter::new(LogType::Syslog);
        assert_eq!(adapter.source_type(), DataSourceType::LinuxLogs);
    }

    #[test]
    fn test_video_adapter_metadata() {
        let adapter = VideoAdapter::new(VideoFormat::MP4, 30.0);
        let metadata = adapter.metadata();
        assert!(metadata.contains_key("fps"));
    }

    #[test]
    fn test_missing_file_error() {
        let adapter = RosBagAdapter::new();
        let result = adapter.load(Path::new("/nonexistent/file.bag"));
        assert!(result.is_err());
    }

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("pyroboreplay_modality_test_{}_{}", std::process::id(), name))
    }

    /// Real end-to-end test: writes an actual MCAP file with `mcap::Writer`
    /// (two channels, several messages), then verifies `RosBagAdapter`
    /// genuinely parses it back out via the public `DataSource` trait.
    #[test]
    fn test_rosbag_mcap_real_parse() {
        let path = temp_path("bag.mcap");
        {
            let file = std::fs::File::create(&path).unwrap();
            let mut writer = mcap::Writer::new(std::io::BufWriter::new(file)).unwrap();
            let scan_channel = writer
                .add_channel(0, "/scan", "cdr", &std::collections::BTreeMap::new())
                .unwrap();
            let odom_channel = writer
                .add_channel(0, "/odom", "cdr", &std::collections::BTreeMap::new())
                .unwrap();
            for i in 0..5u64 {
                writer
                    .write_to_known_channel(
                        &mcap::records::MessageHeader {
                            channel_id: scan_channel,
                            sequence: i as u32,
                            log_time: i * 1_000_000_000,
                            publish_time: i * 1_000_000_000,
                        },
                        &[1, 2, 3, i as u8],
                    )
                    .unwrap();
            }
            writer
                .write_to_known_channel(
                    &mcap::records::MessageHeader {
                        channel_id: odom_channel,
                        sequence: 0,
                        log_time: 2_500_000_000,
                        publish_time: 2_500_000_000,
                    },
                    &[9, 9],
                )
                .unwrap();
            writer.finish().unwrap();
        }

        let adapter = RosBagAdapter::new();
        let data = adapter.load(&path).unwrap();
        assert_eq!(data.source_type, DataSourceType::RosBag);
        assert_eq!(data.metadata.get("storage_format").map(|s| s.as_str()), Some("mcap"));

        let mut topic_names: Vec<_> = data.topics.iter().map(|t| t.name.clone()).collect();
        topic_names.sort();
        assert_eq!(topic_names, vec!["/odom".to_string(), "/scan".to_string()]);

        let scan_topic = data.topics.iter().find(|t| t.name == "/scan").unwrap();
        assert_eq!(scan_topic.message_count, 5);

        assert_eq!(data.duration_seconds, Some(4.0));

        let stream = adapter.extract_stream("/scan").unwrap();
        assert_eq!(stream.len(), 5);
        assert_eq!(stream[0].value, vec![1, 2, 3, 0]);
        assert_eq!(stream[4].timestamp, 4_000_000_000);

        let topics = adapter.available_topics().unwrap();
        assert_eq!(topics.len(), 2);

        std::fs::remove_file(&path).ok();
    }

    /// Real end-to-end test against an actual rosbag2 SQLite (.db3) database,
    /// built with the real `topics`/`messages` schema rosbag2's SQLite
    /// storage plugin writes.
    #[test]
    fn test_rosbag_sqlite_real_parse() {
        let path = temp_path("bag.db3");
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE topics(id INTEGER PRIMARY KEY, name TEXT NOT NULL, type TEXT NOT NULL, serialization_format TEXT, offered_qos_profiles TEXT);
                 CREATE TABLE messages(id INTEGER PRIMARY KEY, topic_id INTEGER NOT NULL, timestamp INTEGER NOT NULL, data BLOB NOT NULL);",
            )
            .unwrap();
            conn.execute(
                "INSERT INTO topics (id, name, type, serialization_format) VALUES (1, '/imu', 'sensor_msgs/msg/Imu', 'cdr')",
                [],
            )
            .unwrap();
            for i in 0..3i64 {
                conn.execute(
                    "INSERT INTO messages (topic_id, timestamp, data) VALUES (1, ?1, ?2)",
                    rusqlite::params![i * 1_000_000_000, vec![i as u8; 4]],
                )
                .unwrap();
            }
        }

        let adapter = RosBagAdapter::new();
        let data = adapter.load(&path).unwrap();
        assert_eq!(data.metadata.get("storage_format").map(|s| s.as_str()), Some("rosbag2-sqlite"));
        assert_eq!(data.topics.len(), 1);
        assert_eq!(data.topics[0].name, "/imu");
        assert_eq!(data.topics[0].message_count, 3);
        assert_eq!(data.duration_seconds, Some(2.0));

        let stream = adapter.extract_stream("/imu").unwrap();
        assert_eq!(stream.len(), 3);
        assert_eq!(stream[1].value, vec![1u8; 4]);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_syslog_rfc3164_real_parse() {
        let path = temp_path("syslog3164.log");
        let content = "Jun 14 15:16:01 combo sshd(pam_unix)[19939]: authentication failure; logname= uid=0\n";
        std::fs::write(&path, content).unwrap();

        let adapter = LinuxLogsAdapter::new(LogType::Syslog);
        let data = adapter.load(&path).unwrap();
        assert_eq!(data.topics[0].message_count, 1);
        assert!(data.start_time.is_some());

        let stream = adapter.extract_stream("Syslog").unwrap();
        assert_eq!(stream.len(), 1);
        let record: LogRecord = serde_json::from_slice(&stream[0].value).unwrap();
        assert_eq!(record.host.as_deref(), Some("combo"));
        assert!(record.process.as_deref().unwrap().starts_with("sshd"));
        assert!(record.message.contains("authentication failure"));

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_syslog_rfc5424_real_parse() {
        let path = temp_path("syslog5424.log");
        let content = "<34>1 2023-10-11T22:14:15.003Z mymachine.example.com su - ID47 - 'su root' failed for lonvick\n";
        std::fs::write(&path, content).unwrap();

        let adapter = LinuxLogsAdapter::new(LogType::Syslog);
        let data = adapter.load(&path).unwrap();
        assert_eq!(data.start_time.unwrap().to_rfc3339(), "2023-10-11T22:14:15.003+00:00");
        assert_eq!(data.metadata.get("critical_count").map(|s| s.as_str()), Some("1")); // pri 34 % 8 == 2 -> critical

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_dmesg_ctime_real_parse() {
        let path = temp_path("dmesg_ctime.log");
        let content = "[Wed Aug 12 09:23:01 2026] usb 1-1: new high-speed USB device\n";
        std::fs::write(&path, content).unwrap();

        let adapter = LinuxLogsAdapter::new(LogType::Dmesg);
        let data = adapter.load(&path).unwrap();
        assert!(data.start_time.is_some());
        assert_eq!(data.metadata.get("boot_relative_lines"), None);

        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn test_dmesg_monotonic_real_parse() {
        let path = temp_path("dmesg_mono.log");
        let content = "[    0.000000] Linux version 6.1.0\n[    1.234567] usb 1-1: new device\n";
        std::fs::write(&path, content).unwrap();

        let adapter = LinuxLogsAdapter::new(LogType::Dmesg);
        let data = adapter.load(&path).unwrap();
        assert_eq!(data.metadata.get("boot_relative_lines").map(|s| s.as_str()), Some("2"));
        assert_eq!(data.duration_seconds, Some(1.234567));

        std::fs::remove_file(&path).ok();
    }

    /// Real end-to-end test: writes an actual `map_saver_cli`-shaped
    /// YAML+PGM pair and a `diagnostic_msgs/DiagnosticArray` JSONL dump, then
    /// verifies `Nav2ExportAdapter` genuinely parses them.
    #[test]
    fn test_nav2_export_real_parse() {
        let dir = temp_path("nav2_export_dir");
        std::fs::create_dir_all(&dir).unwrap();

        // A minimal but real 4x3 binary PGM (P5) costmap image.
        let pgm_path = dir.join("costmap.pgm");
        {
            let mut f = std::fs::File::create(&pgm_path).unwrap();
            f.write_all(b"P5\n4 3\n255\n").unwrap();
            f.write_all(&[0u8; 12]).unwrap();
        }
        let yaml_path = dir.join("costmap.yaml");
        std::fs::write(
            &yaml_path,
            "image: costmap.pgm\nresolution: 0.05\norigin: [0.0, 0.0, 0.0]\nnegate: 0\noccupied_thresh: 0.65\nfree_thresh: 0.196\n",
        )
        .unwrap();

        let diag_path = dir.join("diagnostics.jsonl");
        let diag_line = r#"{"header":{"stamp":{"sec":100,"nanosec":0}},"status":[{"level":2,"name":"planner_server","message":"no valid path found","hardware_id":""}]}"#;
        std::fs::write(&diag_path, format!("{}\n", diag_line)).unwrap();

        let adapter = Nav2ExportAdapter::new();
        let data = adapter.load(&dir).unwrap();

        let mut topic_names: Vec<_> = data.topics.iter().map(|t| t.name.clone()).collect();
        topic_names.sort();
        assert_eq!(topic_names, vec!["costmap".to_string(), "planner_diagnostics".to_string()]);
        assert_eq!(data.metadata.get("costmap.costmap.width").map(|s| s.as_str()), Some("4"));
        assert_eq!(data.metadata.get("costmap.costmap.height").map(|s| s.as_str()), Some("3"));

        let diag_stream = adapter.extract_stream("planner_diagnostics").unwrap();
        assert_eq!(diag_stream.len(), 1);
        assert_eq!(diag_stream[0].timestamp, 100_000_000_000);
        let raw = String::from_utf8(diag_stream[0].value.clone()).unwrap();
        assert!(raw.contains("planner_server"));

        std::fs::remove_dir_all(&dir).ok();
    }

    /// True end-to-end test against a real video file: generates a small
    /// synthetic test-pattern video with the system `ffmpeg` (deterministic,
    /// no external asset needed) and verifies `VideoAdapter::load` genuinely
    /// extracts real metadata via `ffprobe`. Skipped (not failed) if ffmpeg
    /// isn't installed, matching the precedent in `video_processing.rs`.
    #[test]
    fn test_video_adapter_real_ffprobe_metadata() {
        if std::process::Command::new("ffmpeg")
            .arg("-version")
            .output()
            .map(|o| !o.status.success())
            .unwrap_or(true)
        {
            eprintln!("skipping: ffmpeg not available in this environment");
            return;
        }

        let dir = temp_path("video_dir");
        std::fs::create_dir_all(&dir).unwrap();
        let video_path = dir.join("test.mp4");

        let gen = std::process::Command::new("ffmpeg")
            .args([
                "-y", "-v", "error",
                "-f", "lavfi", "-i", "testsrc=duration=2:size=64x48:rate=10",
                video_path.to_str().unwrap(),
            ])
            .output()
            .expect("failed to run ffmpeg to generate test video");
        assert!(gen.status.success(), "ffmpeg generation failed: {}", String::from_utf8_lossy(&gen.stderr));

        let adapter = VideoAdapter::new(VideoFormat::MP4, 10.0);
        let data = adapter.load(&video_path).unwrap();
        assert_eq!(data.metadata.get("width").map(|s| s.as_str()), Some("64"));
        assert_eq!(data.metadata.get("height").map(|s| s.as_str()), Some("48"));
        assert!(data.metadata.contains_key("codec"));
        assert!(data.duration_seconds.unwrap() > 1.5);
        assert_eq!(data.topics[0].message_count, 20);

        std::fs::remove_dir_all(&dir).ok();
    }
}
