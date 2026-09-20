# Stats Dashboard Quick Start Guide

## What is the Stats Dashboard?

A **separate terminal window** that shows real-time metrics about your mission replay:
- Event counts and progress
- Playback speed
- Sensor statistics (FPS, quality, frame counts)
- Data quality indicators

## Quick Start (30 seconds)

### 1. Launch Replay with Dashboard

```bash
cargo run -- replay your_mission.bag --stats-dashboard
```

### 2. Two Windows Open

- **Main Window**: Interactive replay UI (lidar visualization, timeline scrubber)
- **Separate Window**: Stats dashboard (metrics, sensor info)

### 3. Both Run Simultaneously

- Replay in main window
- Stats update in dashboard window
- Independent event loops, zero interference

## Platform Support

| Platform | Terminal | Status |
|----------|----------|--------|
| **macOS** | Terminal.app or iTerm2 | ✅ Fully supported |
| **Linux** | terminator, gnome-terminal, xterm, xfce4-terminal | ✅ Fully supported |
| **Windows** | Not yet supported | ⏳ Coming in v0.4 |

## Real-World Examples

### Example 1: Warehouse Floor Exploration

```bash
# Load warehouse exploration mission with stats
cargo run -- replay warehouse_floor_v3.bag --stats-dashboard

# Main window: Step through lidar scans frame-by-frame
# Dashboard: Monitor sensor quality, FPS, event count
```

### Example 2: Drone Inspection Route

```bash
# Filter to camera + IMU only
cargo run -- replay drone_inspection.bag --sensor camera,imu --stats-dashboard

# Main window: Camera replay
# Dashboard: IMU metrics, dropped frames, quality score
```

### Example 3: Fleet Multi-Robot

```bash
# Show robot 1 only, with dashboard
cargo run -- replay fleet_mission.bag --robot robot_1 --stats-dashboard
```

## Dashboard View

```

 PyRoboReplay Stats Dashboard | Mission: warehouse_v3 | Evt: 1250/5000 

 
 ⚡ Real-time Stats 
 Total Events: 5000 
 Current Position: 1250 / 5000 
 Progress: 25.0% 
 Playback Speed: 1.0x 
 
 Sensor Summary 
 Total: 8500 events across 6 sensors | Avg Quality: 95% 
 
 Sensors: ✅ Lidar (main) 30.5fps (2500 frames) | ✅ Camera 15.2fps 
 ✅ IMU 100fps (5000 samples) | Odometry 50fps 
 

 Controls: +/- (speed) | R (reset) | Q (quit) 

```

## Keyboard Controls

Press these keys **in the dashboard window**:

| Key | Action |
|-----|--------|
| `+` | Speed up replay |
| `-` | Slow down replay |
| `↑` | Speed up (alternative) |
| `↓` | Slow down (alternative) |
| `R` | Reset to beginning |
| `Q` | Quit dashboard |
| `Esc` | Quit dashboard |
| `Ctrl+C` | Force exit |

## Common Workflows

### Workflow 1: Monitor Data Quality During Replay

1. Start replay with dashboard: `cargo run -- replay mission.bag --stats-dashboard`
2. Watch dashboard for quality drops or missing sensors
3. Pause main replay (Space key) if quality issue detected
4. Investigate in main window
5. Resume when ready

### Workflow 2: Speed Analysis

1. Launch with dashboard
2. In main window: set playback speed
3. In dashboard: watch FPS and progress update in real-time
4. Adjust speed to find optimal playback rate for analysis

### Workflow 3: Multi-Sensor Sync Check

1. Filter sensors in main window: `--sensor lidar,camera,imu`
2. Watch dashboard to verify all sensors have similar FPS and frame counts
3. Any FPS drift indicates sync issues
4. Dashboard quality score highlights problems

## Troubleshooting

### Dashboard doesn't open

**macOS:**
```bash
# Check if Terminal.app is installed
ls /Applications/Utilities/Terminal.app

# Or iTerm2
ls /Applications/iTerm.app
```

**Linux:**
```bash
# Install terminator (recommended)
sudo apt install terminator

# Or gnome-terminal
sudo apt install gnome-terminal

# Or xterm
sudo apt install xterm
```

### Dashboard opens but shows nothing

- This is expected behavior for now (v0.1)
- Dashboard shows static mission metadata
- Real-time updates coming in v0.3
- To see dashboard working, check window title and basic info

### Replay runs but dashboard won't launch

You'll see:
```
⚠️ Failed to launch stats dashboard: No suitable terminal emulator found
Continuing with main replay...
```

**Solution:** Install one of the supported terminals (see "Install" above)

## Tips & Tricks

### Tip 1: Maximize Dashboard Window

The dashboard is designed to fit in a small terminal. Maximize the window to see more details:

```bash
# On macOS: Cmd+Ctrl+F (fullscreen)
# On Linux: F11 (fullscreen, terminal-dependent)
```

### Tip 2: Compare Multiple Missions

Run separate replay instances with separate dashboards:

```bash
# Terminal 1
cargo run -- replay mission_v1.bag --stats-dashboard

# Terminal 2 (new terminal)
cargo run -- replay mission_v2.bag --stats-dashboard

# Compare metrics side-by-side
```

### Tip 3: Automate Analysis

Use the dashboard as part of analysis scripts:

```bash
#!/bin/bash
for mission in missions/*.bag; do
 echo "Analyzing: $mission"
 timeout 30 cargo run -- replay "$mission" --stats-dashboard
 # Dashboard auto-closes after timeout
done
```

## What's Next?

### Coming in v0.2
- Dashboard persistence (saves stats to file)
- Historical graphs (plot metrics over time)
- Export dashboard data as CSV

### Coming in v0.3
- Real-time stats sync (live updates from main process)
- Multi-window support (separate sensor windows)
- Custom dashboard layouts

### Coming in v0.4
- Web-based dashboard
- Windows terminal support
- Remote dashboards (SSH support)

## See Also

- [Stats Dashboard Documentation](STATS_DASHBOARD.md) - Complete technical details
- [Example Code](examples/stats_dashboard_demo.rs) - Programmatic usage
- [PyRoboReplay CLI Guide](https://github.com/mullassery/pyroboreplay#cli-reference)

---

**Questions?** Open an issue on [GitHub](https://github.com/mullassery/pyroboreplay/issues)
