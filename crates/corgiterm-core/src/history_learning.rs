//! Integration between command history and learning
//!
//! This module bridges the CommandHistory and CommandLearning systems,
//! automatically feeding history into the learning engine.

use crate::history::{CommandEntry, CommandHistory};
use crate::learning::CommandLearning;
use std::path::PathBuf;

/// Combined history and learning manager
pub struct HistoryLearningManager {
    history: CommandHistory,
    learning: CommandLearning,
    learning_enabled: bool,
    data_path: Option<PathBuf>,
}

impl HistoryLearningManager {
    /// Create new manager
    pub fn new(max_history: usize, window_size: usize, learning_enabled: bool) -> Self {
        Self {
            history: CommandHistory::new(max_history),
            learning: CommandLearning::new(window_size),
            learning_enabled,
            data_path: None,
        }
    }

    /// Set persistence paths
    pub fn with_persistence(mut self, history_path: PathBuf, learning_path: PathBuf) -> Self {
        self.history = self.history.with_persistence(history_path);
        self.data_path = Some(learning_path);
        self
    }

    /// Add a command to both history and learning
    pub fn add_command(&mut self, entry: CommandEntry) {
        // Add to history
        self.history.push(entry.clone());

        // Add to learning if enabled
        if self.learning_enabled {
            self.learning.add_command(entry);
        }
    }

    /// Complete a command (update exit code and duration)
    pub fn complete_command(&mut self, command: &str, exit_code: i32, duration_ms: u64) {
        // Find the most recent matching command in history
        if let Some(recent) = self.history.recent(10).into_iter().find(|e| e.command == command) {
            let mut updated = recent.clone();
            updated.complete(exit_code, duration_ms);

            // Re-add to learning with completion data
            if self.learning_enabled {
                self.learning.add_command(updated);
            }
        }
    }

    /// Get command history
    pub fn history(&self) -> &CommandHistory {
        &self.history
    }

    /// Get learning engine
    pub fn learning(&self) -> &CommandLearning {
        &self.learning
    }

    /// Get mutable learning engine
    pub fn learning_mut(&mut self) -> &mut CommandLearning {
        &mut self.learning
    }

    /// Enable/disable learning
    pub fn set_learning_enabled(&mut self, enabled: bool) {
        self.learning_enabled = enabled;
    }

    /// Is learning enabled?
    pub fn is_learning_enabled(&self) -> bool {
        self.learning_enabled
    }

    /// Save both history and learning data
    pub fn save(&self) -> std::io::Result<()> {
        // Save history
        self.history.save()?;

        // Save learning data
        if let Some(ref path) = self.data_path {
            self.learning.save(path)?;
        }

        Ok(())
    }

    /// Load both history and learning data
    pub fn load(&mut self) -> std::io::Result<()> {
        // Load history
        self.history.load()?;

        // Load learning data
        if let Some(ref path) = self.data_path {
            self.learning.load(path)?;
        }

        Ok(())
    }

    /// Run preference detection
    pub fn detect_preferences(&mut self) {
        if self.learning_enabled {
            self.learning.detect_preferences();
        }
    }

    /// Get statistics for AI context
    pub fn get_learning_context(&self) -> LearningContextData {
        let frequent = self.learning.frequent_commands(20);
        let patterns = self.learning.patterns();
        let preferences = self.learning.preferences();

        LearningContextData {
            frequent_commands: frequent.iter().map(|s| FrequentCommandData {
                command: s.command.clone(),
                count: s.total_count,
                success_rate: s.success_rate(),
                avg_duration_ms: s.avg_duration_ms,
            }).collect(),
            patterns: patterns.iter().map(|p| PatternData {
                sequence: p.sequence.clone(),
                frequency: p.frequency,
                confidence: p.confidence,
            }).collect(),
            preferences: preferences.iter().map(|p| PreferenceData {
                standard: p.standard.clone(),
                preferred: p.preferred.clone(),
                ratio: p.preference_ratio,
            }).collect(),
        }
    }

    /// Clear all learning data (privacy)
    pub fn clear_learning(&mut self) {
        self.learning = CommandLearning::new(100); // Reset with default window
        if let Some(ref path) = self.data_path {
            let _ = std::fs::remove_file(path); // Ignore errors
        }
    }
}

/// Simplified learning context data for serialization/transfer
#[derive(Debug, Clone)]
pub struct LearningContextData {
    pub frequent_commands: Vec<FrequentCommandData>,
    pub patterns: Vec<PatternData>,
    pub preferences: Vec<PreferenceData>,
}

#[derive(Debug, Clone)]
pub struct FrequentCommandData {
    pub command: String,
    pub count: usize,
    pub success_rate: f32,
    pub avg_duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct PatternData {
    pub sequence: Vec<String>,
    pub frequency: usize,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub struct PreferenceData {
    pub standard: String,
    pub preferred: String,
    pub ratio: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = HistoryLearningManager::new(1000, 100, true);
        assert!(manager.is_learning_enabled());
    }

    #[test]
    fn test_add_command() {
        let mut manager = HistoryLearningManager::new(1000, 100, true);
        let entry = CommandEntry::new("ls -la", PathBuf::from("/home"));
        manager.add_command(entry);

        assert_eq!(manager.history().recent(10).len(), 1);
    }

    #[test]
    fn test_disable_learning() {
        let mut manager = HistoryLearningManager::new(1000, 100, true);
        manager.set_learning_enabled(false);
        assert!(!manager.is_learning_enabled());
    }
}
