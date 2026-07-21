/// Demonstration of keyboard shortcuts and help system
/// Shows all available shortcuts and help panels

use pyroboreplay::cli::keyboard::{KeyboardShortcuts, HelpPanel, ShortcutCategory};

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║     PyRoboReplay Keyboard Shortcuts & Help System Demo         ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Show full help
    println!("{}", HelpPanel::render_full());

    // Show quick reference
    println!("\n");
    println!("{}", HelpPanel::render_quick());

    // Show category breakdowns
    println!("\n");
    println!("📚 Detailed Shortcut Breakdown by Category\n");

    for category in KeyboardShortcuts::categories() {
        println!("{}", HelpPanel::render_category(category));
    }

    // Show context-specific help
    println!("\n");
    println!("🎯 Context-Sensitive Help Examples\n");

    for context in &["timeline", "playback", "sensors", "analysis"] {
        println!("{}", HelpPanel::render_context(context));
        println!();
    }

    // Show random tip
    println!("{}", HelpPanel::random_tip());

    // Statistics
    println!("\n\n");
    println!("📊 Keyboard Shortcuts Statistics");
    println!("════════════════════════════════════════════════════════════════");
    let all_shortcuts = KeyboardShortcuts::all();
    println!("Total shortcuts: {}", all_shortcuts.len());

    for category in KeyboardShortcuts::categories() {
        let count = KeyboardShortcuts::by_category(category).len();
        println!("  {} : {} shortcuts", category, count);
    }

    // Find specific shortcuts
    println!("\n");
    println!("🔍 Shortcut Lookup Examples");
    println!("════════════════════════════════════════════════════════════════");

    for key in &["Space", "?", "L", "M", "Home", "Ctrl+E"] {
        if let Some(shortcut) = KeyboardShortcuts::find(key) {
            println!(
                "  {} → {} ({})",
                key, shortcut.description, shortcut.category
            );
        } else {
            println!("  {} → Not found", key);
        }
    }

    // Usage tips
    println!("\n");
    println!("💡 Usage Tips");
    println!("════════════════════════════════════════════════════════════════");
    println!("1. During replay, press '?' to show full keyboard reference");
    println!("2. Press 'F1' for context-sensitive help");
    println!("3. Use Home/End to jump mission start/end");
    println!("4. Press Space to play/pause");
    println!("5. Use ↑/↓ to adjust playback speed");
    println!("6. Press 'M' to see sensor metadata");
    println!("7. Press 'L', 'C', 'I', 'O' to toggle sensors");
    println!("8. Press 'D' to detect anomalies");
    println!("9. Use 'K' to mark important events");
    println!("10. Press 'Q' or Esc to quit");

    // Advanced features
    println!("\n");
    println!("⚡ Advanced Features");
    println!("════════════════════════════════════════════════════════════════");
    println!("• Mouse clicks work on timeline for seeking");
    println!("• Drag timeline to scrub through mission");
    println!("• Scroll to zoom timeline in/out");
    println!("• Custom keybindings (configure in settings)");
    println!("• Keyboard macros for complex workflows");
    println!("• Accessibility: all functions available via keyboard");
}
