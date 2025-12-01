//! Command history storage and pattern extraction
//!
//! Tracks command history, analyzes patterns, and provides learning context

use crate::learning::{CommandPatternInfo, CommandPreference, FrequentCommand, LearningContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Command history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The command that was executed
    pub command: String,
    /// Working directory when executed
    pub directory: String,
    /// Unix timestamp
    pub timestamp: u64,
    /// Exit code (0 = success)
    pub exit_code: Option<i32>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
}

/// Persistent command history store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHistoryStore {
    /// All history entries
    entries: Vec<HistoryEntry>,
    /// Max entries to keep
    #[serde(default = "default_max_entries")]
    max_entries: usize,
}

fn default_max_entries() -> usize {
    10000
}

impl Default for CommandHistoryStore {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: default_max_entries(),
        }
    }
}

impl CommandHistoryStore {
    /// Create a new empty store
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from config directory
    pub fn load() -> Self {
        let path = Self::storage_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(store) => return store,
                    Err(e) => tracing::warn!("Failed to parse history: {}", e),
                },
                Err(e) => tracing::warn!("Failed to read history: {}", e),
            }
        }
        Self::default()
    }

    /// Save to config directory
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::storage_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
    }

    /// Get storage path
    fn storage_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("corgiterm")
            .join("command_history.json")
    }

    /// Record a command execution
    pub fn record(
        &mut self,
        command: String,
        directory: String,
        exit_code: Option<i32>,
        duration_ms: Option<u64>,
    ) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.entries.push(HistoryEntry {
            command,
            directory,
            timestamp,
            exit_code,
            duration_ms,
        });

        // Trim to max entries
        if self.entries.len() > self.max_entries {
            let excess = self.entries.len() - self.max_entries;
            self.entries.drain(0..excess);
        }
    }

    /// Get recent commands
    pub fn recent(&self, limit: usize) -> Vec<&HistoryEntry> {
        self.entries.iter().rev().take(limit).collect()
    }

    /// Get commands for a specific directory
    pub fn for_directory(&self, directory: &str, limit: usize) -> Vec<&HistoryEntry> {
        self.entries
            .iter()
            .rev()
            .filter(|e| e.directory == directory)
            .take(limit)
            .collect()
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Extract learning context from history
    pub fn extract_learning_context(&self) -> LearningContext {
        let mut context = LearningContext::default();

        // Extract frequent commands
        context.frequent_commands = self.extract_frequent_commands(20);

        // Extract preferences
        context.preferences = self.extract_preferences();

        // Extract patterns
        context.patterns = self.extract_patterns(10);

        // Extract directory-specific commands
        context.directory_commands = self.extract_directory_commands();

        context
    }

    /// Extract most frequent commands
    fn extract_frequent_commands(&self, limit: usize) -> Vec<FrequentCommand> {
        let mut counts: HashMap<String, (usize, usize, usize)> = HashMap::new(); // (total, success, fail)

        for entry in &self.entries {
            // Normalize command (just the base command)
            let base_cmd = entry
                .command
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_string();
            if base_cmd.is_empty() {
                continue;
            }

            let counter = counts.entry(base_cmd).or_insert((0, 0, 0));
            counter.0 += 1;
            match entry.exit_code {
                Some(0) => counter.1 += 1,
                Some(_) => counter.2 += 1,
                None => {} // Unknown
            }
        }

        let mut freq_cmds: Vec<_> = counts
            .into_iter()
            .map(|(cmd, (total, success, _fail))| FrequentCommand {
                command: cmd,
                count: total,
                success_rate: if total > 0 {
                    success as f32 / total as f32
                } else {
                    0.0
                },
            })
            .collect();

        freq_cmds.sort_by(|a, b| b.count.cmp(&a.count));
        freq_cmds.truncate(limit);
        freq_cmds
    }

    /// Extract command preferences (e.g., eza over ls, bat over cat)
    fn extract_preferences(&self) -> Vec<CommandPreference> {
        let alternatives = [
            ("ls", &["eza", "exa", "lsd"][..]),
            ("cat", &["bat", "batcat"][..]),
            ("find", &["fd", "fdfind"][..]),
            ("grep", &["rg", "ripgrep"][..]),
            ("du", &["dust", "dua"][..]),
            ("diff", &["delta", "difft"][..]),
            ("ps", &["procs"][..]),
            ("top", &["htop", "btop", "bottom"][..]),
        ];

        let mut preferences = Vec::new();
        let mut cmd_counts: HashMap<&str, usize> = HashMap::new();

        for entry in &self.entries {
            let base_cmd = entry.command.split_whitespace().next().unwrap_or("");
            *cmd_counts.entry(base_cmd).or_insert(0) += 1;
        }

        for (standard, alts) in alternatives {
            let std_count = *cmd_counts.get(standard).unwrap_or(&0);

            for alt in alts {
                let alt_count = *cmd_counts.get(alt).unwrap_or(&0);
                let total = std_count + alt_count;

                if alt_count > 0 && total > 5 {
                    let ratio = alt_count as f32 / total as f32;
                    if ratio > 0.3 {
                        // User uses the alternative at least 30% of the time
                        preferences.push(CommandPreference {
                            standard: standard.to_string(),
                            preferred: alt.to_string(),
                            ratio,
                        });
                        break; // Only one alternative per standard
                    }
                }
            }
        }

        preferences
    }

    /// Extract command patterns (sequences that often occur together)
    fn extract_patterns(&self, limit: usize) -> Vec<CommandPatternInfo> {
        let mut sequences: HashMap<(String, String), usize> = HashMap::new();

        // Look at consecutive commands in the same directory
        for window in self.entries.windows(2) {
            if window[0].directory == window[1].directory {
                // Check time proximity (within 5 minutes)
                if window[1].timestamp.saturating_sub(window[0].timestamp) < 300 {
                    let cmd1 = window[0]
                        .command
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();
                    let cmd2 = window[1]
                        .command
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string();

                    if !cmd1.is_empty() && !cmd2.is_empty() && cmd1 != cmd2 {
                        *sequences.entry((cmd1, cmd2)).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut patterns: Vec<_> = sequences
            .into_iter()
            .filter(|(_, count)| *count >= 3) // At least 3 occurrences
            .map(|((cmd1, cmd2), freq)| CommandPatternInfo {
                sequence: vec![cmd1, cmd2],
                frequency: freq,
                confidence: (freq as f32 / self.entries.len().max(1) as f32 * 10.0).min(0.95),
            })
            .collect();

        patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));
        patterns.truncate(limit);
        patterns
    }

    /// Extract directory-specific frequent commands
    fn extract_directory_commands(&self) -> HashMap<String, Vec<String>> {
        let mut dir_cmds: HashMap<String, HashMap<String, usize>> = HashMap::new();

        for entry in &self.entries {
            let cmd = entry.command.clone();
            *dir_cmds
                .entry(entry.directory.clone())
                .or_default()
                .entry(cmd)
                .or_insert(0) += 1;
        }

        dir_cmds
            .into_iter()
            .map(|(dir, cmds)| {
                let mut sorted: Vec<_> = cmds.into_iter().collect();
                sorted.sort_by(|a, b| b.1.cmp(&a.1));
                let top_cmds: Vec<String> =
                    sorted.into_iter().take(10).map(|(cmd, _)| cmd).collect();
                (dir, top_cmds)
            })
            .filter(|(_, cmds)| !cmds.is_empty())
            .collect()
    }

    /// Search history
    pub fn search(&self, query: &str, limit: usize) -> Vec<&HistoryEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .rev()
            .filter(|e| e.command.to_lowercase().contains(&query_lower))
            .take(limit)
            .collect()
    }

    /// Get statistics
    pub fn stats(&self) -> HistoryStats {
        let total = self.entries.len();
        let successful = self
            .entries
            .iter()
            .filter(|e| e.exit_code == Some(0))
            .count();
        let failed = self
            .entries
            .iter()
            .filter(|e| matches!(e.exit_code, Some(c) if c != 0))
            .count();

        let unique_commands: std::collections::HashSet<_> = self
            .entries
            .iter()
            .filter_map(|e| e.command.split_whitespace().next())
            .collect();

        let unique_dirs: std::collections::HashSet<_> =
            self.entries.iter().map(|e| &e.directory).collect();

        HistoryStats {
            total_commands: total,
            successful_commands: successful,
            failed_commands: failed,
            unique_commands: unique_commands.len(),
            unique_directories: unique_dirs.len(),
        }
    }
}

/// History statistics
#[derive(Debug, Clone)]
pub struct HistoryStats {
    pub total_commands: usize,
    pub successful_commands: usize,
    pub failed_commands: usize,
    pub unique_commands: usize,
    pub unique_directories: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_command() {
        let mut store = CommandHistoryStore::new();
        store.record(
            "ls -la".to_string(),
            "/home/user".to_string(),
            Some(0),
            Some(100),
        );
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_frequent_commands() {
        let mut store = CommandHistoryStore::new();
        for _ in 0..10 {
            store.record(
                "git status".to_string(),
                "/home/user".to_string(),
                Some(0),
                None,
            );
        }
        for _ in 0..5 {
            store.record("ls".to_string(), "/home/user".to_string(), Some(0), None);
        }

        let context = store.extract_learning_context();
        assert!(!context.frequent_commands.is_empty());
        assert_eq!(context.frequent_commands[0].command, "git");
    }

    #[test]
    fn test_patterns() {
        let mut store = CommandHistoryStore::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Simulate cargo build -> cargo test pattern (different base commands)
        for i in 0..5 {
            store.entries.push(HistoryEntry {
                command: "make build".to_string(),
                directory: "/project".to_string(),
                timestamp: now + i * 60,
                exit_code: Some(0),
                duration_ms: None,
            });
            store.entries.push(HistoryEntry {
                command: "pytest tests/".to_string(),
                directory: "/project".to_string(),
                timestamp: now + i * 60 + 30,
                exit_code: Some(0),
                duration_ms: None,
            });
        }

        let context = store.extract_learning_context();
        assert!(!context.patterns.is_empty());
        // Should detect make -> pytest pattern
    }

    #[test]
    fn test_preferences() {
        let mut store = CommandHistoryStore::new();
        // User uses eza more than ls
        for _ in 0..10 {
            store.record("eza".to_string(), "/home".to_string(), Some(0), None);
        }
        for _ in 0..2 {
            store.record("ls".to_string(), "/home".to_string(), Some(0), None);
        }

        let context = store.extract_learning_context();
        let has_eza_pref = context.preferences.iter().any(|p| p.preferred == "eza");
        assert!(has_eza_pref);
    }
}
