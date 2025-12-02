//! Main application setup

use gtk4::Application;
use parking_lot::RwLock;
use std::sync::Arc;

use crate::dialogs;
use crate::window::MainWindow;

/// Global config manager
static CONFIG_MANAGER: std::sync::OnceLock<Arc<RwLock<corgiterm_config::ConfigManager>>> =
    std::sync::OnceLock::new();

/// Global session manager for project persistence
static SESSION_MANAGER: std::sync::OnceLock<Arc<RwLock<corgiterm_core::SessionManager>>> =
    std::sync::OnceLock::new();

/// Global AI manager
static AI_MANAGER: std::sync::OnceLock<Arc<RwLock<corgiterm_ai::AiManager>>> =
    std::sync::OnceLock::new();

/// Global plugin manager
static PLUGIN_MANAGER: std::sync::OnceLock<Arc<RwLock<corgiterm_plugins::PluginManager>>> =
    std::sync::OnceLock::new();

/// Global snippets manager
static SNIPPETS_MANAGER: std::sync::OnceLock<Arc<RwLock<corgiterm_config::SnippetsManager>>> =
    std::sync::OnceLock::new();

/// Global command history store for AI learning
static HISTORY_STORE: std::sync::OnceLock<Arc<RwLock<corgiterm_ai::history::CommandHistoryStore>>> =
    std::sync::OnceLock::new();

/// Global conversation store for AI chat persistence
static CONVERSATION_STORE: std::sync::OnceLock<Arc<RwLock<corgiterm_ai::conversation::ConversationStore>>> =
    std::sync::OnceLock::new();

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

/// Get the global plugin manager
pub fn plugin_manager() -> Option<Arc<RwLock<corgiterm_plugins::PluginManager>>> {
    PLUGIN_MANAGER.get().cloned()
}

/// Get the global snippets manager
pub fn snippets_manager() -> Option<Arc<RwLock<corgiterm_config::SnippetsManager>>> {
    SNIPPETS_MANAGER.get().cloned()
}

/// Get the global command history store
pub fn history_store() -> Option<Arc<RwLock<corgiterm_ai::history::CommandHistoryStore>>> {
    HISTORY_STORE.get().cloned()
}

/// Get the global conversation store
pub fn conversation_store() -> Option<Arc<RwLock<corgiterm_ai::conversation::ConversationStore>>> {
    CONVERSATION_STORE.get().cloned()
}

/// Record a command execution for AI learning
pub fn record_command(command: String, directory: String, exit_code: Option<i32>) {
    if let Some(store) = history_store() {
        let mut store = store.write();
        store.record(command, directory, exit_code, None);
        // Save periodically (every 10 commands)
        if store.len() % 10 == 0 {
            if let Err(e) = store.save() {
                tracing::warn!("Failed to save command history: {}", e);
            }
        }
    }
}

/// Get learning context for AI prompts
pub fn get_learning_context() -> corgiterm_ai::learning::LearningContext {
    if let Some(store) = history_store() {
        let store = store.read();
        store.extract_learning_context()
    } else {
        corgiterm_ai::learning::LearningContext::default()
    }
}

/// Load custom CSS styles with optional hot-reload
fn load_css() {
    // Check config for hot-reload setting
    let hot_reload = if let Some(cm) = config_manager() {
        let config = cm.read().config();
        config
            .appearance
            .hot_reload_css
            .unwrap_or(cfg!(debug_assertions))
    } else {
        cfg!(debug_assertions) // Default to hot-reload in debug builds
    };

    crate::theme::load_theme(hot_reload);
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
    let mut first_provider: Option<String> = None;

    // Get AI config
    if let Some(cm) = config_manager() {
        let config = cm.read().config();

        // Priority 1: CLI-based providers (OAuth, no API key needed)

        // Claude CLI (claude command)
        if std::process::Command::new("which")
            .arg("claude")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            let provider = corgiterm_ai::providers::ClaudeCliProvider::new(Some(
                config.ai.claude.model.clone(),
            ));
            if first_provider.is_none() {
                first_provider = Some("claude-cli".to_string());
            }
            ai_manager.add_provider(Box::new(provider));
            tracing::info!("Claude CLI provider available");
        }

        // Gemini CLI (gemini command)
        if std::process::Command::new("which")
            .arg("gemini")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            let provider = corgiterm_ai::providers::GeminiCliProvider::new(Some(
                config.ai.gemini.model.clone(),
            ));
            if first_provider.is_none() {
                first_provider = Some("gemini-cli".to_string());
            }
            ai_manager.add_provider(Box::new(provider));
            tracing::info!("Gemini CLI provider available");
        }

        // Priority 2: Local Ollama (check if reachable)
        if config.ai.local.enabled && !config.ai.local.endpoint.is_empty() {
            // Quick connectivity check (non-blocking timeout)
            let endpoint = config.ai.local.endpoint.clone();
            let ollama_available = std::process::Command::new("curl")
                .args(["-s", "-m", "1", &format!("{}/api/tags", endpoint)])
                .output()
                .map(|o| o.status.success() && !o.stdout.is_empty())
                .unwrap_or(false);

            if ollama_available {
                let provider = corgiterm_ai::providers::OllamaProvider::new(
                    endpoint,
                    config.ai.local.model.clone(),
                );
                if first_provider.is_none() {
                    first_provider = Some("ollama".to_string());
                }
                ai_manager.add_provider(Box::new(provider));
                tracing::info!("Ollama provider available at {}", config.ai.local.endpoint);
            } else {
                tracing::debug!("Ollama not reachable at {}", config.ai.local.endpoint);
            }
        }

        // Priority 3: API key providers
        if let Some(ref api_key) = config.ai.claude.api_key {
            if !api_key.is_empty() {
                let provider = corgiterm_ai::providers::ClaudeProvider::new(
                    api_key.clone(),
                    Some(config.ai.claude.model.clone()),
                );
                if first_provider.is_none() {
                    first_provider = Some("claude".to_string());
                }
                ai_manager.add_provider(Box::new(provider));
                tracing::info!("Claude API provider configured");
            }
        }

        if let Some(ref api_key) = config.ai.openai.api_key {
            if !api_key.is_empty() {
                let provider = corgiterm_ai::providers::OpenAiProvider::new(
                    api_key.clone(),
                    Some(config.ai.openai.model.clone()),
                );
                if first_provider.is_none() {
                    first_provider = Some("openai".to_string());
                }
                ai_manager.add_provider(Box::new(provider));
                tracing::info!("OpenAI API provider configured");
            }
        }

        if let Some(ref api_key) = config.ai.gemini.api_key {
            if !api_key.is_empty() {
                let provider = corgiterm_ai::providers::GeminiProvider::new(
                    api_key.clone(),
                    Some(config.ai.gemini.model.clone()),
                );
                if first_provider.is_none() {
                    first_provider = Some("gemini".to_string());
                }
                ai_manager.add_provider(Box::new(provider));
                tracing::info!("Gemini API provider configured");
            }
        }

        // Set default provider (auto = first available)
        let default = if config.ai.default_provider == "auto" {
            first_provider.clone().unwrap_or_default()
        } else {
            config.ai.default_provider.clone()
        };

        if !default.is_empty() && ai_manager.set_default(&default) {
            tracing::info!("Default AI provider: {}", default);
        }
    }

    let provider_count = ai_manager.list_providers().len();
    let ai_arc = Arc::new(RwLock::new(ai_manager));
    let _ = AI_MANAGER.set(ai_arc);

    if provider_count > 0 {
        tracing::info!("AI manager initialized with {} provider(s)", provider_count);
    } else {
        tracing::warn!("No AI providers available - install claude CLI, ollama, or add API keys");
    }
}

/// Initialize the plugin system
fn init_plugins() {
    // Get plugin directory from config or use default
    let plugin_dir = if let Some(cm) = config_manager() {
        let config = cm.read().config();
        config.advanced.plugin_dir.clone()
    } else {
        None
    };

    let plugin_dir = plugin_dir.unwrap_or_else(|| corgiterm_config::config_dir().join("plugins"));

    let mut plugin_manager = corgiterm_plugins::PluginManager::new(plugin_dir.clone());

    // Discover and load plugins
    match plugin_manager.discover() {
        Ok(count) => {
            if count > 0 {
                tracing::info!("Loaded {} plugins from {:?}", count, plugin_dir);
            } else {
                tracing::debug!("No plugins found in {:?}", plugin_dir);
            }
        }
        Err(e) => {
            tracing::warn!("Failed to discover plugins: {}", e);
        }
    }

    let plugin_arc = Arc::new(RwLock::new(plugin_manager));
    let _ = PLUGIN_MANAGER.set(plugin_arc);
    tracing::info!("Plugin manager initialized");
}

/// Initialize snippets library
fn init_snippets() {
    match corgiterm_config::SnippetsManager::new() {
        Ok(snippets_manager) => {
            let snippets_arc = Arc::new(RwLock::new(snippets_manager));
            let _ = SNIPPETS_MANAGER.set(snippets_arc.clone());
            crate::snippets::init_snippets(snippets_arc);
            tracing::info!("Snippets library initialized");
        }
        Err(e) => {
            tracing::error!("Failed to initialize snippets: {}", e);
        }
    }
}

/// Initialize command history for AI learning
fn init_history() {
    let history_store = corgiterm_ai::history::CommandHistoryStore::load();
    let stats = history_store.stats();
    let history_arc = Arc::new(RwLock::new(history_store));
    let _ = HISTORY_STORE.set(history_arc);
    tracing::info!(
        "Command history loaded: {} commands, {} unique",
        stats.total_commands,
        stats.unique_commands
    );
}

/// Initialize conversation store for AI chat persistence
fn init_conversations() {
    let mut conv_store = corgiterm_ai::conversation::ConversationStore::new();
    if let Err(e) = conv_store.load() {
        tracing::warn!("Failed to load conversations: {}", e);
    } else {
        let stats = conv_store.stats();
        tracing::info!(
            "Loaded {} conversations (chat: {}, command: {}, explain: {})",
            stats.total_conversations,
            stats.chat_count,
            stats.command_count,
            stats.explain_count
        );
    }
    let conv_arc = Arc::new(RwLock::new(conv_store));
    let _ = CONVERSATION_STORE.set(conv_arc);
}

/// Save conversations to disk (call on app exit or periodically)
pub fn save_conversations() {
    if let Some(store) = conversation_store() {
        let mut store = store.write();
        if let Err(e) = store.save() {
            tracing::error!("Failed to save conversations: {}", e);
        }
    }
}

/// Build the main UI
pub fn build_ui(app: &Application) {
    // Initialize config first
    init_config();

    // Initialize session manager for project persistence
    init_sessions();

    // Initialize AI providers
    init_ai();

    // Initialize plugin system
    init_plugins();

    // Initialize snippets library
    init_snippets();

    // Initialize command history for AI learning
    init_history();

    // Initialize conversation store for AI chat persistence
    init_conversations();

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
