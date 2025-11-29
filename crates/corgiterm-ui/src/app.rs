//! Main application setup

use gtk4::prelude::*;
use gtk4::gdk::Display;
use gtk4::{Application, CssProvider};
use libadwaita::prelude::*;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::window::MainWindow;
use crate::dialogs;

/// Global config manager
static CONFIG_MANAGER: std::sync::OnceLock<Arc<RwLock<corgiterm_config::ConfigManager>>> = std::sync::OnceLock::new();

/// Get the global config manager
pub fn config_manager() -> Option<Arc<RwLock<corgiterm_config::ConfigManager>>> {
    CONFIG_MANAGER.get().cloned()
}

/// Load custom CSS styles
fn load_css() {
    let provider = CssProvider::new();

    // Load CSS from embedded resource
    let css_data = include_str!("style.css");
    provider.load_from_string(css_data);

    // Apply to default display
    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        tracing::info!("Loaded CorgiTerm CSS theme");
    }
}

/// Initialize the global config
fn init_config() {
    match corgiterm_config::ConfigManager::new() {
        Ok(config_manager) => {
            let config_arc = Arc::new(RwLock::new(config_manager));
            let _ = CONFIG_MANAGER.set(config_arc.clone());
            dialogs::init_config(config_arc);
            tracing::info!("Configuration loaded");
        }
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
        }
    }
}

/// Build the main UI
pub fn build_ui(app: &Application) {
    // Initialize config first
    init_config();

    // Load custom CSS
    load_css();

    // Apply libadwaita styling
    let style_manager = libadwaita::StyleManager::default();
    style_manager.set_color_scheme(libadwaita::ColorScheme::PreferDark);

    // Create main window
    let window = MainWindow::new(app);
    window.present();
}

/// Application state
pub struct AppState {
    /// Configuration manager
    pub config: corgiterm_config::ConfigManager,
    /// Session manager
    pub sessions: corgiterm_core::SessionManager,
    /// AI manager
    pub ai: corgiterm_ai::AiManager,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let config_dir = corgiterm_config::config_dir();

        Ok(Self {
            config: corgiterm_config::ConfigManager::new()?,
            sessions: corgiterm_core::SessionManager::new(config_dir.clone()),
            ai: corgiterm_ai::AiManager::new(),
        })
    }
}
