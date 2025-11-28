//! Main application setup

use gtk4::prelude::*;
use gtk4::gdk::Display;
use gtk4::{Application, CssProvider};
use libadwaita::prelude::*;

use crate::window::MainWindow;

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

/// Build the main UI
pub fn build_ui(app: &Application) {
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
