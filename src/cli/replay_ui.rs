use crate::core::event::MissionRecord;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

pub struct ReplayState {
    mission: MissionRecord,
    current_index: usize,
    is_playing: bool,
    playback_speed: f32,
    selected_sensors: Vec<String>,
    #[allow(dead_code)]
    all_sensors: Vec<String>,
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

        Self {
            mission,
            current_index: 0,
            is_playing: false,
            playback_speed: 1.0,
            selected_sensors,
            all_sensors,
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

    fn draw_ui(&self, f: &mut Frame) {
        let size = f.size();

        // Main layout: header, timeline, event details, footer
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
        let middle_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(chunks[1]);

        self.draw_timeline(f, middle_chunks[0]);
        self.draw_event_details(f, middle_chunks[1]);

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
