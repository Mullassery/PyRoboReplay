use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "PyRoboReplay")]
#[command(
    about = "Offline mission replay and forensic analysis for recorded robot missions",
    long_about = None
)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Replay a mission from a ROS bag file
    Replay {
        /// Path to the bag file (.bag or .db3)
        #[arg(value_name = "BAG_FILE")]
        bag_file: String,

        /// Sensor(s) to filter (comma-separated: lidar,camera,imu)
        /// If not specified, shows all sensors
        #[arg(short, long, value_name = "SENSORS")]
        sensor: Option<String>,

        /// Start time (optional, ISO 8601 format)
        #[arg(long, value_name = "TIMESTAMP")]
        start_time: Option<String>,

        /// End time (optional, ISO 8601 format)
        #[arg(long, value_name = "TIMESTAMP")]
        end_time: Option<String>,

        /// Show only events at this robot (default: all)
        #[arg(short, long, value_name = "ROBOT_ID")]
        robot: Option<String>,

        /// Output as JSON (for AI-agent integration)
        #[arg(long)]
        json: bool,

        /// Export camera frames to standalone HTML file
        #[arg(long, value_name = "OUTPUT_FILE")]
        export_camera: Option<String>,

        /// Launch stats dashboard in a separate terminal window
        #[arg(long)]
        stats_dashboard: bool,
    },

    /// Compare two missions side-by-side
    Compare {
        /// First bag file
        #[arg(value_name = "BAG_FILE_1")]
        bag_file_1: String,

        /// Second bag file
        #[arg(value_name = "BAG_FILE_2")]
        bag_file_2: String,
    },

    /// Analyze a mission for statistics and anomalies
    Analyze {
        /// Bag file to analyze
        #[arg(value_name = "BAG_FILE")]
        bag_file: String,

        /// Detect reality gaps (sim-to-real issues)
        #[arg(long)]
        detect_gaps: bool,

        /// Output format: json, text, or html
        #[arg(long, default_value = "text")]
        format: String,

        /// Save results to file
        #[arg(long, value_name = "OUTPUT_FILE")]
        output: Option<String>,

        /// Show detailed findings (not just summary)
        #[arg(long)]
        detail: bool,

        /// Output as JSON (for AI-agent integration) [DEPRECATED: use --format json]
        #[arg(long)]
        json: bool,
    },

    /// List available topics in a bag file
    List {
        /// Bag file to inspect
        #[arg(value_name = "BAG_FILE")]
        bag_file: String,

        /// Output as JSON (for AI-agent integration)
        #[arg(long)]
        json: bool,
    },

    /// Load a PyTerrainMap grid-map JSON export
    Terrain {
        /// Path to a `write_grid_map()` JSON export
        #[arg(value_name = "GRID_MAP")]
        grid_map: String,

        /// Output format: text or json
        #[arg(long, default_value = "text")]
        format: String,

        /// Save results to file
        #[arg(long, value_name = "OUTPUT_FILE")]
        output: Option<String>,
    },

    /// Analyze external RGB/thermal and retrospective DINO artifacts
    Forensics {
        /// JSON array of the robot's YOLO detections
        #[arg(long, value_name = "JSON_FILE")]
        yolo: Option<String>,

        /// JSON array of retrospective DINO detections
        #[arg(long, value_name = "JSON_FILE")]
        dino: Option<String>,

        /// JSON array of RGB detections for thermal fusion
        #[arg(long, value_name = "JSON_FILE")]
        rgb: Option<String>,

        /// JSON thermal frame for fusion
        #[arg(long, value_name = "JSON_FILE")]
        thermal: Option<String>,

        /// Output format: text or json
        #[arg(long, default_value = "text")]
        format: String,

        /// Save results to file
        #[arg(long, value_name = "OUTPUT_FILE")]
        output: Option<String>,
    },

    /// Run Phase 15 Nav2 root-cause heuristics on a measurements document
    Nav2 {
        /// JSON document of Nav2 measurements (see docs/API.md)
        #[arg(value_name = "JSON_FILE")]
        input: String,

        /// Output format: text or json
        #[arg(long, default_value = "text")]
        format: String,

        /// Save results to file
        #[arg(long, value_name = "OUTPUT_FILE")]
        output: Option<String>,
    },

    /// Build a Phase 14 navigation session from recorded sources
    Session {
        /// ROS 2 bag file (.db3 or .mcap)
        #[arg(value_name = "BAG_FILE")]
        bag_file: Option<String>,

        /// Linux log file (syslog, dmesg, or journalctl output)
        #[arg(long, value_name = "LOG_FILE")]
        logs: Option<String>,

        /// Log format: syslog, dmesg, or journalctl
        #[arg(long, default_value = "syslog")]
        log_type: String,

        /// Directory containing a Nav2 map/diagnostics export
        #[arg(long, value_name = "DIRECTORY")]
        nav2_export: Option<String>,

        /// Output format: text or json
        #[arg(long, default_value = "text")]
        format: String,

        /// Save results to file
        #[arg(long, value_name = "OUTPUT_FILE")]
        output: Option<String>,
    },

    /// Export a recorded mission to an interchange format
    Export {
        /// ROS 2 bag file (.db3 or .mcap)
        #[arg(value_name = "BAG_FILE")]
        bag_file: String,

        /// Output path
        #[arg(value_name = "OUTPUT_FILE")]
        output: String,

        /// Format: mcap (native) or rerun (Python API; see docs)
        #[arg(long, default_value = "mcap")]
        format: String,
    },
}
