# PyRoboReplay Stats Dashboard

## Overview

The CLI stats dashboard is a **separate terminal window** that displays real-time mission metrics while a simulation replay is running. It's platform-aware and automatically selects the appropriate terminal launcher for your OS.

## Features

- **Separate Terminal**: Launches in its own window (not shared stdout with the main replay)
- **Platform-Aware**: 
 - **macOS**: Uses Terminal.app or iTerm2 (whichever is available)
 - **Linux**: Tries terminator → gnome-terminal → xterm → xfce4-terminal
- **Real-Time Metrics**:
 - Total events in mission
 - Current playback position
 - Progress percentage
 - Playback speed multiplier
 - Sensor summary (frame count, FPS, quality metrics)
 - Data quality indicators
- **Interactive Controls**: Speed adjustment, reset, quit commands

## Usage

### Launch with Stats Dashboard

```bash
# Replay a mission with stats dashboard in separate window
cargo run -- replay your_mission.bag --stats-dashboard

# With sensor filtering
cargo run -- replay your_mission.bag --sensor lidar,camera --stats-dashboard

# With sensor filtering and stats dashboard
cargo run -- replay your_mission.bag -s lidar --stats-dashboard
```

### Dashboard Controls

| Key | Action |
|-----|--------|
| `+` / `↑` | Increase playback speed |
| `-` / `↓` | Decrease playback speed |
| `R` | Reset to beginning |
| `Q` / `Esc` | Close dashboard |
| `Ctrl+C` | Force exit |

## Dashboard Display

```
 PyRoboReplay Stats Dashboard | Mission: exploration_v1 | Event: 1250/5000

⚡ Real-time Stats
Total Events: 5000
Current Position: 1250 / 5000
Progress: 25.0%
Playback Speed: 1.0x

 Sensor Summary
Total: 8500 events across 6 sensors | Avg Quality: 95%
Sensors: ✅ Lidar (main) 30.5fps (2500 frames) | ✅ Camera (front) 15.2fps (750 frames) | ...

Controls: +/- (speed) | R (reset) | Q (quit)

```

## Implementation Details

### Architecture

```
PyRoboReplay Replay (main process)
 
 → Launch Stats Dashboard (separate process/terminal)
 
 → Continue with replay_ui.rs
```

### Platform Detection

The `Platform` enum detects the OS at runtime:

```rust
pub enum Platform {
 MacOS, // Uses osascript + Terminal.app/iTerm2
 Linux, // Uses terminal emulator (terminator, gnome-terminal, xterm, etc.)
 Unknown, // Returns error
}
```

### Terminal Launching

**macOS** (`launch_macos`):
- Checks if iTerm2 is running
- Falls back to Terminal.app if iTerm2 not available
- Uses AppleScript via `osascript`

```bash
osascript -e '
tell application "Terminal"
 create window with default profile
 tell current window
 tell current tab
 write text "<command>"
 end tell
 end tell
end tell
'
```

**Linux** (`launch_linux`):
- Tries terminal emulators in order of preference
- First match wins and is used to spawn the dashboard
- Command pattern: `terminator -t "PyRoboReplay Stats Dashboard" -e "<command>"`

### Stats Data Flow

1. **Mission loaded** → SensorMetadataPanel extracts stats from events
2. **Dashboard launched** → Separate terminal starts `stats_dashboard.rs`
3. **Dashboard running** → Reads mission metadata, displays in ratatui TUI
4. **Independent loop** → Dashboard has its own event loop (not blocking main replay)

### Key Components

| Component | Purpose |
|-----------|---------|
| `Platform` enum | OS detection (macOS, Linux, Unknown) |
| `TerminalLauncher` | Platform-specific terminal launching logic |
| `StatsDashboard` | TUI state and rendering for the dashboard |
| `launch_stats_dashboard_window()` | Entry point: creates temp script and launches terminal |
| `SensorMetadataPanel` | Renders sensor statistics (reused from sensor_stats.rs) |

## Technical Notes

### Process Independence

The stats dashboard runs in a **completely separate process**:
- Different terminal window (not shared stdout)
- Independent event loop
- No blocking on main replay thread
- Both run simultaneously without interference

### Cleanup

Temporary scripts are automatically cleaned up after 60 seconds:
```rust
// Script persists for IPC, then auto-deletes
thread::spawn(move || {
 thread::sleep(Duration::from_secs(60));
 let _ = fs::remove_file(script_path_clone);
});
```

### Error Handling

If dashboard launch fails:
- Main replay continues uninterrupted
- User sees warning message with reason
- No replay functionality lost

```
⚠️ Failed to launch stats dashboard: No suitable terminal emulator found
Continuing with main replay...
```

## Future Enhancements

- **Live stats sync**: Use Unix sockets or shared memory to sync live stats from main process
- **Multi-window support**: Separate windows for different sensor streams
- **Historical graphs**: Plot metrics over time
- **Export capabilities**: Save dashboard history as CSV/JSON
- **Remote dashboard**: Web-based dashboard for distributed debugging
- **Custom themes**: User-configurable dashboard colors and layouts

## Testing

### Unit Tests

```bash
cargo test stats_dashboard --lib
```

Test coverage includes:
- Platform detection
- Terminal launcher creation
- Dashboard stats initialization
- Key handling

### Manual Testing

**macOS**:
```bash
cargo run -- replay test_mission.bag --stats-dashboard
# Should open new Terminal window with dashboard
```

**Linux**:
```bash
cargo run -- replay test_mission.bag --stats-dashboard
# Should open terminator/gnome-terminal window with dashboard
```

**Without terminal emulator available**:
```bash
# Should see error message and continue replay
⚠️ Failed to launch stats dashboard: No suitable terminal emulator found
```

## Environment Variables

None currently, but the following could be added:

- `PYROBOREPLAY_STATS_TERMINAL`: Force specific terminal emulator
- `PYROBOREPLAY_STATS_DISABLED`: Skip dashboard launch even if --stats-dashboard is set
- `PYROBOREPLAY_STATS_ITERM`: Force iTerm2 on macOS (even if not running)

## Troubleshooting

### Dashboard doesn't open

**macOS**:
- Ensure Terminal.app or iTerm2 is installed
- Check System Preferences → Security & Privacy → accessibility

**Linux**:
- Install terminator: `sudo apt install terminator`
- Or gnome-terminal: `sudo apt install gnome-terminal`
- Or xterm: `sudo apt install xterm`

### Stats not updating

- Currently shows mission metadata (static)
- Real-time updates coming in v0.3
- For now, stats are computed at launch time

### Terminal window doesn't appear

- Check if the terminal emulator started (look in task manager)
- Try launching manually: `terminator -t "test" -e "sleep 10"`
- File an issue with platform and terminal info

## See Also

- [PyRoboReplay Architecture](ARCHITECTURE_COMPLETE.md)
- [Sensor Stats Panel](src/cli/sensor_stats.rs)
- [Replay UI](src/cli/replay_ui.rs)
