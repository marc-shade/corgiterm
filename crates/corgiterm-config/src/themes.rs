//! Theme definitions and management
//!
//! CorgiTerm comes with beautiful built-in themes including
//! the signature "Corgi Collection" themes.

use palette::Srgba;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete color theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme name
    pub name: String,
    /// Theme author
    pub author: Option<String>,
    /// Theme description
    pub description: Option<String>,
    /// Is this a dark theme?
    pub is_dark: bool,
    /// Terminal colors
    pub colors: TerminalColors,
    /// UI colors
    pub ui: UiColors,
    /// Cursor colors
    pub cursor: CursorColors,
    /// Selection colors
    pub selection: SelectionColors,
}

/// Standard terminal colors (16 ANSI + extras)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalColors {
    /// Primary foreground
    pub foreground: String,
    /// Primary background
    pub background: String,
    /// Bold text color
    pub bold: Option<String>,
    /// Dim text color
    pub dim: Option<String>,
    /// ANSI color 0 (Black)
    pub black: String,
    /// ANSI color 1 (Red)
    pub red: String,
    /// ANSI color 2 (Green)
    pub green: String,
    /// ANSI color 3 (Yellow)
    pub yellow: String,
    /// ANSI color 4 (Blue)
    pub blue: String,
    /// ANSI color 5 (Magenta)
    pub magenta: String,
    /// ANSI color 6 (Cyan)
    pub cyan: String,
    /// ANSI color 7 (White)
    pub white: String,
    /// ANSI color 8 (Bright Black)
    pub bright_black: String,
    /// ANSI color 9 (Bright Red)
    pub bright_red: String,
    /// ANSI color 10 (Bright Green)
    pub bright_green: String,
    /// ANSI color 11 (Bright Yellow)
    pub bright_yellow: String,
    /// ANSI color 12 (Bright Blue)
    pub bright_blue: String,
    /// ANSI color 13 (Bright Magenta)
    pub bright_magenta: String,
    /// ANSI color 14 (Bright Cyan)
    pub bright_cyan: String,
    /// ANSI color 15 (Bright White)
    pub bright_white: String,
}

/// UI element colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiColors {
    /// Sidebar background
    pub sidebar_bg: String,
    /// Sidebar text
    pub sidebar_fg: String,
    /// Tab bar background
    pub tab_bar_bg: String,
    /// Active tab background
    pub tab_active_bg: String,
    /// Active tab text
    pub tab_active_fg: String,
    /// Inactive tab background
    pub tab_inactive_bg: String,
    /// Inactive tab text
    pub tab_inactive_fg: String,
    /// Border color
    pub border: String,
    /// Accent color
    pub accent: String,
    /// Success color
    pub success: String,
    /// Warning color
    pub warning: String,
    /// Error color
    pub error: String,
    /// Info color
    pub info: String,
}

/// Cursor colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorColors {
    /// Cursor color
    pub cursor: String,
    /// Cursor text color
    pub cursor_text: String,
}

/// Selection colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionColors {
    /// Selection background
    pub background: String,
    /// Selection text
    pub foreground: Option<String>,
}

impl Theme {
    /// Corgi Dark - The signature dark theme
    pub fn corgi_dark() -> Self {
        Self {
            name: "Corgi Dark".to_string(),
            author: Some("CorgiTerm Team".to_string()),
            description: Some("The signature dark theme with warm corgi tones".to_string()),
            is_dark: true,
            colors: TerminalColors {
                foreground: "#E8DCC4".to_string(),
                background: "#1E1B16".to_string(),
                bold: Some("#FFEFD5".to_string()),
                dim: Some("#8B7355".to_string()),
                black: "#1E1B16".to_string(),
                red: "#E06C60".to_string(),
                green: "#A8C686".to_string(),
                yellow: "#E5C07B".to_string(),
                blue: "#6CA0DC".to_string(),
                magenta: "#C792EA".to_string(),
                cyan: "#7EC8A3".to_string(),
                white: "#E8DCC4".to_string(),
                bright_black: "#5C5346".to_string(),
                bright_red: "#F28779".to_string(),
                bright_green: "#C3E88D".to_string(),
                bright_yellow: "#FFCB6B".to_string(),
                bright_blue: "#82AAFF".to_string(),
                bright_magenta: "#D4BFFF".to_string(),
                bright_cyan: "#89DDFF".to_string(),
                bright_white: "#FFEFD5".to_string(),
            },
            ui: UiColors {
                sidebar_bg: "#16140F".to_string(),
                sidebar_fg: "#C9B896".to_string(),
                tab_bar_bg: "#1E1B16".to_string(),
                tab_active_bg: "#2D2A23".to_string(),
                tab_active_fg: "#FFEFD5".to_string(),
                tab_inactive_bg: "#1E1B16".to_string(),
                tab_inactive_fg: "#8B7355".to_string(),
                border: "#3D3A33".to_string(),
                accent: "#E5A84B".to_string(),
                success: "#A8C686".to_string(),
                warning: "#E5C07B".to_string(),
                error: "#E06C60".to_string(),
                info: "#6CA0DC".to_string(),
            },
            cursor: CursorColors {
                cursor: "#E5A84B".to_string(),
                cursor_text: "#1E1B16".to_string(),
            },
            selection: SelectionColors {
                background: "#4D4536".to_string(),
                foreground: None,
            },
        }
    }

    /// Corgi Light - The signature light theme
    pub fn corgi_light() -> Self {
        Self {
            name: "Corgi Light".to_string(),
            author: Some("CorgiTerm Team".to_string()),
            description: Some("A warm, inviting light theme".to_string()),
            is_dark: false,
            colors: TerminalColors {
                foreground: "#3D3A33".to_string(),
                background: "#FFF8EE".to_string(),
                bold: Some("#1E1B16".to_string()),
                dim: Some("#8B7355".to_string()),
                black: "#3D3A33".to_string(),
                red: "#C74440".to_string(),
                green: "#6A8F4A".to_string(),
                yellow: "#B8860B".to_string(),
                blue: "#4A7AAF".to_string(),
                magenta: "#8B5A9E".to_string(),
                cyan: "#2A9D8F".to_string(),
                white: "#FFF8EE".to_string(),
                bright_black: "#6B6359".to_string(),
                bright_red: "#D65D55".to_string(),
                bright_green: "#7FA863".to_string(),
                bright_yellow: "#D4A017".to_string(),
                bright_blue: "#5D8EC4".to_string(),
                bright_magenta: "#A06DB5".to_string(),
                bright_cyan: "#3EB4A5".to_string(),
                bright_white: "#FFFFFF".to_string(),
            },
            ui: UiColors {
                sidebar_bg: "#F5EBD9".to_string(),
                sidebar_fg: "#3D3A33".to_string(),
                tab_bar_bg: "#FFF8EE".to_string(),
                tab_active_bg: "#FFFFFF".to_string(),
                tab_active_fg: "#1E1B16".to_string(),
                tab_inactive_bg: "#F5EBD9".to_string(),
                tab_inactive_fg: "#6B6359".to_string(),
                border: "#E5D9C3".to_string(),
                accent: "#D4881C".to_string(),
                success: "#6A8F4A".to_string(),
                warning: "#B8860B".to_string(),
                error: "#C74440".to_string(),
                info: "#4A7AAF".to_string(),
            },
            cursor: CursorColors {
                cursor: "#D4881C".to_string(),
                cursor_text: "#FFF8EE".to_string(),
            },
            selection: SelectionColors {
                background: "#E5D9C3".to_string(),
                foreground: None,
            },
        }
    }

    /// Corgi Sunset - Warm orange/pink tones
    pub fn corgi_sunset() -> Self {
        Self {
            name: "Corgi Sunset".to_string(),
            author: Some("CorgiTerm Team".to_string()),
            description: Some("Warm sunset colors for evening coding".to_string()),
            is_dark: true,
            colors: TerminalColors {
                foreground: "#F4E3CF".to_string(),
                background: "#1F1520".to_string(),
                bold: Some("#FFE4C4".to_string()),
                dim: Some("#9E7B89".to_string()),
                black: "#1F1520".to_string(),
                red: "#FF6B6B".to_string(),
                green: "#95D5B2".to_string(),
                yellow: "#FFD93D".to_string(),
                blue: "#6A9BD8".to_string(),
                magenta: "#E879A9".to_string(),
                cyan: "#74C7B8".to_string(),
                white: "#F4E3CF".to_string(),
                bright_black: "#614051".to_string(),
                bright_red: "#FF8585".to_string(),
                bright_green: "#B2E4C6".to_string(),
                bright_yellow: "#FFE566".to_string(),
                bright_blue: "#8BB8E8".to_string(),
                bright_magenta: "#F09AC0".to_string(),
                bright_cyan: "#93D8CC".to_string(),
                bright_white: "#FFE4C4".to_string(),
            },
            ui: UiColors {
                sidebar_bg: "#180F18".to_string(),
                sidebar_fg: "#C9A7B5".to_string(),
                tab_bar_bg: "#1F1520".to_string(),
                tab_active_bg: "#352535".to_string(),
                tab_active_fg: "#FFE4C4".to_string(),
                tab_inactive_bg: "#1F1520".to_string(),
                tab_inactive_fg: "#9E7B89".to_string(),
                border: "#4A3545".to_string(),
                accent: "#FF8C5A".to_string(),
                success: "#95D5B2".to_string(),
                warning: "#FFD93D".to_string(),
                error: "#FF6B6B".to_string(),
                info: "#6A9BD8".to_string(),
            },
            cursor: CursorColors {
                cursor: "#FF8C5A".to_string(),
                cursor_text: "#1F1520".to_string(),
            },
            selection: SelectionColors {
                background: "#4A3545".to_string(),
                foreground: None,
            },
        }
    }

    /// Pembroke - Named after Pembroke Welsh Corgi
    pub fn pembroke() -> Self {
        Self {
            name: "Pembroke".to_string(),
            author: Some("CorgiTerm Team".to_string()),
            description: Some("A regal theme inspired by the Pembroke Welsh Corgi".to_string()),
            is_dark: true,
            colors: TerminalColors {
                foreground: "#D4C5A9".to_string(),
                background: "#1A1614".to_string(),
                bold: Some("#EFE4C9".to_string()),
                dim: Some("#7A6E5D".to_string()),
                black: "#1A1614".to_string(),
                red: "#C75643".to_string(),
                green: "#8DAA6F".to_string(),
                yellow: "#D4A656".to_string(),
                blue: "#5B8FA8".to_string(),
                magenta: "#9E7682".to_string(),
                cyan: "#6B9E8A".to_string(),
                white: "#D4C5A9".to_string(),
                bright_black: "#5A524A".to_string(),
                bright_red: "#D66B5A".to_string(),
                bright_green: "#A4BF86".to_string(),
                bright_yellow: "#E4B86A".to_string(),
                bright_blue: "#72A4BC".to_string(),
                bright_magenta: "#B08C98".to_string(),
                bright_cyan: "#82B49E".to_string(),
                bright_white: "#EFE4C9".to_string(),
            },
            ui: UiColors {
                sidebar_bg: "#14110F".to_string(),
                sidebar_fg: "#B8AA90".to_string(),
                tab_bar_bg: "#1A1614".to_string(),
                tab_active_bg: "#282420".to_string(),
                tab_active_fg: "#EFE4C9".to_string(),
                tab_inactive_bg: "#1A1614".to_string(),
                tab_inactive_fg: "#7A6E5D".to_string(),
                border: "#383430".to_string(),
                accent: "#C78B52".to_string(),
                success: "#8DAA6F".to_string(),
                warning: "#D4A656".to_string(),
                error: "#C75643".to_string(),
                info: "#5B8FA8".to_string(),
            },
            cursor: CursorColors {
                cursor: "#C78B52".to_string(),
                cursor_text: "#1A1614".to_string(),
            },
            selection: SelectionColors {
                background: "#3D3832".to_string(),
                foreground: None,
            },
        }
    }
}

/// Theme manager for loading and switching themes
pub struct ThemeManager {
    themes: HashMap<String, Theme>,
    current: String,
}

impl ThemeManager {
    pub fn new() -> Self {
        let mut themes = HashMap::new();

        // Load built-in themes
        let builtins = vec![
            Theme::corgi_dark(),
            Theme::corgi_light(),
            Theme::corgi_sunset(),
            Theme::pembroke(),
        ];

        for theme in builtins {
            themes.insert(theme.name.clone(), theme);
        }

        Self {
            themes,
            current: "Corgi Dark".to_string(),
        }
    }

    /// Get current theme
    pub fn current(&self) -> &Theme {
        self.themes.get(&self.current).unwrap_or_else(|| {
            self.themes.values().next().expect("No themes available")
        })
    }

    /// Set current theme by name
    pub fn set_current(&mut self, name: &str) -> bool {
        if self.themes.contains_key(name) {
            self.current = name.to_string();
            true
        } else {
            false
        }
    }

    /// List all available themes
    pub fn list(&self) -> Vec<&str> {
        self.themes.keys().map(|s| s.as_str()).collect()
    }

    /// Load custom theme from file
    pub fn load_from_file(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(path)?;
        let theme: Theme = toml::from_str(&content)?;
        self.themes.insert(theme.name.clone(), theme);
        Ok(())
    }

    /// Get theme by name
    pub fn get(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name)
    }
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_themes() {
        let manager = ThemeManager::new();
        assert!(manager.list().contains(&"Corgi Dark"));
        assert!(manager.list().contains(&"Corgi Light"));
    }

    #[test]
    fn test_theme_switching() {
        let mut manager = ThemeManager::new();
        assert!(manager.set_current("Corgi Light"));
        assert_eq!(manager.current().name, "Corgi Light");
    }
}
