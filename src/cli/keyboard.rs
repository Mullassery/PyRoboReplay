/// Keyboard shortcuts and help system for terminal replay
/// Defines all keyboard commands and displays context-sensitive help

/// Category for organizing shortcuts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutCategory {
    Navigation,
    Playback,
    Sensors,
    Analysis,
    Export,
    Display,
    Help,
}

impl std::fmt::Display for ShortcutCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShortcutCategory::Navigation => write!(f, "Navigation"),
            ShortcutCategory::Playback => write!(f, "Playback"),
            ShortcutCategory::Sensors => write!(f, "Sensors"),
            ShortcutCategory::Analysis => write!(f, "Analysis"),
            ShortcutCategory::Export => write!(f, "Export"),
            ShortcutCategory::Display => write!(f, "Display"),
            ShortcutCategory::Help => write!(f, "Help"),
        }
    }
}

/// A keyboard shortcut definition
#[derive(Debug, Clone)]
pub struct Shortcut {
    pub category: ShortcutCategory,
    pub key: &'static str,
    pub description: &'static str,
    pub long_description: &'static str,
}

/// Keyboard shortcuts database
pub struct KeyboardShortcuts;

impl KeyboardShortcuts {
    /// Get all shortcuts
    pub fn all() -> Vec<Shortcut> {
        vec![
            // Navigation
            Shortcut {
                category: ShortcutCategory::Navigation,
                key: "Home",
                description: "Jump to first event",
                long_description: "Go to the beginning of the mission",
            },
            Shortcut {
                category: ShortcutCategory::Navigation,
                key: "End",
                description: "Jump to last event",
                long_description: "Go to the end of the mission",
            },
            Shortcut {
                category: ShortcutCategory::Navigation,
                key: "←",
                description: "Previous event",
                long_description: "Move to the previous event in the timeline",
            },
            Shortcut {
                category: ShortcutCategory::Navigation,
                key: "→",
                description: "Next event",
                long_description: "Move to the next event in the timeline",
            },
            Shortcut {
                category: ShortcutCategory::Navigation,
                key: "P",
                description: "Previous event (alt)",
                long_description: "Move to the previous event (alternative key)",
            },
            Shortcut {
                category: ShortcutCategory::Navigation,
                key: "N",
                description: "Next event (alt)",
                long_description: "Move to the next event (alternative key)",
            },
            Shortcut {
                category: ShortcutCategory::Navigation,
                key: "Page Up",
                description: "Jump 10 events back",
                long_description: "Move 10 events backwards quickly",
            },
            Shortcut {
                category: ShortcutCategory::Navigation,
                key: "Page Down",
                description: "Jump 10 events forward",
                long_description: "Move 10 events forwards quickly",
            },

            // Playback
            Shortcut {
                category: ShortcutCategory::Playback,
                key: "Space",
                description: "Play / Pause",
                long_description: "Start or pause mission playback",
            },
            Shortcut {
                category: ShortcutCategory::Playback,
                key: "↑",
                description: "Increase speed",
                long_description: "Increase playback speed (1.0x → 1.5x → 2.0x → 4.0x)",
            },
            Shortcut {
                category: ShortcutCategory::Playback,
                key: "↓",
                description: "Decrease speed",
                long_description: "Decrease playback speed (1.0x → 0.5x → 0.25x)",
            },
            Shortcut {
                category: ShortcutCategory::Playback,
                key: "R",
                description: "Reset speed",
                long_description: "Reset playback speed to 1.0x",
            },
            Shortcut {
                category: ShortcutCategory::Playback,
                key: "S",
                description: "Step single frame",
                long_description: "Advance one frame (while paused)",
            },

            // Sensors
            Shortcut {
                category: ShortcutCategory::Sensors,
                key: "L",
                description: "Toggle Lidar",
                long_description: "Show/hide Lidar visualization",
            },
            Shortcut {
                category: ShortcutCategory::Sensors,
                key: "C",
                description: "Toggle Camera",
                long_description: "Show/hide Camera frames",
            },
            Shortcut {
                category: ShortcutCategory::Sensors,
                key: "I",
                description: "Toggle IMU",
                long_description: "Show/hide IMU graphs",
            },
            Shortcut {
                category: ShortcutCategory::Sensors,
                key: "O",
                description: "Toggle Odometry",
                long_description: "Show/hide Odometry data",
            },
            Shortcut {
                category: ShortcutCategory::Sensors,
                key: "M",
                description: "Toggle Metadata Panel",
                long_description: "Show/hide sensor metadata and statistics",
            },

            // Analysis
            Shortcut {
                category: ShortcutCategory::Analysis,
                key: "A",
                description: "Show Analysis Panel",
                long_description: "Display event analysis and statistics",
            },
            Shortcut {
                category: ShortcutCategory::Analysis,
                key: "D",
                description: "Detect Anomalies",
                long_description: "Scan mission for anomalies and flag them",
            },
            Shortcut {
                category: ShortcutCategory::Analysis,
                key: "G",
                description: "Show Event Graph",
                long_description: "Display event dependency/causal graph",
            },
            Shortcut {
                category: ShortcutCategory::Analysis,
                key: "K",
                description: "Mark Keyframe",
                long_description: "Mark current event as important for analysis",
            },

            // Export
            Shortcut {
                category: ShortcutCategory::Export,
                key: "E",
                description: "Export Current Frame",
                long_description: "Export current event/frame to file",
            },
            Shortcut {
                category: ShortcutCategory::Export,
                key: "Ctrl+E",
                description: "Export Range",
                long_description: "Export event range to file",
            },
            Shortcut {
                category: ShortcutCategory::Export,
                key: "J",
                description: "Toggle JSON Output",
                long_description: "Switch between human-readable and JSON output",
            },

            // Display
            Shortcut {
                category: ShortcutCategory::Display,
                key: "T",
                description: "Toggle Timestamps",
                long_description: "Show/hide timestamps on events",
            },
            Shortcut {
                category: ShortcutCategory::Display,
                key: "V",
                description: "Cycle View Mode",
                long_description: "Cycle through different display layouts",
            },
            Shortcut {
                category: ShortcutCategory::Display,
                key: "Z",
                description: "Zoom Timeline",
                long_description: "Zoom in/out of timeline view",
            },
            Shortcut {
                category: ShortcutCategory::Display,
                key: "+/-",
                description: "Adjust Panel Size",
                long_description: "Increase or decrease visualization panel size",
            },

            // Help
            Shortcut {
                category: ShortcutCategory::Help,
                key: "?",
                description: "Show Help",
                long_description: "Display this keyboard shortcuts help",
            },
            Shortcut {
                category: ShortcutCategory::Help,
                key: "H",
                description: "Toggle Help (alt)",
                long_description: "Alternative key to show/hide help",
            },
            Shortcut {
                category: ShortcutCategory::Help,
                key: "F1",
                description: "Context Help",
                long_description: "Show help for current view/mode",
            },
            Shortcut {
                category: ShortcutCategory::Help,
                key: "Q / Esc",
                description: "Quit",
                long_description: "Exit the replay application",
            },
        ]
    }

    /// Get shortcuts by category
    pub fn by_category(category: ShortcutCategory) -> Vec<Shortcut> {
        Self::all()
            .into_iter()
            .filter(|s| s.category == category)
            .collect()
    }

    /// Get all categories
    pub fn categories() -> Vec<ShortcutCategory> {
        vec![
            ShortcutCategory::Navigation,
            ShortcutCategory::Playback,
            ShortcutCategory::Sensors,
            ShortcutCategory::Analysis,
            ShortcutCategory::Export,
            ShortcutCategory::Display,
            ShortcutCategory::Help,
        ]
    }

    /// Find shortcut by key
    pub fn find(key: &str) -> Option<Shortcut> {
        Self::all().into_iter().find(|s| s.key == key)
    }
}

/// Help panel renderer
pub struct HelpPanel;

impl HelpPanel {
    /// Render full help with all shortcuts
    pub fn render_full() -> String {
        let mut output = String::new();

        output.push_str("╔════════════════════════════════════════════════════════════════╗\n");
        output.push_str("║           PyRoboReplay Keyboard Shortcuts Reference            ║\n");
        output.push_str("╚════════════════════════════════════════════════════════════════╝\n\n");

        for category in KeyboardShortcuts::categories() {
            output.push_str(&Self::render_category(category));
            output.push('\n');
        }

        output.push_str("╔════════════════════════════════════════════════════════════════╗\n");
        output.push_str("║ Tips: Mouse clicks work on timeline • Drag to scrub • Scroll   ║\n");
        output.push_str("║       to zoom • Press '?' at any time for quick reference      ║\n");
        output.push_str("╚════════════════════════════════════════════════════════════════╝\n");

        output
    }

    /// Render shortcuts for a specific category
    pub fn render_category(category: ShortcutCategory) -> String {
        let mut output = String::new();

        output.push_str(&format!("┌─ {} ", category));
        for _ in 0..(62 - category.to_string().len()) {
            output.push('─');
        }
        output.push_str("┐\n");

        let shortcuts = KeyboardShortcuts::by_category(category);
        for shortcut in shortcuts {
            output.push_str(&format!(
                "│ {:<15} • {:<45} │\n",
                shortcut.key, shortcut.description
            ));
        }

        output.push_str("└──────────────────────────────────────────────────────────────┘\n");

        output
    }

    /// Render quick reference (compact)
    pub fn render_quick() -> String {
        let mut output = String::new();

        output.push_str("Quick Reference\n");
        output.push_str("═══════════════════════════════════════════════════════════════\n");
        output.push_str("Navigation:  Home/End │ ← / → │ PgUp/PgDn\n");
        output.push_str("Playback:    Space   │ ↑ / ↓ │ R (reset)\n");
        output.push_str("Sensors:     L (lidar) │ C (camera) │ I (IMU) │ M (metadata)\n");
        output.push_str("Analysis:    A (panel) │ D (detect) │ G (graph) │ K (mark)\n");
        output.push_str("Export:      E (frame) │ Ctrl+E (range) │ J (JSON)\n");
        output.push_str("Display:     T (time) │ V (view) │ Z (zoom)\n");
        output.push_str("Help:        ? (full) │ F1 (context)\n");
        output.push_str("═══════════════════════════════════════════════════════════════\n");

        output
    }

    /// Render context-specific help
    pub fn render_context(context: &str) -> String {
        let mut output = String::new();

        output.push_str(&format!("Help: {}\n", context));
        output.push_str("═══════════════════════════════════════════════════════════════\n");

        match context {
            "timeline" => {
                output.push_str("Timeline Navigation:\n");
                output.push_str("  Home/End    : Jump to mission start/end\n");
                output.push_str("  ← / →       : Step one event at a time\n");
                output.push_str("  PgUp/PgDn   : Jump 10 events quickly\n");
                output.push_str("  Click/Drag  : Seek to position or scrub\n");
            }
            "playback" => {
                output.push_str("Playback Control:\n");
                output.push_str("  Space       : Play/Pause mission\n");
                output.push_str("  ↑ / ↓       : Increase/Decrease speed\n");
                output.push_str("  R           : Reset to 1.0x speed\n");
                output.push_str("  Available: 0.25x, 0.5x, 1.0x, 1.5x, 2.0x, 4.0x\n");
            }
            "sensors" => {
                output.push_str("Sensor Toggle:\n");
                output.push_str("  L           : Lidar visualization\n");
                output.push_str("  C           : Camera frames\n");
                output.push_str("  I           : IMU graphs\n");
                output.push_str("  O           : Odometry data\n");
                output.push_str("  M           : Sensor metadata panel\n");
            }
            "analysis" => {
                output.push_str("Analysis Tools:\n");
                output.push_str("  A           : Show analysis panel\n");
                output.push_str("  D           : Detect anomalies in data\n");
                output.push_str("  G           : Display event dependency graph\n");
                output.push_str("  K           : Mark keyframe for review\n");
            }
            _ => {
                output.push_str("Press '?' to see all keyboard shortcuts\n");
            }
        }

        output
    }

    /// Get tip of the day
    pub fn random_tip() -> &'static str {
        const TIPS: &[&str] = &[
            "💡 Press '?' at any time to see keyboard shortcuts",
            "💡 Use ↑/↓ to adjust playback speed without pausing",
            "💡 Press Home/End to jump to mission start/end",
            "💡 Click the timeline to scrub to any position",
            "💡 Press 'M' to see sensor metadata and quality indicators",
            "💡 Press 'D' to detect anomalies in the mission data",
            "💡 Use 'K' to mark keyframes for analysis",
            "💡 Press 'J' to toggle JSON output for AI agents",
            "💡 Try 'V' to cycle through different view layouts",
            "💡 Use Ctrl+E to export a range of events",
            "💡 Press 'Z' to zoom in/out of the timeline",
            "💡 Use 'L', 'C', 'I', 'O' to toggle individual sensors",
            "💡 Press 'A' to open the analysis panel",
            "💡 Use PgUp/PgDn to jump 10 events quickly",
            "💡 Try clicking and dragging on the timeline to scrub",
        ];

        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hash, Hasher};

        let mut hasher = RandomState::new().build_hasher();
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .hash(&mut hasher);

        let index = (hasher.finish() as usize) % TIPS.len();
        TIPS[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shortcuts_count() {
        let all = KeyboardShortcuts::all();
        assert!(all.len() > 20);
    }

    #[test]
    fn test_category_count() {
        let categories = KeyboardShortcuts::categories();
        assert_eq!(categories.len(), 7);
    }

    #[test]
    fn test_shortcuts_by_category() {
        let nav = KeyboardShortcuts::by_category(ShortcutCategory::Navigation);
        assert!(!nav.is_empty());
    }

    #[test]
    fn test_find_shortcut() {
        let shortcut = KeyboardShortcuts::find("Space");
        assert!(shortcut.is_some());
        assert_eq!(shortcut.unwrap().description, "Play / Pause");
    }

    #[test]
    fn test_help_panel_full() {
        let help = HelpPanel::render_full();
        assert!(help.contains("Keyboard Shortcuts"));
        assert!(help.contains("Navigation"));
        assert!(help.contains("Playback"));
    }

    #[test]
    fn test_help_panel_quick() {
        let quick = HelpPanel::render_quick();
        assert!(quick.contains("Quick Reference"));
        assert!(quick.contains("Space"));
    }

    #[test]
    fn test_help_context() {
        let context = HelpPanel::render_context("playback");
        assert!(context.contains("Playback Control"));
        assert!(context.contains("1.0x"));
    }

    #[test]
    fn test_random_tip() {
        let tip = HelpPanel::random_tip();
        assert!(tip.contains("💡"));
    }
}
