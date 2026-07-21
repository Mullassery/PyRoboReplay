# PyRoboReplay Keyboard Shortcuts Reference

Comprehensive keyboard shortcut guide for CLI replay mode.

## Navigation & Timeline Control

### Timeline Movement
| Key | Action | Description |
|-----|--------|-------------|
| `←` / `→` | Previous/Next Event | Move one event backward/forward |
| `Page Up` | Back 10 Events | Jump 10 events earlier |
| `Page Down` | Forward 10 Events | Jump 10 events later |
| `Home` | First Event | Jump to start of mission |
| `End` | Last Event | Jump to end of mission |
| `1-9` | Jump to 10%-90% | Quick jump (1=10%, 5=50%, 9=90%) |
| `G` | Go to Timestamp | Open dialog to jump to specific ISO 8601 timestamp |
| `J` | Jump by Index | Enter event number to jump to |

### Timeline Positioning
| Key | Action | Description |
|-----|--------|-------------|
| `Shift+Home` | Mark Start | Set bookmark at current position |
| `Shift+End` | Mark End | Set bookmark at current position |
| `[` / `]` | Previous/Next Bookmark | Jump between bookmarked positions |

---

## Playback Control

| Key | Action | Description |
|-----|--------|-------------|
| `Space` | Play/Pause | Toggle playback state |
| `+` / `=` | Speed Up | Increase playback speed (0.25x → 4.0x) |
| `-` / `_` | Slow Down | Decrease playback speed |
| `0` | Normal Speed | Reset to 1.0x playback |
| `*` | Pause | Immediately pause (if playing) |
| `.` | Step Forward | Advance one event (while paused) |
| `,` | Step Backward | Go back one event (while paused) |

---

## Sensor Control & Visualization

### Toggle Sensors
| Key | Action | Description |
|-----|--------|-------------|
| `L` | Toggle Lidar | Show/hide lidar visualization |
| `C` | Toggle Camera | Show/hide camera frame info |
| `I` | Toggle IMU | Show/hide IMU graphs |
| `O` | Toggle Odometry | Show/hide odometry display |
| `E` | Toggle Events | Show/hide event log |
| `M` | Toggle All | Show/hide all visualizations (min UI) |
| `Shift+M` | Cycle Themes | Cycle through visualization themes |

### Sensor Selection
| Key | Action | Description |
|-----|--------|-------------|
| `S` | Sensor Menu | Show list of available sensors |
| `S` `L` | Filter Lidar | Show only lidar events |
| `S` `C` | Filter Camera | Show only camera events |
| `S` `I` | Filter IMU | Show only IMU events |
| `S` `O` | Filter Odometry | Show only odometry events |
| `S` `A` | Filter All | Show all event types |
| `S` `N` | Filter None | Hide all events (useful with visualizations) |

### Visualization Options
| Key | Action | Description |
|-----|--------|-------------|
| `Shift+L` | Lidar Options | Adjust lidar visualization (resolution, intensity) |
| `Shift+I` | IMU Options | Adjust IMU graph scale/smoothing |
| `Shift+O` | Odometry Options | Show pose/velocity/orientation options |

---

## Analysis & Queries

| Key | Action | Description |
|-----|--------|-------------|
| `F` | Find Events | Search for events by type or robot_id |
| `Ctrl+F` | Find Again | Repeat last search |
| `T` | Show Stats | Display event count by type for current view |
| `Ctrl+T` | Global Stats | Display stats for entire mission |
| `D` | Show Diagnostics | Analysis: gaps, anomalies, quality issues |
| `A` | Anomalies | Highlight anomalous events (out-of-range, gaps) |
| `Shift+A` | Anomaly Options | Configure anomaly detection sensitivity |
| `R` | Event Details | Show raw event data for current event |
| `N` | Event Notes | Add/view notes for current event |

---

## Export & Output

| Key | Action | Description |
|-----|--------|-------------|
| `Ctrl+E` | Export Events | Export visible events as JSON/CSV |
| `Ctrl+C` | Export Camera | Generate camera HTML file |
| `Ctrl+L` | Export Lidar | Export lidar frames as .pcd files |
| `Ctrl+S` | Save State | Save current replay position & filters |
| `Ctrl+P` | Print Report | Generate text mission report |
| `Ctrl+J` | JSON Output | Output current view as JSON |

---

## Display & UI

| Key | Action | Description |
|-----|--------|-------------|
| `H` / `?` | Show Help | Display keyboard shortcuts |
| `Ctrl+H` | Help (Advanced) | Show advanced keyboard commands |
| `V` | Toggle Verbose | Show/hide detailed status messages |
| `Shift+V` | Verbosity Level | Cycle through verbosity levels |
| `Shift+D` | Toggle Dark/Light | Cycle color scheme (if supported) |
| `Tab` | Next Panel | Focus next UI panel |
| `Shift+Tab` | Previous Panel | Focus previous UI panel |
| `Ctrl+L` | Clear Screen | Redraw entire UI |

---

## Multi-Robot Control

| Key | Action | Description |
|-----|--------|-------------|
| `R` | Robot Menu | Show list of robots in mission |
| `R` `1-9` | Select Robot | Filter to specific robot (1=robot_1, etc.) |
| `R` `A` | All Robots | Show all robots' events |
| `Shift+R` | Compare Robots | Side-by-side comparison mode |

---

## Advanced Navigation

| Key | Action | Description |
|-----|--------|-------------|
| `W` | Where Am I? | Show current position in mission (% complete) |
| `X` | Time Statistics | Show mission time info (duration, frame rates) |
| `/` | Command Palette | Open command search (search by name) |
| `:` | Command Mode | Enter vim-like command mode (advanced) |

---

## Context-Sensitive Help

| Context | Command |
|---------|---------|
| During playback | `Space` = Play/Pause, `←/→` = Navigate |
| At start of mission | `End` = Jump to last event |
| At end of mission | `Home` = Jump to first event |
| Single robot | `R` = No-op, `R A` = Show all robots |
| Multi-robot | `R` = Robot menu, `1-9` = Select robot |
| Lidar visible | `L` = Hide, `Shift+L` = Options |
| Camera visible | `C` = Hide, `Ctrl+C` = Export |

---

## Quick Reference (Cheat Sheet)

### Most Used
```
Space      Play/Pause
←/→        Previous/Next event
Home/End   First/Last event
H/?       Help
Q/ESC     Quit
```

### Sensor Replay
```
L          Lidar only
C          Camera only
I          IMU only
O          Odometry only
M          All (toggle)
```

### Analysis
```
F          Find event
T          Statistics
D          Diagnostics
A          Show anomalies
```

### Export
```
Ctrl+E     Export events
Ctrl+C     Export camera
Ctrl+J     JSON output
```

---

## Custom Keybindings

Users can customize keybindings by creating `~/.pyroboreplay/keybindings.toml`:

```toml
# Example custom keybindings
[replay]
play_pause = "Space"
step_forward = "."
step_backward = ","
jump_to_start = "Home"
jump_to_end = "End"

[sensors]
toggle_lidar = "l"
toggle_camera = "c"
toggle_imu = "i"

[export]
export_events = "ctrl+e"
export_camera = "ctrl+c"
```

Run with `pyroboreplay replay mission.bag --keybindings ~/.pyroboreplay/keybindings.toml`

---

## Accessibility Features

### For Vision Impairment
- `V` toggles verbose mode (all actions announced in terminal)
- Screen reader support via text-based output
- High-contrast mode via `Shift+D`

### For Motor/Dexterity Issues
- Customizable keybindings (avoid complex chords)
- Command palette (`:` for vim-style commands)
- Mouse support (click UI elements, drag timeline)

### For Cognitive Load
- Context help (`?` in each mode)
- Simple defaults (only essential shortcuts)
- Advanced mode (`Ctrl+H`) for power users

---

## Tips & Tricks

### Efficient Workflow
1. Start: `End` (go to end of mission first to see length)
2. Navigate: `G` + timestamp to jump to region of interest
3. Replay: `Space` to play through, `←/→` to fine-tune
4. Analyze: `T` for stats, `F` to find specific events
5. Export: `Ctrl+C` for camera, `Ctrl+J` for JSON

### Batch Analysis
```bash
# Export multiple missions
for mission in *.bag; do
  pyroboreplay replay "$mission" --export-events "$mission.json" --json
done

# Agents can then analyze all missions
```

### Real-Time Debugging
```bash
# Watch a specific sensor during playback
pyroboreplay replay mission.bag -s lidar --verbose
# Then use 'L' to toggle visualization, 'A' for anomalies
```

---

For more help, see [QUICKSTART.md](QUICKSTART.md) and [API.md](API.md).
