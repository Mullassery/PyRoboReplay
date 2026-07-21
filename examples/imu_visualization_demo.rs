/// Demonstration of IMU ASCII visualization
/// Shows accelerometer, gyroscope, and magnetometer graphs

use pyroboreplay::cli::imu_viz::{IMUVisualization, IMUVizConfig};

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║          PyRoboReplay IMU Visualization Demo                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Demo 1: Stationary robot (minimal sensor noise)
    println!("Demo 1: Stationary Robot (Baseline)");
    println!("─────────────────────────────────────────────────────────────────");
    demo_stationary();

    println!("\n\nDemo 2: Robot Accelerating Forward");
    println!("─────────────────────────────────────────────────────────────────");
    demo_acceleration();

    println!("\n\nDemo 3: Robot Turning (Rotating)");
    println!("─────────────────────────────────────────────────────────────────");
    demo_rotation();

    println!("\n\nDemo 4: Impact Detection (Collision)");
    println!("─────────────────────────────────────────────────────────────────");
    demo_impact();

    println!("\n\nDemo 5: Sensor Drift Over Time");
    println!("─────────────────────────────────────────────────────────────────");
    demo_drift();
}

fn demo_stationary() {
    let mut viz = IMUVisualization::new();
    let config = IMUVizConfig::default();

    // Simulate 30 readings at rest (slight noise from gravity)
    for i in 0..30 {
        let time = format!("T+{:02}ms", i * 33);
        // Accel: ~9.8 m/s² in Z (gravity), minimal noise
        let accel = [
            0.1 * (i as f64 * 0.1).sin(),
            0.1 * (i as f64 * 0.12).sin(),
            9.8 + 0.2 * (i as f64 * 0.15).sin(),
        ];
        let gyro = [
            0.01 * (i as f64 * 0.2).sin(),
            0.01 * (i as f64 * 0.1).sin(),
            0.01 * (i as f64 * 0.18).sin(),
        ];
        let mag = [20.0, 15.0, 35.0];

        viz.add_reading(&time, accel, gyro, Some(mag));
    }

    viz.detect_peaks(&config);

    println!("{}", viz.render_dashboard(&config));
}

fn demo_acceleration() {
    let mut viz = IMUVisualization::new();
    let config = IMUVizConfig::default();

    // Simulate 30 readings with acceleration in X direction
    for i in 0..30 {
        let time = format!("T+{:02}ms", i * 33);
        let accel_x = (i as f64 / 30.0) * 5.0; // Ramping acceleration
        let accel = [
            accel_x,
            0.1 * (i as f64 * 0.15).sin(),
            9.8 + 0.3 * (i as f64 * 0.12).sin(),
        ];
        let gyro = [0.0, 0.0, 0.0]; // No rotation
        let mag = [20.0, 15.0, 35.0];

        viz.add_reading(&time, accel, gyro, Some(mag));
    }

    viz.detect_peaks(&config);

    println!("{}", viz.render_dashboard(&config));
}

fn demo_rotation() {
    let mut viz = IMUVisualization::new();
    let config = IMUVizConfig::default();

    // Simulate 30 readings with rotation (turning left)
    for i in 0..30 {
        let time = format!("T+{:02}ms", i * 33);
        let angle = (i as f64 / 30.0) * std::f64::consts::PI;

        // Accel: gravity + centrifugal effects
        let accel = [
            2.0 * angle.sin(), // Centrifugal in X
            0.1 * (i as f64 * 0.1).sin(),
            9.8 + 0.2 * (i as f64 * 0.1).cos(),
        ];

        // Gyro: strong rotation around Z (yaw)
        let gyro = [
            0.1 * (i as f64 * 0.2).sin(),
            0.1 * (i as f64 * 0.15).sin(),
            (i as f64 / 30.0) * 4.0, // Ramping angular velocity
        ];

        let mag = [20.0, 15.0, 35.0];

        viz.add_reading(&time, accel, gyro, Some(mag));
    }

    viz.detect_peaks(&config);

    println!("{}", viz.render_dashboard(&config));
}

fn demo_impact() {
    let mut viz = IMUVisualization::new();
    let config = IMUVizConfig::default();

    // Simulate impact event
    for i in 0..30 {
        let time = format!("T+{:02}ms", i * 33);

        // Normal motion initially
        let mut accel = [0.0, 0.0, 9.8];
        let mut gyro = [0.0, 0.0, 0.0];

        // Impact at frame 15 (sudden acceleration)
        if i == 15 {
            accel[0] = 15.0; // Sharp impact in X
            accel[2] = 5.0;  // Drop in Z
            gyro[0] = 5.0;   // Sudden rotation
            gyro[1] = 3.0;
        } else if i > 15 && i < 20 {
            // Ringing after impact (damping oscillation)
            let damping = (-(i as f64 - 15.0) * 0.5).exp();
            accel[0] = 15.0 * (((i as f64) - 15.0) * 0.5).cos() * damping;
        }

        let mag = [20.0, 15.0, 35.0];

        viz.add_reading(&time, accel, gyro, Some(mag));
    }

    viz.detect_peaks(&config);

    println!("{}", viz.render_dashboard(&config));
}

fn demo_drift() {
    let mut viz = IMUVisualization::new();
    let config = IMUVizConfig::default();

    // Simulate sensor drift over time (typical gyro bias)
    for i in 0..60 {
        let time = format!("T+{:03}ms", i * 16);
        let accel = [
            0.1 * (i as f64 * 0.05).sin(),
            0.1 * (i as f64 * 0.08).sin(),
            9.8 + 0.15 * (i as f64 * 0.1).sin(),
        ];

        // Gyro with increasing bias (drift)
        let drift = (i as f64 / 60.0) * 0.5;
        let gyro = [
            0.05 * (i as f64 * 0.1).sin() + drift * 0.1,
            0.05 * (i as f64 * 0.08).sin() + drift * 0.15,
            0.05 * (i as f64 * 0.12).sin() + drift * 0.2,
        ];

        let mag = [20.0 + drift, 15.0 + drift * 0.7, 35.0];

        viz.add_reading(&time, accel, gyro, Some(mag));
    }

    viz.detect_peaks(&config);

    println!("{}", viz.render_dashboard(&config));
}
