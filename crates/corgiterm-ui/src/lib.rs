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
pub mod dialogs;
pub mod document_view;
pub mod emoji_picker;
pub mod history_search;
pub mod keyboard;
pub mod recording_panel;
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

use gtk4::gio;
use gtk4::prelude::*;
use gtk4::{glib, Application};

/// Application ID
pub const APP_ID: &str = "dev.corgiterm.CorgiTerm";

/// Initialize and run the application
pub fn run() -> glib::ExitCode {
    // Initialize GTK
    gtk4::init().expect("Failed to initialize GTK");

    // Create application with NON_UNIQUE flag to avoid D-Bus registration issues
    // on some X11 desktop environments (e.g., Budgie, older XFCE)
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

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
