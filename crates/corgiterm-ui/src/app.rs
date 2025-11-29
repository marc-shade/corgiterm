//! Main application setup

use gtk4::gdk::Display;
use gtk4::{Application, CssProvider};
use std::sync::Arc;
use parking_lot::RwLock;

use crate::window::MainWindow;
use crate::dialogs;

/// Global config manager
static CONFIG_MANAGER: std::sync::OnceLock<Arc<RwLock<corgiterm_config::ConfigManager>>> = std::sync::OnceLock::new();

/// Global session manager for project persistence
static SESSION_MANAGER: std::sync::OnceLock<Arc<RwLock<corgiterm_core::SessionManager>>> = std::sync::OnceLock::new();

/// Global AI manager
static AI_MANAGER: std::sync::OnceLock<Arc<RwLock<corgiterm_ai::AiManager>>> = std::sync::OnceLock::new();

/// Get the global config manager
pub fn config_manager() -> Option<Arc<RwLock<corgiterm_config::ConfigManager>>> {
    CONFIG_MANAGER.get().cloned()
}

/// Get the global session manager
pub fn session_manager() -> Option<Arc<RwLock<corgiterm_core::SessionManager>>> {
    SESSION_MANAGER.get().cloned()
}

/// Get the global AI manager
pub fn ai_manager() -> Option<Arc<RwLock<corgiterm_ai::AiManager>>> {
    AI_MANAGER.get().cloned()
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

/// Initialize the session manager for project persistence
fn init_sessions() {
    let config_dir = corgiterm_config::config_dir();
    let mut session_manager = corgiterm_core::SessionManager::new(config_dir);

    // Load saved projects
    if let Err(e) = session_manager.load() {
        tracing::warn!("Failed to load saved projects: {}", e);
    } else {
        tracing::info!("Loaded {} saved projects", session_manager.projects().len());
    }

    let session_arc = Arc::new(RwLock::new(session_manager));
    let _ = SESSION_MANAGER.set(session_arc);
}

/// Initialize AI providers from config
fn init_ai() {
    let mut ai_manager = corgiterm_ai::AiManager::new();

    // Get AI config
    if let Some(cm) = config_manager() {
        let config = cm.read().config();

        // Add Claude provider if API key is configured
        if let Some(ref api_key) = config.ai.claude.api_key {
            if !api_key.is_empty() {
                let provider = corgiterm_ai::providers::ClaudeProvider::new(
                    api_key.clone(),
                    Some(config.ai.claude.model.clone()),
                );
                ai_manager.add_provider(Box::new(provider));
                tracing::info!("Claude AI provider initialized");
            }
        }

        // Add OpenAI provider if API key is configured
        if let Some(ref api_key) = config.ai.openai.api_key {
            if !api_key.is_empty() {
                let provider = corgiterm_ai::providers::OpenAiProvider::new(
                    api_key.clone(),
                    Some(config.ai.openai.model.clone()),
                );
                ai_manager.add_provider(Box::new(provider));
                tracing::info!("OpenAI provider initialized");
            }
        }

        // Add Ollama (local) provider if enabled
        if config.ai.local.enabled && !config.ai.local.endpoint.is_empty() {
            let provider = corgiterm_ai::providers::OllamaProvider::new(
                config.ai.local.endpoint.clone(),
                config.ai.local.model.clone(),
            );
            ai_manager.add_provider(Box::new(provider));
            tracing::info!("Ollama local provider initialized");
        }

        // Set default provider
        let _ = ai_manager.set_default(&config.ai.default_provider);
    }

    let ai_arc = Arc::new(RwLock::new(ai_manager));
    let _ = AI_MANAGER.set(ai_arc);
    tracing::info!("AI manager initialized");
}

/// Build the main UI
pub fn build_ui(app: &Application) {
    // Initialize config first
    init_config();

    // Initialize session manager for project persistence
    init_sessions();

    // Initialize AI providers
    init_ai();

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
