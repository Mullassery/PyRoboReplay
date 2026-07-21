# PyRoboReplay Python API Reference

Complete reference for the PyRoboReplay Python API.

## Mission Class

The main entry point for working with robot missions.

### Creating a Mission

```python
from pyroboreplay import Mission

# Load from ROS 2 bag file
mission = Mission.from_ros_bag("path/to/mission.db3")
```

**Parameters:**
- `path` (str): Path to ROS 2 bag file (.db3 or .bag format)

**Returns:**
- `Mission`: Mission object with all events loaded

**Raises:**
- `IOError`: If file not found or cannot be read
- `ValueError`: If file is not a valid ROS 2 bag

### Properties & Methods

#### Basic Info

```python
mission.mission_id()           # → str: Unique mission identifier (UUID)
mission.name()                 # → str: Mission name (from filename or metadata)
mission.event_count()          # → int: Total number of events
mission.duration_seconds()     # → Optional[int]: Mission duration in seconds
```

#### Sensor Discovery

```python
sensors = mission.get_available_sensors()
# → List[str]: ["lidar", "camera", "imu", "odometry"]
```

Returns all unique sensor types in the mission, sorted alphabetically.

#### Sensor Queries (Individual Replay)

```python
# Get all frames from one sensor type
lidar_frames = mission.get_sensor_frames("lidar")
# → List[Event]: All lidar scan events

camera_frames = mission.get_sensor_frames("camera")
# → List[Event]: All camera frame events

imu_data = mission.get_sensor_frames("imu")
# → List[Event]: All IMU data events

odom_updates = mission.get_sensor_frames("odometry")
# → List[Event]: All odometry updates
```

**Sensor Types:**
- `"lidar"` - LidarScan events (laser scanner)
- `"camera"` - CameraFrame events (image data)
- `"imu"` - IMUData events (accelerometer, gyroscope, magnetometer)
- `"odometry"` - Odometry events (pose, velocity)
- `"costmap"` - Costmap events (occupancy grid)

#### Multi-Sensor Queries

```python
# Get frames from multiple sensors
frames = mission.get_multi_sensor_frames(["lidar", "camera"])
# → List[Event]: All lidar + camera events (24k total in warehouse mission)

# Useful for synchronized multi-sensor analysis
for frame in frames:
    if frame.get_sensor_type() == "lidar":
        print(f"Lidar at {frame.get_timestamp()}")
    elif frame.get_sensor_type() == "camera":
        print(f"Camera at {frame.get_timestamp()}")
```

#### Holistic Queries (All Sensors)

```python
# Get all events at a specific timestamp
timestamp = "2026-07-21T13:37:47Z"  # ISO 8601 format
events_at_t = mission.get_events_at_timestamp(timestamp)
# → List[Event]: All sensor observations at this moment

print(f"At {timestamp}, sensors recorded:")
for event in events_at_t:
    print(f"  - {event.get_sensor_type()}")
```

#### Event Statistics

```python
# Get breakdown of events by type
counts = mission.get_event_counts()
# → List[Tuple[str, int]]: [("imu_data", 60000), ("camera_frame", 18000), ...]

for event_type, count in counts:
    percentage = (count / mission.event_count()) * 100
    print(f"{event_type}: {count} ({percentage:.1f}%)")
```

#### Get All Events

```python
all_events = mission.get_all_events()
# → List[Event]: Every event in the mission (chronologically sorted)

for event in all_events[:10]:
    print(f"{event.get_timestamp()}: {event.get_event_type()}")
```

#### Export

```python
json_str = mission.to_json()
# → str: Mission serialized as JSON (for data export/sharing)

with open("mission_export.json", "w") as f:
    f.write(json_str)
```

---

## Event Class

Represents a single sensor or navigation event.

### Properties

```python
event = mission.get_sensor_frames("lidar")[0]

event.get_event_type()      # → str: "lidar_scan", "camera_frame", etc.
event.get_timestamp()        # → str: ISO 8601 timestamp "2026-07-21T13:37:47Z"
event.get_robot_id()         # → Optional[str]: Robot identifier, e.g. "warehouse_robot_1"
event.get_sensor_type()      # → Optional[str]: "lidar", "camera", "imu", "odometry", or None
```

### Examples

```python
# Print event details
event = mission.get_all_events()[0]
print(f"Event: {event}")
# Output: Event(type='lidar_scan', timestamp='2026-07-21T13:37:47.123456Z', 
#         robot='warehouse_robot_1', sensor='lidar')

# Filter events by type
lidar_events = [e for e in mission.get_all_events() 
                if e.get_sensor_type() == "lidar"]

# Get robot-specific events
robot_1_events = [e for e in mission.get_all_events() 
                  if e.get_robot_id() == "robot_1"]

# Time-range filtering
from datetime import datetime, timedelta
start = datetime.fromisoformat("2026-07-21T13:37:47Z")
end = start + timedelta(minutes=5)
events_in_range = [e for e in mission.get_all_events()
                   if start.isoformat() <= e.get_timestamp() <= end.isoformat()]
```

---

## Common Patterns

### Replay Specific Sensor Over Time

```python
mission = Mission.from_ros_bag("mission.bag")
lidar_frames = mission.get_sensor_frames("lidar")

print(f"Replaying {len(lidar_frames)} lidar scans...")
for i, frame in enumerate(lidar_frames):
    print(f"[{i+1}/{len(lidar_frames)}] {frame.get_timestamp()}")
    # Process lidar data...
```

### Analyze Coverage Efficiency

```python
mission = Mission.from_ros_bag("mission.bag")

# Count sensor observations per second
duration = mission.duration_seconds()
lidar_hz = len(mission.get_sensor_frames("lidar")) / duration
camera_hz = len(mission.get_sensor_frames("camera")) / duration

print(f"Lidar frequency: {lidar_hz:.1f} Hz")
print(f"Camera frequency: {camera_hz:.1f} Hz")
```

### Compare Two Missions

```python
mission_a = Mission.from_ros_bag("strategy_a.bag")
mission_b = Mission.from_ros_bag("strategy_b.bag")

# Compare mission lengths
print(f"Strategy A: {mission_a.duration_seconds()}s")
print(f"Strategy B: {mission_b.duration_seconds()}s")

# Compare sensor coverage
print(f"Strategy A lidar: {len(mission_a.get_sensor_frames('lidar'))} scans")
print(f"Strategy B lidar: {len(mission_b.get_sensor_frames('lidar'))} scans")
```

### Jupyter Notebook Analysis

```python
import pandas as pd
from pyroboreplay import Mission

# Load mission
mission = Mission.from_ros_bag("mission.bag")

# Create analysis dataframe
events_data = []
for event in mission.get_all_events():
    events_data.append({
        'timestamp': event.get_timestamp(),
        'type': event.get_event_type(),
        'sensor': event.get_sensor_type(),
        'robot': event.get_robot_id(),
    })

df = pd.DataFrame(events_data)

# Analyze
print(df.groupby('sensor')['type'].count())
print(df.groupby('robot')['type'].count())
```

---

## Error Handling

```python
from pyroboreplay import Mission

try:
    mission = Mission.from_ros_bag("nonexistent.bag")
except IOError as e:
    print(f"Failed to load bag: {e}")

# Graceful handling of empty queries
mission = Mission.from_ros_bag("mission.bag")
events = mission.get_sensor_frames("nonexistent_sensor")
if not events:
    print("No events found for that sensor type")
```

---

## Performance Notes

All operations are optimized for interactive use:

- **Query latency**: <10ms (even for 1M+ events)
- **Memory efficient**: References to events, not copies
- **Batch processing**: Linear iteration over all events

For very large missions (>10M events), consider:
- Filtering by sensor type first (`get_sensor_frames`)
- Using time ranges in your own Python code
- Exporting to pandas DataFrame for vectorized operations

---

## Version Info

- **PyRoboReplay**: v0.1.0 (Phase 1)
- **Python**: 3.10+ required
- **Supported bag formats**: ROS 2 .db3 (SQLite)
