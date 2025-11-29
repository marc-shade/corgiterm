//! Command pattern learning and analysis
//!
//! This module learns from user command patterns to provide:
//! - Frequently used commands
//! - Command sequences (patterns)
//! - Directory-specific commands
//! - Success/failure tracking
//! - User preferences (alternative commands)

use crate::history::CommandEntry;
use chrono::{DateTime, Utc, Timelike};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

/// A sequence of commands that often occur together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPattern {
    /// The sequence of commands
    pub sequence: Vec<String>,
    /// How often this pattern has occurred
    pub frequency: usize,
    /// Contexts where this pattern appears
    pub contexts: Vec<PatternContext>,
    /// Average time between commands in sequence
    pub avg_interval_ms: u64,
    /// Last time this pattern was seen
    pub last_seen: DateTime<Utc>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

/// Context for a command pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternContext {
    /// Working directory
    pub directory: PathBuf,
    /// Project identifier (if known)
    pub project_id: Option<String>,
    /// Time of day category
    pub time_category: TimeCategory,
}

/// Time of day categories for pattern analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeCategory {
    Morning,   // 6am-12pm
    Afternoon, // 12pm-6pm
    Evening,   // 6pm-12am
    Night,     // 12am-6am
}

impl TimeCategory {
    pub fn from_datetime(dt: &DateTime<Utc>) -> Self {
        let hour = dt.time().hour();
        match hour {
            6..=11 => Self::Morning,
            12..=17 => Self::Afternoon,
            18..=23 => Self::Evening,
            _ => Self::Night,
        }
    }
}

/// User preference for alternative commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreference {
    /// The standard command
    pub standard: String,
    /// User's preferred alternative
    pub preferred: String,
    /// How often user chooses preferred over standard
    pub preference_ratio: f32,
    /// Common flags/options used with this command
    pub common_flags: Vec<String>,
}

/// Statistics for a single command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandStats {
    /// The command (without arguments)
    pub command: String,
    /// Total executions
    pub total_count: usize,
    /// Successful executions (exit code 0)
    pub success_count: usize,
    /// Failed executions
    pub failure_count: usize,
    /// Average duration in milliseconds
    pub avg_duration_ms: u64,
    /// Directories where this command is used
    pub directories: Vec<PathBuf>,
    /// Most recent execution
    pub last_used: DateTime<Utc>,
    /// Frequency score (based on recency and count)
    pub frequency_score: f32,
}

impl CommandStats {
    pub fn success_rate(&self) -> f32 {
        if self.total_count == 0 {
            0.0
        } else {
            self.success_count as f32 / self.total_count as f32
        }
    }
}

/// Command learning engine
pub struct CommandLearning {
    /// Recent command window for pattern detection
    recent_commands: VecDeque<CommandEntry>,
    /// Window size for pattern detection
    window_size: usize,
    /// Detected patterns
    patterns: Vec<CommandPattern>,
    /// User preferences
    preferences: Vec<UserPreference>,
    /// Command statistics
    stats: HashMap<String, CommandStats>,
    /// Minimum pattern frequency for detection
    min_pattern_frequency: usize,
    /// Maximum pattern length
    max_pattern_length: usize,
}

impl CommandLearning {
    /// Create new learning engine
    pub fn new(window_size: usize) -> Self {
        Self {
            recent_commands: VecDeque::with_capacity(window_size),
            window_size,
            patterns: Vec::new(),
            preferences: Vec::new(),
            stats: HashMap::new(),
            min_pattern_frequency: 3,
            max_pattern_length: 5,
        }
    }

    /// Add a command to learning analysis
    pub fn add_command(&mut self, entry: CommandEntry) {
        // Update command statistics
        self.update_stats(&entry);

        // Add to recent window
        self.recent_commands.push_back(entry);
        if self.recent_commands.len() > self.window_size {
            self.recent_commands.pop_front();
        }

        // Detect patterns periodically
        if self.recent_commands.len() >= 10 {
            self.detect_patterns();
        }
    }

    /// Update statistics for a command
    fn update_stats(&mut self, entry: &CommandEntry) {
        let base_command = extract_base_command(&entry.command);

        let stats = self.stats.entry(base_command.clone()).or_insert_with(|| {
            CommandStats {
                command: base_command.clone(),
                total_count: 0,
                success_count: 0,
                failure_count: 0,
                avg_duration_ms: 0,
                directories: Vec::new(),
                last_used: entry.timestamp,
                frequency_score: 0.0,
            }
        });

        stats.total_count += 1;

        if let Some(success) = entry.success {
            if success {
                stats.success_count += 1;
            } else {
                stats.failure_count += 1;
            }
        }

        if let Some(duration) = entry.duration_ms {
            // Update running average
            let total_duration = stats.avg_duration_ms * (stats.total_count - 1) as u64 + duration;
            stats.avg_duration_ms = total_duration / stats.total_count as u64;
        }

        if !stats.directories.contains(&entry.cwd) {
            stats.directories.push(entry.cwd.clone());
        }

        stats.last_used = entry.timestamp;

        // Calculate frequency score separately to avoid borrow conflicts
        let total_count = stats.total_count;
        let last_used = stats.last_used;
        // Release the mutable borrow before accessing self.stats again

        // Now we can borrow self immutably
        let max_count = self.stats.values().map(|s| s.total_count).max().unwrap_or(1);
        let frequency_score = Self::calculate_frequency_score_static(total_count, last_used, max_count);

        // Update the frequency score
        if let Some(stats) = self.stats.get_mut(&base_command) {
            stats.frequency_score = frequency_score;
        }
    }

    /// Calculate frequency score based on count and recency
    fn calculate_frequency_score(&self, stats: &CommandStats) -> f32 {
        let max_count = self.stats.values().map(|s| s.total_count).max().unwrap_or(1);
        Self::calculate_frequency_score_static(stats.total_count, stats.last_used, max_count)
    }

    /// Static version of frequency score calculation
    fn calculate_frequency_score_static(total_count: usize, last_used: DateTime<Utc>, max_count: usize) -> f32 {
        let recency_weight = 0.6;
        let count_weight = 0.4;

        // Recency score (last 30 days)
        let days_since = (Utc::now() - last_used).num_days();
        let recency_score = if days_since < 30 {
            1.0 - (days_since as f32 / 30.0)
        } else {
            0.0
        };

        // Count score (normalized by max count)
        let count_score = total_count as f32 / max_count as f32;

        recency_weight * recency_score + count_weight * count_score
    }

    /// Detect command patterns in recent history
    fn detect_patterns(&mut self) {
        let commands: Vec<_> = self.recent_commands.iter().collect();

        for pattern_len in 2..=self.max_pattern_length.min(commands.len()) {
            for i in 0..=(commands.len() - pattern_len) {
                let sequence: Vec<String> = commands[i..i + pattern_len]
                    .iter()
                    .map(|c| extract_base_command(&c.command))
                    .collect();

                // Check if this pattern already exists
                if let Some(pattern) = self.patterns.iter_mut().find(|p| p.sequence == sequence) {
                    pattern.frequency += 1;
                    pattern.last_seen = Utc::now();
                    pattern.confidence = (pattern.frequency as f32 / self.recent_commands.len() as f32).min(1.0);
                } else if self.should_create_pattern(&sequence) {
                    // Create new pattern
                    let context = PatternContext {
                        directory: commands[i].cwd.clone(),
                        project_id: commands[i].project_id.clone(),
                        time_category: TimeCategory::from_datetime(&commands[i].timestamp),
                    };

                    let pattern = CommandPattern {
                        sequence,
                        frequency: 1,
                        contexts: vec![context],
                        avg_interval_ms: self.calculate_avg_interval(&commands[i..i + pattern_len]),
                        last_seen: Utc::now(),
                        confidence: 0.1,
                    };

                    self.patterns.push(pattern);
                }
            }
        }

        // Prune low-confidence patterns
        self.patterns.retain(|p| p.frequency >= self.min_pattern_frequency);
    }

    /// Check if a sequence should become a pattern
    fn should_create_pattern(&self, sequence: &[String]) -> bool {
        // Don't create patterns for identical consecutive commands
        if sequence.len() == 2 && sequence[0] == sequence[1] {
            return false;
        }

        // Don't create patterns for very common single commands
        let common_singles = ["ls", "cd", "pwd", "clear"];
        if sequence.len() == 1 && common_singles.contains(&sequence[0].as_str()) {
            return false;
        }

        true
    }

    /// Calculate average interval between commands
    fn calculate_avg_interval(&self, commands: &[&CommandEntry]) -> u64 {
        if commands.len() < 2 {
            return 0;
        }

        let mut total_ms = 0u64;
        for i in 1..commands.len() {
            let interval = (commands[i].timestamp - commands[i - 1].timestamp).num_milliseconds();
            total_ms += interval.max(0) as u64;
        }

        total_ms / (commands.len() - 1) as u64
    }

    /// Get most frequent commands
    pub fn frequent_commands(&self, limit: usize) -> Vec<&CommandStats> {
        let mut sorted: Vec<_> = self.stats.values().collect();
        sorted.sort_by(|a, b| b.frequency_score.partial_cmp(&a.frequency_score).unwrap());
        sorted.truncate(limit);
        sorted
    }

    /// Get commands used in a specific directory
    pub fn directory_commands(&self, dir: &PathBuf, limit: usize) -> Vec<&CommandStats> {
        let mut dir_stats: Vec<_> = self.stats
            .values()
            .filter(|s| s.directories.contains(dir))
            .collect();

        dir_stats.sort_by(|a, b| b.frequency_score.partial_cmp(&a.frequency_score).unwrap());
        dir_stats.truncate(limit);
        dir_stats
    }

    /// Get likely next command based on current command
    pub fn predict_next_command(&self, current: &str) -> Option<CommandSuggestion> {
        let base_current = extract_base_command(current);

        // Look for patterns starting with current command
        let matching_patterns: Vec<_> = self.patterns
            .iter()
            .filter(|p| p.sequence.first().map(|s| s == &base_current).unwrap_or(false))
            .collect();

        if let Some(pattern) = matching_patterns.iter().max_by_key(|p| p.frequency) {
            if let Some(next_cmd) = pattern.sequence.get(1) {
                return Some(CommandSuggestion {
                    command: next_cmd.clone(),
                    reason: format!("Often follows '{}'", base_current),
                    confidence: pattern.confidence,
                    source: SuggestionSource::Pattern,
                });
            }
        }

        None
    }

    /// Get command patterns
    pub fn patterns(&self) -> &[CommandPattern] {
        &self.patterns
    }

    /// Get command statistics
    pub fn stats(&self) -> &HashMap<String, CommandStats> {
        &self.stats
    }

    /// Detect user preferences (e.g., prefers `exa` over `ls`)
    pub fn detect_preferences(&mut self) {
        // Common command alternatives
        let alternatives = vec![
            ("ls", vec!["exa", "lsd", "eza"]),
            ("cat", vec!["bat"]),
            ("find", vec!["fd"]),
            ("grep", vec!["rg", "ag"]),
            ("du", vec!["dust"]),
            ("top", vec!["htop", "btop", "bottom"]),
        ];

        for (standard, alts) in alternatives {
            let standard_count = self.stats.get(standard).map(|s| s.total_count).unwrap_or(0);

            for alt in alts {
                let alt_count = self.stats.get(alt).map(|s| s.total_count).unwrap_or(0);

                if alt_count > 0 && alt_count > standard_count {
                    let total = standard_count + alt_count;
                    let preference_ratio = alt_count as f32 / total as f32;

                    // Update or create preference
                    if let Some(pref) = self.preferences.iter_mut().find(|p| p.standard == standard) {
                        pref.preferred = alt.to_string();
                        pref.preference_ratio = preference_ratio;
                    } else {
                        self.preferences.push(UserPreference {
                            standard: standard.to_string(),
                            preferred: alt.to_string(),
                            preference_ratio,
                            common_flags: Vec::new(),
                        });
                    }
                }
            }
        }
    }

    /// Get user preferences
    pub fn preferences(&self) -> &[UserPreference] {
        &self.preferences
    }

    /// Save learning data
    pub fn save(&self, path: &PathBuf) -> std::io::Result<()> {
        let data = LearningData {
            patterns: self.patterns.clone(),
            preferences: self.preferences.clone(),
            stats: self.stats.clone(),
        };

        let json = serde_json::to_string_pretty(&data)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load learning data
    pub fn load(&mut self, path: &PathBuf) -> std::io::Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let json = std::fs::read_to_string(path)?;
        let data: LearningData = serde_json::from_str(&json)?;

        self.patterns = data.patterns;
        self.preferences = data.preferences;
        self.stats = data.stats;

        Ok(())
    }
}

/// Serializable learning data
#[derive(Debug, Serialize, Deserialize)]
struct LearningData {
    patterns: Vec<CommandPattern>,
    preferences: Vec<UserPreference>,
    stats: HashMap<String, CommandStats>,
}

/// A suggested command
#[derive(Debug, Clone)]
pub struct CommandSuggestion {
    pub command: String,
    pub reason: String,
    pub confidence: f32,
    pub source: SuggestionSource,
}

/// Source of a command suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionSource {
    Pattern,    // From learned patterns
    Frequency,  // From frequent commands
    Directory,  // From directory-specific history
    Preference, // From user preferences
}

/// Extract base command (without arguments)
fn extract_base_command(full_command: &str) -> String {
    full_command
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_base_command() {
        assert_eq!(extract_base_command("ls -la /home"), "ls");
        assert_eq!(extract_base_command("git status"), "git");
        assert_eq!(extract_base_command("cargo build --release"), "cargo");
    }

    #[test]
    fn test_command_stats() {
        let mut learning = CommandLearning::new(100);

        let mut entry = CommandEntry::new("ls -la", PathBuf::from("/home"));
        entry.complete(0, 100);
        learning.add_command(entry.clone());

        let stats = learning.stats.get("ls").unwrap();
        assert_eq!(stats.total_count, 1);
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.success_rate(), 1.0);
    }

    #[test]
    fn test_time_category() {
        let morning = Utc::now().with_hour(9).unwrap();
        assert_eq!(TimeCategory::from_datetime(&morning), TimeCategory::Morning);

        let evening = Utc::now().with_hour(20).unwrap();
        assert_eq!(TimeCategory::from_datetime(&evening), TimeCategory::Evening);
    }
}
