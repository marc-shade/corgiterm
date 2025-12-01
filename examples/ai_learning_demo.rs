//! AI Command History Learning Demo
//!
//! This example demonstrates how the AI learning system works:
//! - Tracking command patterns
//! - Detecting user preferences
//! - Providing context-aware suggestions

use corgiterm_ai::learning::{
    CommandPatternInfo, CommandPreference, FrequentCommand, LearningContext,
};
use corgiterm_core::history::CommandEntry;
use corgiterm_core::HistoryLearningManager;
use std::path::PathBuf;

fn main() {
    println!("🐕 CorgiTerm AI Learning Demo\n");

    // Create a history manager with learning enabled
    let mut manager = HistoryLearningManager::new(
        10000, // max history
        100,   // learning window size
        true,  // learning enabled
    );

    // Simulate a typical development session
    println!("📝 Simulating command history...\n");

    let project_dir = PathBuf::from("/home/user/my-project");

    // Simulate Git workflow pattern
    simulate_commands(
        &mut manager,
        &project_dir,
        vec![
            "git status",
            "git add .",
            "git commit -m 'Update'",
            "git push",
        ],
    );

    // Simulate Rust development pattern
    simulate_commands(
        &mut manager,
        &project_dir,
        vec!["cargo build", "cargo test", "cargo run"],
    );

    // Simulate user prefers modern alternatives
    for _ in 0..5 {
        add_command(&mut manager, "exa -la", &project_dir);
    }
    for _ in 0..2 {
        add_command(&mut manager, "ls -la", &project_dir);
    }

    for _ in 0..4 {
        add_command(&mut manager, "bat README.md", &project_dir);
    }
    add_command(&mut manager, "cat README.md", &project_dir);

    // Add some directory-specific commands
    add_command(&mut manager, "docker-compose up", &project_dir);
    add_command(&mut manager, "npm install", &project_dir);
    add_command(&mut manager, "yarn dev", &project_dir);

    // Detect preferences
    println!("🔍 Detecting user preferences...");
    manager.detect_preferences();

    // Display learning results
    println!("\n📊 Learning Results:\n");

    let context = manager.get_learning_context();

    println!("Top 5 Frequent Commands:");
    for (i, cmd) in context.frequent_commands.iter().take(5).enumerate() {
        println!(
            "  {}. {} - {} executions ({}% success rate)",
            i + 1,
            cmd.command,
            cmd.count,
            (cmd.success_rate * 100.0) as u32
        );
    }

    println!("\n🎨 User Preferences Detected:");
    for pref in &context.preferences {
        println!(
            "  • Prefers '{}' over '{}' ({}% of the time)",
            pref.preferred,
            pref.standard,
            (pref.ratio * 100.0) as u32
        );
    }

    println!("\n🔗 Command Patterns Learned:");
    for (i, pattern) in context.patterns.iter().take(5).enumerate() {
        println!(
            "  {}. {} → (seen {} times, confidence: {:.1}%)",
            i + 1,
            pattern.sequence.join(" → "),
            pattern.frequency,
            pattern.confidence * 100.0
        );
    }

    // Demonstrate AI integration
    println!("\n🤖 AI Integration Demo:\n");

    // Create learning context for AI
    let ai_context = LearningContext {
        frequent_commands: context
            .frequent_commands
            .iter()
            .map(|c| FrequentCommand {
                command: c.command.clone(),
                count: c.count,
                success_rate: c.success_rate,
            })
            .collect(),
        preferences: context
            .preferences
            .iter()
            .map(|p| CommandPreference {
                standard: p.standard.clone(),
                preferred: p.preferred.clone(),
                ratio: p.ratio,
            })
            .collect(),
        patterns: context
            .patterns
            .iter()
            .map(|p| CommandPatternInfo {
                sequence: p.sequence.clone(),
                frequency: p.frequency,
                confidence: p.confidence,
            })
            .collect(),
        directory_commands: std::collections::HashMap::new(),
    };

    println!("AI now has context about:");
    println!(
        "  • {} frequent commands",
        ai_context.frequent_commands.len()
    );
    println!("  • {} user preferences", ai_context.preferences.len());
    println!("  • {} learned patterns", ai_context.patterns.len());

    // Predict next command
    println!("\n🔮 Pattern Prediction:");
    if let Some(suggestion) = manager.learning().predict_next_command("git add") {
        println!(
            "  After 'git add', you usually run: '{}' ({}% confidence)",
            suggestion.command,
            (suggestion.confidence * 100.0) as u32
        );
    }

    // Directory-specific suggestions
    println!("\n📁 Directory-Specific Suggestions:");
    let dir_cmds = manager.learning().directory_commands(&project_dir, 5);
    for (i, stat) in dir_cmds.iter().enumerate() {
        println!(
            "  {}. {} (used {} times in this directory)",
            i + 1,
            stat.command,
            stat.directories.len()
        );
    }

    println!("\n✅ Demo complete!");
    println!("\n💡 This data would be used to:");
    println!("  • Suggest commands as you type");
    println!("  • Predict next command in sequences");
    println!("  • Provide directory-specific completions");
    println!("  • Customize AI suggestions to your preferences");
}

fn simulate_commands(manager: &mut HistoryLearningManager, dir: &PathBuf, commands: Vec<&str>) {
    for cmd in commands {
        add_command(manager, cmd, dir);
    }
}

fn add_command(manager: &mut HistoryLearningManager, cmd: &str, dir: &PathBuf) {
    let mut entry = CommandEntry::new(cmd, dir.clone());
    entry.complete(0, 100); // Simulate successful execution
    manager.add_command(entry);
}
