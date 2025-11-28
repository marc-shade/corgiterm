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

pub mod error;
pub mod pty;
pub mod terminal;
pub mod session;
pub mod history;
pub mod safe_mode;

pub use error::{CoreError, Result};
pub use pty::{Pty, PtySize};
pub use terminal::{Terminal, TerminalEvent, TerminalSize};
pub use session::{Session, SessionId, SessionManager};
pub use history::{CommandHistory, OutputHistory, SearchableHistory};
pub use safe_mode::{SafeMode, CommandPreview, RiskLevel};

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
