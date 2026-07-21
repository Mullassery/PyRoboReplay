pub mod args;
pub mod replay_ui;
pub mod json_output;

use args::{Cli, Commands};
use clap::Parser;
use crate::adapters::{MissionAdapter, ros2::Ros2Adapter};
use replay_ui::ReplayState;
use json_output::{JsonResponse, MissionAnalysisJson};
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
            export_camera: _,
        } => {
            tracing::info!("Replaying: {}", bag_file);

            // Parse the bag file
            let adapter = Ros2Adapter::new();
            let mission = adapter.read(&bag_file)?;

            tracing::info!("Loaded mission with {} events", mission.event_count());

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

        Commands::Analyze { bag_file, json: output_json } => {
            tracing::info!("Analyzing: {}", bag_file);
            let adapter = Ros2Adapter::new();
            let mission = adapter.read(&bag_file)?;

            if output_json {
                // Output as JSON for AI-agent integration
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
