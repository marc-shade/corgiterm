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
pub mod shortcuts;
pub mod themes;

use directories::ProjectDirs;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use notify::{recommended_watcher, Event, RecursiveMode, Watcher};
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    /// SSH configuration
    pub ssh: SshConfig,
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
    /// Configurable shortcuts
    pub shortcuts: ShortcutsConfig,
}

/// A single keybinding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinding {
    pub key: String,
    pub mods: Vec<String>,
    pub action: String,
}

/// Configurable keyboard shortcuts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShortcutsConfig {
    // Tab management
    pub new_tab: Option<String>,
    pub close_tab: Option<String>,
    pub next_tab: Option<String>,
    pub prev_tab: Option<String>,
    pub new_document_tab: Option<String>,

    // Tab switching (1-9)
    pub switch_to_tab_1: Option<String>,
    pub switch_to_tab_2: Option<String>,
    pub switch_to_tab_3: Option<String>,
    pub switch_to_tab_4: Option<String>,
    pub switch_to_tab_5: Option<String>,
    pub switch_to_tab_6: Option<String>,
    pub switch_to_tab_7: Option<String>,
    pub switch_to_tab_8: Option<String>,
    pub switch_to_tab_9: Option<String>,

    // Pane management
    pub split_horizontal: Option<String>,
    pub split_vertical: Option<String>,
    pub close_pane: Option<String>,
    pub focus_next_pane: Option<String>,
    pub focus_prev_pane: Option<String>,

    // UI features
    pub toggle_ai: Option<String>,
    pub quick_switcher: Option<String>,
    pub ssh_manager: Option<String>,
    pub emoji_picker: Option<String>,
    pub open_file: Option<String>,
    pub toggle_broadcast: Option<String>,

    // Application
    pub quit: Option<String>,
}

impl Default for ShortcutsConfig {
    fn default() -> Self {
        Self {
            // Tab management
            new_tab: Some("Ctrl+T".to_string()),
            close_tab: Some("Ctrl+W".to_string()),
            next_tab: Some("Ctrl+Tab".to_string()),
            prev_tab: Some("Ctrl+Shift+Tab".to_string()),
            new_document_tab: Some("Ctrl+O".to_string()),

            // Tab switching
            switch_to_tab_1: Some("Ctrl+1".to_string()),
            switch_to_tab_2: Some("Ctrl+2".to_string()),
            switch_to_tab_3: Some("Ctrl+3".to_string()),
            switch_to_tab_4: Some("Ctrl+4".to_string()),
            switch_to_tab_5: Some("Ctrl+5".to_string()),
            switch_to_tab_6: Some("Ctrl+6".to_string()),
            switch_to_tab_7: Some("Ctrl+7".to_string()),
            switch_to_tab_8: Some("Ctrl+8".to_string()),
            switch_to_tab_9: Some("Ctrl+9".to_string()),

            // Pane management
            split_horizontal: Some("Ctrl+Shift+H".to_string()),
            split_vertical: Some("Ctrl+Shift+D".to_string()),
            close_pane: Some("Ctrl+Shift+W".to_string()),
            focus_next_pane: Some("Ctrl+Shift+]".to_string()),
            focus_prev_pane: Some("Ctrl+Shift+[".to_string()),

            // UI features
            toggle_ai: Some("Ctrl+Shift+A".to_string()),
            quick_switcher: Some("Ctrl+K".to_string()),
            ssh_manager: Some("Ctrl+S".to_string()),
            emoji_picker: Some("Ctrl+Shift+E".to_string()),
            open_file: Some("Ctrl+Shift+O".to_string()),
            toggle_broadcast: Some("Ctrl+Shift+B".to_string()),

            // Application
            quit: Some("Ctrl+Q".to_string()),
        }
    }
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
    /// Command learning settings
    pub learning: LearningConfig,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_provider: "auto".to_string(), // Auto-detect best available provider
            claude: ClaudeConfig::default(),
            openai: OpenAiConfig::default(),
            gemini: GeminiConfig::default(),
            local: LocalLlmConfig::default(),
            natural_language: true,
            auto_suggest: true,
            show_panel: false,
            panel_position: AiPanelPosition::Right,
            learning: LearningConfig::default(),
        }
    }
}

/// Command learning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LearningConfig {
    /// Enable command learning
    pub enabled: bool,
    /// Maximum history size for learning
    pub max_history: usize,
    /// Minimum pattern frequency for detection
    pub min_pattern_frequency: usize,
    /// Maximum pattern length
    pub max_pattern_length: usize,
    /// Learning window size (number of recent commands)
    pub window_size: usize,
    /// Auto-detect user preferences (e.g., exa vs ls)
    pub detect_preferences: bool,
    /// Suggest next command based on patterns
    pub suggest_next: bool,
    /// Show directory-specific suggestions
    pub directory_suggestions: bool,
    /// Privacy: opt-out of learning
    pub opt_out: bool,
    /// Path to learning data file
    pub data_path: Option<PathBuf>,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_history: 10000,
            min_pattern_frequency: 3,
            max_pattern_length: 5,
            window_size: 100,
            detect_preferences: true,
            suggest_next: true,
            directory_suggestions: true,
            opt_out: false,
            data_path: None, // Will default to config_dir/learning.json
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
            enabled: true, // Enable by default - auto-detects if Ollama is running
            endpoint: "http://localhost:11434".to_string(), // Standard Ollama port
            model: "codellama".to_string(), // Common model for command generation
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

/// Snippet for commonly used commands
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snippet {
    /// Unique identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// The command to execute
    pub command: String,
    /// Optional description
    pub description: String,
    /// Tags for organization and searching
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: i64,
    /// Last used timestamp
    pub last_used: Option<i64>,
    /// Usage count
    pub use_count: u32,
}

impl Snippet {
    /// Create a new snippet
    pub fn new(name: String, command: String, description: String, tags: Vec<String>) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            command,
            description,
            tags,
            created_at: timestamp,
            last_used: None,
            use_count: 0,
        }
    }

    /// Record usage of this snippet
    pub fn record_use(&mut self) {
        use std::time::{SystemTime, UNIX_EPOCH};

        self.use_count += 1;
        self.last_used = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        );
    }
}

/// Snippets configuration and storage
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SnippetsConfig {
    /// Collection of snippets
    pub snippets: Vec<Snippet>,
}

impl SnippetsConfig {
    /// Add a new snippet
    pub fn add(&mut self, snippet: Snippet) {
        self.snippets.push(snippet);
    }

    /// Remove a snippet by ID
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(pos) = self.snippets.iter().position(|s| s.id == id) {
            self.snippets.remove(pos);
            true
        } else {
            false
        }
    }

    /// Update a snippet
    pub fn update(&mut self, snippet: Snippet) -> bool {
        if let Some(existing) = self.snippets.iter_mut().find(|s| s.id == snippet.id) {
            *existing = snippet;
            true
        } else {
            false
        }
    }

    /// Find a snippet by ID
    pub fn find(&self, id: &str) -> Option<&Snippet> {
        self.snippets.iter().find(|s| s.id == id)
    }

    /// Find a snippet by ID (mutable)
    pub fn find_mut(&mut self, id: &str) -> Option<&mut Snippet> {
        self.snippets.iter_mut().find(|s| s.id == id)
    }

    /// Search snippets by name or tags (fuzzy)
    pub fn search(&self, query: &str) -> Vec<&Snippet> {
        let query_lower = query.to_lowercase();

        self.snippets
            .iter()
            .filter(|s| {
                s.name.to_lowercase().contains(&query_lower)
                    || s.description.to_lowercase().contains(&query_lower)
                    || s.command.to_lowercase().contains(&query_lower)
                    || s.tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Get snippets sorted by usage (most used first)
    pub fn by_usage(&self) -> Vec<&Snippet> {
        let mut sorted: Vec<&Snippet> = self.snippets.iter().collect();
        sorted.sort_by(|a, b| b.use_count.cmp(&a.use_count));
        sorted
    }

    /// Get snippets sorted by recency (most recently used first)
    pub fn by_recency(&self) -> Vec<&Snippet> {
        let mut sorted: Vec<&Snippet> = self.snippets.iter().collect();
        sorted.sort_by(|a, b| match (b.last_used, a.last_used) {
            (Some(b_time), Some(a_time)) => b_time.cmp(&a_time),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => b.created_at.cmp(&a.created_at),
        });
        sorted
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

/// Snippets manager with file storage
pub struct SnippetsManager {
    snippets: Arc<RwLock<SnippetsConfig>>,
    snippets_path: PathBuf,
}

impl SnippetsManager {
    /// Create a new snippets manager
    pub fn new() -> anyhow::Result<Self> {
        let snippets_path = config_dir().join("snippets.json");

        let snippets = if snippets_path.exists() {
            Self::load_from_file(&snippets_path)?
        } else {
            SnippetsConfig::default()
        };

        Ok(Self {
            snippets: Arc::new(RwLock::new(snippets)),
            snippets_path,
        })
    }

    /// Load snippets from file
    fn load_from_file(path: &PathBuf) -> anyhow::Result<SnippetsConfig> {
        let content = std::fs::read_to_string(path)?;
        let snippets: SnippetsConfig = serde_json::from_str(&content)?;
        Ok(snippets)
    }

    /// Get all snippets
    pub fn snippets(&self) -> SnippetsConfig {
        self.snippets.read().clone()
    }

    /// Add a new snippet
    pub fn add(&self, snippet: Snippet) -> anyhow::Result<()> {
        self.snippets.write().add(snippet);
        self.save()
    }

    /// Remove a snippet by ID
    pub fn remove(&self, id: &str) -> anyhow::Result<bool> {
        let removed = self.snippets.write().remove(id);
        if removed {
            self.save()?;
        }
        Ok(removed)
    }

    /// Update a snippet
    pub fn update(&self, snippet: Snippet) -> anyhow::Result<bool> {
        let updated = self.snippets.write().update(snippet);
        if updated {
            self.save()?;
        }
        Ok(updated)
    }

    /// Record usage of a snippet
    pub fn record_use(&self, id: &str) -> anyhow::Result<()> {
        if let Some(snippet) = self.snippets.write().find_mut(id) {
            snippet.record_use();
            self.save()?;
        }
        Ok(())
    }

    /// Search snippets
    pub fn search(&self, query: &str) -> Vec<Snippet> {
        self.snippets
            .read()
            .search(query)
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get snippets by usage
    pub fn by_usage(&self) -> Vec<Snippet> {
        self.snippets
            .read()
            .by_usage()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Get snippets by recency
    pub fn by_recency(&self) -> Vec<Snippet> {
        self.snippets
            .read()
            .by_recency()
            .into_iter()
            .cloned()
            .collect()
    }

    /// Save snippets to file
    pub fn save(&self) -> anyhow::Result<()> {
        let snippets = self.snippets.read();
        let content = serde_json::to_string_pretty(&*snippets)?;

        if let Some(parent) = self.snippets_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&self.snippets_path, content)?;
        Ok(())
    }
}

/// SSH configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SshConfig {
    /// Saved SSH hosts
    pub hosts: Vec<SshHost>,
    /// Auto-import from ~/.ssh/config
    pub auto_import: bool,
    /// Default port
    pub default_port: u16,
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            hosts: Vec::new(),
            auto_import: true,
            default_port: 22,
        }
    }
}

/// A saved SSH host
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SshHost {
    /// Display name for this connection
    pub name: String,
    /// Hostname or IP address
    pub hostname: String,
    /// SSH port (default: 22)
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    /// Username for login
    pub username: Option<String>,
    /// Path to identity file (private key)
    pub identity_file: Option<PathBuf>,
    /// Additional SSH options
    #[serde(default)]
    pub options: Vec<String>,
    /// Tags for organization
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_ssh_port() -> u16 {
    22
}

impl SshHost {
    /// Build SSH command from this host configuration
    pub fn build_command(&self) -> Vec<String> {
        let mut cmd = vec!["ssh".to_string()];

        // Add port if not default
        if self.port != 22 {
            cmd.push("-p".to_string());
            cmd.push(self.port.to_string());
        }

        // Add identity file if specified
        if let Some(ref identity) = self.identity_file {
            cmd.push("-i".to_string());
            cmd.push(identity.display().to_string());
        }

        // Add custom options
        for opt in &self.options {
            cmd.push(opt.clone());
        }

        // Build user@host string
        let target = if let Some(ref user) = self.username {
            format!("{}@{}", user, self.hostname)
        } else {
            self.hostname.clone()
        };
        cmd.push(target);

        cmd
    }

    /// Get display string for this host
    pub fn display_string(&self) -> String {
        if let Some(ref user) = self.username {
            format!("{}@{}:{}", user, self.hostname, self.port)
        } else {
            format!("{}:{}", self.hostname, self.port)
        }
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

    #[test]
    fn test_ssh_host_command() {
        let host = SshHost {
            name: "Test Server".to_string(),
            hostname: "example.com".to_string(),
            port: 2222,
            username: Some("user".to_string()),
            identity_file: Some(PathBuf::from("/home/user/.ssh/id_rsa")),
            options: vec!["-o".to_string(), "StrictHostKeyChecking=no".to_string()],
            tags: vec!["production".to_string()],
        };

        let cmd = host.build_command();
        assert_eq!(cmd[0], "ssh");
        assert!(cmd.contains(&"-p".to_string()));
        assert!(cmd.contains(&"2222".to_string()));
        assert!(cmd.contains(&"-i".to_string()));
        assert_eq!(*cmd.last().unwrap(), "user@example.com");
    }

    #[test]
    fn test_ssh_host_display() {
        let host = SshHost {
            name: "Test".to_string(),
            hostname: "example.com".to_string(),
            port: 22,
            username: Some("user".to_string()),
            identity_file: None,
            options: Vec::new(),
            tags: Vec::new(),
        };

        assert_eq!(host.display_string(), "user@example.com:22");
    }
}
