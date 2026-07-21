use chrono::Utc;
use pyroboreplay::core::{
    event::{Costmap, LidarData, Location, MissionEvent, MissionRecord, Odometry, Pose},
    CausalGraphBuilder,
};
use pyroboreplay::cli::causal_viz::CausalViz;

fn create_visualization_demo_mission() -> MissionRecord {
    let base_time = Utc::now();
    let mut events = Vec::new();

    // Scenario: Robot exploration with environmental detection
    events.push(MissionEvent::RobotPose {
        robot_id: "robot_1".to_string(),
        timestamp: base_time,
        pose: Pose {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            qx: 0.0,
            qy: 0.0,
            qz: 0.0,
            qw: 1.0,
        },
        confidence: Some(0.99),
    });

    events.push(MissionEvent::LidarScan {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(200),
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

    events.push(MissionEvent::ObstacleDetected {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(500),
        location: Location {
            x: 3.0,
            y: 0.5,
            z: 0.0,
        },
        obstacle_type: "wall".to_string(),
        confidence: Some(0.96),
    });

    events.push(MissionEvent::CostmapUpdate {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(700),
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

    events.push(MissionEvent::NavigationDecision {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(900),
        decision_type: "perimeter_follow".to_string(),
        rationale: Some("Following wall perimeter for exploration".to_string()),
    });

    events.push(MissionEvent::OdometryUpdate {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(1200),
        data: Odometry {
            frame_id: "odom".to_string(),
            child_frame_id: "base_link".to_string(),
            pose: Pose {
                x: 0.8,
                y: 0.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            twist_linear: [0.5, 0.0, 0.0],
            twist_angular: [0.0, 0.0, 0.0],
        },
    });

    let mut record = MissionRecord::new("Wall-Following Exploration Mission");
    record.events = events;
    record
}

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║    PyRoboReplay: Causal Visualization Demo - Phase 3           ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let mission = create_visualization_demo_mission();

    // Build causal graph
    let builder = CausalGraphBuilder::new(mission.events.clone()).with_window(2000);
    let graph = builder.build();

    println!("Mission: {}", mission.name);
    println!("Events: {}", mission.events.len());
    println!("Causal links found: {}\n", graph.links().len());

    // Display: Query what caused the odometry update (robot motion)
    println!("═══════════════════════════════════════════════════════════════════\n");
    println!("ANALYSIS 1: What caused the robot to move? (Odometry event)\n");

    let query_motion = graph.query_what_caused(5, &mission.events);
    let flowchart = CausalViz::render_query(&query_motion, &mission.events);
    println!("{}", flowchart.diagram);

    println!("\nStatistics:");
    println!("{}", CausalViz::render_summary(&query_motion));

    // Display: Hypothesis comparison
    println!("\n═══════════════════════════════════════════════════════════════════\n");
    println!("ANALYSIS 2: Comparing all causal hypotheses\n");

    if !query_motion.hypotheses.is_empty() {
        println!("{}", CausalViz::render_comparison(&query_motion.hypotheses, &mission.events));
    }

    // Display: Confidence timeline
    println!("\n═══════════════════════════════════════════════════════════════════\n");
    println!("ANALYSIS 3: Confidence Timeline (Event Contribution)\n");

    println!("{}\n", CausalViz::render_confidence_timeline(&query_motion, &mission.events));

    // Display: Navigation decision analysis
    println!("═══════════════════════════════════════════════════════════════════\n");
    println!("ANALYSIS 4: What triggered the navigation decision?\n");

    let query_nav = graph.query_what_caused(4, &mission.events);
    let nav_flowchart = CausalViz::render_query(&query_nav, &mission.events);
    println!("{}", nav_flowchart.diagram);

    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("\n✨ SUMMARY\n");
    println!("   Total causal chains analyzed: {}", query_motion.hypotheses.len());
    println!("   Confidence range: {:.0}% - {:.0}%",
        query_motion.hypotheses.iter().map(|h| h.confidence * 100.0).fold(f32::INFINITY, f32::min),
        query_motion.hypotheses.iter().map(|h| h.confidence * 100.0).fold(0.0, f32::max)
    );
    println!("\n✅ Phase 3 Task #17 Complete: Causal Visualization Engine");
}
