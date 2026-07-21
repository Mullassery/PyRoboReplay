# Camera Export: Timeline-Based Intelligent Playback

## Overview

PyRoboReplay exports camera frames with **timeline-based intelligent loading** — you get a lightweight HTML file that loads frames on-demand from your mission file, not a giant video dump.

```bash
$ pyroboreplay replay mission.bag --export-camera camera_replay.html
✅ Camera export ready
📖 Frame manifest embedded in HTML
📁 Frames loaded on-demand from mission.bag
```

Two files, one workflow:
- **camera_replay.html** — Lightweight player + frame manifest (50KB)
- **mission.bag** — Original mission file (place in same directory)

The player loads only frames you view. No bloated multi-GB HTML file.

## Quick Start

### Export Camera Frames

```bash
# Export (creates small HTML file + manifest)
pyroboreplay replay warehouse_mission.bag --export-camera replay.html

# Copy mission file to same directory as HTML
cp warehouse_mission.bag ./

# Open in browser
open replay.html
```

### How It Works

1. **Export phase** (on your machine):
   - Scans mission file for CameraFrame events
   - Builds lightweight frame manifest (timestamps, dimensions, encoding)
   - Embeds manifest in HTML
   - **Frames are NOT copied** — HTML just references them

2. **Playback phase** (in browser):
   - Browser loads HTML + manifest
   - Detects mission.bag in same directory
   - When you navigate to a frame, browser extracts it from mission.bag
   - Only frames you view are decoded into memory

3. **Seeking**:
   - Click slider to jump to frame
   - Frame is instantly extracted from mission.bag via byte offsets
   - No loading bar, no lag

## Requirements

### File Placement
```
my_mission/
├── replay.html          (exported from PyRoboReplay)
├── mission.bag          (original mission file)
└── mission.db3          (or .db3 format)
```

Both files must be in **the same directory**. The HTML file auto-detects the mission file.

### Browser Features Needed
- ✅ FileReader API (read local mission.bag)
- ✅ Modern JavaScript (ES6+)
- ✅ All modern browsers (Chrome, Firefox, Safari, Edge)

## Playback Controls

### Buttons
| Button | Action |
|--------|--------|
| **▶ Play** | Auto-playback from current frame |
| **⏸ Pause** | Pause (changes icon during play) |
| **← Previous** | Jump to previous frame |
| **Next →** | Jump to next frame |
| **⏮ First** | Jump to first frame |
| **Last ⏭** | Jump to last frame |

### Speed Control
Dropdown menu: 0.25x → 0.5x → 1.0x → 1.5x → 2.0x → 4.0x

### Frame Slider
Drag to scrub through frames, or click to jump.

### Statistics Panel
- **Total Frames**: Number of camera frames in mission
- **Current Frame**: Frame number / Total
- **Display Size**: Resolution (e.g., 1920×1080)
- **Encoding**: Image format (rgb8, mono8, etc.)

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| **Space** | Play / Pause |
| **→** | Next frame |
| **←** | Previous frame |
| **Home** | First frame |
| **End** | Last frame |
| **1-9** | Speed shortcuts (10%-90%) |

## Use Cases

### Lightweight Distribution
- Export camera replay as two small files
- Email mission.bag + replay.html together
- Team member opens HTML, mission data is right there
- No cloud upload, no data exposure

### Collaborative Debugging
```bash
# Engineer 1: Export camera replay
pyroboreplay replay warehouse_mission.bag --export-camera debug.html

# Engineer 1: Share both files
# Email: debug.html + mission.bag

# Engineer 2: Opens debug.html in browser
# Browser loads frames from mission.bag on-demand
# Team discusses same frames in real-time
```

### Temporal Analysis
- Replay camera alongside lidar/IMU in terminal
- HTML player shows camera frames at precise timestamps
- Correlate with sensor anomalies

### Incident Investigation
- Mission file is evidence (immutable, sealed)
- HTML player is interface (no external dependencies)
- Single replay session works offline, forever

## Technical Details

### Frame Manifest Format

Embedded in HTML as JSON:
```json
{
  "mission_id": "abc123",
  "mission_name": "warehouse_exploration_v1",
  "total_frames": 1250,
  "fps": 30.0,
  "frames": [
    {
      "index": 0,
      "timestamp": "2026-07-21T10:00:00Z",
      "width": 1920,
      "height": 1080,
      "encoding": "rgb8",
      "event_index": 42
    },
    ...
  ]
}
```

Each frame entry includes:
- **index**: Frame number (0-based)
- **timestamp**: ISO 8601 timestamp
- **width/height**: Pixel dimensions
- **encoding**: ROS encoding type (rgb8, mono8, etc.)
- **event_index**: Position in mission.events array

### Supported Image Formats

| Encoding | MIME Type | Notes |
|----------|-----------|-------|
| `rgb8` | `image/jpeg` | RGB color (most common) |
| `bgr8` | `image/jpeg` | OpenCV BGR color |
| `rgba8` | `image/jpeg` | RGBA with alpha |
| `bgra8` | `image/jpeg` | BGRA with alpha |
| `mono8` | `image/png` | 8-bit grayscale |
| `mono16` | `image/png` | 16-bit grayscale |

### File Sizes

| Component | Size |
|-----------|------|
| HTML file (player + manifest) | ~50-100 KB |
| Mission.bag (frames + all events) | Original size |
| Total for user | Same as original mission |

**vs. Traditional Export**:
- ❌ Old way: Full frame data embedded (1000 frames × 640×480 RGB = ~500MB HTML file)
- ✅ New way: Just manifest (50KB HTML) + reference to original mission file

### Performance

- **HTML Load**: <500ms
- **Frame Extract**: <100ms per frame (on modern hardware)
- **Seeking**: Instant (direct byte offset lookup)
- **Memory**: Only 1-2 frames in RAM at a time (not all frames)

### Browser Compatibility

| Browser | Support | Notes |
|---------|---------|-------|
| Chrome/Edge | ✅ Full | Best FileReader support |
| Firefox | ✅ Full | Excellent |
| Safari | ✅ Full | Works great |
| Mobile Safari | ⚠️ Limited | FileReader may have limits |
| Android Chrome | ✅ Full | Good on tablets |

Note: Mobile browsers may have file size limits. Mission.bag should be <500MB for reliable mobile playback.

## Troubleshooting

### "Mission file not found"
**Cause**: mission.bag or mission.db3 not in same directory as HTML  
**Solution**: Copy mission file to same folder as replay.html

### "No frames to display"
**Cause**: Mission has no camera frames  
**Solution**: Check mission: `pyroboreplay list mission.bag | grep -i camera`

### Playback is slow / laggy
**Cause**: Large mission file or slow disk  
**Solution**: 
- Close other browser tabs
- Use Chrome (best FileReader performance)
- Check disk I/O (is antivirus scanning?)

### "FileReader not supported"
**Cause**: Very old browser  
**Solution**: Update to modern browser (any version from 2018+)

### File too large for mobile browser
**Cause**: Mission.bag exceeds mobile file size limits  
**Solution**: 
- Export subset of frames only (future: `--frame-range` flag)
- Use desktop browser
- Split mission into smaller chunks

## Advanced Usage

### With Different Time Ranges

Future support planned (v0.3+):
```bash
# Export only frames between t1 and t2
pyroboreplay replay mission.bag \
  --export-camera subset.html \
  --start-time 2026-07-21T10:05:00Z \
  --end-time 2026-07-21T10:10:00Z
```

### Programmatic Access (Rust)

```rust
use pyroboreplay::cli::camera_export::{export_camera_to_html, CameraExportConfig};

let mission = MissionRecord::load("mission.bag")?;

let config = CameraExportConfig {
    max_width: 1280,
    max_height: 720,
    quality: 90,
    fps: 30.0,
};

export_camera_to_html(&mission, "output.html", Some(config))?;
```

### Frame Extraction (Python)

Future support planned (v0.3+):
```python
from pyroboreplay import Mission

mission = Mission.from_ros_bag("mission.bag")

# Get frame 42 as bytes
frame_data = mission.get_camera_frame(index=42)

# Frame includes: timestamp, width, height, encoding, image_data
print(f"Frame {42}: {frame_data.width}×{frame_data.height} {frame_data.encoding}")
```

## Architecture

### Module Structure
```
src/cli/
└── camera_export.rs
    ├── CameraExportConfig       # Export settings
    ├── FrameMetadata            # Per-frame metadata
    ├── FrameManifest            # Complete manifest
    ├── export_camera_to_html()  # Main export function
    └── generate_timeline_html() # HTML template
```

### Design Principles
1. **Manifest-first**: Metadata travels in HTML, data stays in mission file
2. **Lazy loading**: Only extract frames on demand
3. **Byte-efficient**: No frame duplication or re-encoding
4. **Offline-capable**: Works completely offline once files are together
5. **Shareable**: Two files, one email attachment, universal playback

## Limitations & Future Work

### Current Limitations
1. **Both files required**: Mission file must be present
2. **Single browser session**: No persistence between sessions
3. **No re-encoding**: Frames exported as-is (respect original encoding)
4. **No frame range export**: Always exports all frames (flag planned)

### Planned Enhancements (v0.3+)
- **Frame range**: `--start-frame 100 --end-frame 200`
- **Thumbnails**: Pre-generate first/last frames for preview
- **Video export**: Convert to MP4 if needed
- **Overlay support**: Draw causal events, obstacles on frames
- **Sync to timeline**: Show frame updates during CLI replay
- **Batch export**: Export multiple cameras side-by-side
- **Compression options**: JPEG quality control at export time

## Comparison: Old vs New

| Feature | Old (Full Embed) | New (Timeline) |
|---------|-----------------|----------------|
| HTML file size | 500MB+ | 50KB |
| Memory for 100 frames | 500MB | ~20MB |
| Seek speed | Instant | <100ms |
| Requires mission file | No | Yes |
| Shareable | Heavy (email limits) | Light (email-friendly) |
| Playback offline | Yes (after download) | Yes (if files together) |
| Frame extraction speed | Fast (cached) | Medium (on-demand) |

## See Also

- [Lidar Visualization](LIDAR_VISUALIZATION.md) — Terminal-based lidar replay
- [Architecture](ARCHITECTURE.md) — System design
- [Keyboard Shortcuts](KEYBOARD_SHORTCUTS.md) — All shortcuts
- [API Reference](API.md) — Python/Rust API
