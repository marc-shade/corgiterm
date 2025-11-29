// Clippy allows for intentional patterns in GTK code
#![allow(clippy::type_complexity)] // Complex callback types are common in GTK
#![allow(clippy::collapsible_if)] // Sometimes nested ifs are clearer
#![allow(clippy::single_match)] // Match is often clearer than if-let for enums
#![allow(clippy::await_holding_lock)] // GTK sync primitives with async code
#![allow(clippy::needless_range_loop)] // Sometimes index access is clearer
#![allow(clippy::only_used_in_recursion)] // Common in tree/pane traversal methods
#![allow(clippy::needless_borrow)] // Sometimes borrows are clearer
#![allow(clippy::needless_borrows_for_generic_args)] // Sometimes borrows are clearer
#![allow(clippy::map_clone)] // Minor optimization vs readability
#![allow(clippy::ptr_arg)] // PathBuf vs Path in GTK callbacks

//! # CorgiTerm UI
//!
//! GTK4/libadwaita user interface for CorgiTerm.
//!
//! Layout:
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ CorgiTerm - ~/projects/website                        _ □ ✕    │
//! ├────────────┬────────────────────────────────────────────────────┤
//! │ PROJECTS   │ [🏠 home] [🔧 dev] [📦 build] [+]                  │
//! │            ├────────────────────────────────────────────────────┤
//! │ 📁 website │                                                    │
//! │   ├ home   │ ~/projects/website $ npm run build                 │
//! │   ├ dev    │ > corgiterm-website@1.0.0 build                    │
//! │   └ build  │ > next build                                       │
//! │            │                                                    │
//! │ 📁 api     │ ✓ Compiled successfully                            │
//! │   └ server │                                                    │
//! │            │ ~/projects/website $                               │
//! │ + New      │                                                    │
//! │            ├────────────────────────────────────────────────────┤
//! │ ─────────  │ 🐕 Type naturally: "show large files"              │
//! │ 🤖 AI      │                                                    │
//! └────────────┴────────────────────────────────────────────────────┘
//! ```

pub mod ai_panel;
pub mod app;
pub mod ascii_art_dialog;
pub mod broadcast;
pub mod dialogs;
pub mod document_view;
pub mod emoji_picker;
pub mod keyboard;
pub mod sidebar;
pub mod snippets;
pub mod split_pane;
pub mod ssh_manager;
pub mod tab_bar;
pub mod terminal_view;
pub mod theme;
pub mod theme_creator;
pub mod widgets;
pub mod window;

use gtk4::prelude::*;
use gtk4::{glib, Application};

/// Application ID
pub const APP_ID: &str = "dev.corgiterm.CorgiTerm";

/// Initialize and run the application
pub fn run() -> glib::ExitCode {
    // Initialize GTK
    gtk4::init().expect("Failed to initialize GTK");

    // Create application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect signals
    app.connect_activate(|app| {
        app::build_ui(app);
    });

    // Run
    app.run()
}

/// Application version
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_id() {
        assert!(APP_ID.contains("corgiterm"));
    }
}
