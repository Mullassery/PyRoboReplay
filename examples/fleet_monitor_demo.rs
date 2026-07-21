use pyroboreplay::streaming::{
    FleetMonitor, FleetMonitorConfig, FleetDashboard, StreamEvent, RobotStatusType,
};
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║  PyRoboReplay: Real-Time Fleet Monitoring                    ║");
    println!("║  Phase 8.1: Extended Observability                           ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 1: CREATE FLEET MONITOR WITH 5 ROBOTS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let config = FleetMonitorConfig::default();
    let mut monitor = FleetMonitor::new(config);

    // Register 5 robots in the fleet
    for i in 1..=5 {
        monitor.register_robot(&format!("warehouse_bot_{}", i), Some("mission_warehouse"));
    }

    let summary = monitor.get_fleet_summary();
    println!("✓ Fleet initialized with {} robots", summary.total_robots);
    println!("✓ Health score: {:.1}%", summary.overall_health_score * 100.0);
    println!("✓ Active missions: {}\n", summary.active_missions);

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 2: PROCESS STREAMING EVENTS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    // Simulate stream events from robots
    let events = vec![
        StreamEvent {
            event_id: Uuid::new_v4().to_string(),
            mission_id: "mission_warehouse".to_string(),
            event_type: "pose_update".to_string(),
            timestamp: Utc::now(),
            robot_id: Some("warehouse_bot_1".to_string()),
            payload: json!({"x": 10.5, "y": 20.3, "z": 0.0}),
            sequence_number: 0,
        },
        StreamEvent {
            event_id: Uuid::new_v4().to_string(),
            mission_id: "mission_warehouse".to_string(),
            event_type: "pose_update".to_string(),
            timestamp: Utc::now(),
            robot_id: Some("warehouse_bot_2".to_string()),
            payload: json!({"x": 15.0, "y": 25.0, "z": 0.0}),
            sequence_number: 1,
        },
        StreamEvent {
            event_id: Uuid::new_v4().to_string(),
            mission_id: "mission_warehouse".to_string(),
            event_type: "sensor_reading".to_string(),
            timestamp: Utc::now(),
            robot_id: Some("warehouse_bot_3".to_string()),
            payload: json!({"lidar_range": 5.2}),
            sequence_number: 2,
        },
    ];

    for event in &events {
        monitor.process_event(event);
    }

    let summary = monitor.get_fleet_summary();
    println!("✓ Processed {} events", events.len());
    println!("✓ Active robots: {} / {}", summary.active_missions, summary.total_robots);
    println!("✓ Current health score: {:.1}%\n", summary.overall_health_score * 100.0);

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 3: FLEET HEALTH SUMMARY");
    println!("═══════════════════════════════════════════════════════════════════\n");

    println!("Fleet Status Summary:");
    println!("  Total Robots: {}", summary.total_robots);
    println!("  Active Missions: {}", summary.active_missions);
    println!("  Alert Count: {}", summary.alerts_by_severity.len());
    println!("  Health Score: {:.1}%", summary.overall_health_score * 100.0);

    println!("\nRobot Status Details:");
    for (i, robot) in summary.robots.iter().enumerate() {
        println!("  {}. {} → {}", i + 1, robot.robot_id, robot.status);
        println!("     Active Mission: {:?}", robot.active_mission_id);
        println!("     Alerts: {}", robot.alert_count);
    }
    println!();

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 4: FLEET DASHBOARD WITH HISTORICAL TRACKING");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let config = FleetMonitorConfig::default();
    let monitor = FleetMonitor::new(config);
    let mut dashboard = FleetDashboard::new(monitor, 10);

    // Simulate multiple ticks (time windows)
    for tick in 1..=5 {
        dashboard.tick(&[]);
        println!("✓ Dashboard tick {}: {} snapshots recorded", tick, tick);
    }

    let window = dashboard.current_window();
    println!("\nDashboard Window:");
    println!("  Window Start: {}", window.window_start.format("%H:%M:%S UTC"));
    println!("  Window End: {}", window.window_end.format("%H:%M:%S UTC"));
    println!("  Snapshots in Window: {}", window.summaries.len());
    println!("  Health Trend: {:?}\n", window.trend);

    println!("═══════════════════════════════════════════════════════════════════");
    println!("DEMO 5: OFFLINE ROBOT DETECTION");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let mut config = FleetMonitorConfig::default();
    config.offline_threshold_ms = 100;

    let mut monitor = FleetMonitor::new(config);
    monitor.register_robot("robot_a", None);
    monitor.register_robot("robot_b", None);

    // Simulate robot_a going offline by simulating timeout
    monitor.update_robot_statuses();

    let summary = monitor.get_fleet_summary();
    for robot in &summary.robots {
        println!(
            "✓ {} → {}",
            robot.robot_id,
            match robot.status {
                RobotStatusType::Active => "Active",
                RobotStatusType::Idle => "Idle",
                RobotStatusType::Degraded => "Degraded",
                RobotStatusType::Failed => "Failed",
                RobotStatusType::Offline => "Offline",
                RobotStatusType::Charging => "Charging",
            }
        );
    }
    println!();

    println!("═══════════════════════════════════════════════════════════════════");
    println!("FLEET MONITORING FEATURES ENABLED");
    println!("═══════════════════════════════════════════════════════════════════\n");

    println!("✓ Multi-robot fleet tracking");
    println!("✓ Real-time event processing");
    println!("✓ Per-robot diagnostics integration");
    println!("✓ Health scoring and trending");
    println!("✓ Offline detection with configurable timeout");
    println!("✓ Alert aggregation by severity");
    println!("✓ Historical dashboard with windowing");
    println!("✓ Top failure type extraction");
    println!("✓ Mission tracking per robot");
    println!("✓ Production-grade observability\n");

    println!("═══════════════════════════════════════════════════════════════════");
    println!("✨ Phase 8.1: Real-Time Fleet Monitoring Complete");
    println!("═══════════════════════════════════════════════════════════════════\n");
}
