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

    // UI features
    ToggleAi,
    ToggleSidebar,
    QuickSwitcher,
    SshManager,
    Snippets,
    AsciiArt,
    OpenFile,

    // Application
    Quit,
}

/// Keyboard shortcuts manager
pub struct KeyboardShortcuts {
    shortcuts: HashMap<ShortcutAction, ParsedShortcut>,
}

impl KeyboardShortcuts {
    /// Create from configuration
    pub fn from_config(config: &ShortcutsConfig) -> Self {
        let mut shortcuts = HashMap::new();

        // Tab management
        if let Some(ref s) = config.new_tab {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::NewTab, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'new_tab': {}", s);
            }
        }
        if let Some(ref s) = config.close_tab {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::CloseTab, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'close_tab': {}", s);
            }
        }
        if let Some(ref s) = config.next_tab {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::NextTab, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'next_tab': {}", s);
            }
        }
        if let Some(ref s) = config.prev_tab {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::PrevTab, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'prev_tab': {}", s);
            }
        }
        if let Some(ref s) = config.new_document_tab {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::NewDocumentTab, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'new_document_tab': {}", s);
            }
        }

        // Tab switching
        if let Some(ref s) = config.switch_to_tab_1 {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::SwitchToTab1, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'switch_to_tab_1': {}", s);
            }
        }
        if let Some(ref s) = config.switch_to_tab_2 {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::SwitchToTab2, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'switch_to_tab_2': {}", s);
            }
        }
        if let Some(ref s) = config.switch_to_tab_3 {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::SwitchToTab3, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'switch_to_tab_3': {}", s);
            }
        }
        if let Some(ref s) = config.switch_to_tab_4 {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::SwitchToTab4, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'switch_to_tab_4': {}", s);
            }
        }
        if let Some(ref s) = config.switch_to_tab_5 {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::SwitchToTab5, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'switch_to_tab_5': {}", s);
            }
        }
        if let Some(ref s) = config.switch_to_tab_6 {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::SwitchToTab6, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'switch_to_tab_6': {}", s);
            }
        }
        if let Some(ref s) = config.switch_to_tab_7 {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::SwitchToTab7, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'switch_to_tab_7': {}", s);
            }
        }
        if let Some(ref s) = config.switch_to_tab_8 {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::SwitchToTab8, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'switch_to_tab_8': {}", s);
            }
        }
        if let Some(ref s) = config.switch_to_tab_9 {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::SwitchToTab9, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'switch_to_tab_9': {}", s);
            }
        }

        // Pane management
        if let Some(ref s) = config.split_horizontal {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::SplitHorizontal, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'split_horizontal': {}", s);
            }
        }
        if let Some(ref s) = config.split_vertical {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::SplitVertical, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'split_vertical': {}", s);
            }
        }
        if let Some(ref s) = config.close_pane {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::ClosePane, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'close_pane': {}", s);
            }
        }
        if let Some(ref s) = config.focus_next_pane {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::FocusNextPane, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'focus_next_pane': {}", s);
            }
        }
        if let Some(ref s) = config.focus_prev_pane {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::FocusPrevPane, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'focus_prev_pane': {}", s);
            }
        }

        // UI features
        if let Some(ref s) = config.toggle_ai {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::ToggleAi, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'toggle_ai': {}", s);
            }
        }
        if let Some(ref s) = config.toggle_sidebar {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::ToggleSidebar, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'toggle_sidebar': {}", s);
            }
        }
        if let Some(ref s) = config.quick_switcher {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::QuickSwitcher, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'quick_switcher': {}", s);
            }
        }
        if let Some(ref s) = config.ssh_manager {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::SshManager, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'ssh_manager': {}", s);
            }
        }
        if let Some(ref s) = config.snippets {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::Snippets, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'snippets': {}", s);
            }
        }
        if let Some(ref s) = config.ascii_art {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::AsciiArt, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'ascii_art': {}", s);
            }
        }
        if let Some(ref s) = config.open_file {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::OpenFile, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'open_file': {}", s);
            }
        }

        // Application
        if let Some(ref s) = config.quit {
            if let Ok(parsed) = parse_shortcut(s) {
                shortcuts.insert(ShortcutAction::Quit, parsed);
            } else {
                tracing::warn!("Failed to parse shortcut 'quit': {}", s);
            }
        }

        Self { shortcuts }
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
