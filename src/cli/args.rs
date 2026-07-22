use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "PyRoboReplay")]
#[command(about = "Time-travel debugger for robot fleets", long_about = None)]
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
}
