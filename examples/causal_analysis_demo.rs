use chrono::Utc;
use pyroboreplay::core::{
    event::{
        Costmap, IMUData, LidarData, Location, MissionEvent,
        MissionRecord, Odometry, Pose,
    },
    CausalGraphBuilder,
};

fn create_demo_mission() -> MissionRecord {
    let base_time = Utc::now();

    let mut events = Vec::new();

    // Scenario 1: Robot encounters obstacle
    // t=0: Lidar detects clear path
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

    // t=500ms: Obstacle appears (lidar detects it)
    events.push(MissionEvent::LidarScan {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(500),
        data: LidarData {
            ranges: (0..360).map(|i| if i == 180 { 1.5 } else { 5.0 }).collect(),
            intensities: None,
            frame_id: "lidar".to_string(),
            min_angle: 0.0,
            max_angle: 6.28,
            angle_increment: 0.01745,
            range_min: 0.1,
            range_max: 10.0,
        },
    });

    // t=700ms: Obstacle is detected and localized
    events.push(MissionEvent::ObstacleDetected {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(700),
        location: Location {
            x: 2.0,
            y: 0.0,
            z: 0.0,
        },
        obstacle_type: "static".to_string(),
        confidence: Some(0.95),
    });

    // t=800ms: Costmap is updated to reflect obstacle
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

    // t=1000ms: Navigation decision to avoid obstacle
    events.push(MissionEvent::NavigationDecision {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(1000),
        decision_type: "path_replan".to_string(),
        rationale: Some("Obstacle detected ahead, replanning path".to_string()),
    });

    // t=1200ms: Robot begins to decelerate (IMU spike)
    events.push(MissionEvent::IMUData {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(1200),
        data: IMUData {
            frame_id: "imu".to_string(),
            linear_acceleration: [-2.5, 0.0, 0.0],
            angular_velocity: [0.0, 0.0, 0.0],
            magnetometer: None,
            orientation: None,
        },
    });

    // t=1500ms: Odometry shows robot has stopped
    events.push(MissionEvent::OdometryUpdate {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(1500),
        data: Odometry {
            frame_id: "odom".to_string(),
            child_frame_id: "base_link".to_string(),
            pose: Pose {
                x: 1.0,
                y: 0.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            twist_linear: [0.0, 0.0, 0.0],
            twist_angular: [0.0, 0.0, 0.0],
        },
    });

    let mut record = MissionRecord::new("Obstacle Avoidance Demo");
    record.events = events;
    record
}

fn main() {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║        PyRoboReplay: Causal Analysis Demo - Phase 3           ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    let mission = create_demo_mission();
    println!("Created demo mission: {} ({} events)", mission.name, mission.events.len());
    println!("Time range: {} ms\n", {
        let first = mission.events.first().map(|e| e.timestamp());
        let last = mission.events.last().map(|e| e.timestamp());
        if let (Some(f), Some(l)) = (first, last) {
            format!("{}", (l - f).num_milliseconds())
        } else {
            "N/A".to_string()
        }
    });

    // Build causal graph
    println!("🔗 Building causal graph...");
    let builder = CausalGraphBuilder::new(mission.events.clone()).with_window(2000);
    let graph = builder.build();

    println!(
        "✅ Graph built with {} causal links\n",
        graph.links().len()
    );

    // Analyze causal structure
    println!("📊 Causal Analysis Results:");
    println!("───────────────────────────────────────────────────────────────");

    for (idx, link) in graph.links().iter().enumerate() {
        let event_type_from = mission.events[link.source_event_idx].event_type();
        let event_type_to = mission.events[link.target_event_idx].event_type();

        println!(
            "Link {}: {} → {} (confidence: {:.0}%, gap: {}ms)",
            idx,
            event_type_from,
            event_type_to,
            link.confidence * 100.0,
            link.time_gap_ms
        );

        match link.relationship_type.as_str() {
            "lidar_detected_obstacle" => {
                println!("  └─ Lidar detected obstacle that triggered detection");
            }
            "obstacle_triggered_nav" => {
                println!("  └─ Obstacle detection triggered navigation replanning");
            }
            "costmap_influenced_nav" => {
                println!("  └─ Costmap update influenced navigation decision");
            }
            "imu_caused_motion" => {
                println!("  └─ IMU acceleration caused motion change");
            }
            _ => {
                println!("  └─ {} relationship", link.relationship_type);
            }
        }
    }

    println!("\n───────────────────────────────────────────────────────────────");

    // Trace causality for the navigation decision event (index 4)
    println!("\n🔍 Tracing causality for NavigationDecision (event #4):");
    println!("───────────────────────────────────────────────────────────────");

    let nav_decision_idx = 4;
    let causes = graph.get_direct_causes(nav_decision_idx);

    println!("Direct causes of NavigationDecision:");
    for cause in &causes {
        let source_type = mission.events[cause.source_event_idx].event_type();
        println!(
            "  • {} (confidence: {:.0}%, {}ms before)",
            source_type,
            cause.confidence * 100.0,
            cause.time_gap_ms
        );
    }

    // Trace full causal chain
    println!("\n🔗 Full causal chain leading to robot stop:");
    println!("───────────────────────────────────────────────────────────────");

    let odometry_idx = 6; // Robot stop event
    let chains = graph.trace_causes(odometry_idx, 5);

    for (idx, chain) in chains.iter().enumerate() {
        print!("Path {}: ", idx + 1);
        for (i, event_idx) in chain.event_chain.iter().enumerate() {
            let event_type = mission.events[*event_idx].event_type();
            if i > 0 {
                print!(" → ");
            }
            print!("{}", event_type);
        }
        println!(" (confidence: {:.0}%)", chain.total_confidence * 100.0);
    }

    println!("\n───────────────────────────────────────────────────────────────");
    println!("\n💡 Key Findings:");
    println!("  • Obstacle detection -> Navigation decision: clear causal link");
    println!("  • Navigation decision -> Robot deceleration: inferred causality");
    println!("  • Full chain: Lidar → Detection → Navigation → Motion");
    println!("\n✨ Phase 3 Task #15 Complete: Event Dependency Graph Built");
}
