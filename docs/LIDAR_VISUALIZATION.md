# Lidar Visualization in PyRoboReplay

## Overview

PyRoboReplay provides terminal-based ASCII visualization of lidar scans using a **2D polar projection** (bird's-eye view). This feature enables quick visual diagnosis of sensor readings, obstacle detection, and anomaly flagging without leaving the terminal.

## How It Works

### Coordinate System
- **Center point** (`●`): Observer location (robot center)
- **Angles** (0-360°): 0° = right (East), 90° = up (North), 180° = left (West), 270° = down (South)
- **Rings** (`·`): Reference distance lines at 5m intervals
- **Characters**: Represent intensity/signal strength at each angle

### Display Modes

#### In Terminal Replay
When replaying a mission and stepping to a **LidarScan** event:
- On terminals **wider than 120 columns**: Automatically shows lidar visualization alongside timeline
- On smaller terminals: Shows standard event details instead
- Press arrow keys to navigate through events and see real-time lidar updates

#### Intensity Encoding
Each pixel's intensity determines the character shown:

```
█ (Solid Block)   → High intensity (>0.8)    — Strong reflection
▓ (Dark Shade)    → High-medium (0.6-0.8)   — Good signal
▒ (Medium Shade)  → Medium (0.4-0.6)        — Moderate signal
░ (Light Shade)   → Low-medium (0.2-0.4)    — Weak signal
· (Dot)           → Low intensity (<0.2)     — Very weak/grid line
X (X Mark)        → Anomaly                  — Out-of-range or gap
```

## Features

### 1. **2D Polar Projection**
- Bird's-eye view (top-down perspective)
- Angle (θ) mapped to horizontal/vertical position
- Range (r) mapped to distance from center
- Efficient terminal rendering

### 2. **Anomaly Detection**
Automatically marks:
- **Out-of-range readings** (> max_range)
- **Close readings** (< min_range)
- **Sensor gaps** (no reading at angle)
- Marked with `X` character

### 3. **Configurable Resolution**
```rust
pub struct LidarVizConfig {
    pub width: usize,        // Terminal width (default: 80)
    pub height: usize,       // Terminal height (default: 40)
    pub max_range: f32,      // Max visualization range (default: 30m)
    pub min_range: f32,      // Min valid range (default: 0.1m)
    pub show_grid: bool,     // Display reference rings (default: true)
    pub show_anomalies: bool,// Highlight anomalies (default: true)
}
```

### 4. **Real-Time Updates During Replay**
As you navigate the timeline:
- Visualization updates immediately for each LidarScan event
- Shows temporal evolution of sensor readings
- Helps diagnose dynamic obstacles or sensor degradation

### 5. **Legend & Statistics**
Integrated footer shows:
- Frame count (total angles scanned)
- Average range
- Anomaly count
- Character intensity legend

## Usage Examples

### In Terminal Replay

```bash
# Start replay with wide terminal
$ pyroboreplay replay mission.bag

# When you step to a LidarScan event on a wide terminal:
# → Visualization appears on the right side
# → Shows bird's-eye view of obstacles around robot
# → Use arrow keys (← →) to navigate through scans
```

### Programmatic Usage (Rust API)

```rust
use pyroboreplay::cli::lidar_viz::{LidarVisualization, LidarVizConfig};

// Create visualization with custom config
let mut config = LidarVizConfig::default();
config.max_range = 50.0;  // 50-meter range
config.width = 120;       // Wider display

let mut viz = LidarVisualization::new(&config);

// Add readings from sensor
for reading in lidar_scan.readings {
    viz.add_reading(
        reading.angle_degrees,
        reading.range_meters,
        reading.intensity,
        &config
    );
}

// Render and display
println!("{}", viz.render_with_legend(
    reading_count,
    avg_range,
    anomaly_count
));
```

### CLI JSON Export with Visualization

Future versions will support:
```bash
# Export lidar scan as ASCII art in JSON
$ pyroboreplay replay mission.bag --export-lidar scans.json

# Output includes rendered ASCII visualization in JSON for agent parsing
```

## Interpretation Guide

### Clear Environment
```
Shows: Uniform shield of block/shade characters
Means: Robot has unobstructed 360° view
Action: None needed
```

### Obstacle Detection
```
Shows: Dense characters in specific angular range
Means: Obstacle in that direction
Action: Check navigation decision (did planner route around it?)
```

### Signal Strength Variation
```
Shows: Different shade intensity across scan
Means: Materials with different reflectivity
Example: Dense wall (█) vs cloth (░)
Action: Consider for terrain classification
```

### Sensor Anomalies
```
Shows: X marks scattered in visualization
Means: Out-of-range or missing readings
Action: Investigate sensor malfunction or environmental noise
```

### Sparse Coverage
```
Shows: Fewer characters with larger gaps
Means: Low update rate or partial scans
Action: Check sensor FPS during this interval
```

## Performance Characteristics

- **Rendering time**: <1ms per scan (80×40 grid)
- **Memory**: ~3KB per visualization instance
- **CPU**: Negligible (<1% per frame in terminal)
- **Works on**: All terminals supporting Unicode block characters

## Terminal Requirements

### Supported Terminals
- ✅ macOS: Terminal.app, iTerm2, Kitty
- ✅ Linux: GNOME Terminal, Konsole, xterm (with UTF-8)
- ✅ Windows: Windows Terminal (v1.0+), WSL2 terminals
- ✅ Remote: SSH terminals with UTF-8 encoding

### Character Set
Requires terminal support for Unicode block elements:
- `█` (U+2588) Solid block
- `▓` (U+2593) Dark shade
- `▒` (U+2592) Medium shade
- `░` (U+2591) Light shade
- `●` (U+25CF) Circle
- `·` (U+00B7) Dot

If your terminal doesn't support these, fall back to ASCII mode:
```
# Future CLI option
$ pyroboreplay replay mission.bag --ascii-mode
# Uses: # @ o - . instead of Unicode
```

## Limitations & Future Work

### Current Limitations
- Single scan per display (not continuous streaming)
- 2D projection only (elevation angle lost)
- Max range visualization (beyond config.max_range not shown)
- Terminal width-dependent (small terminals show degraded visualization)

### Planned Enhancements (v0.3+)
- **3D visualization option**: Show elevation rings
- **Time-series overlay**: Last N scans as fading layers
- **Statistical heatmap**: Density of obstacles over time
- **Export to image**: Convert terminal visualization to PNG
- **Real-time streaming**: Live lidar display without replay
- **Interactive zoom**: Focus on specific angular range

## Architecture

### Module Structure
```
src/cli/
├── lidar_viz.rs           # Visualization engine
│   ├── LidarVizConfig     # Configuration
│   ├── LidarVisualization # Rendering
│   └── Tests              # Unit tests
└── replay_ui.rs           # Integration with replay
    ├── draw_lidar_visualization()  # Render in UI
    └── Auto-detect LidarScan events
```

### API Stability
- **Stable**: Public `LidarVisualization` struct, `LidarVizConfig` configuration
- **Stable**: `render()`, `render_with_legend()` output format (used by agents)
- **Internal**: `add_reading()`, `mark_anomaly()` (implementation details)

## Troubleshooting

### Visualization Shows Garbled Characters
**Solution**: Ensure terminal has UTF-8 encoding enabled
```bash
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8
pyroboreplay replay mission.bag
```

### Visualization Not Appearing in Replay
**Reasons**:
1. Terminal width < 120 columns → Switch to wider terminal or maximize window
2. No LidarScan event selected → Navigate to a LidarScan event (press →)
3. No intensity data → Intensity defaults to grid visualization (still readable)

### Performance Issues
**Solution**: Reduce visualization resolution in narrow terminals
```rust
let mut config = LidarVizConfig::default();
config.width = 40;   // Smaller grid
config.height = 20;
```

## See Also

- [Keyboard Shortcuts](KEYBOARD_SHORTCUTS.md) — Navigation controls
- [CLI Reference](API.md) — All command-line options
- [Architecture](ARCHITECTURE.md) — System design details
