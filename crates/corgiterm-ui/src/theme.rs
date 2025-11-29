//! Theme management with hot-reload support

use gtk4::gdk::Display;
use gtk4::CssProvider;
use notify::{recommended_watcher, Event, EventKind, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;

/// Load CSS theme (embedded or from file with optional hot-reload)
pub fn load_theme(hot_reload: bool) {
    // Check if there's a user CSS file
    let user_css = corgiterm_config::config_dir().join("style.css");

    if user_css.exists() {
        if let Err(e) = load_from_file(&user_css, hot_reload) {
            tracing::warn!("Failed to load user CSS, falling back to embedded: {}", e);
            load_embedded();
        }
    } else {
        // Use embedded CSS
        load_embedded();
        tracing::debug!("No user CSS found at {:?}, using embedded theme", user_css);
    }
}

/// Load CSS from embedded data (default)
fn load_embedded() {
    let provider = CssProvider::new();
    let css_data = include_str!("style.css");
    provider.load_from_string(css_data);
    apply_provider(&provider);
    tracing::info!("Loaded embedded CSS theme");
}

/// Load CSS from a file path with optional hot-reload
fn load_from_file(path: &PathBuf, hot_reload: bool) -> anyhow::Result<()> {
    let css_data = std::fs::read_to_string(path)?;

    let provider = CssProvider::new();
    provider.load_from_string(&css_data);
    apply_provider(&provider);
    tracing::info!("Loaded CSS theme from {:?}", path);

    if hot_reload {
        setup_file_watcher(path.clone(), provider)?;
    }

    Ok(())
}

/// Apply the CSS provider to the default display
fn apply_provider(provider: &CssProvider) {
    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

/// Set up file watcher for hot-reload
fn setup_file_watcher(path: PathBuf, provider: CssProvider) -> anyhow::Result<()> {
    // Use a channel to communicate between watcher thread and GTK main thread
    let (tx, rx) = mpsc::channel::<PathBuf>();

    let path_for_watcher = path.clone();
    let mut watcher = recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                if event.paths.iter().any(|p| *p == path_for_watcher) {
                    let _ = tx.send(path_for_watcher.clone());
                }
            }
        }
    })?;

    // Watch the CSS file's parent directory
    if let Some(parent) = path.parent() {
        watcher.watch(parent, RecursiveMode::NonRecursive)?;
    }

    // Keep watcher alive by storing in a thread-local
    // The watcher will be dropped when the thread ends
    std::thread::spawn(move || {
        let _watcher = watcher; // Keep alive
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        }
    });

    // Set up a GTK idle callback to check for updates
    let path_for_reload = path.clone();
    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
        // Check for pending file change notifications
        while let Ok(changed_path) = rx.try_recv() {
            if changed_path == path_for_reload {
                match std::fs::read_to_string(&changed_path) {
                    Ok(css_data) => {
                        provider.load_from_string(&css_data);
                        tracing::info!("Hot-reloaded CSS theme from {:?}", changed_path);
                    }
                    Err(e) => {
                        tracing::error!("Failed to reload CSS: {}", e);
                    }
                }
            }
        }
        gtk4::glib::ControlFlow::Continue
    });

    tracing::info!("Enabled CSS hot-reload for {:?}", path);
    Ok(())
}
