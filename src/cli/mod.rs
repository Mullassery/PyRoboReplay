pub mod args;
pub mod camera_export;
pub mod causal_viz;
pub mod consolidated_output;
pub mod export;
pub mod gap_analysis;
pub mod imu_viz;
pub mod json_output;
pub mod keyboard;
pub mod lidar_viz;
pub mod replay_ui;
pub mod sensor_stats;
pub mod stats_dashboard;
mod tests_consolidation_integration;

use crate::adapters::{ros2::Ros2Adapter, MissionAdapter};
use args::{Cli, Commands};
use camera_export::export_camera_to_html;
use clap::Parser;
use json_output::{JsonResponse, MissionAnalysisJson};
use replay_ui::ReplayState;
use stats_dashboard::launch_stats_dashboard_window;
use std::error::Error;
use tracing_subscriber;

pub fn run() -> Result<(), Box<dyn Error>> {
    run_with_args(std::env::args())
}

/// Run the CLI from an explicit argument iterator (first item is argv[0]).
///
/// The standalone binary uses [`run`]; Python's console entry point passes
/// `sys.argv` here through the `run_cli` PyO3 function.
pub fn run_with_args<I, T>(args: I) -> Result<(), Box<dyn Error>>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    // `try_init` (not `init`) so an in-process second invocation — e.g. the
    // Python entry point called twice — does not panic on a duplicate global
    // tracing subscriber.
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    let cli = Cli::parse_from(args);

    match cli.command {
        Commands::Replay {
            bag_file,
            sensor,
            start_time: _,
            end_time: _,
            robot: _,
            json: output_json,
            export_camera,
            stats_dashboard: launch_dashboard,
        } => {
            tracing::info!("Replaying: {}", bag_file);

            // Parse the bag file
            let adapter = Ros2Adapter::new();
            let mission = adapter.read(&bag_file)?;

            tracing::info!("Loaded mission with {} events", mission.event_count());

            // Launch stats dashboard in separate window if requested
            if launch_dashboard {
                match launch_stats_dashboard_window(&mission, "PyRoboReplay Stats Dashboard") {
                    Ok(child) => {
                        println!("✅ Stats dashboard launched in a separate terminal");
                        println!("📊 To close: press Ctrl+C in the dashboard window");
                        // Don't wait for the child; let it run independently
                        let _ = child.id();
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to launch stats dashboard: {}", e);
                        eprintln!("Continuing with main replay...");
                    }
                }
            }

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
            json: output_json,
        } => {
            tracing::info!("Analyzing: {}", bag_file);
            let adapter = Ros2Adapter::new();
            let mission = adapter.read(&bag_file)?;

            // Handle gap detection if requested
            if detect_gaps {
                use crate::analyzers::{MissionAnalysisData, RealityGapDetector};
                use gap_analysis::GapFormatter;

                tracing::info!("Detecting reality gaps...");

                // Convert mission to analysis data (simplified - normally would populate all fields)
                let analysis_data = MissionAnalysisData {
                    mission_id: mission.id.to_string(),
                    duration_sec: mission
                        .duration()
                        .map(|d| d.num_seconds() as f32)
                        .unwrap_or(0.0),
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

        Commands::Terrain {
            grid_map,
            format,
            output,
        } => {
            use crate::core::pyterrainmap::GridMapDocument;

            let document = GridMapDocument::from_path(&grid_map)?;
            let summary = document.summary();
            let text = if format == "json" {
                serde_json::to_string_pretty(&serde_json::json!({
                    "document": document,
                    "summary": summary,
                }))?
            } else {
                format!(
                    "PyTerrainMap grid: {} cells, {} observations\n\
                     Cell size: {:?} m | elevation cells: {}\n\
                     Elevation: {:?} .. {:?} m (mean {:?})\n\
                     Mean confidence: {:?}\n",
                    summary.cells,
                    summary.total_observations,
                    summary.cell_size_m,
                    summary.elevation_cells,
                    summary.min_elevation_m,
                    summary.max_elevation_m,
                    summary.mean_elevation_m,
                    summary.mean_confidence
                )
            };
            write_output(output, &text)?;
        }

        Commands::Forensics {
            yolo,
            dino,
            rgb,
            thermal,
            format,
            output,
        } => {
            use crate::forensics;
            use crate::fusion::thermal_model::ThermalFrame;
            use crate::perception::object_detection::DetectedObject;
            use crate::perception::retrospective_detection::DINODetection;

            if yolo.is_none() && dino.is_none() && rgb.is_none() && thermal.is_none() {
                return Err(
                    "forensics requires at least one of --yolo, --dino, --rgb, --thermal".into(),
                );
            }

            let yolo_detections: Vec<DetectedObject> = match yolo {
                Some(path) => read_json(&path)?,
                None => Vec::new(),
            };
            let dino_detections: Vec<DINODetection> = match dino {
                Some(path) => read_json(&path)?,
                None => Vec::new(),
            };
            let rgb_detections: Vec<DetectedObject> = match rgb {
                Some(path) => read_json(&path)?,
                None => Vec::new(),
            };
            let thermal_frame: Option<ThermalFrame> = match thermal {
                Some(path) => Some(read_json(&path)?),
                None => None,
            };

            let gap_analysis = forensics::analyze_detection_gaps(yolo_detections, dino_detections);
            let fusion_analysis =
                thermal_frame.map(|frame| forensics::fuse_rgb_thermal(rgb_detections, frame));

            let text = if format == "json" {
                serde_json::to_string_pretty(&serde_json::json!({
                    "detection_gaps": gap_analysis,
                    "fusion": fusion_analysis,
                }))?
            } else {
                let mut text = forensics::gap_analysis_text(&gap_analysis);
                if let Some(fusion) = &fusion_analysis {
                    text.push('\n');
                    text.push_str(&forensics::fusion_analysis_text(fusion));
                }
                text
            };
            write_output(output, &text)?;
        }

        Commands::Nav2 {
            input,
            format,
            output,
        } => {
            let diagnostics: crate::phase15::Nav2Diagnostics = read_json(&input)?;
            let report = crate::phase15::analyze_nav2_diagnostics(&diagnostics);
            let text = if format == "json" {
                serde_json::to_string_pretty(&report)?
            } else {
                crate::phase15::nav2_report_text(&report)
            };
            write_output(output, &text)?;
        }

        Commands::Session {
            bag_file,
            logs,
            log_type,
            nav2_export,
            format,
            output,
        } => {
            use crate::phase14::modality_adapters::LogType;
            use crate::phase14::session_report::{build_navigation_session, SessionSources};
            use std::path::Path;

            let log_type = match log_type.to_ascii_lowercase().as_str() {
                "syslog" => LogType::Syslog,
                "dmesg" => LogType::Dmesg,
                "journalctl" => LogType::Journalctl,
                other => {
                    return Err(format!(
                        "unknown log type {other:?}; expected syslog, dmesg, or journalctl"
                    )
                    .into())
                }
            };
            let sources = SessionSources {
                bag: bag_file.as_deref().map(Path::new),
                log_file: logs.as_deref().map(Path::new),
                log_type: Some(log_type),
                nav2_export_dir: nav2_export.as_deref().map(Path::new),
            };
            let session = build_navigation_session("cli-session", &sources)?;
            let summary = session.public_summary();
            let text = if format == "json" {
                serde_json::to_string_pretty(&summary)?
            } else {
                let mut text = format!(
                    "Navigation session: {} events, {:.1}s, {:.0}% complete\nModalities: {}\n",
                    summary.stats.total_events,
                    summary.stats.duration_seconds,
                    summary.stats.data_completeness * 100.0,
                    summary.modalities.join(", ")
                );
                if let Some(ts) = summary.first_error_time_ns {
                    text.push_str(&format!("First error timestamp (ns): {ts}\n"));
                }
                for (topic, count) in &summary.topic_event_counts {
                    text.push_str(&format!("  {topic}: {count}\n"));
                }
                text
            };
            write_output(output, &text)?;
        }

        Commands::Export {
            bag_file,
            output,
            format,
        } => {
            let adapter = Ros2Adapter::new();
            let mission = adapter.read(&bag_file)?;
            match format.to_ascii_lowercase().as_str() {
                "mcap" => {
                    export::export_mcap(&mission, &output)?;
                    println!("✅ MCAP export saved to: {}", output);
                }
                "rerun" => {
                    return Err("rerun export is available through the Python API: \
                         `from pyroboreplay import export_mission`. The native binary \
                         exports MCAP only."
                        .into())
                }
                other => {
                    return Err(
                        format!("unknown export format {other:?}; expected mcap or rerun").into(),
                    )
                }
            }
        }

        Commands::List {
            bag_file,
            json: output_json,
        } => {
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
                            .filter(|e| e.sensor_type() == Some(*sensor))
                            .count();
                        serde_json::json!({
                            "sensor": sensor,
                            "frame_count": count
                        })
                    })
                    .collect();

                let response = JsonResponse::success(serde_json::json!({
                    "mission_id": mission.id.to_string(),
                    "mission_name": mission.name,
                    "sensors": sensor_data
                }));
                println!("{}", serde_json::to_string_pretty(&response)?);
            } else {
                // Print human-readable format
                println!("\n📡 Available Sensors:");
                for sensor in sensors {
                    let count = mission
                        .events
                        .iter()
                        .filter(|e| e.sensor_type() == Some(sensor))
                        .count();
                    println!("  {} ({} frames)", sensor, count);
                }
            }
        }
    }

    Ok(())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, Box<dyn Error>> {
    let content =
        std::fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))?;
    let parsed =
        serde_json::from_str(&content).map_err(|e| format!("invalid JSON in {path}: {e}"))?;
    Ok(parsed)
}

fn write_output(output: Option<String>, text: &str) -> Result<(), Box<dyn Error>> {
    match output {
        Some(path) => {
            std::fs::write(&path, text).map_err(|e| format!("failed to write {path}: {e}"))?;
            println!("✅ Saved to: {path}");
        }
        None => println!("{text}"),
    }
    Ok(())
}
