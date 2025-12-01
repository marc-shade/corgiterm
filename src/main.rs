//! CorgiTerm - A next-generation, AI-powered terminal emulator
//!
//! ```text
//!    ∩＿∩
//!   (・ω・)  Welcome to CorgiTerm!
//!   /　 つ   The friendliest terminal ever.
//! ```

use clap::Parser;
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// CorgiTerm - AI-Powered Terminal Emulator
#[derive(Parser, Debug)]
#[command(name = "corgiterm")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Working directory to start in
    #[arg(short = 'd', long)]
    directory: Option<PathBuf>,

    /// Execute a command and exit
    #[arg(short = 'e', long)]
    execute: Option<String>,

    /// Open a specific project
    #[arg(short = 'p', long)]
    project: Option<PathBuf>,

    /// Start in Safe Mode
    #[arg(long)]
    safe_mode: bool,

    /// Disable AI features
    #[arg(long)]
    no_ai: bool,

    /// Use a specific theme
    #[arg(short = 't', long)]
    theme: Option<String>,

    /// Config file path
    #[arg(short = 'c', long)]
    config: Option<PathBuf>,

    /// Enable debug logging
    #[arg(long)]
    debug: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Set up logging
    let log_level = if args.debug { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| format!("corgiterm={}", log_level)),
        ))
        .init();

    tracing::info!("Starting CorgiTerm v{}", env!("CARGO_PKG_VERSION"));

    // Print welcome message in debug mode
    if args.debug {
        println!(
            r#"
   ∩＿∩
  (・ω・)  CorgiTerm v{}
  /　 つ   Debug mode enabled
        "#,
            env!("CARGO_PKG_VERSION")
        );
    }

    // Initialize core
    corgiterm_core::init()?;
    tracing::debug!("Core initialized");

    // Handle command execution mode
    if let Some(ref cmd) = args.execute {
        tracing::info!("Executing command: {}", cmd);
        // TODO: Execute command in headless mode
        return Ok(());
    }

    // Run the GTK4 application
    let exit_code = corgiterm_ui::run();

    std::process::exit(exit_code.into());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_parsing() {
        let args = Args::try_parse_from(["corgiterm"]).unwrap();
        assert!(!args.safe_mode);
        assert!(!args.no_ai);
    }

    #[test]
    fn test_arg_parsing_with_options() {
        let args =
            Args::try_parse_from(["corgiterm", "--safe-mode", "--no-ai", "-d", "/tmp"]).unwrap();
        assert!(args.safe_mode);
        assert!(args.no_ai);
        assert_eq!(args.directory, Some(PathBuf::from("/tmp")));
    }
}
