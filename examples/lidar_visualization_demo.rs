/// Demonstration of lidar ASCII visualization
/// Shows how the 2D polar projection renders different sensor configurations

use pyroboreplay::cli::lidar_viz::{LidarVisualization, LidarVizConfig};

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║          PyRoboReplay Lidar ASCII Visualization Demo           ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Demo 1: Clear environment
    println!("Demo 1: Clear Environment (Good Visibility)");
    println!("─────────────────────────────────────────────────────────────────");
    demo_clear_environment();

    println!("\n\nDemo 2: Obstacle Detection");
    println!("─────────────────────────────────────────────────────────────────");
    demo_obstacle_detection();

    println!("\n\nDemo 3: Signal Strength Variation (Dense Obstacle)");
    println!("─────────────────────────────────────────────────────────────────");
    demo_signal_variation();

    println!("\n\nDemo 4: Sensor Anomalies (Gaps & Out-of-Range)");
    println!("─────────────────────────────────────────────────────────────────");
    demo_sensor_anomalies();

    println!("\n\nDemo 5: Sparse Readings (Low FPS/Partial Scan)");
    println!("─────────────────────────────────────────────────────────────────");
    demo_sparse_readings();
}

fn demo_clear_environment() {
    let config = LidarVizConfig {
        width: 80,
        height: 40,
        max_range: 30.0,
        min_range: 0.1,
        show_grid: true,
        show_anomalies: true,
    };

    let mut viz = LidarVisualization::new(&config);

    // Simulate clear environment with uniform readings
    for angle in (0..360).step_by(2) {
        let angle_f = angle as f32;
        let range = 25.0; // Consistent range
        let intensity = Some(0.7 + (angle_f.sin() * 0.2).abs()); // Slight variation
        viz.add_reading(angle_f, range, intensity, &config);
    }

    println!("{}", viz.render_with_legend(360, 25.0, 0));
}

fn demo_obstacle_detection() {
    let config = LidarVizConfig {
        width: 80,
        height: 40,
        max_range: 30.0,
        min_range: 0.1,
        show_grid: true,
        show_anomalies: true,
    };

    let mut viz = LidarVisualization::new(&config);

    // Obstacle in front (0-45°) and back (180-225°)
    for angle in (0..360).step_by(2) {
        let angle_f = angle as f32;
        let range = if (angle >= 0 && angle <= 45) || (angle >= 180 && angle <= 225) {
            8.0 // Obstacle detected
        } else {
            25.0 // Clear path
        };
        let intensity = Some(0.8);
        viz.add_reading(angle_f, range, intensity, &config);
    }

    println!("{}", viz.render_with_legend(360, 16.5, 0));
}

fn demo_signal_variation() {
    let config = LidarVizConfig {
        width: 80,
        height: 40,
        max_range: 30.0,
        min_range: 0.1,
        show_grid: true,
        show_anomalies: true,
    };

    let mut viz = LidarVisualization::new(&config);

    // Dense wall ahead (high intensity)
    for angle in (0..360).step_by(2) {
        let angle_f = angle as f32;
        let range = if angle >= 0 && angle <= 90 || angle >= 270 {
            12.0 // Wall
        } else {
            25.0
        };
        let intensity = if angle >= 0 && angle <= 90 || angle >= 270 {
            Some(0.95) // High reflection
        } else {
            Some(0.5) // Lower reflection
        };
        viz.add_reading(angle_f, range, intensity, &config);
    }

    println!("{}", viz.render_with_legend(360, 18.5, 0));
}

fn demo_sensor_anomalies() {
    let config = LidarVizConfig {
        width: 80,
        height: 40,
        max_range: 30.0,
        min_range: 0.1,
        show_grid: true,
        show_anomalies: true,
    };

    let mut viz = LidarVisualization::new(&config);

    let mut anomaly_count = 0;

    for angle in (0..360).step_by(2) {
        let angle_f = angle as f32;
        // Simulated anomalies: gaps and out-of-range readings
        if angle % 30 == 0 {
            // Out-of-range reading (simulated noise)
            viz.add_reading(angle_f, 100.0, Some(0.1), &config);
            anomaly_count += 1;
        } else {
            let range = 20.0 + (angle_f.sin() * 8.0);
            let intensity = Some(0.6 + (angle_f.cos() * 0.3).abs());
            viz.add_reading(angle_f, range, intensity, &config);
        }
    }

    println!("{}", viz.render_with_legend(360, 20.0, anomaly_count));
}

fn demo_sparse_readings() {
    let config = LidarVizConfig {
        width: 80,
        height: 40,
        max_range: 30.0,
        min_range: 0.1,
        show_grid: true,
        show_anomalies: true,
    };

    let mut viz = LidarVisualization::new(&config);

    // Only 90 readings (1/4 normal density)
    for angle in (0..360).step_by(8) {
        let angle_f = angle as f32;
        let range = 18.0 + (angle_f.sin() * 5.0);
        let intensity = Some(0.65);
        viz.add_reading(angle_f, range, intensity, &config);
    }

    println!("{}", viz.render_with_legend(90, 18.0, 0));
}
