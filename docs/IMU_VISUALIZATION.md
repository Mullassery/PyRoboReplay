# IMU Visualization in Terminal

## Overview

PyRoboReplay provides terminal-based ASCII visualization of Inertial Measurement Unit (IMU) data—accelerometer, gyroscope, and magnetometer—rendered as real-time graphs during mission replay.

The visualization shows:
- **Accelerometer** (3-axis): Movement and impacts (m/s²)
- **Gyroscope** (3-axis roll/pitch/yaw): Rotation and angular velocity (rad/s)
- **Magnetometer** (3-axis): Magnetic field strength (µT)
- **Peak detection**: Automatic flagging of impacts and significant events
- **Drift analysis**: Sensor bias over time
- **Statistics**: Mean, peak, and drift for each axis

## Quick Start

### During Mission Replay

```bash
# Start replay
$ pyroboreplay replay warehouse_mission.bag

# Navigate to an IMUData event (press arrow keys)
# When you select an IMU event, the graph appears automatically
```

### Programmatic Usage (Rust)

```rust
use pyroboreplay::cli::imu_viz::{IMUVisualization, IMUVizConfig};

let mut viz = IMUVisualization::new();
let config = IMUVizConfig::default();

// Add IMU readings
viz.add_reading(
    "2026-07-21T10:00:00Z",
    [0.1, 0.2, 9.8],  // accel (m/s²)
    [0.01, 0.02, 0.03],  // gyro (rad/s)
    Some([20.0, 15.0, 35.0]),  // mag (µT)
);

// Render graphs
println!("{}", viz.render_accel(&config));
println!("{}", viz.render_gyro(&config));
println!("{}", viz.render_mag(&config));

// Full dashboard with statistics
println!("{}", viz.render_dashboard(&config));
```

## Graph Interpretation

### Accelerometer (m/s²)

Shows linear acceleration on X, Y, Z axes. Includes gravity (~9.8 m/s² on Z when stationary).

```
X: Forward/backward acceleration (up to ~20 m/s²)
Y: Left/right acceleration (up to ~20 m/s²)
Z: Up/down acceleration (normally 9.8 + variation)
```

**What to look for**:
- **Smooth rise**: Constant acceleration (robot speeding up)
- **Sharp spike**: Impact or collision (impulsive force)
- **Oscillation**: Bouncing or vibration (poor terrain, loose parts)
- **Bias shift**: Slow drift (sensor miscalibration)

### Gyroscope (rad/s)

Shows rotational velocity around X, Y, Z axes (roll, pitch, yaw).

```
Roll (X): Rotation around forward axis
Pitch (Y): Rotation around lateral axis
Yaw (Z): Rotation around vertical axis (heading change)
```

**What to look for**:
- **Ramping**: Continuous turn (smooth navigation)
- **Spike**: Sharp rotation (sudden direction change, evasion)
- **Drift**: Slow creep (gyro bias, temperature effects)
- **High frequency noise**: Vibration from wheels/terrain

### Magnetometer (µT)

Shows magnetic field magnitude on X, Y, Z axes (Earth's magnetic field ~50 µT).

```
X, Y, Z: Components of ambient magnetic field
```

**What to look for**:
- **Constant values**: No magnetic anomalies (good)
- **Sudden shifts**: Nearby ferrous metal (interference)
- **Oscillation**: Rotating magnet nearby (distortion)
- **Magnitude change**: Varying magnetic environment (tunnels, buildings)

## Character Encoding

ASCII graphs use block characters to show intensity:

```
█ (Solid block)   → High intensity
▓ (Dark shade)    → High-medium
▒ (Medium shade)  → Medium
░ (Light shade)   → Low-medium
· (Dot)           → Low intensity
- (Dash)          → Near zero / baseline
_ (Underscore)    → Flat / constant
```

## Statistics Panel

After the graphs, the dashboard shows numerical statistics:

```
Accelerometer:
  X → Mean: 0.15, Peak: 2.34, Drift: 0.42
  Y → Mean: -0.08, Peak: 1.90, Drift: 0.15
  Z → Mean: 9.82, Peak: 10.20, Drift: 0.38

Gyroscope:
  Roll → Mean: 0.001, Peak: 0.234, Drift: 0.015
  Pitch → Mean: -0.002, Peak: 0.187, Drift: 0.012
  Yaw → Mean: 0.005, Peak: 0.456, Drift: 0.034

Magnetometer:
  X → Mean: 18.5, Peak: 22.3
  Y → Mean: 14.2, Peak: 16.8
  Z → Mean: 34.6, Peak: 36.2

Detected Peaks: 5
```

**Metrics**:
- **Mean**: Average value over time window
- **Peak**: Maximum absolute value (highest magnitude event)
- **Drift**: Difference between first and last reading (bias accumulation)

## Peak Detection

Automatically identifies significant events (impacts, sharp turns, vibrations).

```
Detected Peaks: 5
├─ Frame 42: X axis impact at 15.2 m/s²
├─ Frame 89: Yaw rotation at 0.8 rad/s
├─ Frame 124: Z axis impact at 5.3 m/s²
└─ ...
```

**Peak threshold**:
- **Accelerometer**: >2.0 m/s² (impacts)
- **Gyroscope**: >1.0 rad/s (sharp turns)

Configure sensitivity via `IMUVizConfig`:

```rust
let mut config = IMUVizConfig::default();
config.detect_peaks = true;
viz.detect_peaks(&config);
```

## Use Cases

### 1. Impact/Collision Detection
When a robot hits an obstacle, the accelerometer shows a sharp spike. Peaks are automatically flagged.

```
Before collision:  X axis flat
Collision moment:  X axis spike (█ █ █)
After collision:   X axis settles
```

### 2. Navigation Event Correlation
Link navigation decisions to sensor events:
- Sharp turn → Gyro Yaw spike (high Yaw rotation)
- Climb ramp → Accel Z spike + Pitch rotation
- Emergency stop → Large negative X accel

### 3. Sensor Calibration Check
Compare accel Z with known gravity (~9.8 m/s²). Drift indicates calibration issues.

```
Good sensor:     Z hovering around 9.8
Bad sensor:      Z drifting from 9.2 to 10.4 (bias)
Uncalibrated:    Z oscillating wildly
```

### 4. Vibration Analysis
High-frequency oscillations in accelerometer indicate:
- Loose mechanical components
- Rough terrain
- Motor resonance
- Bearing wear

### 5. Environmental Hazard Detection
Magnetometer interference from:
- Ferrous metal objects (fencing, pipes)
- Electrical equipment (motors, power lines)
- Underground metal (buried cables)

## Configuration

### IMUVizConfig

```rust
pub struct IMUVizConfig {
    pub width: usize,           // Graph width (default: 60 chars)
    pub height: usize,          // Graph height (default: 12 lines)
    pub accel_range: f64,       // Max accel to display (default: 20 m/s²)
    pub gyro_range: f64,        // Max gyro to display (default: 10 rad/s)
    pub mag_range: f64,         // Max mag to display (default: 100 µT)
    pub show_stats: bool,       // Show statistics panel (default: true)
    pub detect_peaks: bool,     // Auto-detect peaks (default: true)
}
```

### Example: High-Sensitivity Impact Detection

```rust
let mut config = IMUVizConfig::default();
config.accel_range = 50.0;    // Show up to 50 m/s² (more sensitive)
config.detect_peaks = true;
config.show_stats = true;
```

### Example: Low-Noise Gyro Analysis

```rust
let mut config = IMUVizConfig::default();
config.gyro_range = 0.5;      // Show only small rotations
config.detect_peaks = true;
```

## Common Patterns

### Stationary Robot
```
Accel Z: ═══════════════════════ (constant ~9.8)
Gyro:    ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄ (near zero, slight noise)
Impact: No peaks detected
```

### Accelerating Forward
```
Accel X: ▄▄▄▄▄████████▓▓▓▓▓▒ (ramping)
Gyro:    ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄ (minimal)
Impact: No peaks (smooth acceleration)
```

### Turning Left
```
Gyro Yaw: ▄▄▄▄▄▄▓▓▓▓▓██████ (ramping)
Accel:    █ (centrifugal effect in Y)
Impact: Yaw peak detected
```

### Collision/Impact
```
Accel X: ▄▄▄▄▄▄▄████████████ (sharp spike)
Then drops back to baseline
Impact: High acceleration peak detected at frame N
```

### Sensor Drift
```
Gyro Yaw at start: 0.05 rad/s
Gyro Yaw at end:   0.25 rad/s
Drift: 0.20 rad/s (bias accumulation)
```

## Terminal Requirements

### Supported Terminals
- ✅ macOS: Terminal.app, iTerm2, Kitty
- ✅ Linux: GNOME Terminal, Konsole, xterm
- ✅ Windows: Windows Terminal (v1.0+), WSL2
- ✅ Remote: SSH with UTF-8 encoding

### Character Set
Requires Unicode block elements:
- `█` (U+2588) Solid block
- `▓` (U+2593) Dark shade
- `▒` (U+2592) Medium shade
- `░` (U+2591) Light shade
- `·` (U+00B7) Dot

If unsupported, fall back to ASCII rendering:
```bash
# Future: ASCII mode support
pyroboreplay replay mission.bag --ascii-mode
# Uses: # @ o - . instead of Unicode
```

## Performance

- **Rendering time**: <5ms per graph (60×12 grid)
- **Memory**: ~50KB per 1000 readings
- **CPU**: Negligible (<1% during playback)
- **Storage**: ~50 bytes per IMU sample in event model

## Limitations & Future Work

### Current Limitations
1. Single IMU visualization (next event overwrites)
2. No time-series overlay (previous frames fade)
3. No 3D visualization
4. Limited to 2D projection
5. No advanced spectral analysis (FFT)

### Planned Enhancements (v0.3+)
- **Multi-panel view**: Accel/Gyro/Mag side-by-side
- **Time-series overlay**: Show last N readings fading
- **Heatmap mode**: Show frequency content (FFT)
- **Correlation overlay**: Link to navigation decisions
- **Sensor fusion indicators**: Orientation estimate (pitch/roll/yaw)
- **Custom peak thresholds**: Per-axis sensitivity configuration
- **Export to CSV**: Raw IMU data export
- **Comparative mode**: Multiple robots' IMU data overlaid

## Integration with Other Sensors

### Accelerometer + Navigation
When acceleration spikes near a turn, suggests collision avoidance.

### Gyroscope + Odometry
When Yaw velocity changes, correlate with odometry heading change for validation.

### Magnetometer + Navigation
When magnetometer shows interference, robot may have difficulty with compass-based navigation.

### IMU + Lidar
Impacts (accel spikes) often precede lidar seeing obstacles.

## See Also

- [Lidar Visualization](LIDAR_VISUALIZATION.md) — Terminal-based lidar graphs
- [Camera Export](CAMERA_EXPORT.md) — Frame-by-frame video replay
- [Keyboard Shortcuts](KEYBOARD_SHORTCUTS.md) — Navigation keys
- [API Reference](API.md) — Python/Rust API details
- [Architecture](ARCHITECTURE.md) — System design
