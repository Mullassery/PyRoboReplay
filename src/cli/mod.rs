pub mod args;
pub mod replay_ui;
pub mod json_output;
pub mod lidar_viz;
pub mod camera_export;
pub mod imu_viz;
pub mod sensor_stats;
pub mod keyboard;
pub mod causal_viz;
pub mod gap_analysis;
pub mod consolidated_output;
mod tests_consolidation_integration;

use args::{Cli, Commands};
use clap::Parser;
use crate::adapters::{MissionAdapter, ros2::Ros2Adapter};
use replay_ui::ReplayState;
use json_output::{JsonResponse, MissionAnalysisJson};
use camera_export::export_camera_to_html;
use std::error::Error;
use tracing_subscriber;

pub fn run() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Replay {
            bag_file,
            sensor,
            start_time: _,
            end_time: _,
            robot: _,
            json: output_json,
            export_camera,
        } => {
            tracing::info!("Replaying: {}", bag_file);

            // Parse the bag file
            let adapter = Ros2Adapter::new();
            let mission = adapter.read(&bag_file)?;

            tracing::info!("Loaded mission with {} events", mission.event_count());

            // Handle camera export if requested
            if let Some(export_path) = export_camera {
                tracing::info!("Exporting camera frames to: {}", export_path);
                export_camera_to_html(&mission, &export_path, None)?;
                println!("✅ Camera frames exported to: {}", export_path);
                println!("📖 Open in browser: open {}", export_path);
                return Ok(());
            }

            if output_json {
                // Output mission analysis as JSON
                let analysis = MissionAnalysisJson::from_mission(&mission);
                let response = JsonResponse::success(analysis);
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                // Parse sensor filter
                let sensors = sensor.map(|s| {
                    s.split(',')
                        .map(|sensor| sensor.trim().to_string())
                        .collect()
                });

                // Run interactive replay
                let mut replay = ReplayState::new(mission, sensors);
                replay.run()?;
            }
        }

        Commands::Compare {
            bag_file_1,
            bag_file_2,
        } => {
            tracing::info!("Comparing: {} vs {}", bag_file_1, bag_file_2);
            println!("Multi-mission comparison coming in v0.4");
        }

        Commands::Analyze {
            bag_file,
            detect_gaps,
            format: output_format,
            output: output_file,
            detail,
            json: output_json
        } => {
            tracing::info!("Analyzing: {}", bag_file);
            let adapter = Ros2Adapter::new();
            let mission = adapter.read(&bag_file)?;

            // Handle gap detection if requested
            if detect_gaps {
                use crate::analyzers::{
                    MissionAnalysisData, RealityGapDetector, ControlMessage, JointState,
                    OdometryMessage, CameraFrame, LidarScan, IMUMeasurement, EncoderReading,
                    MotorCurrent, ThermalReading, BatteryReading, DetectionResult, PerceptionError,
                    MessageTimestamp
                };
                use gap_analysis::GapFormatter;

                tracing::info!("Detecting reality gaps...");

                // Convert mission to analysis data (simplified - normally would populate all fields)
                let analysis_data = MissionAnalysisData {
                    mission_id: mission.id.to_string(),
                    duration_sec: mission.duration().map(|d| d.num_seconds() as f32).unwrap_or(0.0),
                    robot_type: "unknown".to_string(),
                    control_messages: Vec::new(),
                    joint_states: Vec::new(),
                    odometry_messages: Vec::new(),
                    camera_frames: Vec::new(),
                    lidar_scans: Vec::new(),
                    imu_measurements: Vec::new(),
                    encoder_data: Vec::new(),
                    motor_currents: Vec::new(),
                    thermal_readings: Vec::new(),
                    battery_data: Vec::new(),
                    detection_results: Vec::new(),
                    perception_errors: Vec::new(),
                    message_timestamps: Vec::new(),
                };

                let detector = RealityGapDetector::new();
                let findings = detector.analyze_mission(&analysis_data);

                // Format output
                let formatter = GapFormatter::new(detail);
                let output_text = match output_format.as_str() {
                    "json" => formatter.format_json(&findings).to_string(),
                    "html" => formatter.format_html(&findings, &mission.id.to_string()),
                    _ => formatter.format_text(&findings),
                };

                // Save or print
                if let Some(path) = output_file {
                    std::fs::write(&path, &output_text)?;
                    println!("✅ Gap analysis saved to: {}", path);
                } else {
                    println!("{}", output_text);
                }
            } else if output_json || output_format == "json" {
                // Legacy: output mission analysis as JSON
                let analysis = MissionAnalysisJson::from_mission(&mission);
                let response = JsonResponse::success(analysis);
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                // Print statistics (human-readable)
                println!("\n📊 Mission Analysis: {}", mission.name);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("Total events: {}", mission.event_count());

                if let Some(duration) = mission.duration() {
                    println!("Duration: {}s", duration.num_seconds());
                }

                // Count by type
                let mut event_counts: std::collections::HashMap<&str, usize> =
                    std::collections::HashMap::new();
                for event in &mission.events {
                    *event_counts.entry(event.event_type()).or_insert(0) += 1;
                }

                println!("\nEvent types:");
                for (event_type, count) in event_counts {
                    println!("  {}: {}", event_type, count);
                }
            }
        }

        Commands::List { bag_file, json: output_json } => {
            tracing::info!("Listing topics in: {}", bag_file);
            let adapter = Ros2Adapter::new();
            let mission = adapter.read(&bag_file)?;

            // Get unique sensors
            let sensors: std::collections::HashSet<_> = mission
                .events
                .iter()
                .filter_map(|e| e.sensor_type())
                .collect();

            if output_json {
                // Output sensor list as JSON
                let sensor_data: Vec<_> = sensors
                    .iter()
                    .map(|sensor| {
                        let count = mission
                            .events
                            .iter()
                            .filter(|e| e.sensor_type().map_or(false, |s| s == *sensor))
                            .count();
                        serde_json::json!({
                            "sensor": sensor,
                            "frame_count": count
                        })
                    })
                    .collect();

                let response = JsonResponse::success(
                    serde_json::json!({
                        "mission_id": mission.id.to_string(),
                        "mission_name": mission.name,
                        "sensors": sensor_data
                    })
                );
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                // Print human-readable format
                println!("\n📡 Available Sensors:");
                for sensor in sensors {
                    let count = mission
                        .events
                        .iter()
                        .filter(|e| e.sensor_type().map_or(false, |s| s == sensor))
                        .count();
                    println!("  {} ({} frames)", sensor, count);
                }
            }
        }
    }

    Ok(())
}
