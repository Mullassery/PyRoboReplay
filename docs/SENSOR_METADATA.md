# Sensor Metadata Panel

## Overview

PyRoboReplay displays a real-time sensor metadata panel during mission replay showing:
- **Frame counts** per sensor
- **Frame rates** (frames per second)
- **Data quality** indicators
- **Temporal coverage** (first/last timestamp, duration)
- **Sensor specifications** (encoding, resolution, etc.)

This helps operators quickly assess data health and identify anomalies like dropped frames or missing sensors.

## Quick View

### Full Panel Display

```
📊 Sensor Metadata Panel
════════════════════════════════════════════════════════════════
├─ Lidar (lidar_0) 🟢 [███████░░░]
│  Frames: 1250 (29.8 fps avg)
│  Duration: 42.0s
│  First: 2026-07-21T10:05:12.123456+00:00
│  Last:  2026-07-21T10:05:54.456789+00:00
│  Quality: 95% complete

├─ Camera (camera_0) 🟡 [██████░░░░]
│  Frames: 600 (14.3 fps avg)
│  Duration: 41.9s
│  First: 2026-07-21T10:05:12.234567+00:00
│  Last:  2026-07-21T10:05:54.345678+00:00
│  Encoding: rgb8 (1920×1080)
│  Quality: 75% complete

├─ IMU 🟢 [████████░░]
│  Frames: 1250 (29.8 fps avg)
│  Duration: 42.0s
│  First: 2026-07-21T10:05:12.111111+00:00
│  Last:  2026-07-21T10:05:54.555555+00:00
│  Quality: 85% complete

└─ Odometry 🔴 [███░░░░░░░]
   Frames: 400 (9.5 fps avg)
   Duration: 42.1s
   First: 2026-07-21T10:05:12.345678+00:00
   Last:  2026-07-21T10:05:54.654321+00:00
   Quality: 45% complete
```

### Compact Summary

```
Sensors: 🟢 Lidar 29.8fps (1250 frames) | 🟡 Camera 14.3fps (600 frames) | 
         🟢 IMU 29.8fps (1250 frames) | 🔴 Odometry 9.5fps (400 frames)

Total: 3500 events across 4 sensors | Avg Quality: 76%
```

## Quality Indicators

### Quality Emoji Scale

| Emoji | Quality Range | Meaning |
|-------|---------------|---------|
| ✅ | 95-100% | Excellent data, no gaps |
| 🟢 | 75-94% | Good data, minimal gaps |
| 🟡 | 50-74% | Fair data, some gaps |
| 🟠 | 25-49% | Poor data, many gaps |
| 🔴 | 0-24% | Bad data, mostly missing |

### Quality Bar Visualization

```
████████░░ = 80% (8 of 10 blocks filled)
██████░░░░ = 60% (6 of 10 blocks filled)
█░░░░░░░░░ = 10% (1 of 10 blocks filled)
```

## Metrics Explained

### Frame Count
Total number of frames received for this sensor.

```
Frames: 1250
```

Helpful for detecting:
- Missing sensors (0 frames)
- Low-frequency sensors (expected)
- Data loss (fewer frames than expected)

### Frame Rate (FPS)

Average frames per second over the mission duration.

```
Frames: 1250 (29.8 fps avg)
Duration: 42.0s
Calculation: 1250 / 42.0 = 29.8 fps
```

**Typical sensor rates**:
- Lidar: 20-40 Hz
- Camera: 10-30 Hz
- IMU: 50-200 Hz
- Odometry: 10-50 Hz
- Costmap: 0.1-10 Hz

### Duration

Time span from first to last frame of that sensor.

```
Duration: 42.0s
```

Helps identify:
- Sensors starting/stopping early
- Data gaps or dropout periods
- Sync issues between sensors

### Quality Score

Percentage of expected frames received (frame distribution completeness).

```
Quality: 85% complete
```

Calculated as: `actual_frames / (total_events * average_density)`

Lower quality indicates:
- Missed frames
- Bursty data patterns
- Sensor intermittency
- Network packet loss

### Sensor Specifications (Camera)

```
Encoding: rgb8 (1920×1080)
```

Shows image format and resolution:
- **rgb8**: 24-bit color (3 bytes per pixel)
- **mono8**: 8-bit grayscale
- **bgr8**: OpenCV BGR color format
- Resolution in pixels (width × height)

## Display Modes

### Mode 1: Full Panel (Default)

Shows all details for each sensor. Used when:
- Starting replay (one-time overview)
- Checking data integrity
- Investigating sensor issues

Press **`?`** during replay to toggle help, which shows this panel.

### Mode 2: Compact Summary

Single-line overview of all sensors. Used when:
- Running multiple missions (quick glance)
- Terminal space limited
- Monitoring active replay

Auto-displayed when panel is toggled during playback.

### Mode 3: Statistics Only

Numerical breakdown without visualization:

```
Lidar:
  Type: LidarScan
  Frames: 1250
  FPS: 29.8
  Quality: 95%
  Status: 🟢
```

## Usage During Replay

### Viewing the Panel

**Initial display** (automatically shown):
- Panel appears on first few frames of replay
- Shows all detected sensors and current stats

**Toggle panel** (keyboard):
- Press **`?`** (question mark) to show/hide help panel
- Panel remains visible during scrubbing and playback

**Update frequency**:
- Panel updates every 500ms during playback
- Updated on every frame when paused
- Reflects current position in timeline

### Interpreting Results

#### Healthy Mission
```
🟢 High quality (95%+)
🟢 All expected sensors present
🟢 Consistent frame rates
🟢 No time gaps
```

**Action**: Proceed with analysis

#### Degraded Mission
```
🟡 Medium quality (50-75%)
🟡 Some sensor dropout
🔴 One critical sensor very low
🟡 Variable frame rates
```

**Action**: Investigate which sensor degraded and when

#### Problematic Mission
```
🔴 Poor quality (<50%)
🔴 Missing sensors
🔴 High dropout rate
🔴 Significant time gaps
```

**Action**: 
- Verify mission file integrity
- Check if mission was aborted early
- Investigate sensor failures

## Advanced Analysis

### Cross-Sensor Sync Check

Compare timestamps of different sensors to detect sync issues:

```
Lidar first:  10:05:12.123
Camera first: 10:05:12.234 (+111ms offset)
IMU first:    10:05:12.111 (-12ms offset)

Result: Camera started ~111ms late (possible initialization issue)
```

### Data Completeness

Use quality score to assess mission success:

```
Mission duration: 60 seconds
Lidar quality: 95% (52/60 seconds with data)
Camera quality: 75% (45/60 seconds with data)
Odometry quality: 40% (24/60 seconds with data)

Interpretation: Odometry sensor failed halfway through mission
```

### Temporal Coverage

Identify when each sensor was active:

```
Lidar:   ████████████████████ (continuous 0-60s)
Camera:  ████████████░░░░░░░░ (30s, then dropout)
Odometry:████░░░░░░░░░░░░░░░░ (first 20s only)
```

## Performance Impact

- **Display**: <5ms render time per panel update
- **Memory**: ~100KB per 1000 events tracked
- **CPU**: Negligible (<1% during playback)

## Terminal Requirements

Requires Unicode emoji support:
- ✅ ✅ Checkmark (U+2705)
- 🟢 Green circle (U+1F7E2)
- 🟡 Yellow circle (U+1F7E1)
- 🟠 Orange circle (U+1F7E0)
- 🔴 Red circle (U+1F534)

All modern terminals support these. Fallback to text mode:
```bash
# Future: ASCII fallback mode
pyroboreplay replay mission.bag --ascii-mode
# Uses: A G Y O R instead of emoji
```

## Limitations & Future Work

### Current Limitations
1. Quality calculated per-sensor (not per-robot)
2. No frame-rate variation analysis
3. No predictive dropout detection
4. Panel updates on timer (not real-time)

### Planned Enhancements (v0.3+)
- **Per-robot aggregation**: Overall quality by robot
- **Sensor health trends**: Quality over time
- **Predictive alerts**: Warn if pattern suggests dropout
- **Export as CSV**: Raw statistics for external analysis
- **Comparative mode**: Compare two missions' sensor health side-by-side
- **Anomaly highlights**: Auto-detect unusual patterns
- **Latency measurement**: Frame-to-frame timing variance

## Integration with Other Features

### With Lidar Visualization
Metadata panel shows "Lidar: 95%" → Lidar viz shows dense point cloud (confirms data quality visually)

### With Camera Export
Metadata "Camera: 50%" → Some frames may be missing in HTML export (plan accordingly)

### With IMU Graphs
Metadata "IMU: 99%" → Dense IMU graphs without gaps (good for trend analysis)

## Example Workflow

```bash
# 1. Start replay
pyroboreplay replay warehouse_mission.bag

# 2. Press '?' to see metadata panel
# → Shows: Lidar 95%, Camera 40%, IMU 98%, Odometry 60%

# 3. Camera quality is low (40%)
# → Scrub to different parts of mission to find when camera failed
# → Use frame rate data to identify exact failure point

# 4. Export camera frames only for the good part
pyroboreplay replay warehouse_mission.bag \
  --export-camera camera_partial.html \
  --start-time 2026-07-21T10:00:00Z \
  --end-time 2026-07-21T10:15:00Z

# 5. Lidar and IMU fine, camera partial
# → Use lidar + IMU for main analysis
# → Camera useful as secondary validation where available
```

## See Also

- [Lidar Visualization](LIDAR_VISUALIZATION.md) — Terminal graphs for lidar
- [IMU Visualization](IMU_VISUALIZATION.md) — Terminal graphs for IMU
- [Camera Export](CAMERA_EXPORT.md) — Browser replay with metadata
- [Keyboard Shortcuts](KEYBOARD_SHORTCUTS.md) — All available shortcuts
- [API Reference](API.md) — Programmatic access
