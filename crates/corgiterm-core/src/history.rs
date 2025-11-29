//! Command and output history with full-text search
//!
//! CorgiTerm keeps searchable history of:
//! - All commands executed
//! - All terminal output
//! - Timestamps and context
//!
//! This enables "Time-Travel" debugging and never losing output.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;

/// A single command in history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    /// The command that was executed
    pub command: String,
    /// Working directory when executed
    pub cwd: PathBuf,
    /// When it was executed
    pub timestamp: DateTime<Utc>,
    /// Exit code (if completed)
    pub exit_code: Option<i32>,
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Was this successful?
    pub success: Option<bool>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Project ID if part of a project
    pub project_id: Option<String>,
}

impl CommandEntry {
    pub fn new(command: impl Into<String>, cwd: PathBuf) -> Self {
        Self {
            command: command.into(),
            cwd,
            timestamp: Utc::now(),
            exit_code: None,
            duration_ms: None,
            success: None,
            tags: Vec::new(),
            project_id: None,
        }
    }

    /// Mark command as completed
    pub fn complete(&mut self, exit_code: i32, duration_ms: u64) {
        self.exit_code = Some(exit_code);
        self.duration_ms = Some(duration_ms);
        self.success = Some(exit_code == 0);
    }
}

/// Command history with search capabilities
pub struct CommandHistory {
    /// All commands (most recent first)
    entries: VecDeque<CommandEntry>,
    /// Maximum entries to keep
    max_entries: usize,
    /// Persistence file path
    persist_path: Option<PathBuf>,
}

impl CommandHistory {
    /// Create new command history
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries,
            persist_path: None,
        }
    }

    /// Set persistence path
    pub fn with_persistence(mut self, path: PathBuf) -> Self {
        self.persist_path = Some(path);
        self
    }

    /// Add a command to history
    pub fn push(&mut self, entry: CommandEntry) {
        self.entries.push_front(entry);
        while self.entries.len() > self.max_entries {
            self.entries.pop_back();
        }
    }

    /// Search commands by query
    pub fn search(&self, query: &str) -> Vec<&CommandEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.command.to_lowercase().contains(&query_lower))
            .collect()
    }

    /// Get recent commands
    pub fn recent(&self, count: usize) -> Vec<&CommandEntry> {
        self.entries.iter().take(count).collect()
    }

    /// Get commands in a specific directory
    pub fn in_directory(&self, cwd: &PathBuf) -> Vec<&CommandEntry> {
        self.entries.iter().filter(|e| &e.cwd == cwd).collect()
    }

    /// Get frequently used commands
    pub fn frequent(&self, count: usize) -> Vec<(String, usize)> {
        use std::collections::HashMap;
        let mut counts: HashMap<String, usize> = HashMap::new();

        for entry in &self.entries {
            *counts.entry(entry.command.clone()).or_insert(0) += 1;
        }

        let mut sorted: Vec<_> = counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(count);
        sorted
    }

    /// Save history to disk
    pub fn save(&self) -> std::io::Result<()> {
        if let Some(ref path) = self.persist_path {
            let content = serde_json::to_string_pretty(&self.entries.iter().collect::<Vec<_>>())?;
            std::fs::write(path, content)?;
        }
        Ok(())
    }

    /// Load history from disk
    pub fn load(&mut self) -> std::io::Result<()> {
        if let Some(ref path) = self.persist_path {
            if path.exists() {
                let content = std::fs::read_to_string(path)?;
                let entries: Vec<CommandEntry> = serde_json::from_str(&content)?;
                self.entries = entries.into_iter().collect();
            }
        }
        Ok(())
    }
}

/// A chunk of terminal output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputChunk {
    /// The output text
    pub text: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Associated command (if known)
    pub command: Option<String>,
    /// Is this stderr?
    pub is_stderr: bool,
}

/// Searchable output history
pub struct OutputHistory {
    /// Output chunks
    chunks: VecDeque<OutputChunk>,
    /// Maximum chunks to keep
    max_chunks: usize,
    /// Maximum total size in bytes
    max_bytes: usize,
    /// Current size
    current_bytes: usize,
}

impl OutputHistory {
    /// Create new output history
    pub fn new(max_chunks: usize, max_bytes: usize) -> Self {
        Self {
            chunks: VecDeque::new(),
            max_chunks,
            max_bytes,
            current_bytes: 0,
        }
    }

    /// Add output chunk
    pub fn push(&mut self, chunk: OutputChunk) {
        self.current_bytes += chunk.text.len();
        self.chunks.push_front(chunk);

        // Trim if necessary
        while self.chunks.len() > self.max_chunks || self.current_bytes > self.max_bytes {
            if let Some(removed) = self.chunks.pop_back() {
                self.current_bytes = self.current_bytes.saturating_sub(removed.text.len());
            } else {
                break;
            }
        }
    }

    /// Search output for a pattern
    pub fn search(&self, pattern: &str) -> Vec<SearchResult> {
        let pattern_lower = pattern.to_lowercase();
        let mut results = Vec::new();

        for (idx, chunk) in self.chunks.iter().enumerate() {
            let text_lower = chunk.text.to_lowercase();
            for (pos, _) in text_lower.match_indices(&pattern_lower) {
                // Get context around match
                let start = pos.saturating_sub(50);
                let end = (pos + pattern.len() + 50).min(chunk.text.len());
                let context = chunk.text[start..end].to_string();

                results.push(SearchResult {
                    chunk_index: idx,
                    position: pos,
                    context,
                    timestamp: chunk.timestamp,
                    command: chunk.command.clone(),
                });
            }
        }

        results
    }

    /// Get output at a specific time ("Time-Travel")
    pub fn at_time(&self, time: DateTime<Utc>) -> Vec<&OutputChunk> {
        self.chunks.iter().filter(|c| c.timestamp <= time).collect()
    }
}

/// A search result in output history
#[derive(Debug)]
pub struct SearchResult {
    /// Index of the chunk
    pub chunk_index: usize,
    /// Position in the chunk
    pub position: usize,
    /// Context around the match
    pub context: String,
    /// When this was outputted
    pub timestamp: DateTime<Utc>,
    /// Associated command
    pub command: Option<String>,
}

/// Combined searchable history trait
pub trait SearchableHistory {
    /// Search across all history types
    fn global_search(&self, query: &str) -> GlobalSearchResults;
}

/// Results from a global search
#[derive(Debug, Default)]
pub struct GlobalSearchResults {
    pub commands: Vec<CommandEntry>,
    pub output_matches: Vec<SearchResult>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_history() {
        let mut history = CommandHistory::new(100);
        history.push(CommandEntry::new("ls -la", PathBuf::from("/home")));
        history.push(CommandEntry::new("cd projects", PathBuf::from("/home")));

        assert_eq!(history.recent(10).len(), 2);
    }

    #[test]
    fn test_command_search() {
        let mut history = CommandHistory::new(100);
        history.push(CommandEntry::new("ls -la", PathBuf::from("/home")));
        history.push(CommandEntry::new("git status", PathBuf::from("/home")));
        history.push(CommandEntry::new("git push", PathBuf::from("/home")));

        let results = history.search("git");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_output_search() {
        let mut output = OutputHistory::new(1000, 1024 * 1024);
        output.push(OutputChunk {
            text: "Error: file not found".to_string(),
            timestamp: Utc::now(),
            command: Some("cat missing.txt".to_string()),
            is_stderr: true,
        });

        let results = output.search("error");
        assert_eq!(results.len(), 1);
    }
}
