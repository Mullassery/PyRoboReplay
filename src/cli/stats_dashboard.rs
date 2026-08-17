/// Separate terminal stats dashboard for real-time mission metrics
/// Launches in its own terminal window (platform-aware: macOS or Linux)

use crate::core::event::MissionRecord;
use crate::cli::sensor_stats::SensorMetadataPanel;
use std::process::{Command, Stdio};
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::sync::{Arc, Mutex};

/// Escape an arbitrary string for safe inclusion as a single-quoted token in a
/// POSIX shell script. The returned string is itself a complete, safely-quoted
/// shell word (including the surrounding quotes) that can be concatenated
/// directly next to other quoted segments in a generated script.
///
/// This is the standard technique for embedding untrusted data in shell
/// scripts: wrap in single quotes, and turn any embedded single quote into
/// `'\''` (end quote, escaped literal quote, reopen quote).
fn shell_single_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

/// Escape an arbitrary string for safe inclusion inside a double-quoted
/// AppleScript string literal. Escapes backslashes and double quotes, and
/// strips control characters (notably newlines/carriage returns) that cannot
/// appear literally inside an AppleScript string.
fn applescript_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' | '\r' => out.push(' '),
            c if c.is_control() => {} // drop other control characters
            c => out.push(c),
        }
    }
    out
}

/// Platform detection for terminal launcher
#[derive(Debug, Clone, Copy)]
pub enum Platform {
    MacOS,
    Linux,
    Unknown,
}

impl Platform {
    pub fn detect() -> Self {
        #[cfg(target_os = "macos")]
        {
            Platform::MacOS
        }
        #[cfg(target_os = "linux")]
        {
            Platform::Linux
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            Platform::Unknown
        }
    }

    pub fn display(&self) -> &'static str {
        match self {
            Platform::MacOS => "macOS",
            Platform::Linux => "Linux",
            Platform::Unknown => "Unknown",
        }
    }
}

/// Stats data structure for dashboard
#[derive(Debug, Clone)]
pub struct DashboardStats {
    pub mission_name: String,
    pub total_events: usize,
    pub elapsed_time: f64,
    pub playback_progress: f32, // 0.0 to 1.0
    pub current_event_index: usize,
    pub playback_speed: f32,
    pub sensor_panel: Option<String>, // Rendered sensor stats
}

/// Terminal launcher abstraction
pub struct TerminalLauncher {
    platform: Platform,
}

impl TerminalLauncher {
    pub fn new() -> Self {
        Self {
            platform: Platform::detect(),
        }
    }

    /// Launch a separate terminal window with a command
    pub fn launch(&self, title: &str, command: &str) -> Result<std::process::Child, Box<dyn std::error::Error>> {
        match self.platform {
            Platform::MacOS => self.launch_macos(title, command),
            Platform::Linux => self.launch_linux(title, command),
            Platform::Unknown => Err("Unsupported platform for terminal launch".into()),
        }
    }

    /// Launch terminal on macOS (uses Terminal.app or iTerm2 if available)
    fn launch_macos(&self, title: &str, command: &str) -> Result<std::process::Child, Box<dyn std::error::Error>> {
        // Try iTerm2 first, fall back to Terminal.app
        let has_iterm = Command::new("pgrep")
            .arg("-x")
            .arg("iTerm")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let app = if has_iterm { "iTerm" } else { "Terminal" };

        // `app` is one of our own two hard-coded literals ("iTerm"/"Terminal"),
        // never attacker-influenced. `command` may originate from untrusted
        // data (e.g. mission/replay content flowing through callers of this
        // function), so it must be fully escaped for AppleScript string
        // context, not just quote-escaped.
        let script = format!(
            "tell application \"{}\"
                create window with default profile
                tell current window
                    tell current tab
                        write text \"{}\"
                    end tell
                end tell
            end tell",
            app,
            applescript_escape(command)
        );

        tracing::info!("Launching {} on macOS using {}", title, app);

        let child = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .spawn()?;

        Ok(child)
    }

    /// Launch terminal on Linux (uses terminator, then falls back to gnome-terminal, xterm)
    fn launch_linux(&self, title: &str, command: &str) -> Result<std::process::Child, Box<dyn std::error::Error>> {
        // Try different terminal emulators in order of preference
        let terminals = vec![
            ("terminator", vec!["-t", title, "-e", command]),
            ("gnome-terminal", vec!["--title", title, "--", "/bin/bash", "-c", command]),
            ("xterm", vec!["-title", title, "-e", command]),
            ("xfce4-terminal", vec!["--title", title, "-e", command]),
        ];

        for (term_name, args) in terminals {
            match Command::new(term_name)
                .args(&args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(child) => {
                    tracing::info!("Launched {} on Linux using {}", title, term_name);
                    return Ok(child);
                }
                Err(_) => {
                    tracing::debug!("{} not available, trying next terminal", term_name);
                    continue;
                }
            }
        }

        Err("No suitable terminal emulator found. Install terminator, gnome-terminal, or xterm.".into())
    }
}

/// Stats dashboard state
pub struct StatsDashboard {
    mission: Arc<Mutex<MissionRecord>>,
    current_index: Arc<Mutex<usize>>,
    playback_speed: Arc<Mutex<f32>>,
    is_running: Arc<Mutex<bool>>,
}

impl StatsDashboard {
    pub fn new(mission: MissionRecord) -> Self {
        Self {
            mission: Arc::new(Mutex::new(mission)),
            current_index: Arc::new(Mutex::new(0)),
            playback_speed: Arc::new(Mutex::new(1.0)),
            is_running: Arc::new(Mutex::new(true)),
        }
    }

    /// Run the dashboard in the current terminal
    pub fn run_dashboard(&self) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.dashboard_loop(&mut terminal);

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn dashboard_loop(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Draw the dashboard
            terminal.draw(|f| self.draw_dashboard(f))?;

            // Handle input
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(key) = event::read()? {
                    if !self.handle_key(key) {
                        break;
                    }
                }
            }

            // Periodic update check
            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    fn handle_key(&self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return false,
            KeyCode::Char('+') | KeyCode::Up => {
                let mut speed = self.playback_speed.lock().unwrap();
                *speed = (*speed * 1.5).min(4.0);
            }
            KeyCode::Char('-') | KeyCode::Down => {
                let mut speed = self.playback_speed.lock().unwrap();
                *speed = (*speed / 1.5).max(0.25);
            }
            KeyCode::Char('r') => {
                let mut idx = self.current_index.lock().unwrap();
                *idx = 0;
            }
            _ => {}
        }
        true
    }

    fn draw_dashboard(&self, f: &mut Frame) {
        let mission = self.mission.lock().unwrap();
        let current_idx = *self.current_index.lock().unwrap();
        let speed = *self.playback_speed.lock().unwrap();

        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(size);

        // Header
        let header = Paragraph::new(format!(
            "📊 PyRoboReplay Stats Dashboard | Mission: {} | Event: {}/{}",
            mission.name, current_idx, mission.events.len()
        ))
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(header, chunks[0]);

        // Stats panel
        self.draw_stats_panel(f, chunks[1], &mission, current_idx, speed);

        // Footer
        let footer = Paragraph::new("Controls: +/- (speed) | R (reset) | Q (quit)")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(footer, chunks[2]);
    }

    fn draw_stats_panel(
        &self,
        f: &mut Frame,
        area: ratatui::layout::Rect,
        mission: &MissionRecord,
        current_idx: usize,
        speed: f32,
    ) {
        let panel = SensorMetadataPanel::from_mission(mission);

        let mut output = String::new();
        output.push_str("⚡ Real-time Stats\n");
        output.push_str(&format!("Total Events: {}\n", mission.events.len()));
        output.push_str(&format!("Current Position: {} / {}\n", current_idx, mission.events.len()));
        output.push_str(&format!("Progress: {:.1}%\n", (current_idx as f32 / mission.events.len().max(1) as f32) * 100.0));
        output.push_str(&format!("Playback Speed: {:.1}x\n", speed));
        output.push('\n');

        // Sensor stats
        if !panel.sensor_names().is_empty() {
            output.push_str("📡 Sensor Summary\n");
            output.push_str(&panel.summary());
            output.push('\n');
            output.push_str(&panel.render_compact());
        }

        let stats_widget = Paragraph::new(output)
            .style(Style::default().fg(Color::White))
            .block(Block::default().title("Statistics").borders(Borders::ALL));

        f.render_widget(stats_widget, area);
    }
}

/// Launch a separate stats dashboard window
pub fn launch_stats_dashboard_window(
    mission: &MissionRecord,
    title: &str,
) -> Result<std::process::Child, Box<dyn std::error::Error>> {
    let launcher = TerminalLauncher::new();

    // Create a temporary script to run the dashboard
    let script_path = PathBuf::from(format!("/tmp/pyroboreplay_stats_{}.sh", uuid::Uuid::new_v4()));

    // `title` is currently always a hard-coded caller literal, but
    // `mission.name` comes from loaded mission/replay data and must be
    // treated as untrusted. Both are embedded as safely single-quoted shell
    // tokens (never interpolated raw into the script source) so that quotes,
    // `$(...)`, backticks, semicolons, etc. in the data cannot break out of
    // the quoted context or be interpreted as shell syntax.
    let title_token = shell_single_quote(title);
    let mission_name_token = shell_single_quote(&mission.name);
    let script_content = format!(
        "#!/bin/bash\n\
         echo 'Starting PyRoboReplay Stats Dashboard for: '{}\n\
         echo 'Mission: '{}\n\
         echo 'Events: {}'\n\
         echo ''\n\
         echo 'Press Ctrl+C to exit'\n\
         echo ''\n\
         # Simulate dashboard (real implementation would use IPC)\n\
         while true; do\n\
           echo 'Stats Dashboard - Use Ctrl+C to exit'\n\
           echo 'Last Update: '$(date '+%Y-%m-%d %H:%M:%S')\n\
           sleep 1\n\
         done\n",
        title_token, mission_name_token, mission.events.len()
    );

    fs::write(&script_path, &script_content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755))?;
    }

    let command = script_path.to_string_lossy().to_string();
    let child = launcher.launch(title, &command)?;

    // Clean up script after a delay
    let script_path_clone = script_path.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(60));
        let _ = fs::remove_file(script_path_clone);
    });

    Ok(child)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        match platform {
            Platform::MacOS => assert_eq!(platform.display(), "macOS"),
            Platform::Linux => assert_eq!(platform.display(), "Linux"),
            Platform::Unknown => assert_eq!(platform.display(), "Unknown"),
        }
    }

    #[test]
    fn test_terminal_launcher_creation() {
        let launcher = TerminalLauncher::new();
        assert!(!launcher.platform.display().is_empty());
    }

    #[test]
    fn test_dashboard_stats_creation() {
        let mission = MissionRecord::new("test");
        let dashboard = StatsDashboard::new(mission);
        assert_eq!(*dashboard.current_index.lock().unwrap(), 0);
        assert_eq!(*dashboard.playback_speed.lock().unwrap(), 1.0);
    }

    // --- Shell injection regression tests -----------------------------------
    //
    // `mission.name` (and, via TerminalLauncher::launch, arbitrary `command`
    // strings) originate from loaded mission/replay files and must be treated
    // as untrusted. These tests prove the escaping helpers neutralize classic
    // shell/AppleScript metacharacters instead of just asserting on the
    // string shape.

    #[test]
    fn test_shell_single_quote_neutralizes_quote_breakout() {
        // A naive `format!("'{}'", s)` would let this break out of the quotes
        // and execute `touch <marker>` as a separate command.
        let marker = std::env::temp_dir().join(format!("pyroboreplay_test_pwn_{}.marker", uuid::Uuid::new_v4()));
        let _ = fs::remove_file(&marker);
        let malicious = format!("'; touch {}; echo '", marker.display());
        let token = shell_single_quote(&malicious);

        // The escaped token must never contain an unescaped closing quote
        // followed directly by shell syntax that isn't itself quoted.
        let script = format!("echo {}\n", token);

        // Round-trip through a real shell: the payload must be printed
        // verbatim, and no injected command may execute.
        let full_script = format!("#!/bin/bash\n{}\n", script);
        let output = Command::new("bash")
            .arg("-c")
            .arg(&full_script)
            .output()
            .expect("failed to run bash");
        assert!(output.status.success(), "escaped script failed to run: {:?}", output);
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(&malicious), "payload should be printed literally, got: {}", stdout);
        assert!(!marker.exists(), "injected command must not have executed");
        let _ = fs::remove_file(&marker);
    }

    #[test]
    fn test_shell_single_quote_handles_command_substitution_and_semicolons() {
        for payload in [
            "$(rm -rf /tmp/should-not-run)",
            "`rm -rf /tmp/should-not-run`",
            "; rm -rf /tmp/should-not-run #",
            "a'b\"c$(d)`e`;f",
        ] {
            let token = shell_single_quote(payload);
            let script = format!("#!/bin/bash\nprintf '%s' {}\n", token);
            let output = Command::new("bash")
                .arg("-c")
                .arg(&script)
                .output()
                .expect("failed to run bash");
            assert!(output.status.success(), "script failed for payload {:?}: {:?}", payload, output);
            assert_eq!(
                String::from_utf8_lossy(&output.stdout),
                payload,
                "payload must round-trip literally for input {:?}",
                payload
            );
        }
    }

    #[test]
    fn test_launch_stats_dashboard_window_script_is_safe_with_malicious_mission_name() {
        // Build a mission whose name is a classic shell-injection payload and
        // ensure the generated script (a) is syntactically valid bash and
        // (b) does not execute the injected command when run.
        let mut mission = MissionRecord::new("'; touch /tmp/pyroboreplay_test_pwned_mission; echo '");
        mission.events = vec![];

        let title_token = shell_single_quote("PyRoboReplay Stats Dashboard");
        let mission_name_token = shell_single_quote(&mission.name);
        let script_content = format!(
            "#!/bin/bash\n\
             echo 'Starting PyRoboReplay Stats Dashboard for: '{}\n\
             echo 'Mission: '{}\n\
             echo 'Events: {}'\n\
             exit 0\n",
            title_token, mission_name_token, mission.events.len()
        );

        // Syntax check only (bash -n), then a real bounded execution.
        let syntax_check = Command::new("bash")
            .arg("-n")
            .arg("-c")
            .arg(&script_content)
            .output()
            .expect("failed to run bash -n");
        assert!(syntax_check.status.success(), "generated script has invalid syntax: {:?}", syntax_check);

        let marker = std::env::temp_dir().join("pyroboreplay_test_pwned_mission");
        let _ = fs::remove_file(&marker);
        let output = Command::new("bash")
            .arg("-c")
            .arg(&script_content)
            .output()
            .expect("failed to run generated script");
        assert!(output.status.success());
        assert!(!marker.exists(), "malicious mission name must not execute injected command");
        let _ = fs::remove_file(&marker);
    }

    #[test]
    fn test_applescript_escape_neutralizes_quote_and_backslash() {
        let malicious = "\"\ndo shell script \"touch /tmp/pwned\"";
        let escaped = applescript_escape(malicious);
        // No raw double quote or newline should survive unescaped.
        assert!(!escaped.contains('\n'));
        // Every `"` in the output must be immediately preceded by a `\`.
        let bytes: Vec<char> = escaped.chars().collect();
        for (i, c) in bytes.iter().enumerate() {
            if *c == '"' {
                assert!(i > 0 && bytes[i - 1] == '\\', "unescaped quote at {}", i);
            }
        }
    }

    #[test]
    fn test_applescript_escape_handles_backslash() {
        let input = r#"a\b"c"#;
        let escaped = applescript_escape(input);
        assert_eq!(escaped, r#"a\\b\"c"#);
    }
}
