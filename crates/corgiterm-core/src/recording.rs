//! Session recording and playback
//!
//! Records all terminal I/O events with timestamps for later replay.
//! This is useful for:
//! - Training/demo videos
//! - Reviewing past sessions
//! - Sharing terminal workflows
//!
//! Recording format is designed to be compact and easy to replay at
//! different speeds (1x, 2x, 0.5x, etc.)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// Unique identifier for a recording
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordingId(Uuid);

impl RecordingId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        Uuid::parse_str(s).ok().map(Self)
    }
}

impl Default for RecordingId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RecordingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of recorded event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// Output from the terminal (PTY to screen)
    Output(Vec<u8>),
    /// Input to the terminal (keyboard to PTY)
    Input(Vec<u8>),
    /// Terminal resize event
    Resize { rows: u16, cols: u16 },
    /// Marker event (e.g., command started, command completed)
    Marker(String),
}

/// A single recorded event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingEvent {
    /// Timestamp relative to recording start (in milliseconds)
    pub timestamp_ms: u64,
    /// The event data
    pub event: EventType,
}

/// Recording metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMeta {
    /// Recording ID
    pub id: RecordingId,
    /// Title/name of the recording
    pub title: String,
    /// Description
    pub description: Option<String>,
    /// When recording started
    pub started_at: DateTime<Utc>,
    /// When recording ended
    pub ended_at: Option<DateTime<Utc>>,
    /// Total duration in milliseconds
    pub duration_ms: u64,
    /// Initial terminal size
    pub initial_rows: u16,
    /// Initial terminal size
    pub initial_cols: u16,
    /// Working directory when recording started
    pub cwd: PathBuf,
    /// Shell that was used
    pub shell: Option<String>,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Number of events
    pub event_count: usize,
}

/// A complete terminal recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    /// Metadata
    pub meta: RecordingMeta,
    /// All recorded events
    pub events: Vec<RecordingEvent>,
}

impl Recording {
    /// Create a new recording
    pub fn new(title: impl Into<String>, cwd: PathBuf, rows: u16, cols: u16) -> Self {
        Self {
            meta: RecordingMeta {
                id: RecordingId::new(),
                title: title.into(),
                description: None,
                started_at: Utc::now(),
                ended_at: None,
                duration_ms: 0,
                initial_rows: rows,
                initial_cols: cols,
                cwd,
                shell: None,
                tags: Vec::new(),
                event_count: 0,
            },
            events: Vec::new(),
        }
    }

    /// Add an output event
    pub fn add_output(&mut self, data: &[u8]) {
        let timestamp_ms = self.elapsed_ms();
        self.events.push(RecordingEvent {
            timestamp_ms,
            event: EventType::Output(data.to_vec()),
        });
        self.meta.event_count = self.events.len();
    }

    /// Add an input event
    pub fn add_input(&mut self, data: &[u8]) {
        let timestamp_ms = self.elapsed_ms();
        self.events.push(RecordingEvent {
            timestamp_ms,
            event: EventType::Input(data.to_vec()),
        });
        self.meta.event_count = self.events.len();
    }

    /// Add a resize event
    pub fn add_resize(&mut self, rows: u16, cols: u16) {
        let timestamp_ms = self.elapsed_ms();
        self.events.push(RecordingEvent {
            timestamp_ms,
            event: EventType::Resize { rows, cols },
        });
        self.meta.event_count = self.events.len();
    }

    /// Add a marker event
    pub fn add_marker(&mut self, label: impl Into<String>) {
        let timestamp_ms = self.elapsed_ms();
        self.events.push(RecordingEvent {
            timestamp_ms,
            event: EventType::Marker(label.into()),
        });
        self.meta.event_count = self.events.len();
    }

    /// Finalize the recording
    pub fn finalize(&mut self) {
        self.meta.ended_at = Some(Utc::now());
        self.meta.duration_ms = self.elapsed_ms();
    }

    /// Get elapsed time since recording started (in milliseconds)
    fn elapsed_ms(&self) -> u64 {
        let elapsed = Utc::now() - self.meta.started_at;
        elapsed.num_milliseconds().max(0) as u64
    }

    /// Get duration as a human-readable string
    pub fn duration_string(&self) -> String {
        let secs = self.meta.duration_ms / 1000;
        let mins = secs / 60;
        let secs = secs % 60;
        if mins > 0 {
            format!("{}m {}s", mins, secs)
        } else {
            format!("{}s", secs)
        }
    }
}

/// State of a recording playback
#[derive(Debug)]
pub struct PlaybackState {
    /// The recording being played
    pub recording: Recording,
    /// Current event index
    pub current_index: usize,
    /// Playback speed (1.0 = normal, 2.0 = double speed)
    pub speed: f64,
    /// Is playback paused?
    pub paused: bool,
    /// Current playback position in milliseconds
    pub position_ms: u64,
}

impl PlaybackState {
    /// Create new playback state
    pub fn new(recording: Recording) -> Self {
        Self {
            recording,
            current_index: 0,
            speed: 1.0,
            paused: false,
            position_ms: 0,
        }
    }

    /// Get the next event(s) that should be played at the current position
    pub fn next_events(&mut self) -> Vec<RecordingEvent> {
        if self.paused {
            return Vec::new();
        }

        let mut events = Vec::new();
        while self.current_index < self.recording.events.len() {
            let event = &self.recording.events[self.current_index];
            if event.timestamp_ms <= self.position_ms {
                events.push(event.clone());
                self.current_index += 1;
            } else {
                break;
            }
        }
        events
    }

    /// Advance playback by delta_ms (real time)
    pub fn advance(&mut self, delta_ms: u64) {
        if !self.paused {
            let adjusted = (delta_ms as f64 * self.speed) as u64;
            self.position_ms = self.position_ms.saturating_add(adjusted);
        }
    }

    /// Seek to a specific position
    pub fn seek(&mut self, position_ms: u64) {
        self.position_ms = position_ms.min(self.recording.meta.duration_ms);
        // Find the event index that matches this position
        self.current_index = self
            .recording
            .events
            .iter()
            .position(|e| e.timestamp_ms > position_ms)
            .unwrap_or(self.recording.events.len());
    }

    /// Toggle pause
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Set playback speed
    pub fn set_speed(&mut self, speed: f64) {
        self.speed = speed.max(0.1).min(10.0);
    }

    /// Check if playback is complete
    pub fn is_complete(&self) -> bool {
        self.current_index >= self.recording.events.len()
    }

    /// Get progress as percentage (0.0 to 1.0)
    pub fn progress(&self) -> f64 {
        if self.recording.meta.duration_ms == 0 {
            return 1.0;
        }
        self.position_ms as f64 / self.recording.meta.duration_ms as f64
    }
}

/// Manages saved recordings
pub struct RecordingStore {
    /// Directory where recordings are stored
    storage_path: PathBuf,
    /// Cached list of recordings (metadata only)
    recordings: Vec<RecordingMeta>,
}

impl RecordingStore {
    /// Create a new recording store
    pub fn new() -> Self {
        let storage_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("corgiterm")
            .join("recordings");

        Self {
            storage_path,
            recordings: Vec::new(),
        }
    }

    /// Load recording list from disk
    pub fn load(&mut self) -> Result<(), std::io::Error> {
        if !self.storage_path.exists() {
            std::fs::create_dir_all(&self.storage_path)?;
        }

        self.recordings.clear();

        for entry in std::fs::read_dir(&self.storage_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                // Read just the metadata
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(recording) = serde_json::from_str::<Recording>(&content) {
                        self.recordings.push(recording.meta);
                    }
                }
            }
        }

        // Sort by date, newest first
        self.recordings
            .sort_by(|a, b| b.started_at.cmp(&a.started_at));

        Ok(())
    }

    /// Save a recording to disk
    pub fn save(&self, recording: &Recording) -> Result<PathBuf, std::io::Error> {
        std::fs::create_dir_all(&self.storage_path)?;

        let filename = format!("{}.json", recording.meta.id);
        let path = self.storage_path.join(&filename);

        let content = serde_json::to_string_pretty(recording)?;
        std::fs::write(&path, content)?;

        Ok(path)
    }

    /// Load a full recording by ID
    pub fn load_recording(&self, id: RecordingId) -> Result<Recording, std::io::Error> {
        let filename = format!("{}.json", id);
        let path = self.storage_path.join(&filename);

        let content = std::fs::read_to_string(&path)?;
        let recording: Recording = serde_json::from_str(&content)?;

        Ok(recording)
    }

    /// Delete a recording
    pub fn delete(&mut self, id: RecordingId) -> Result<(), std::io::Error> {
        let filename = format!("{}.json", id);
        let path = self.storage_path.join(&filename);

        if path.exists() {
            std::fs::remove_file(&path)?;
        }

        self.recordings.retain(|r| r.id != id);
        Ok(())
    }

    /// Get all recording metadata
    pub fn list(&self) -> &[RecordingMeta] {
        &self.recordings
    }

    /// Get recordings matching a search query
    pub fn search(&self, query: &str) -> Vec<&RecordingMeta> {
        let query_lower = query.to_lowercase();
        self.recordings
            .iter()
            .filter(|r| {
                r.title.to_lowercase().contains(&query_lower)
                    || r.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || r.tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Get total number of recordings
    pub fn count(&self) -> usize {
        self.recordings.len()
    }
}

impl Default for RecordingStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_creation() {
        let mut recording = Recording::new("Test Recording", PathBuf::from("/home/user"), 24, 80);

        recording.add_output(b"Hello, ");
        recording.add_input(b"World");
        recording.add_output(b"!");
        recording.add_marker("test complete");
        recording.finalize();

        assert_eq!(recording.meta.title, "Test Recording");
        assert_eq!(recording.events.len(), 4);
        assert!(recording.meta.duration_ms >= 0);
    }

    #[test]
    fn test_playback_state() {
        let mut recording = Recording::new("Test", PathBuf::from("/"), 24, 80);
        recording.events.push(RecordingEvent {
            timestamp_ms: 0,
            event: EventType::Output(b"Start".to_vec()),
        });
        recording.events.push(RecordingEvent {
            timestamp_ms: 100,
            event: EventType::Output(b"Middle".to_vec()),
        });
        recording.events.push(RecordingEvent {
            timestamp_ms: 200,
            event: EventType::Output(b"End".to_vec()),
        });
        recording.meta.duration_ms = 200;

        let mut playback = PlaybackState::new(recording);

        // At position 0, should get first event
        let events = playback.next_events();
        assert_eq!(events.len(), 1);

        // Advance to 150ms, should get second event
        playback.advance(150);
        let events = playback.next_events();
        assert_eq!(events.len(), 1);

        // Advance to 250ms, should get third event
        playback.advance(100);
        let events = playback.next_events();
        assert_eq!(events.len(), 1);

        assert!(playback.is_complete());
    }

    #[test]
    fn test_playback_speed() {
        let recording = Recording::new("Test", PathBuf::from("/"), 24, 80);
        let mut playback = PlaybackState::new(recording);

        playback.set_speed(2.0);
        playback.advance(100); // 100ms real time = 200ms playback time

        assert_eq!(playback.position_ms, 200);
    }
}
