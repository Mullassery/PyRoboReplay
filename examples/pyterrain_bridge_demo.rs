use chrono::Utc;
use pyroboreplay::core::{
    event::{MissionEvent, MissionRecord, Odometry, Pose},
    Obstacle, PyTerrainBridge, SpatialCausalityAnalyzer, TerrainKnowledgeGraph,
    TraversabilityZone,
};

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║   PyRoboReplay: PyTerrainMap Bridge - Phase 4 Task #2          ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Create terrain knowledge graph from PyTerrainMap
    let mut graph = TerrainKnowledgeGraph::new();

    let base_time = Utc::now();

    // Add detected obstacles from PyTerrainMap
    let obstacle1 = Obstacle {
        id: "wall_north".to_string(),
        position: (0.0, 5.0, 0.0),
        dimensions: (10.0, 0.5, 2.0),
        first_seen: base_time,
        last_seen: base_time + chrono::Duration::seconds(600),
        obstacle_type: "wall".to_string(),
        confidence: 0.98,
        is_dynamic: false,
    };

    let obstacle2 = Obstacle {
        id: "object_center".to_string(),
        position: (2.5, 2.5, 0.0),
        dimensions: (1.0, 1.0, 1.5),
        first_seen: base_time + chrono::Duration::seconds(100),
        last_seen: base_time + chrono::Duration::seconds(500),
        obstacle_type: "object".to_string(),
        confidence: 0.85,
        is_dynamic: true,
    };

    graph.add_obstacle(obstacle1);
    graph.add_obstacle(obstacle2);

    // Add traversability zones
    let zone_open = TraversabilityZone {
        id: "zone_open_area".to_string(),
        center: (0.0, 0.0, 0.0),
        radius_m: 3.0,
        traversability: 0.95,
        terrain_type: "open".to_string(),
        successful_passages: 15,
        failed_attempts: 0,
    };

    let zone_confined = TraversabilityZone {
        id: "zone_between_walls".to_string(),
        center: (2.5, 2.5, 0.0),
        radius_m: 2.0,
        traversability: 0.60,
        terrain_type: "confined".to_string(),
        successful_passages: 3,
        failed_attempts: 2,
    };

    graph.add_traversability_zone(zone_open);
    graph.add_traversability_zone(zone_confined);

    // Build coverage map
    graph.coverage_grid.update_coverage(0.0, 0.0, 1.0); // Initial position
    graph.coverage_grid.update_coverage(1.0, 1.0, 0.95); // Explored region
    graph.coverage_grid.update_coverage(2.5, 2.5, 0.80); // Confined region
    graph.coverage_grid.update_coverage(3.0, 0.0, 0.90); // Another region

    println!("═══════════════════════════════════════════════════════════════════");
    println!("TERRAIN KNOWLEDGE GRAPH LOADED");
    println!("═══════════════════════════════════════════════════════════════════\n");

    println!("Obstacles tracked from PyTerrainMap:");
    for (_, obstacle) in &graph.obstacles {
        println!(
            "  • {} ({}): at ({:.1}, {:.1}, {:.1}) | confidence {:.0}% | {}",
            obstacle.id,
            obstacle.obstacle_type,
            obstacle.position.0,
            obstacle.position.1,
            obstacle.position.2,
            obstacle.confidence * 100.0,
            if obstacle.is_dynamic {
                "DYNAMIC"
            } else {
                "static"
            }
        );
    }

    println!("\nTraversability zones:");
    for (_, zone) in &graph.traversability_zones {
        println!(
            "  • {} ({}): center ({:.1}, {:.1}) | radius {:.1}m | traversability {:.0}%",
            zone.id, zone.terrain_type, zone.center.0, zone.center.1, zone.radius_m, zone.traversability * 100.0
        );
    }

    println!("\nCoverage Statistics:");
    println!(
        "  Total coverage: {:.1}%",
        graph.coverage_grid.total_coverage_percentage()
    );
    println!(
        "  Grid: {}×{} cells @ {:.2}m/cell",
        graph.coverage_grid.width, graph.coverage_grid.height, graph.coverage_grid.cell_size
    );

    // Create PyTerrainMap bridge
    let mut bridge = PyTerrainBridge::new(graph);

    // Create sample mission with odometry events
    let mut mission = MissionRecord::new("Terrain-Aware Navigation Mission");

    // Event 0: Start in open area
    mission.add_event(MissionEvent::OdometryUpdate {
        robot_id: "robot_1".to_string(),
        timestamp: base_time,
        data: Odometry {
            frame_id: "odom".to_string(),
            child_frame_id: "base_link".to_string(),
            pose: Pose {
                x: 0.0,
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

    // Event 1: Move toward object
    mission.add_event(MissionEvent::OdometryUpdate {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(500),
        data: Odometry {
            frame_id: "odom".to_string(),
            child_frame_id: "base_link".to_string(),
            pose: Pose {
                x: 1.0,
                y: 1.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            twist_linear: [1.0, 1.0, 0.0],
            twist_angular: [0.0, 0.0, 0.0],
        },
    });

    // Event 2: Reach object area (confined)
    mission.add_event(MissionEvent::OdometryUpdate {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(1000),
        data: Odometry {
            frame_id: "odom".to_string(),
            child_frame_id: "base_link".to_string(),
            pose: Pose {
                x: 2.5,
                y: 2.5,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            twist_linear: [0.3, 0.3, 0.0],
            twist_angular: [0.0, 0.0, 0.0],
        },
    });

    // Event 3: Navigate around object
    mission.add_event(MissionEvent::OdometryUpdate {
        robot_id: "robot_1".to_string(),
        timestamp: base_time + chrono::Duration::milliseconds(1500),
        data: Odometry {
            frame_id: "odom".to_string(),
            child_frame_id: "base_link".to_string(),
            pose: Pose {
                x: 4.0,
                y: 1.0,
                z: 0.0,
                qx: 0.0,
                qy: 0.0,
                qz: 0.0,
                qw: 1.0,
            },
            twist_linear: [1.0, 0.0, 0.0],
            twist_angular: [0.0, 0.0, 0.1],
        },
    });

    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("SPATIAL CAUSALITY WITH TERRAIN CONTEXT");
    println!("═══════════════════════════════════════════════════════════════════\n");

    // Create spatial causality analyzer
    let mut analyzer = SpatialCausalityAnalyzer::new();

    // Apply terrain knowledge to enrich spatial contexts
    bridge.apply_to_analyzer(&mut analyzer, &mission.events);

    println!("Events enriched with PyTerrainMap data:");
    for (idx, event) in mission.events.iter().enumerate() {
        match event {
            MissionEvent::OdometryUpdate { data, timestamp, .. } => {
                println!(
                    "\nEvent {}: OdometryUpdate at ({:.1}, {:.1}, {:.1})",
                    idx, data.pose.x, data.pose.y, data.pose.z
                );

                // Query obstacles at this time
                let obstacles_at_time = bridge.knowledge_graph.query_obstacles_at_time(*timestamp);
                if !obstacles_at_time.is_empty() {
                    println!("  Active obstacles at this time:");
                    for obs in obstacles_at_time {
                        println!(
                            "    • {} at distance {:.1}m",
                            obs.id,
                            (
                                (obs.position.0 - data.pose.x).powi(2)
                                    + (obs.position.1 - data.pose.y).powi(2)
                            )
                            .sqrt()
                        );
                    }
                }

                // Query traversability
                let traversability = bridge
                    .knowledge_graph
                    .query_traversability_at_position((data.pose.x, data.pose.y, data.pose.z));
                println!("  Traversability: {:.0}%", traversability * 100.0);
            }
            _ => {}
        }
    }

    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("COVERAGE EVOLUTION");
    println!("═══════════════════════════════════════════════════════════════════\n");

    let evolution = bridge
        .knowledge_graph
        .coverage_evolution(base_time, base_time + chrono::Duration::seconds(2));

    println!("Coverage statistics from t0 to t1:");
    println!("  Initial coverage: {:.1}%", evolution.initial_coverage);
    println!("  Final coverage: {:.1}%", evolution.final_coverage);
    println!("  New obstacles discovered: {}", evolution.new_obstacles_found);

    println!("\n═══════════════════════════════════════════════════════════════════");
    println!("BRIDGE REPORT");
    println!("═══════════════════════════════════════════════════════════════════\n");

    println!("{}\n", bridge.generate_report());

    println!("💡 Spatial-Causal Capabilities Enabled:");
    println!("  ✓ Query obstacles at specific timestamps");
    println!("  ✓ Traversability-aware navigation analysis");
    println!("  ✓ Coverage evolution tracking");
    println!("  ✓ Obstacle-causal relationship detection");
    println!("  ✓ Multi-dimensional mission understanding");

    println!("\n✨ Phase 4 Task #2 Complete: PyTerrainMap Bridge Integrated");
}
