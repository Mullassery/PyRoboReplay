/// Example: Launch PyRoboReplay with separate stats dashboard
///
/// This example demonstrates how to:
/// 1. Load a mission from a bag file
/// 2. Launch a stats dashboard in a separate terminal window
/// 3. Run the main replay UI independently
///
/// Usage:
///   cargo run --example stats_dashboard_demo -- your_mission.bag
///   cargo run --example stats_dashboard_demo -- --help

use pyroboreplay::adapters::{MissionAdapter, ros2::Ros2Adapter};
use pyroboreplay::cli::stats_dashboard::{launch_stats_dashboard_window, Platform};
use std::env;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args[1] == "--help" || args[1] == "-h" {
        println!("Usage: {} <bag_file> [--no-dashboard]", args[0]);
        println!("\nOptions:");
        println!("  <bag_file>        Path to ROS 2 bag file");
        println!("  --no-dashboard    Don't launch stats dashboard");
        println!("\nExample:");
        println!("  cargo run --example stats_dashboard_demo -- exploration_v1.bag");
        return Ok(());
    }

    let bag_file = &args[1];
    let launch_dashboard = !args.contains(&"--no-dashboard".to_string());

    println!("🤖 PyRoboReplay Stats Dashboard Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Platform: {}", Platform::detect().display());
    println!("Loading mission: {}", bag_file);

    // Load mission
    let adapter = Ros2Adapter::new();
    let mission = adapter.read(bag_file)?;

    println!("✅ Loaded mission with {} events", mission.event_count());

    // Launch stats dashboard in separate window if requested
    if launch_dashboard {
        println!("\n📊 Launching stats dashboard in separate terminal...");
        match launch_stats_dashboard_window(&mission, "PyRoboReplay Example Dashboard") {
            Ok(mut child) => {
                println!("✅ Dashboard launched successfully!");
                println!("   PID: {}", child.id());
                println!("   Close the dashboard window or press Ctrl+C to exit");

                // Give user time to see the dashboard before exiting
                println!("\n⏳ Dashboard will stay open for 30 seconds...");
                thread::sleep(Duration::from_secs(30));

                // Try to kill the child process gracefully
                let _ = child.kill();
                let _ = child.wait();

                println!("✅ Example complete!");
            }
            Err(e) => {
                eprintln!("⚠️  Failed to launch dashboard: {}", e);
                eprintln!("   This example requires a terminal emulator:");
                eprintln!("   macOS: Terminal.app or iTerm2");
                eprintln!("   Linux: terminator, gnome-terminal, xterm, or xfce4-terminal");
            }
        }
    } else {
        println!("📊 Dashboard launch disabled (use --no-dashboard to skip)");
    }

    println!("\n📊 Mission Summary:");
    println!("  Name: {}", mission.name);
    println!("  Events: {}", mission.event_count());

    if let Some(duration) = mission.duration() {
        println!("  Duration: {}s", duration.num_seconds());
    }

    // Count by event type
    let mut type_counts: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();
    for event in &mission.events {
        *type_counts.entry(event.event_type()).or_insert(0) += 1;
    }

    println!("\n  Event Types:");
    for (event_type, count) in type_counts {
        println!("    {}: {}", event_type, count);
    }

    Ok(())
}
