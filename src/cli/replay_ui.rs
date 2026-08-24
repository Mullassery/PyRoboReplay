use crate::core::event::MissionRecord;
use crate::cli::lidar_viz::{LidarVisualization, LidarVizConfig};
use crossbeam_channel::{Receiver, Sender};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

/// Below this size, panels can't lay out sensibly (ratatui doesn't panic on
/// tiny areas, but content gets silently clipped/garbled) -- show an explicit
/// message instead.
const MIN_TERMINAL_WIDTH: u16 = 60;
const MIN_TERMINAL_HEIGHT: u16 = 15;

/// A request for the background lidar-render worker: render the scan at
/// `event_index` at the given panel size.
struct LidarRenderRequest {
    event_index: usize,
    width: usize,
    height: usize,
}

/// A completed frame from the worker. Cached and reused until a newer one
/// (matching the current index/size) arrives.
struct LidarRenderResult {
    event_index: usize,
    width: usize,
    height: usize,
    text: String,
}

/// Spawn the background thread that does lidar-frame rendering off the UI
/// thread. Bounded, capacity-1 channels in both directions: the UI thread
/// uses `try_send`, so if the worker is still busy on a previous frame it
/// simply skips queuing another request rather than blocking -- the next
/// redraw will pick up whatever the worker finishes. This is the
/// crossbeam-channel worker pattern the per-frame decode/render path needs
/// to exist *before* real per-event decoding (currently a synthetic
/// demonstration pattern -- see the loop below) replaces it, so adding that
/// later doesn't mean retrofitting threading onto an already-blocking UI.
fn spawn_lidar_worker() -> (Sender<LidarRenderRequest>, Receiver<LidarRenderResult>) {
    let (req_tx, req_rx) = crossbeam_channel::bounded::<LidarRenderRequest>(1);
    let (res_tx, res_rx) = crossbeam_channel::bounded::<LidarRenderResult>(1);

    std::thread::spawn(move || {
        while let Ok(request) = req_rx.recv() {
            let config = LidarVizConfig {
                width: request.width,
                height: request.height,
                ..Default::default()
            };

            let mut viz = LidarVisualization::new(&config);
            // Sample readings (in production, would extract from the real
            // event) -- the point of this worker is *where* this computation
            // runs, not what it computes; a real decoder slots in here later
            // without touching the threading.
            for angle in (0..360).step_by(5) {
                let angle_f = angle as f32;
                let range = 10.0 + (angle_f.sin() * 5.0);
                let intensity = Some(0.5 + (angle_f.cos() * 0.5).abs());
                viz.add_reading(angle_f, range, intensity, &config);
            }
            let text = viz.render();

            let result = LidarRenderResult {
                event_index: request.event_index,
                width: request.width,
                height: request.height,
                text,
            };
            // An error here means the receiver was dropped (app shutting
            // down) -- exit the thread rather than looping forever.
            if res_tx.send(result).is_err() {
                break;
            }
        }
    });

    (req_tx, res_rx)
}

pub struct ReplayState {
    mission: MissionRecord,
    current_index: usize,
    is_playing: bool,
    playback_speed: f32,
    selected_sensors: Vec<String>,
    #[allow(dead_code)]
    all_sensors: Vec<String>,
    lidar_request_tx: Sender<LidarRenderRequest>,
    lidar_result_rx: Receiver<LidarRenderResult>,
    lidar_cache: Option<LidarRenderResult>,
}

impl ReplayState {
    pub fn new(mission: MissionRecord, sensors: Option<Vec<String>>) -> Self {
        // Get all available sensors from mission
        let all_sensors: Vec<String> = mission
            .events
            .iter()
            .filter_map(|e| e.sensor_type())
            .map(|s| s.to_string())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let selected_sensors = sensors.unwrap_or_else(|| all_sensors.clone());
        let (lidar_request_tx, lidar_result_rx) = spawn_lidar_worker();

        Self {
            mission,
            current_index: 0,
            is_playing: false,
            playback_speed: 1.0,
            selected_sensors,
            all_sensors,
            lidar_request_tx,
            lidar_result_rx,
            lidar_cache: None,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Run UI loop
        let result = self.ui_loop(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    fn ui_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Draw UI
            terminal.draw(|f| self.draw_ui(f))?;

            // Handle input
            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if !self.handle_key(key) {
                        break;
                    }
                }

                // Frame-skipping: if more key events are already queued (e.g.
                // the user is holding an arrow key to scrub rapidly through a
                // high-density mission), apply all of them before the next
                // redraw instead of redrawing once per keypress. Redrawing --
                // and re-requesting a lidar frame -- for every single queued
                // event during a fast scrub is wasted work nobody sees, since
                // only the state after the burst is ever actually rendered.
                while crossterm::event::poll(std::time::Duration::from_millis(0))? {
                    if let Event::Key(key) = event::read()? {
                        if !self.handle_key(key) {
                            return Ok(());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return false,
            KeyCode::Char(' ') => self.is_playing = !self.is_playing,
            KeyCode::Right | KeyCode::Char('n') => self.next_event(),
            KeyCode::Left | KeyCode::Char('p') => self.previous_event(),
            KeyCode::Up => self.playback_speed = (self.playback_speed * 1.5).min(4.0),
            KeyCode::Down => self.playback_speed = (self.playback_speed / 1.5).max(0.25),
            KeyCode::Home => self.current_index = 0,
            KeyCode::End => self.current_index = self.mission.events.len().saturating_sub(1),
            KeyCode::Char('?') => {
                // Show help (in real impl, would toggle help panel)
            }
            _ => {}
        }
        true
    }

    fn next_event(&mut self) {
        self.current_index = (self.current_index + 1).min(self.mission.events.len().saturating_sub(1));
    }

    fn previous_event(&mut self) {
        self.current_index = self.current_index.saturating_sub(1);
    }

    fn draw_ui(&mut self, f: &mut Frame) {
        let size = f.size();

        if size.width < MIN_TERMINAL_WIDTH || size.height < MIN_TERMINAL_HEIGHT {
            let message = format!(
                "Terminal too small ({}x{}). Need at least {}x{}.",
                size.width, size.height, MIN_TERMINAL_WIDTH, MIN_TERMINAL_HEIGHT
            );
            let warning = Paragraph::new(message)
                .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
                .alignment(Alignment::Center)
                .block(Block::default().borders(Borders::ALL).title("Window Too Small"));
            f.render_widget(warning, size);
            return;
        }

        // Check if current event is a lidar scan
        let is_lidar_event = self.current_index < self.mission.events.len() &&
            self.mission.events[self.current_index].event_type() == "LidarScan";

        // Main layout: header, timeline, event details/lidar, footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(4),
                Constraint::Length(2),
            ])
            .split(size);

        // Header
        self.draw_header(f, chunks[0]);

        // Timeline and details
        let middle_chunks = if is_lidar_event && size.width > 120 {
            // Wide layout: timeline + lidar visualization
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(chunks[1])
        } else {
            // Standard layout: timeline + details
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(chunks[1])
        };

        self.draw_timeline(f, middle_chunks[0]);

        if is_lidar_event && size.width > 120 {
            self.draw_lidar_visualization(f, middle_chunks[1]);
        } else {
            self.draw_event_details(f, middle_chunks[1]);
        }

        // Progress
        self.draw_progress(f, chunks[2]);

        // Footer (help)
        self.draw_footer(f, chunks[3]);
    }

    fn draw_header(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let status = if self.is_playing { "▶ Playing" } else { "⏸ Paused" };
        let title = format!(
            "{}  {}x  Events: {}",
            status,
            self.playback_speed,
            self.mission.events.len()
        );

        let header = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::BOTTOM));

        f.render_widget(header, area);
    }

    fn draw_timeline(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let filtered_events: Vec<_> = self
            .mission
            .events
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                e.sensor_type()
                    .map_or(false, |st| self.selected_sensors.contains(&st.to_string()))
            })
            .collect();

        let items: Vec<ListItem> = filtered_events
            .iter()
            .enumerate()
            .map(|(_idx, (orig_idx, event))| {
                let timestamp = event.timestamp().format("%H:%M:%S%.3f");
                let event_type = event.event_type();
                let is_current = *orig_idx == self.current_index;
                let marker = if is_current { "→ " } else { "  " };

                let text = format!("{}{} [{}] {}", marker, timestamp, event_type,
                    event.robot_id().unwrap_or("?"));

                let style = if is_current {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(text).style(style)
            })
            .collect();

        let timeline = List::new(items)
            .block(Block::default().title("Timeline").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));

        f.render_widget(timeline, area);
    }

    fn draw_event_details(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        if self.current_index >= self.mission.events.len() {
            return;
        }

        let event = &self.mission.events[self.current_index];
        let mut lines = vec![];

        lines.push(Line::from(Span::styled(
            "Event Details",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )));

        lines.push(Line::from(""));
        lines.push(Line::from(format!("Type: {}", event.event_type())));
        lines.push(Line::from(format!("Timestamp: {}", event.timestamp())));

        if let Some(robot) = event.robot_id() {
            lines.push(Line::from(format!("Robot: {}", robot)));
        }

        if let Some(sensor) = event.sensor_type() {
            lines.push(Line::from(format!("Sensor: {}", sensor)));
        }

        let details = Paragraph::new(lines)
            .block(Block::default().title("Details").borders(Borders::ALL))
            .style(Style::default().fg(Color::Green));

        f.render_widget(details, area);
    }

    fn draw_progress(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let progress = if self.mission.events.is_empty() {
            0.0
        } else {
            (self.current_index as f64 / self.mission.events.len() as f64) * 100.0
        };

        let gauge = Gauge::default()
            .block(Block::default().title("Progress").borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(progress / 100.0)
            .label(format!("{:.1}% ({}/{})", progress, self.current_index, self.mission.events.len()));

        f.render_widget(gauge, area);
    }

    fn draw_lidar_visualization(&mut self, f: &mut Frame, area: Rect) {
        if self.current_index >= self.mission.events.len() {
            return;
        }

        let event = &self.mission.events[self.current_index];
        if event.event_type() != "LidarScan" {
            return;
        }

        let width = (area.width as usize).saturating_sub(4).max(40);
        let height = (area.height as usize).saturating_sub(4).max(10);

        // Pick up whatever the background worker has finished since the last
        // draw; keep only the most recent (a burst of scrubbing may have
        // queued/produced more than one, and only the latest matters).
        while let Ok(result) = self.lidar_result_rx.try_recv() {
            self.lidar_cache = Some(result);
        }

        let needs_new_frame = match &self.lidar_cache {
            Some(cached) => {
                cached.event_index != self.current_index
                    || cached.width != width
                    || cached.height != height
            }
            None => true,
        };
        if needs_new_frame {
            // try_send, not send: if the worker is still rendering a
            // previous frame, drop this request rather than blocking the UI
            // thread on a full channel -- the next draw call tries again,
            // and by then either the worker has caught up or the user has
            // moved on to a different event anyway.
            let _ = self.lidar_request_tx.try_send(LidarRenderRequest {
                event_index: self.current_index,
                width,
                height,
            });
        }

        let Some(cached) = &self.lidar_cache else {
            let placeholder = Paragraph::new("Rendering lidar scan...")
                .block(Block::default().title("Lidar Scan (2D Polar)").borders(Borders::ALL))
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(placeholder, area);
            return;
        };

        let lines: Vec<Line> = cached
            .text
            .lines()
            .map(|line| Line::from(Span::raw(line.to_string())))
            .collect();

        let stale = cached.event_index != self.current_index;
        let title = if stale {
            "Lidar Scan (2D Polar) [updating...]"
        } else {
            "Lidar Scan (2D Polar)"
        };
        let visualization = Paragraph::new(lines)
            .block(Block::default().title(title).borders(Borders::ALL))
            .style(Style::default().fg(Color::Green));

        f.render_widget(visualization, area);
    }

    fn draw_footer(&self, f: &mut Frame, area: ratatui::layout::Rect) {
        let help = vec![
            Span::raw("Space: "),
            Span::styled("Play/Pause", Style::default().fg(Color::Yellow)),
            Span::raw(" | ←→: "),
            Span::styled("Step", Style::default().fg(Color::Yellow)),
            Span::raw(" | ↑↓: "),
            Span::styled("Speed", Style::default().fg(Color::Yellow)),
            Span::raw(" | Q: "),
            Span::styled("Quit", Style::default().fg(Color::Yellow)),
        ];

        let footer = Paragraph::new(Line::from(help))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);

        f.render_widget(footer, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::{MissionEvent, MissionRecord};
    use chrono::Utc;
    use ratatui::backend::TestBackend;
    use std::time::Duration;

    fn nav_event(robot_id: &str) -> MissionEvent {
        MissionEvent::NavigationDecision {
            robot_id: robot_id.to_string(),
            timestamp: Utc::now(),
            decision_type: "test".to_string(),
            rationale: None,
        }
    }

    fn mission_with_events(count: usize) -> MissionRecord {
        let mut mission = MissionRecord::new("test_mission");
        for i in 0..count {
            mission.add_event(nav_event(&format!("robot_{i}")));
        }
        mission
    }

    // --- Lidar render worker (crossbeam-channel background thread) ---

    #[test]
    fn lidar_worker_renders_a_frame_off_the_calling_thread() {
        let (req_tx, res_rx) = spawn_lidar_worker();

        req_tx
            .send(LidarRenderRequest { event_index: 3, width: 50, height: 20 })
            .expect("worker thread should be alive to receive");

        let result = res_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("worker should produce a result");

        assert_eq!(result.event_index, 3);
        assert_eq!(result.width, 50);
        assert_eq!(result.height, 20);
        assert!(!result.text.is_empty());
    }

    #[test]
    fn lidar_worker_handles_multiple_sequential_requests_without_deadlock() {
        let (req_tx, res_rx) = spawn_lidar_worker();

        for i in 0..5 {
            req_tx
                .send(LidarRenderRequest { event_index: i, width: 40, height: 15 })
                .unwrap();
            let result = res_rx.recv_timeout(Duration::from_secs(2)).unwrap();
            assert_eq!(result.event_index, i);
        }
    }

    #[test]
    fn lidar_worker_exits_cleanly_when_sender_is_dropped() {
        let (req_tx, res_rx) = spawn_lidar_worker();
        drop(req_tx);
        // The worker's recv() now returns Err and the thread exits; the
        // result channel should observe the disconnect rather than hang.
        let outcome = res_rx.recv_timeout(Duration::from_secs(2));
        assert!(outcome.is_err());
    }

    // --- Navigation / frame-skip-relevant state transitions ---

    #[test]
    fn next_and_previous_event_clamp_at_bounds() {
        let mission = mission_with_events(3);
        let mut state = ReplayState::new(mission, None);

        assert_eq!(state.current_index, 0);
        state.previous_event(); // already at 0, must not underflow
        assert_eq!(state.current_index, 0);

        state.next_event();
        state.next_event();
        state.next_event(); // one past the last real event, must clamp
        assert_eq!(state.current_index, 2);
    }

    #[test]
    fn handle_key_toggles_play_state_and_clamps_speed() {
        let mission = mission_with_events(2);
        let mut state = ReplayState::new(mission, None);

        assert!(!state.is_playing);
        state.handle_key(KeyEvent::from(KeyCode::Char(' ')));
        assert!(state.is_playing);

        for _ in 0..20 {
            state.handle_key(KeyEvent::from(KeyCode::Up));
        }
        assert!(state.playback_speed <= 4.0);

        for _ in 0..20 {
            state.handle_key(KeyEvent::from(KeyCode::Down));
        }
        assert!(state.playback_speed >= 0.25);
    }

    #[test]
    fn handle_key_quit_returns_false() {
        let mission = mission_with_events(1);
        let mut state = ReplayState::new(mission, None);
        assert!(!state.handle_key(KeyEvent::from(KeyCode::Char('q'))));
    }

    // --- Minimum-viewport guard ---

    #[test]
    fn draw_ui_shows_warning_below_minimum_terminal_size() {
        let mission = mission_with_events(1);
        let mut state = ReplayState::new(mission, None);

        let backend = TestBackend::new(MIN_TERMINAL_WIDTH - 5, MIN_TERMINAL_HEIGHT - 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| state.draw_ui(f)).unwrap();

        let buffer_text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(buffer_text.contains("too small") || buffer_text.contains("Too Small"));
    }

    #[test]
    fn draw_ui_renders_normal_layout_above_minimum_terminal_size() {
        let mission = mission_with_events(1);
        let mut state = ReplayState::new(mission, None);

        let backend = TestBackend::new(MIN_TERMINAL_WIDTH + 40, MIN_TERMINAL_HEIGHT + 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| state.draw_ui(f)).unwrap();

        let buffer_text: String = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect();
        assert!(!buffer_text.contains("Too Small"));
        // The normal layout renders the timeline panel's title.
        assert!(buffer_text.contains("Timeline"));
    }
}
