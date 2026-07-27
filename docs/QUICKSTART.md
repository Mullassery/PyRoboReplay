# PyRoboReplay Quick Start Guide

Welcome! PyRoboReplay is a time-travel debugger for robot fleets. Replay missions, filter by sensor, and understand why your robots did what they did.

## 🎯 30-Second Setup

### Install

```bash
# Clone the repo
git clone https://git.example.com/user/pyroboreplay.git
cd pyroboreplay

# Build the CLI
cargo build --release

# Test with synthetic data
cargo run --example generate_warehouse_mission --release
```

### Your First Replay

```bash
# Interactive timeline scrubber with keyboard controls
./target/release/pyroboreplay replay warehouse_exploration_v1.db3

# Keyboard shortcuts:
#   Space: Play/Pause
#   ←→:    Previous/Next event
#   ↑↓:    Speed up/down (0.25x to 4.0x)
#   Home/End: Jump to start/end
#   Q:     Quit
```

## 🐍 Python API (30 seconds)

```python
from pyroboreplay import Mission

# Load mission from ROS 2 bag
mission = Mission.from_ros_bag("warehouse.db3")

# Explore sensors
print(mission.get_available_sensors())  # ["lidar", "camera", "imu", "odometry"]

# Replay lidar only
lidar_frames = mission.get_sensor_frames("lidar")
print(f"Total lidar scans: {len(lidar_frames)}")

# Get all events
events = mission.get_all_events()
for event in events[:5]:
    print(f"  {event.get_timestamp()}: {event.get_event_type()}")

# Statistics
print(f"Mission duration: {mission.duration_seconds()}s")
print(f"Total events: {mission.event_count()}")
print(f"Event breakdown: {mission.get_event_counts()}")
```

## CLI Commands

### `pyroboreplay replay`
Interactive timeline replay with sensor filtering

```bash
# Replay all sensors
pyroboreplay replay mission.bag

# Replay only lidar
pyroboreplay replay mission.bag --sensor lidar

# Replay lidar + camera
pyroboreplay replay mission.bag --sensor lidar,camera

# Show only robot_1's events
pyroboreplay replay mission.bag --robot robot_1

# Replay specific time range
pyroboreplay replay mission.bag --start-time 2026-07-21T13:37:47Z --end-time 2026-07-21T13:40:00Z
```

### `pyroboreplay analyze`
Mission statistics and breakdown

```bash
pyroboreplay analyze mission.bag

# Output:
#   📊 Mission Analysis: warehouse_exploration_v1
#   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#   Total events: 96000
#   Duration: 599s
#   
#   Event types:
#     odometry_update: 12000
#     lidar_scan: 6000
#     imu_data: 60000
#     camera_frame: 18000
```

### `pyroboreplay list`
Discover available sensors in a mission

```bash
pyroboreplay list mission.bag

# Output:
#   📡 Available Sensors:
#     lidar (6000 frames)
#     camera (18000 frames)
#     imu (60000 frames)
#     odometry (12000 frames)
```

## 🏗️ Typical Workflows

### Debug Why Coverage Was Incomplete

```bash
# 1. List available sensors
pyroboreplay list mission.bag

# 2. Replay only lidar to see exploration
pyroboreplay replay mission.bag --sensor lidar

# 3. Analyze coverage gaps
pyroboreplay analyze mission.bag

# 4. Python: Correlate lidar with robot pose
from pyroboreplay import Mission
mission = Mission.from_ros_bag("mission.bag")
lidar = mission.get_sensor_frames("lidar")
poses = mission.get_sensor_frames("robot_pose")
# Correlate in your own analysis
```

### Compare Two Missions

```python
from pyroboreplay import Mission

mission_a = Mission.from_ros_bag("exploration_strategy_a.bag")
mission_b = Mission.from_ros_bag("exploration_strategy_b.bag")

print(f"Mission A: {mission_a.event_count()} events, {mission_a.duration_seconds()}s")
print(f"Mission B: {mission_b.event_count()} events, {mission_b.duration_seconds()}s")

# Strategy A lidar efficiency
lidar_a = len(mission_a.get_sensor_frames("lidar"))
lidar_b = len(mission_b.get_sensor_frames("lidar"))
print(f"Lidar efficiency: A={lidar_a/mission_a.duration_seconds():.1f} Hz, " 
      f"B={lidar_b/mission_b.duration_seconds():.1f} Hz")
```

### Investigate Why Robot Stopped

```python
from pyroboreplay import Mission

mission = Mission.from_ros_bag("mission.bag")

# Get events around timestamp where robot stopped
events = mission.get_events_at_timestamp("2026-07-21T13:38:00Z")
for event in events:
    print(f"{event.get_event_type()}: {event.get_sensor_type()}")

# Likely causes: obstacle detection, battery low, communication loss, etc.
```

## 📊 Performance Characteristics

- **Query latency**: <10ms (96k events)
- **Parsing latency**: <5s (100MB bag file)
- **Memory usage**: ~50MB (96k events)
- **Throughput**: 240k events/second

## ⚙️ Requirements

- **Rust**: 1.70+ (for building)
- **Python**: 3.10+ (for Python API)
- **ROS 2**: Not required (works with standalone .db3 files)

## 🆘 Troubleshooting

### "Command not found: pyroboreplay"
```bash
# Use full path or add to PATH
./target/release/pyroboreplay replay mission.bag

# Or install globally
cargo install --path .
```

### "Failed to parse bag file"
```bash
# Check file exists and is valid ROS 2 format
file mission.bag  # Should say "SQLite 3.x database"

# Generate test data if learning
cargo run --example generate_warehouse_mission --release
```

### "No events found for sensor 'X'"
```bash
# List available sensors first
pyroboreplay list mission.bag

# Check spelling (case-sensitive)
pyroboreplay replay mission.bag --sensor lidar  # ✅ correct
pyroboreplay replay mission.bag --sensor Lidar  # ❌ wrong
```

## 📚 Next Steps

- [Python API Reference](API.md) - Full class and method documentation
- [Architecture Guide](ARCHITECTURE.md) - How PyRoboReplay works internally
- [Roadmap](../ROADMAP.md) - What's coming in v0.2+

## 💡 Tips

- **Large missions**: Filter by sensor (`--sensor lidar`) for faster navigation
- **Playback speed**: Use arrow keys to adjust playback speed in replay UI
- **Batch analysis**: Use Python API in Jupyter notebooks for complex queries
- **Time ranges**: ISO 8601 format for `--start-time` and `--end-time`

## 🤝 Contributing

Found a bug? Have a suggestion? [Open an issue](https://git.example.com/user/pyroboreplay/issues)

---

**Happy debugging! 🚀**
