//! # CorgiTerm Core
//!
//! The core terminal emulation engine for CorgiTerm.
//!
//! This crate provides:
//! - PTY management and process spawning
//! - VT100/xterm terminal emulation
//! - Unicode and emoji handling
//! - Session management with thumbnails
//! - Command history and searchable output
//! - Safe Mode command preview
//!
//! ```text
//!    ∩＿∩
//!   (・ω・)  CorgiTerm Core Engine
//!   /　 つ   Making terminals friendly!
//! ```

pub mod ascii_art;
pub mod error;
pub mod history;
pub mod history_learning;
pub mod learning;
pub mod pty;
pub mod safe_mode;
pub mod session;
pub mod terminal;

pub use ascii_art::{
    AsciiArtConfig, AsciiArtGenerator, AsciiFont, CharacterSet, CorgiArt, FONT_SMALL, FONT_STANDARD,
};
pub use error::{CoreError, Result};
pub use history::{CommandHistory, OutputHistory, SearchableHistory};
pub use history_learning::{
    FrequentCommandData, HistoryLearningManager, LearningContextData, PatternData, PreferenceData,
};
pub use learning::{
    CommandLearning, CommandPattern, CommandStats, CommandSuggestion, SuggestionSource,
    UserPreference,
};
pub use pty::{Pty, PtySize};
pub use safe_mode::{CommandPreview, RiskLevel, SafeMode};
pub use session::{Session, SessionId, SessionManager};
pub use terminal::{Terminal, TerminalEvent, TerminalSize};

/// Core version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the core engine with default settings
pub fn init() -> Result<()> {
    tracing::info!("Initializing CorgiTerm Core v{}", VERSION);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
}
