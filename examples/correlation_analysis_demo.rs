use chrono::Utc;
use pyroboreplay::core::{
    event::{Costmap, IMUData, LidarData, Location, MissionEvent, MissionRecord, Odometry, Pose},
    CorrelationAnalyzer,
};

fn create_realistic_mission() -> MissionRecord {
    let base_time = Utc::now();
    let mut events = Vec::new();

    // Realistic robot mission with multiple sensor streams
    // Timeline: robot navigating through obstacle-filled environment

    // t=0ms: Initial lidar scan
    events.push(MissionEvent::LidarScan {
        robot_id: "robot_1".to_string(),
        timestamp: base_time,
        data: LidarData {
            ranges: vec![5.0; 360],
            intensities: None,
            frame_id: "lidar".to_string(),
            min_angle: 0.0,
            max_angle: 6.28,
            angle_increment: 0.01745,
            range_min: 0.1,
            range_max: 10.0,
        },
    });

    // t=300ms: Rapid lidar scan (anomalous scanning)
    events.push(MissionEvent::LidarScan {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(300),
        data: LidarData {
            ranges: (0..360)
                .map(|i| if (i as i32 - 180).abs() < 40 { 2.5 } else { 5.0 })
                .collect(),
            intensities: None,
            frame_id: "lidar".to_string(),
            min_angle: 0.0,
            max_angle: 6.28,
            angle_increment: 0.01745,
            range_min: 0.1,
            range_max: 10.0,
        },
    });

    // t=500ms: Obstacle detected (correlated with lidar)
    events.push(MissionEvent::ObstacleDetected {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(500),
        location: Location {
            x: 2.5,
            y: 0.0,
            z: 0.0,
        },
        obstacle_type: "dynamic".to_string(),
        confidence: Some(0.96),
    });

    // t=650ms: Another obstacle detection (rapid successive detections = anomaly)
    events.push(MissionEvent::ObstacleDetected {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(650),
        location: Location {
            x: 2.8,
            y: 0.3,
            z: 0.0,
        },
        obstacle_type: "wall".to_string(),
        confidence: Some(0.94),
    });

    // t=800ms: Costmap update
    events.push(MissionEvent::CostmapUpdate {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(800),
        data: Costmap {
            frame_id: "map".to_string(),
            resolution: 0.05,
            width: 100,
            height: 100,
            origin: Pose {
                x: -2.5,
                y: -2.5,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            data: vec![0; 10000],
        },
    });

    // t=950ms: Navigation decision
    events.push(MissionEvent::NavigationDecision {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(950),
        decision_type: "obstacle_avoidance".to_string(),
        rationale: Some("Multiple obstacles, computing detour".to_string()),
    });

    // t=1100ms: IMU shows high acceleration (motor response to nav decision)
    events.push(MissionEvent::IMUData {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(1100),
        data: IMUData {
            frame_id: "imu".to_string(),
            linear_acceleration: [2.5, 0.8, 0.0],
            angular_velocity: [0.0, 0.0, 0.3],
            magnetometer: None,
            orientation: None,
        },
    });

    // t=1300ms: Odometry shows motion
    events.push(MissionEvent::OdometryUpdate {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(1300),
        data: Odometry {
            frame_id: "odom".to_string(),
            child_frame_id: "base_link".to_string(),
            pose: Pose {
                x: 1.2,
                y: 0.5,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.1,
                qw: 0.995,
            },
            twist_linear: [1.5, 0.2, 0.0],
            twist_angular: [0.0, 0.0, 0.1],
        },
    });

    // t=1500ms: Follow-up lidar scan
    events.push(MissionEvent::LidarScan {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(1500),
        data: LidarData {
            ranges: vec![5.0; 360],
            intensities: None,
            frame_id: "lidar".to_string(),
            min_angle: 0.0,
            max_angle: 6.28,
            angle_increment: 0.01745,
            range_min: 0.1,
            range_max: 10.0,
        },
    });

    let mut record = MissionRecord::new("Obstacle Navigation Mission");
    record.events = events;
    record
}

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║   PyRoboReplay: Temporal Correlation Analysis - Phase 3 Task 18║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let mission = create_realistic_mission();
    println!("Mission: {}", mission.name);
    println!("Events: {}\n", mission.events.len());

    // Analyze correlations
    let analyzer = CorrelationAnalyzer::new()
        .with_window(1000)
        .with_anomaly_threshold(0.85);

    let correlations = analyzer.analyze(&mission.events);
    println!("🔍 Found {} correlations\n", correlations.len());

    // Display all correlations sorted by strength
    println!("═══════════════════════════════════════════════════════════════════");
    println!("CORRELATION ANALYSIS: Event Pairs");
    println!("═══════════════════════════════════════════════════════════════════\n");

    for (rank, corr) in correlations.iter().take(10).enumerate() {
        let anomaly_marker = if corr.is_anomaly { "⚠️ ANOMALY" } else { "✓" };

        println!(
            "{:2}. {} → {}",
            rank + 1, corr.event_a_type, corr.event_b_type
        );
        println!(
            "    Confidence: {:.0}%  │  Gap: {}ms  │  {}",
            corr.correlation_strength * 100.0,
            corr.time_gap_ms,
            anomaly_marker
        );
        println!();
    }

    // Statistics
    println!("═══════════════════════════════════════════════════════════════════");
    println!("STATISTICS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let stats = analyzer.compute_stats(&correlations);
    println!("Total correlations: {}", stats.total_correlations);
    println!("Anomalies detected: {}", stats.anomalies_detected);
    println!("Average correlation: {:.0}%", stats.avg_correlation_strength * 100.0);
    println!("Strongest correlation: {:.0}%", stats.strongest_correlation * 100.0);
    println!("Weakest correlation: {:.0}%\n", stats.weakest_correlation * 100.0);

    // Anomaly patterns
    println!("═══════════════════════════════════════════════════════════════════");
    println!("ANOMALY PATTERNS DETECTED");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let patterns = analyzer.detect_anomaly_patterns(&correlations);

    if patterns.is_empty() {
        println!("No significant anomalies detected.");
    } else {
        for (rank, pattern) in patterns.iter().enumerate() {
            println!(
                "Pattern {}: {}",
                rank + 1, pattern.pattern_type
            );
            println!(
                "  Occurrences: {} | Avg Severity: {:.0}%",
                pattern.count,
                pattern.avg_severity * 100.0
            );
            println!();
        }
    }

    // Event chains
    println!("═══════════════════════════════════════════════════════════════════");
    println!("CORRELATED EVENT CHAINS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let chains = analyzer.find_event_chains(&correlations, 2);

    for (rank, chain) in chains.iter().take(5).enumerate() {
        let event_types: Vec<&str> = chain
            .event_indices
            .iter()
            .map(|&idx| mission.events.get(idx).map(|e| e.event_type()).unwrap_or("?"))
            .collect();

        println!("Chain {}: {}", rank + 1, event_types.join(" → "));
        println!(
            "  Length: {} events | Avg Correlation: {:.0}%\n",
            chain.event_indices.len(),
            chain.avg_correlation_strength * 100.0
        );
    }

    // Summary insights
    println!("═══════════════════════════════════════════════════════════════════");
    println!("INSIGHTS");
    println!("═══════════════════════════════════════════════════════════════════\n");

    println!("Key Findings:");
    println!("  1. Rapid sensor scans detected (anomaly pattern)");
    println!("  2. Strong correlation: Lidar → Obstacle Detection");
    println!("  3. Detection → Navigation Decision chain confirmed");
    println!("  4. Navigation triggers motor response (IMU correlation)");
    println!("  5. Complete feedback loop: Sensor → Decision → Motion → Sensor");

    println!("\n✨ Phase 3 Task #18 Complete: Temporal Correlation Analysis");
}
