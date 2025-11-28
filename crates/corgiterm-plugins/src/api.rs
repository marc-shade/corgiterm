//! Plugin API definitions

use serde::{Deserialize, Serialize};

/// API version
pub const API_VERSION: &str = "0.1.0";

/// Terminal API for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminalCommand {
    /// Write text to terminal
    Write { text: String },
    /// Execute a shell command
    Execute { command: String },
    /// Get current working directory
    GetCwd,
    /// Get terminal size
    GetSize,
    /// Get scrollback buffer
    GetScrollback { lines: usize },
}

/// Terminal API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminalResponse {
    /// Success with optional data
    Ok { data: Option<String> },
    /// Error with message
    Error { message: String },
    /// Current directory
    Cwd { path: String },
    /// Terminal size
    Size { rows: usize, cols: usize },
    /// Scrollback content
    Scrollback { lines: Vec<String> },
}

/// UI API for plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiCommand {
    /// Show a notification
    Notify { title: String, message: String },
    /// Show a dialog
    Dialog { title: String, content: String, buttons: Vec<String> },
    /// Add a status bar item
    StatusItem { id: String, text: String },
    /// Remove a status bar item
    RemoveStatusItem { id: String },
}

/// UI API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiResponse {
    /// Success
    Ok,
    /// Dialog result (button index)
    DialogResult { button: usize },
    /// Error
    Error { message: String },
}

/// Plugin events that can be subscribed to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
    /// Terminal output received
    TerminalOutput { text: String },
    /// Command executed
    CommandExecuted { command: String, exit_code: i32 },
    /// Session created
    SessionCreated { id: String },
    /// Session closed
    SessionClosed { id: String },
    /// Theme changed
    ThemeChanged { name: String },
}

/// Plugin hook points
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookPoint {
    /// Before command execution
    PreExecute,
    /// After command execution
    PostExecute,
    /// On terminal output
    OnOutput,
    /// On session start
    OnSessionStart,
    /// On session end
    OnSessionEnd,
}
