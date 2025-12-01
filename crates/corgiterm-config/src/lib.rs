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
    /// SSH configuration
    pub ssh: SshConfig,
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
            ssh: SshConfig::default(),
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeybindingsConfig {
    /// Custom keybindings
    pub bindings: Vec<Keybinding>,
    /// Configurable shortcuts
    pub shortcuts: ShortcutsConfig,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        Self {
            bindings: Vec::new(),
            shortcuts: ShortcutsConfig::default(),
        }
    }
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
    pub snippets: Option<String>,
    pub ascii_art: Option<String>,
    pub open_file: Option<String>,

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
            snippets: Some("Ctrl+Shift+S".to_string()),
            ascii_art: Some("Ctrl+Shift+G".to_string()),
            open_file: Some("Ctrl+Shift+O".to_string()),

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
    /// The command to execute (supports {{var_name}} placeholders)
    pub command: String,
    /// Optional description
    pub description: String,
    /// Tags for organization and searching
    pub tags: Vec<String>,
    /// Hierarchical category (e.g., "Git/Commit", "Docker/Build")
    #[serde(default)]
    pub category: String,
    /// Pinned/favorite snippet
    #[serde(default)]
    pub pinned: bool,
    /// Creation timestamp
    pub created_at: i64,
    /// Last used timestamp
    pub last_used: Option<i64>,
    /// Usage count
    pub use_count: u32,
}

/// A variable placeholder extracted from a snippet command
#[derive(Debug, Clone, PartialEq)]
pub struct SnippetVariable {
    /// Variable name (without braces)
    pub name: String,
    /// Default value if specified (e.g., {{var:default}})
    pub default: Option<String>,
    /// Description/hint if specified (e.g., {{var|hint}})
    pub hint: Option<String>,
}

impl Snippet {
    /// Create a new snippet
    pub fn new(name: String, command: String, description: String, tags: Vec<String>) -> Self {
        Self::with_category(name, command, description, tags, String::new())
    }

    /// Create a new snippet with category
    pub fn with_category(
        name: String,
        command: String,
        description: String,
        tags: Vec<String>,
        category: String,
    ) -> Self {
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
            category,
            pinned: false,
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

    /// Extract variable placeholders from the command
    /// Supports formats:
    /// - {{var}} - simple variable
    /// - {{var:default}} - with default value
    /// - {{var|hint}} - with input hint
    /// - {{var:default|hint}} - with both
    pub fn extract_variables(&self) -> Vec<SnippetVariable> {
        use std::collections::HashSet;

        let re = regex::Regex::new(r"\{\{([^}]+)\}\}").unwrap();
        let mut seen = HashSet::new();
        let mut vars = Vec::new();

        for cap in re.captures_iter(&self.command) {
            let inner = cap.get(1).unwrap().as_str();

            // Parse: name[:default][|hint]
            let (name_part, hint) = if let Some(idx) = inner.find('|') {
                (&inner[..idx], Some(inner[idx + 1..].to_string()))
            } else {
                (inner, None)
            };

            let (name, default) = if let Some(idx) = name_part.find(':') {
                (
                    name_part[..idx].trim().to_string(),
                    Some(name_part[idx + 1..].trim().to_string()),
                )
            } else {
                (name_part.trim().to_string(), None)
            };

            // Skip duplicates
            if seen.insert(name.clone()) {
                vars.push(SnippetVariable {
                    name,
                    default,
                    hint,
                });
            }
        }

        vars
    }

    /// Check if this snippet has variable placeholders
    pub fn has_variables(&self) -> bool {
        self.command.contains("{{") && self.command.contains("}}")
    }

    /// Substitute variables in the command with provided values
    /// Returns the command with all {{var}} replaced
    pub fn substitute_variables(
        &self,
        values: &std::collections::HashMap<String, String>,
    ) -> String {
        let re = regex::Regex::new(r"\{\{([^}]+)\}\}").unwrap();

        re.replace_all(&self.command, |caps: &regex::Captures| {
            let inner = caps.get(1).unwrap().as_str();

            // Parse name (strip default/hint)
            let name_part = inner.split('|').next().unwrap_or(inner);
            let name = name_part.split(':').next().unwrap_or(name_part).trim();

            // Look up value, fall back to default, or keep original
            if let Some(val) = values.get(name) {
                val.clone()
            } else {
                // Try default
                if let Some(idx) = name_part.find(':') {
                    let default = name_part[idx + 1..].trim();
                    default.to_string()
                } else {
                    format!("{{{{{}}}}}", name) // Keep placeholder if no value
                }
            }
        })
        .to_string()
    }

    /// Get the category parts as a vector (split by "/")
    pub fn category_parts(&self) -> Vec<&str> {
        if self.category.is_empty() {
            vec![]
        } else {
            self.category.split('/').collect()
        }
    }

    /// Get the top-level category
    pub fn top_category(&self) -> Option<&str> {
        self.category_parts().first().copied()
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

    /// Get pinned snippets (order by name)
    pub fn pinned(&self) -> Vec<&Snippet> {
        let mut pinned: Vec<&Snippet> = self.snippets.iter().filter(|s| s.pinned).collect();
        pinned.sort_by(|a, b| a.name.cmp(&b.name));
        pinned
    }

    /// List unique tags sorted alphabetically
    pub fn tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self.snippets.iter().flat_map(|s| s.tags.clone()).collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// Filter snippets by tag
    pub fn with_tag(&self, tag: &str) -> Vec<&Snippet> {
        let tag_lower = tag.to_lowercase();
        let mut filtered: Vec<&Snippet> = self
            .snippets
            .iter()
            .filter(|s| s.tags.iter().any(|t| t.to_lowercase() == tag_lower))
            .collect();
        filtered.sort_by(|a, b| a.name.cmp(&b.name));
        filtered
    }

    /// List unique categories sorted alphabetically
    pub fn categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self
            .snippets
            .iter()
            .filter(|s| !s.category.is_empty())
            .map(|s| s.category.clone())
            .collect();
        cats.sort();
        cats.dedup();
        cats
    }

    /// List unique top-level categories
    pub fn top_categories(&self) -> Vec<String> {
        let mut cats: Vec<String> = self
            .snippets
            .iter()
            .filter_map(|s| s.top_category().map(|c| c.to_string()))
            .collect();
        cats.sort();
        cats.dedup();
        cats
    }

    /// Filter snippets by category (exact match or prefix)
    pub fn with_category(&self, category: &str) -> Vec<&Snippet> {
        let cat_lower = category.to_lowercase();
        let mut filtered: Vec<&Snippet> = self
            .snippets
            .iter()
            .filter(|s| {
                let s_cat = s.category.to_lowercase();
                s_cat == cat_lower || s_cat.starts_with(&format!("{}/", cat_lower))
            })
            .collect();
        filtered.sort_by(|a, b| a.name.cmp(&b.name));
        filtered
    }

    /// Get snippets without a category (uncategorized)
    pub fn uncategorized(&self) -> Vec<&Snippet> {
        let mut filtered: Vec<&Snippet> = self
            .snippets
            .iter()
            .filter(|s| s.category.is_empty())
            .collect();
        filtered.sort_by(|a, b| a.name.cmp(&b.name));
        filtered
    }

    /// Build a category tree structure for UI display
    /// Returns Vec of (category_path, depth, snippet_count)
    pub fn category_tree(&self) -> Vec<(String, usize, usize)> {
        use std::collections::BTreeMap;

        let mut tree: BTreeMap<String, usize> = BTreeMap::new();

        for snippet in &self.snippets {
            if snippet.category.is_empty() {
                continue;
            }

            // Add all parent categories too
            let parts: Vec<&str> = snippet.category.split('/').collect();
            let mut path = String::new();
            for (i, part) in parts.iter().enumerate() {
                if i > 0 {
                    path.push('/');
                }
                path.push_str(part);
                *tree.entry(path.clone()).or_insert(0) += 0;
            }
            // Increment count for the full path
            *tree.entry(snippet.category.clone()).or_insert(0) += 1;
        }

        tree.into_iter()
            .map(|(path, count)| {
                let depth = path.matches('/').count();
                (path, depth, count)
            })
            .collect()
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

/// Parse a basic subset of ~/.ssh/config entries into SshHost definitions.
/// This is intentionally conservative: Host/HostName/User/Port/IdentityFile are mapped; other options go to `options`.
pub fn parse_ssh_config(contents: &str, default_port: u16) -> Vec<SshHost> {
    use std::path::PathBuf;
    let mut hosts = Vec::new();
    let mut current: Option<SshHost> = None;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let mut parts = trimmed.split_whitespace();
        let key = parts.next().unwrap_or_default();
        let value = parts.collect::<Vec<&str>>().join(" ");

        match key.to_lowercase().as_str() {
            "host" => {
                if let Some(host) = current.take() {
                    hosts.push(host);
                }
                let name = value.trim().to_string();
                current = Some(SshHost {
                    name: name.clone(),
                    hostname: name,
                    port: default_port,
                    username: None,
                    identity_file: None,
                    options: Vec::new(),
                    tags: Vec::new(),
                    favorite: false,
                });
            }
            "hostname" => {
                if let Some(ref mut host) = current {
                    host.hostname = value.trim().to_string();
                }
            }
            "user" => {
                if let Some(ref mut host) = current {
                    host.username = Some(value.trim().to_string());
                }
            }
            "port" => {
                if let Some(ref mut host) = current {
                    if let Ok(port) = value.trim().parse::<u16>() {
                        host.port = port;
                    }
                }
            }
            "identityfile" => {
                if let Some(ref mut host) = current {
                    host.identity_file = Some(PathBuf::from(value.trim()));
                }
            }
            _ => {
                if let Some(ref mut host) = current {
                    host.options.push(format!("{} {}", key, value));
                }
            }
        }
    }

    if let Some(host) = current {
        hosts.push(host);
    }

    hosts
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

    /// Get pinned snippets
    pub fn pinned(&self) -> Vec<Snippet> {
        self.snippets.read().pinned().into_iter().cloned().collect()
    }

    /// Toggle pinned flag for a snippet
    pub fn set_pinned(&self, id: &str, pinned: bool) -> anyhow::Result<()> {
        if let Some(snippet) = self.snippets.write().find_mut(id) {
            snippet.pinned = pinned;
            self.save()?;
        }
        Ok(())
    }

    /// Get snippets filtered by tag (case-insensitive)
    pub fn with_tag(&self, tag: &str) -> Vec<Snippet> {
        self.snippets
            .read()
            .with_tag(tag)
            .into_iter()
            .cloned()
            .collect()
    }

    /// List all unique tags
    pub fn tags(&self) -> Vec<String> {
        self.snippets.read().tags()
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

    /// Export snippets to a specific path
    pub fn export_to_path(&self, path: &PathBuf) -> anyhow::Result<()> {
        let snippets = self.snippets.read();
        let content = serde_json::to_string_pretty(&*snippets)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Import snippets from a path. If replace is false, merges new snippets by ID.
    pub fn import_from_path(&self, path: &PathBuf, replace: bool) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(path)?;
        let incoming: SnippetsConfig = serde_json::from_str(&content)?;

        if replace {
            *self.snippets.write() = incoming;
        } else {
            let mut guard = self.snippets.write();
            for snippet in incoming.snippets {
                if guard.find(&snippet.id).is_none() {
                    guard.add(snippet);
                }
            }
        }

        self.save()
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
    /// Auto-merge tags and known_hosts aliases on import
    pub merge_imports: bool,
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            hosts: Vec::new(),
            auto_import: true,
            default_port: 22,
            merge_imports: true,
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
    /// Favorite host
    #[serde(default)]
    pub favorite: bool,
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

    /// Merge tags and options from another host with same hostname/username/port
    pub fn merge_from(&mut self, other: &SshHost) {
        // Merge tags
        for tag in &other.tags {
            if !self.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                self.tags.push(tag.clone());
            }
        }
        // Merge options
        for opt in &other.options {
            if !self.options.contains(opt) {
                self.options.push(opt.clone());
            }
        }
        // Favor identity file if missing
        if self.identity_file.is_none() {
            self.identity_file = other.identity_file.clone();
        }
        // Favorite flag: keep true if either is true
        self.favorite = self.favorite || other.favorite;
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
            favorite: false,
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
            favorite: false,
        };

        assert_eq!(host.display_string(), "user@example.com:22");
    }

    #[test]
    fn test_merge_host() {
        let mut base = SshHost {
            name: "Base".to_string(),
            hostname: "example.com".to_string(),
            port: 22,
            username: Some("user".to_string()),
            identity_file: None,
            options: vec!["-o StrictHostKeyChecking=no".to_string()],
            tags: vec!["prod".to_string()],
            favorite: false,
        };

        let other = SshHost {
            name: "Other".to_string(),
            hostname: "example.com".to_string(),
            port: 22,
            username: Some("user".to_string()),
            identity_file: Some(PathBuf::from("/id_ed25519")),
            options: vec!["-o LogLevel=ERROR".to_string()],
            tags: vec!["db".to_string()],
            favorite: true,
        };

        base.merge_from(&other);
        assert_eq!(base.identity_file, other.identity_file);
        assert!(base.tags.contains(&"prod".to_string()));
        assert!(base.tags.contains(&"db".to_string()));
        assert!(base
            .options
            .contains(&"-o StrictHostKeyChecking=no".to_string()));
        assert!(base.options.contains(&"-o LogLevel=ERROR".to_string()));
        assert!(base.favorite);
    }

    #[test]
    fn test_parse_basic_ssh_config() {
        let cfg = r#"
Host mybox
  HostName example.com
  User alice
  Port 2200
  IdentityFile ~/.ssh/id_ed25519
  ForwardAgent yes
        "#;

        let hosts = parse_ssh_config(cfg, 22);
        assert_eq!(hosts.len(), 1);
        let h = &hosts[0];
        assert_eq!(h.name, "mybox");
        assert_eq!(h.hostname, "example.com");
        assert_eq!(h.username.as_deref(), Some("alice"));
        assert_eq!(h.port, 2200);
        assert!(h.identity_file.is_some());
        assert!(h.options.iter().any(|o| o.contains("ForwardAgent")));
    }
}
