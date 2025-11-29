//! # CorgiTerm Configuration
//!
//! Configuration management with 500+ settings organized into categories.
//! Supports hot-reloading, schema validation, and project-specific overrides.
//!
//! Configuration sources (in priority order):
//! 1. CLI arguments
//! 2. Environment variables
//! 3. Project-specific config (in project folder)
//! 4. User config (~/.config/corgiterm/config.toml)
//! 5. System defaults

pub mod schema;
pub mod themes;

use directories::ProjectDirs;
use figment::{Figment, providers::{Format, Toml, Env}};
use notify::{Watcher, RecursiveMode, Event, recommended_watcher};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Get the configuration directory
pub fn config_dir() -> PathBuf {
    ProjectDirs::from("dev", "corgiterm", "CorgiTerm")
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("~/.config/corgiterm"))
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// General settings
    pub general: GeneralConfig,
    /// Appearance settings
    pub appearance: AppearanceConfig,
    /// Terminal behavior
    pub terminal: TerminalConfig,
    /// Keyboard shortcuts
    pub keybindings: KeybindingsConfig,
    /// AI integration
    pub ai: AiConfig,
    /// Safe Mode settings
    pub safe_mode: SafeModeConfig,
    /// Session management
    pub sessions: SessionsConfig,
    /// Performance settings
    pub performance: PerformanceConfig,
    /// Accessibility
    pub accessibility: AccessibilityConfig,
    /// Advanced settings
    pub advanced: AdvancedConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            appearance: AppearanceConfig::default(),
            terminal: TerminalConfig::default(),
            keybindings: KeybindingsConfig::default(),
            ai: AiConfig::default(),
            safe_mode: SafeModeConfig::default(),
            sessions: SessionsConfig::default(),
            performance: PerformanceConfig::default(),
            accessibility: AccessibilityConfig::default(),
            advanced: AdvancedConfig::default(),
        }
    }
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Default shell
    pub shell: String,
    /// Starting directory
    pub working_directory: PathBuf,
    /// Check for updates
    pub check_updates: bool,
    /// Send anonymous usage statistics
    pub telemetry: bool,
    /// Language/locale
    pub language: String,
    /// Show welcome screen on first launch
    pub show_welcome: bool,
    /// Confirm before closing with active sessions
    pub confirm_close: bool,
    /// Restore sessions on startup
    pub restore_sessions: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            shell: std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()),
            working_directory: dirs::home_dir().unwrap_or_else(|| PathBuf::from("/")),
            check_updates: true,
            telemetry: false, // Privacy by default
            language: "en".to_string(),
            show_welcome: true,
            confirm_close: true,
            restore_sessions: true,
        }
    }
}

/// Appearance and theming
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    /// Color theme name
    pub theme: String,
    /// Font family
    pub font_family: String,
    /// Font size in points
    pub font_size: f32,
    /// Line height multiplier
    pub line_height: f32,
    /// Enable font ligatures
    pub ligatures: bool,
    /// Cursor style
    pub cursor_style: CursorStyle,
    /// Cursor blink
    pub cursor_blink: bool,
    /// Cursor blink rate in milliseconds
    pub cursor_blink_rate: u32,
    /// Background opacity (0.0 - 1.0)
    pub opacity: f32,
    /// Enable blur behind terminal
    pub blur: bool,
    /// Blur radius
    pub blur_radius: u32,
    /// Window decorations
    pub decorations: WindowDecorations,
    /// Tab bar position
    pub tab_position: TabPosition,
    /// Show sidebar
    pub show_sidebar: bool,
    /// Sidebar width
    pub sidebar_width: u32,
    /// Sidebar position
    pub sidebar_position: SidebarPosition,
    /// Show session thumbnails in sidebar
    pub show_thumbnails: bool,
    /// Thumbnail update interval in ms
    pub thumbnail_interval: u32,
    /// Minimum contrast ratio (accessibility)
    pub min_contrast: f32,
    /// UI scale factor
    pub ui_scale: f32,
    /// Icon theme
    pub icon_theme: String,
    /// Window padding
    pub padding: Padding,
    /// Enable CSS hot-reload (watches style.css for changes)
    #[serde(default)]
    pub hot_reload_css: Option<bool>,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: "Corgi Dark".to_string(),
            font_family: "Source Code Pro".to_string(),
            font_size: 11.0,
            line_height: 1.2,
            ligatures: true,
            cursor_style: CursorStyle::Block,
            cursor_blink: true,
            cursor_blink_rate: 530,
            opacity: 1.0,
            blur: false,
            blur_radius: 20,
            decorations: WindowDecorations::Full,
            tab_position: TabPosition::Top,
            show_sidebar: true,
            sidebar_width: 220,
            sidebar_position: SidebarPosition::Left,
            show_thumbnails: true,
            thumbnail_interval: 1000,
            min_contrast: 4.5,
            ui_scale: 1.0,
            icon_theme: "corgi".to_string(),
            padding: Padding::default(),
            hot_reload_css: None, // Default to None (uses debug_assertions at runtime)
        }
    }
}

/// Cursor styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
    Hollow,
}

/// Window decoration styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowDecorations {
    Full,
    None,
    Transparent,
    Custom,
}

/// Tab bar position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TabPosition {
    Top,
    Bottom,
    Hidden,
}

/// Sidebar position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SidebarPosition {
    Left,
    Right,
}

/// Padding configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct Padding {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

impl Default for Padding {
    fn default() -> Self {
        Self {
            top: 8,
            bottom: 8,
            left: 8,
            right: 8,
        }
    }
}

/// Terminal behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalConfig {
    /// Scrollback lines
    pub scrollback_lines: usize,
    /// Scroll on output
    pub scroll_on_output: bool,
    /// Scroll on keystroke
    pub scroll_on_keystroke: bool,
    /// Mouse reporting
    pub mouse_reporting: bool,
    /// Copy on select
    pub copy_on_select: bool,
    /// Paste on middle click
    pub paste_on_middle_click: bool,
    /// Bell style
    pub bell_style: BellStyle,
    /// Bell audio file (if audible)
    pub bell_audio: Option<PathBuf>,
    /// Word separators for double-click selection
    pub word_separators: String,
    /// Enable hyperlinks
    pub hyperlinks: bool,
    /// Enable bracketed paste
    pub bracketed_paste: bool,
    /// Environment variables to set
    pub env: std::collections::HashMap<String, String>,
    /// TERM environment variable
    pub term: String,
    /// Close tab on exit
    pub close_on_exit: CloseOnExit,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            scrollback_lines: 10000,
            scroll_on_output: false,
            scroll_on_keystroke: true,
            mouse_reporting: true,
            copy_on_select: false,
            paste_on_middle_click: true,
            bell_style: BellStyle::Visual,
            bell_audio: None,
            word_separators: " \t\n{}[]()\"'`,;:".to_string(),
            hyperlinks: true,
            bracketed_paste: true,
            env: std::collections::HashMap::new(),
            term: "xterm-256color".to_string(),
            close_on_exit: CloseOnExit::IfClean,
        }
    }
}

/// Bell notification style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BellStyle {
    None,
    Visual,
    Audible,
    Both,
}

/// When to close tab on exit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloseOnExit {
    Always,
    Never,
    IfClean, // Only if exit code is 0
}

/// Keyboard shortcuts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct KeybindingsConfig {
    /// Custom keybindings
    pub bindings: Vec<Keybinding>,
}

/// A single keybinding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    pub key: String,
    pub mods: Vec<String>,
    pub action: String,
}

/// AI integration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AiConfig {
    /// Enable AI features
    pub enabled: bool,
    /// Default AI provider
    pub default_provider: String,
    /// Claude settings
    pub claude: ClaudeConfig,
    /// OpenAI/Codex settings
    pub openai: OpenAiConfig,
    /// Gemini settings
    pub gemini: GeminiConfig,
    /// Local LLM settings
    pub local: LocalLlmConfig,
    /// Natural language input
    pub natural_language: bool,
    /// Auto-suggest completions
    pub auto_suggest: bool,
    /// Show AI panel
    pub show_panel: bool,
    /// Panel position
    pub panel_position: AiPanelPosition,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_provider: "auto".to_string(),  // Auto-detect best available provider
            claude: ClaudeConfig::default(),
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
            local: LocalLlmConfig::default(),
            natural_language: true,
            auto_suggest: true,
            show_panel: false,
            panel_position: AiPanelPosition::Right,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ClaudeConfig {
    pub api_key: Option<String>,
    pub model: String,
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: "claude-sonnet-4-20250514".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OpenAiConfig {
    pub api_key: Option<String>,
    pub model: String,
}

impl Default for OpenAiConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: "gpt-4o".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeminiConfig {
    pub api_key: Option<String>,
    pub model: String,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: "gemini-2.0-flash".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LocalLlmConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub model: String,
}

impl Default for LocalLlmConfig {
    fn default() -> Self {
        Self {
            enabled: true,  // Enable by default - auto-detects if Ollama is running
            endpoint: "http://localhost:11434".to_string(),  // Standard Ollama port
            model: "codellama".to_string(),  // Common model for command generation
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiPanelPosition {
    Left,
    Right,
    Bottom,
    Float,
}

/// Safe Mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SafeModeConfig {
    /// Enable Safe Mode
    pub enabled: bool,
    /// Show preview for all commands
    pub preview_all: bool,
    /// Only preview dangerous commands
    pub preview_dangerous_only: bool,
    /// Use AI for explanations
    pub ai_explanations: bool,
    /// Custom dangerous patterns
    pub dangerous_patterns: Vec<String>,
    /// Custom safe patterns
    pub safe_patterns: Vec<String>,
}

impl Default for SafeModeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            preview_all: false,
            preview_dangerous_only: true,
            ai_explanations: true,
            dangerous_patterns: Vec::new(),
            safe_patterns: Vec::new(),
        }
    }
}

/// Session management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionsConfig {
    /// Default session name format
    pub default_name: String,
    /// Auto-rename based on running process
    pub auto_rename: bool,
    /// Show process in tab title
    pub show_process: bool,
    /// Show working directory in tab title
    pub show_cwd: bool,
    /// Maximum sessions per project
    pub max_per_project: usize,
    /// Warn before closing multiple sessions
    pub warn_multiple_close: bool,
}

impl Default for SessionsConfig {
    fn default() -> Self {
        Self {
            default_name: "Terminal".to_string(),
            auto_rename: true,
            show_process: true,
            show_cwd: true,
            max_per_project: 50,
            warn_multiple_close: true,
        }
    }
}

/// Performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PerformanceConfig {
    /// Enable GPU rendering
    pub gpu_rendering: bool,
    /// Target frame rate
    pub target_fps: u32,
    /// VSync
    pub vsync: bool,
    /// Batch size for text rendering
    pub text_batch_size: usize,
    /// Enable render caching
    pub render_cache: bool,
    /// Maximum cached frames
    pub cache_frames: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            gpu_rendering: true,
            target_fps: 60,
            vsync: true,
            text_batch_size: 1024,
            render_cache: true,
            cache_frames: 3,
        }
    }
}

/// Accessibility settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AccessibilityConfig {
    /// Enable screen reader support
    pub screen_reader: bool,
    /// Reduce motion
    pub reduce_motion: bool,
    /// High contrast mode
    pub high_contrast: bool,
    /// Minimum font size
    pub min_font_size: f32,
    /// Enable keyboard navigation indicators
    pub focus_indicators: bool,
    /// Announce notifications
    pub announce_notifications: bool,
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            screen_reader: false,
            reduce_motion: false,
            high_contrast: false,
            min_font_size: 10.0,
            focus_indicators: true,
            announce_notifications: true,
        }
    }
}

/// Advanced/experimental settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AdvancedConfig {
    /// Enable debug mode
    pub debug: bool,
    /// Log level
    pub log_level: String,
    /// Log file path
    pub log_file: Option<PathBuf>,
    /// Enable experimental features
    pub experimental: bool,
    /// Custom CSS path
    pub custom_css: Option<PathBuf>,
    /// Plugin directory
    pub plugin_dir: Option<PathBuf>,
    /// IPC socket path
    pub ipc_socket: Option<PathBuf>,
}

impl Default for AdvancedConfig {
    fn default() -> Self {
        Self {
            debug: false,
            log_level: "info".to_string(),
            log_file: None,
            experimental: false,
            custom_css: None,
            plugin_dir: None,
            ipc_socket: None,
        }
    }
}

/// Configuration manager with hot-reloading
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    config_path: PathBuf,
    watcher: Option<notify::RecommendedWatcher>,
}

impl ConfigManager {
    /// Create a new config manager
    pub fn new() -> anyhow::Result<Self> {
        let config_path = config_dir().join("config.toml");

        let config = if config_path.exists() {
            Self::load_from_file(&config_path)?
        } else {
            Config::default()
        };

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            watcher: None,
        })
    }

    /// Load configuration from file
    fn load_from_file(path: &PathBuf) -> anyhow::Result<Config> {
        let figment = Figment::new()
            .merge(Toml::file(path))
            .merge(Env::prefixed("CORGITERM_"));

        Ok(figment.extract()?)
    }

    /// Get current configuration
    pub fn config(&self) -> Config {
        self.config.read().clone()
    }

    /// Update configuration
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut Config),
    {
        let mut config = self.config.write();
        f(&mut config);
    }

    /// Save configuration to file
    pub fn save(&self) -> anyhow::Result<()> {
        let config = self.config.read();
        let content = toml::to_string_pretty(&*config)?;

        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    /// Enable hot-reloading of configuration
    pub fn enable_hot_reload(&mut self) -> anyhow::Result<()> {
        let config = Arc::clone(&self.config);
        let config_path = self.config_path.clone();

        let mut watcher = recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                if event.kind.is_modify() {
                    if let Ok(new_config) = Self::load_from_file(&config_path) {
                        *config.write() = new_config;
                        tracing::info!("Configuration reloaded");
                    }
                }
            }
        })?;

        watcher.watch(&self.config_path, RecursiveMode::NonRecursive)?;
        self.watcher = Some(watcher);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.appearance.font_size, 11.0);
        assert_eq!(config.terminal.scrollback_lines, 10000);
        assert!(!config.general.telemetry);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("[general]"));
        assert!(toml.contains("[appearance]"));
    }
}
