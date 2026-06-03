//! Keyboard shortcut handling
//!
//! Manages configurable keyboard shortcuts loaded from config.

use corgiterm_config::shortcuts::{matches_shortcut, parse_shortcut, ParsedShortcut};
use corgiterm_config::ShortcutsConfig;
use std::collections::HashMap;

/// Action types for keyboard shortcuts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShortcutAction {
    // Tab management
    NewTab,
    CloseTab,
    NextTab,
    PrevTab,
    NewDocumentTab,

    // Tab switching
    SwitchToTab1,
    SwitchToTab2,
    SwitchToTab3,
    SwitchToTab4,
    SwitchToTab5,
    SwitchToTab6,
    SwitchToTab7,
    SwitchToTab8,
    SwitchToTab9,

    // Pane management
    SplitHorizontal,
    SplitVertical,
    ClosePane,
    FocusNextPane,
    FocusPrevPane,

    // Terminal actions
    Copy,
    Paste,
    SelectAll,
    FindTerminal,
    ActivateHints,
    ZoomIn,
    ZoomOut,
    ResetZoom,

    // UI features
    ToggleAi,
    ToggleSidebar,
    QuickSwitcher,
    SshManager,
    Snippets,
    AsciiArt,
    OpenFile,
    HistorySearch,

    // Application
    Quit,
}

/// Display metadata for a configurable keyboard shortcut.
#[derive(Debug, Clone, Copy)]
pub struct ShortcutDefinition {
    pub action: ShortcutAction,
    pub group: &'static str,
    pub title: &'static str,
    pub description: &'static str,
}

pub const SHORTCUT_GROUPS: &[&str] = &["Tabs", "Panes", "Terminal", "Tools and UI", "Application"];

pub const SHORTCUT_DEFINITIONS: &[ShortcutDefinition] = &[
    ShortcutDefinition {
        action: ShortcutAction::NewTab,
        group: "Tabs",
        title: "New Tab",
        description: "Create a terminal tab",
    },
    ShortcutDefinition {
        action: ShortcutAction::CloseTab,
        group: "Tabs",
        title: "Close Tab",
        description: "Close the current tab",
    },
    ShortcutDefinition {
        action: ShortcutAction::NextTab,
        group: "Tabs",
        title: "Next Tab",
        description: "Switch to the next tab",
    },
    ShortcutDefinition {
        action: ShortcutAction::PrevTab,
        group: "Tabs",
        title: "Previous Tab",
        description: "Switch to the previous tab",
    },
    ShortcutDefinition {
        action: ShortcutAction::NewDocumentTab,
        group: "Tabs",
        title: "New Document",
        description: "Create a document tab",
    },
    ShortcutDefinition {
        action: ShortcutAction::SwitchToTab1,
        group: "Tabs",
        title: "Switch to Tab 1",
        description: "Select tab 1",
    },
    ShortcutDefinition {
        action: ShortcutAction::SwitchToTab2,
        group: "Tabs",
        title: "Switch to Tab 2",
        description: "Select tab 2",
    },
    ShortcutDefinition {
        action: ShortcutAction::SwitchToTab3,
        group: "Tabs",
        title: "Switch to Tab 3",
        description: "Select tab 3",
    },
    ShortcutDefinition {
        action: ShortcutAction::SwitchToTab4,
        group: "Tabs",
        title: "Switch to Tab 4",
        description: "Select tab 4",
    },
    ShortcutDefinition {
        action: ShortcutAction::SwitchToTab5,
        group: "Tabs",
        title: "Switch to Tab 5",
        description: "Select tab 5",
    },
    ShortcutDefinition {
        action: ShortcutAction::SwitchToTab6,
        group: "Tabs",
        title: "Switch to Tab 6",
        description: "Select tab 6",
    },
    ShortcutDefinition {
        action: ShortcutAction::SwitchToTab7,
        group: "Tabs",
        title: "Switch to Tab 7",
        description: "Select tab 7",
    },
    ShortcutDefinition {
        action: ShortcutAction::SwitchToTab8,
        group: "Tabs",
        title: "Switch to Tab 8",
        description: "Select tab 8",
    },
    ShortcutDefinition {
        action: ShortcutAction::SwitchToTab9,
        group: "Tabs",
        title: "Switch to Tab 9",
        description: "Select tab 9",
    },
    ShortcutDefinition {
        action: ShortcutAction::SplitHorizontal,
        group: "Panes",
        title: "Split Horizontal",
        description: "Split the current terminal horizontally",
    },
    ShortcutDefinition {
        action: ShortcutAction::SplitVertical,
        group: "Panes",
        title: "Split Vertical",
        description: "Split the current terminal vertically",
    },
    ShortcutDefinition {
        action: ShortcutAction::ClosePane,
        group: "Panes",
        title: "Close Pane",
        description: "Close the focused pane",
    },
    ShortcutDefinition {
        action: ShortcutAction::FocusNextPane,
        group: "Panes",
        title: "Focus Next Pane",
        description: "Move focus to the next pane",
    },
    ShortcutDefinition {
        action: ShortcutAction::FocusPrevPane,
        group: "Panes",
        title: "Focus Previous Pane",
        description: "Move focus to the previous pane",
    },
    ShortcutDefinition {
        action: ShortcutAction::Copy,
        group: "Terminal",
        title: "Copy",
        description: "Copy visible terminal text",
    },
    ShortcutDefinition {
        action: ShortcutAction::Paste,
        group: "Terminal",
        title: "Paste",
        description: "Paste clipboard text",
    },
    ShortcutDefinition {
        action: ShortcutAction::SelectAll,
        group: "Terminal",
        title: "Select All",
        description: "Copy scrollback and visible terminal text",
    },
    ShortcutDefinition {
        action: ShortcutAction::FindTerminal,
        group: "Terminal",
        title: "Find in Terminal",
        description: "Show terminal search",
    },
    ShortcutDefinition {
        action: ShortcutAction::ActivateHints,
        group: "Terminal",
        title: "Activate Hints",
        description: "Open keyboard hints for URLs and detected text",
    },
    ShortcutDefinition {
        action: ShortcutAction::ZoomIn,
        group: "Terminal",
        title: "Zoom In",
        description: "Increase terminal font size",
    },
    ShortcutDefinition {
        action: ShortcutAction::ZoomOut,
        group: "Terminal",
        title: "Zoom Out",
        description: "Decrease terminal font size",
    },
    ShortcutDefinition {
        action: ShortcutAction::ResetZoom,
        group: "Terminal",
        title: "Reset Zoom",
        description: "Reset terminal font size",
    },
    ShortcutDefinition {
        action: ShortcutAction::ToggleAi,
        group: "Tools and UI",
        title: "Toggle AI Panel",
        description: "Show or hide the AI assistant",
    },
    ShortcutDefinition {
        action: ShortcutAction::ToggleSidebar,
        group: "Tools and UI",
        title: "Toggle Sidebar",
        description: "Show or hide the left navigation",
    },
    ShortcutDefinition {
        action: ShortcutAction::QuickSwitcher,
        group: "Tools and UI",
        title: "Quick Switcher",
        description: "Open the quick switcher",
    },
    ShortcutDefinition {
        action: ShortcutAction::SshManager,
        group: "Tools and UI",
        title: "SSH Manager",
        description: "Open SSH Manager",
    },
    ShortcutDefinition {
        action: ShortcutAction::Snippets,
        group: "Tools and UI",
        title: "Snippets",
        description: "Open snippets",
    },
    ShortcutDefinition {
        action: ShortcutAction::AsciiArt,
        group: "Tools and UI",
        title: "ASCII Art",
        description: "Open ASCII art tools",
    },
    ShortcutDefinition {
        action: ShortcutAction::OpenFile,
        group: "Tools and UI",
        title: "Open File",
        description: "Open a file in a document tab",
    },
    ShortcutDefinition {
        action: ShortcutAction::HistorySearch,
        group: "Tools and UI",
        title: "History Search",
        description: "Search command history",
    },
    ShortcutDefinition {
        action: ShortcutAction::Quit,
        group: "Application",
        title: "Quit",
        description: "Close CorgiTerm",
    },
];

impl ShortcutAction {
    pub fn get(self, config: &ShortcutsConfig) -> Option<&str> {
        match self {
            ShortcutAction::NewTab => config.new_tab.as_deref(),
            ShortcutAction::CloseTab => config.close_tab.as_deref(),
            ShortcutAction::NextTab => config.next_tab.as_deref(),
            ShortcutAction::PrevTab => config.prev_tab.as_deref(),
            ShortcutAction::NewDocumentTab => config.new_document_tab.as_deref(),
            ShortcutAction::SwitchToTab1 => config.switch_to_tab_1.as_deref(),
            ShortcutAction::SwitchToTab2 => config.switch_to_tab_2.as_deref(),
            ShortcutAction::SwitchToTab3 => config.switch_to_tab_3.as_deref(),
            ShortcutAction::SwitchToTab4 => config.switch_to_tab_4.as_deref(),
            ShortcutAction::SwitchToTab5 => config.switch_to_tab_5.as_deref(),
            ShortcutAction::SwitchToTab6 => config.switch_to_tab_6.as_deref(),
            ShortcutAction::SwitchToTab7 => config.switch_to_tab_7.as_deref(),
            ShortcutAction::SwitchToTab8 => config.switch_to_tab_8.as_deref(),
            ShortcutAction::SwitchToTab9 => config.switch_to_tab_9.as_deref(),
            ShortcutAction::SplitHorizontal => config.split_horizontal.as_deref(),
            ShortcutAction::SplitVertical => config.split_vertical.as_deref(),
            ShortcutAction::ClosePane => config.close_pane.as_deref(),
            ShortcutAction::FocusNextPane => config.focus_next_pane.as_deref(),
            ShortcutAction::FocusPrevPane => config.focus_prev_pane.as_deref(),
            ShortcutAction::Copy => config.copy.as_deref(),
            ShortcutAction::Paste => config.paste.as_deref(),
            ShortcutAction::SelectAll => config.select_all.as_deref(),
            ShortcutAction::FindTerminal => config.find_terminal.as_deref(),
            ShortcutAction::ActivateHints => config.activate_hints.as_deref(),
            ShortcutAction::ZoomIn => config.zoom_in.as_deref(),
            ShortcutAction::ZoomOut => config.zoom_out.as_deref(),
            ShortcutAction::ResetZoom => config.reset_zoom.as_deref(),
            ShortcutAction::ToggleAi => config.toggle_ai.as_deref(),
            ShortcutAction::ToggleSidebar => config.toggle_sidebar.as_deref(),
            ShortcutAction::QuickSwitcher => config.quick_switcher.as_deref(),
            ShortcutAction::SshManager => config.ssh_manager.as_deref(),
            ShortcutAction::Snippets => config.snippets.as_deref(),
            ShortcutAction::AsciiArt => config.ascii_art.as_deref(),
            ShortcutAction::OpenFile => config.open_file.as_deref(),
            ShortcutAction::HistorySearch => config.history_search.as_deref(),
            ShortcutAction::Quit => config.quit.as_deref(),
        }
    }

    pub fn set(self, config: &mut ShortcutsConfig, value: Option<String>) {
        match self {
            ShortcutAction::NewTab => config.new_tab = value,
            ShortcutAction::CloseTab => config.close_tab = value,
            ShortcutAction::NextTab => config.next_tab = value,
            ShortcutAction::PrevTab => config.prev_tab = value,
            ShortcutAction::NewDocumentTab => config.new_document_tab = value,
            ShortcutAction::SwitchToTab1 => config.switch_to_tab_1 = value,
            ShortcutAction::SwitchToTab2 => config.switch_to_tab_2 = value,
            ShortcutAction::SwitchToTab3 => config.switch_to_tab_3 = value,
            ShortcutAction::SwitchToTab4 => config.switch_to_tab_4 = value,
            ShortcutAction::SwitchToTab5 => config.switch_to_tab_5 = value,
            ShortcutAction::SwitchToTab6 => config.switch_to_tab_6 = value,
            ShortcutAction::SwitchToTab7 => config.switch_to_tab_7 = value,
            ShortcutAction::SwitchToTab8 => config.switch_to_tab_8 = value,
            ShortcutAction::SwitchToTab9 => config.switch_to_tab_9 = value,
            ShortcutAction::SplitHorizontal => config.split_horizontal = value,
            ShortcutAction::SplitVertical => config.split_vertical = value,
            ShortcutAction::ClosePane => config.close_pane = value,
            ShortcutAction::FocusNextPane => config.focus_next_pane = value,
            ShortcutAction::FocusPrevPane => config.focus_prev_pane = value,
            ShortcutAction::Copy => config.copy = value,
            ShortcutAction::Paste => config.paste = value,
            ShortcutAction::SelectAll => config.select_all = value,
            ShortcutAction::FindTerminal => config.find_terminal = value,
            ShortcutAction::ActivateHints => config.activate_hints = value,
            ShortcutAction::ZoomIn => config.zoom_in = value,
            ShortcutAction::ZoomOut => config.zoom_out = value,
            ShortcutAction::ResetZoom => config.reset_zoom = value,
            ShortcutAction::ToggleAi => config.toggle_ai = value,
            ShortcutAction::ToggleSidebar => config.toggle_sidebar = value,
            ShortcutAction::QuickSwitcher => config.quick_switcher = value,
            ShortcutAction::SshManager => config.ssh_manager = value,
            ShortcutAction::Snippets => config.snippets = value,
            ShortcutAction::AsciiArt => config.ascii_art = value,
            ShortcutAction::OpenFile => config.open_file = value,
            ShortcutAction::HistorySearch => config.history_search = value,
            ShortcutAction::Quit => config.quit = value,
        }
    }
}

/// Keyboard shortcuts manager
pub struct KeyboardShortcuts {
    shortcuts: HashMap<ShortcutAction, ParsedShortcut>,
}

impl KeyboardShortcuts {
    /// Create from configuration
    pub fn from_config(config: &ShortcutsConfig) -> Self {
        let mut shortcuts = HashMap::new();

        for definition in SHORTCUT_DEFINITIONS {
            if let Some(shortcut) = definition.action.get(config) {
                if let Ok(parsed) = parse_shortcut(shortcut) {
                    shortcuts.insert(definition.action, parsed);
                } else {
                    tracing::warn!(
                        "Failed to parse shortcut '{}': {}",
                        definition.title,
                        shortcut
                    );
                }
            }
        }

        Self { shortcuts }
    }

    /// Create from the current live configuration.
    pub fn current() -> Self {
        if let Some(config_manager) = crate::app::config_manager() {
            let config = config_manager.read().config();
            Self::from_config(&config.keybindings.shortcuts)
        } else {
            Self::default()
        }
    }

    /// Get the configured shortcut label for an action.
    pub fn label_for(action: ShortcutAction) -> String {
        if let Some(config_manager) = crate::app::config_manager() {
            let config = config_manager.read().config();
            action
                .get(&config.keybindings.shortcuts)
                .unwrap_or("")
                .to_string()
        } else {
            action
                .get(&ShortcutsConfig::default())
                .unwrap_or("")
                .to_string()
        }
    }

    /// Check whether a shortcut string is valid.
    pub fn validate(shortcut: &str) -> bool {
        let shortcut = shortcut.trim();
        shortcut.is_empty() || parse_shortcut(shortcut).is_ok()
    }

    /// Save one shortcut to the live configuration.
    pub fn save_shortcut(action: ShortcutAction, shortcut: &str) -> Result<(), String> {
        let shortcut = shortcut.trim();
        if !shortcut.is_empty() {
            parse_shortcut(shortcut)?;
        }

        let config_manager = crate::app::config_manager()
            .ok_or_else(|| "Configuration is not available".to_string())?;
        config_manager.read().update(|config| {
            action.set(
                &mut config.keybindings.shortcuts,
                (!shortcut.is_empty()).then(|| shortcut.to_string()),
            );
        });
        let save_result = config_manager
            .read()
            .save()
            .map_err(|error| error.to_string());
        save_result
    }

    /// Reset all shortcuts to defaults.
    pub fn reset_all_to_defaults() -> Result<(), String> {
        let config_manager = crate::app::config_manager()
            .ok_or_else(|| "Configuration is not available".to_string())?;
        config_manager.read().update(|config| {
            config.keybindings.shortcuts = ShortcutsConfig::default();
        });
        let save_result = config_manager
            .read()
            .save()
            .map_err(|error| error.to_string());
        save_result
    }

    /// Get a GTK accelerator string for a shortcut action.
    pub fn accelerator_for(action: ShortcutAction) -> Option<String> {
        let label = Self::label_for(action);
        shortcut_to_gtk_accelerator(&label)
    }

    /// Get the default shortcut for an action.
    pub fn default_label_for(action: ShortcutAction) -> String {
        action
            .get(&ShortcutsConfig::default())
            .unwrap_or("")
            .to_string()
    }

    /// Check if a key event matches an action
    pub fn matches(
        &self,
        action: ShortcutAction,
        key: gtk4::gdk::Key,
        modifiers: gtk4::gdk::ModifierType,
    ) -> bool {
        if let Some(shortcut) = self.shortcuts.get(&action) {
            matches_shortcut(shortcut, key, modifiers)
        } else {
            false
        }
    }

    /// Get all configured shortcuts
    pub fn all(&self) -> &HashMap<ShortcutAction, ParsedShortcut> {
        &self.shortcuts
    }
}

impl Default for KeyboardShortcuts {
    fn default() -> Self {
        Self::from_config(&ShortcutsConfig::default())
    }
}

pub fn shortcut_to_gtk_accelerator(shortcut: &str) -> Option<String> {
    parse_shortcut(shortcut).ok()?;

    let parts: Vec<&str> = shortcut.split('+').map(|part| part.trim()).collect();
    let key = parts.last()?;
    let mut accelerator = String::new();

    for modifier in &parts[..parts.len().saturating_sub(1)] {
        match modifier.to_lowercase().as_str() {
            "ctrl" | "control" => accelerator.push_str("<Ctrl>"),
            "shift" => accelerator.push_str("<Shift>"),
            "alt" | "meta" => accelerator.push_str("<Alt>"),
            "super" | "win" | "cmd" => accelerator.push_str("<Super>"),
            _ => return None,
        }
    }

    let key = match key.to_lowercase().as_str() {
        "[" | "bracketleft" => "bracketleft".to_string(),
        "]" | "bracketright" => "bracketright".to_string(),
        "+" | "plus" => "plus".to_string(),
        "-" | "minus" => "minus".to_string(),
        "=" | "equal" => "equal".to_string(),
        "/" | "slash" => "slash".to_string(),
        "\\" | "backslash" => "backslash".to_string(),
        "," | "comma" => "comma".to_string(),
        "." | "period" => "period".to_string(),
        ";" | "semicolon" => "semicolon".to_string(),
        "'" | "apostrophe" => "apostrophe".to_string(),
        "`" | "grave" => "grave".to_string(),
        "escape" | "esc" => "Escape".to_string(),
        "enter" | "return" => "Return".to_string(),
        "pageup" | "pgup" => "Page_Up".to_string(),
        "pagedown" | "pgdn" => "Page_Down".to_string(),
        other => other.to_string(),
    };

    accelerator.push_str(&key);
    Some(accelerator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_default_shortcut_parses() {
        let config = ShortcutsConfig::default();

        for definition in SHORTCUT_DEFINITIONS {
            if let Some(shortcut) = definition.action.get(&config) {
                parse_shortcut(shortcut).unwrap_or_else(|error| {
                    panic!(
                        "default shortcut for {} did not parse: {}",
                        definition.title, error
                    )
                });
            } else {
                panic!("missing default shortcut for {}", definition.title);
            }
        }
    }

    #[test]
    fn shortcut_labels_convert_to_gtk_accelerators() {
        assert_eq!(
            shortcut_to_gtk_accelerator("Ctrl+Shift+A").as_deref(),
            Some("<Ctrl><Shift>a")
        );
        assert_eq!(
            shortcut_to_gtk_accelerator("Ctrl+Shift+]").as_deref(),
            Some("<Ctrl><Shift>bracketright")
        );
        assert_eq!(
            shortcut_to_gtk_accelerator("Ctrl+Plus").as_deref(),
            Some("<Ctrl>plus")
        );
    }
}
