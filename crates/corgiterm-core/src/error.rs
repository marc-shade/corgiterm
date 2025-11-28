//! Error types for CorgiTerm Core

use thiserror::Error;

/// Result type for CorgiTerm Core operations
pub type Result<T> = std::result::Result<T, CoreError>;

/// Core error types
#[derive(Error, Debug)]
pub enum CoreError {
    /// PTY creation or operation failed
    #[error("PTY error: {0}")]
    Pty(String),

    /// Process spawning failed
    #[error("Failed to spawn process: {0}")]
    ProcessSpawn(String),

    /// Terminal emulation error
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// Session management error
    #[error("Session error: {0}")]
    Session(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// UTF-8 encoding error
    #[error("UTF-8 encoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Channel communication error
    #[error("Channel error: {0}")]
    Channel(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
}
