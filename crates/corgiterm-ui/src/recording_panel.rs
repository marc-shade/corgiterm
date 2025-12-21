//! Recording controls panel
//!
//! Provides UI controls for session recording and playback:
//! - Start/Stop recording
//! - Recording list with metadata
//! - Playback controls (play, pause, speed, seek)

use chrono::Local;
use corgiterm_core::{PlaybackState, Recording, RecordingEvent, RecordingId, RecordingStore};
use gtk4::prelude::*;
use gtk4::{
    Box, Button, Label, ListBox, ListBoxRow, Orientation, ProgressBar, Scale, ScrolledWindow,
    Separator, ToggleButton,
};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

/// Recording state shared between UI and terminal
pub struct RecordingState {
    /// Current active recording (if recording)
    pub current_recording: Option<Recording>,
    /// Is currently recording
    pub is_recording: bool,
    /// Playback state (if playing)
    pub playback: Option<PlaybackState>,
    /// Recording store
    pub store: RecordingStore,
}

impl Default for RecordingState {
    fn default() -> Self {
        Self::new()
    }
}

impl RecordingState {
    pub fn new() -> Self {
        let mut store = RecordingStore::new();
        let _ = store.load(); // Ignore errors on initial load

        Self {
            current_recording: None,
            is_recording: false,
            playback: None,
            store,
        }
    }

    /// Start a new recording
    pub fn start_recording(&mut self, title: &str, cwd: PathBuf, rows: u16, cols: u16) {
        if self.is_recording {
            return;
        }

        let recording = Recording::new(title, cwd, rows, cols);
        self.current_recording = Some(recording);
        self.is_recording = true;
        tracing::info!("Started recording: {}", title);
    }

    /// Stop current recording and save it
    pub fn stop_recording(&mut self) -> Option<RecordingId> {
        if !self.is_recording {
            return None;
        }

        if let Some(mut recording) = self.current_recording.take() {
            recording.finalize();
            let id = recording.meta.id;

            match self.store.save(&recording) {
                Ok(path) => {
                    tracing::info!("Recording saved to {:?}", path);
                    let _ = self.store.load(); // Refresh list
                }
                Err(e) => {
                    tracing::error!("Failed to save recording: {}", e);
                }
            }

            self.is_recording = false;
            Some(id)
        } else {
            self.is_recording = false;
            None
        }
    }

    /// Add output data to current recording
    pub fn add_output(&mut self, data: &[u8]) {
        if let Some(ref mut recording) = self.current_recording {
            recording.add_output(data);
        }
    }

    /// Add input data to current recording
    pub fn add_input(&mut self, data: &[u8]) {
        if let Some(ref mut recording) = self.current_recording {
            recording.add_input(data);
        }
    }

    /// Add resize event to current recording
    pub fn add_resize(&mut self, rows: u16, cols: u16) {
        if let Some(ref mut recording) = self.current_recording {
            recording.add_resize(rows, cols);
        }
    }

    /// Start playback of a recording
    pub fn start_playback(&mut self, id: RecordingId) -> bool {
        match self.store.load_recording(id) {
            Ok(recording) => {
                self.playback = Some(PlaybackState::new(recording));
                true
            }
            Err(e) => {
                tracing::error!("Failed to load recording: {}", e);
                false
            }
        }
    }

    /// Stop playback
    pub fn stop_playback(&mut self) {
        self.playback = None;
    }
}

/// Recording controls panel widget
pub struct RecordingPanel {
    container: Box,
    record_button: ToggleButton,
    recording_list: ListBox,
    playback_controls: Box,
    progress_bar: ProgressBar,
    speed_scale: Scale,
    play_pause_button: Button,
    state: Arc<RwLock<RecordingState>>,
    on_playback_event: Rc<RefCell<Option<std::boxed::Box<dyn Fn(&RecordingEvent)>>>>,
}

impl RecordingPanel {
    pub fn new() -> Self {
        let state = Arc::new(RwLock::new(RecordingState::new()));

        let container = Box::new(Orientation::Vertical, 8);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.set_margin_start(12);
        container.set_margin_end(12);
        container.add_css_class("recording-panel");

        // Header
        let header = Label::new(Some("Session Recording"));
        header.add_css_class("title-4");
        header.set_margin_bottom(8);
        container.append(&header);

        // Record button section
        let record_section = Box::new(Orientation::Horizontal, 8);

        let record_button = ToggleButton::new();
        record_button.set_icon_name("media-record-symbolic");
        record_button.set_tooltip_text(Some("Start/Stop Recording"));
        record_button.add_css_class("circular");
        record_button.add_css_class("destructive-action");
        record_section.append(&record_button);

        let record_label = Label::new(Some("Click to start recording"));
        record_label.add_css_class("dim-label");
        record_label.set_hexpand(true);
        record_label.set_xalign(0.0);
        record_section.append(&record_label);

        container.append(&record_section);

        container.append(&Separator::new(Orientation::Horizontal));

        // Recording list header
        let list_header = Label::new(Some("Saved Recordings"));
        list_header.add_css_class("title-5");
        list_header.set_xalign(0.0);
        list_header.set_margin_top(8);
        container.append(&list_header);

        // Recording list
        let recording_list = ListBox::new();
        recording_list.add_css_class("boxed-list");
        recording_list.set_selection_mode(gtk4::SelectionMode::Single);

        let scrolled = ScrolledWindow::new();
        scrolled.set_child(Some(&recording_list));
        scrolled.set_vexpand(true);
        scrolled.set_min_content_height(200);
        container.append(&scrolled);

        container.append(&Separator::new(Orientation::Horizontal));

        // Playback controls
        let playback_controls = Box::new(Orientation::Vertical, 8);
        playback_controls.set_margin_top(8);
        playback_controls.set_visible(false); // Hidden until recording selected

        let playback_header = Label::new(Some("Playback"));
        playback_header.add_css_class("title-5");
        playback_header.set_xalign(0.0);
        playback_controls.append(&playback_header);

        // Progress bar
        let progress_bar = ProgressBar::new();
        progress_bar.set_show_text(true);
        progress_bar.set_text(Some("0:00 / 0:00"));
        playback_controls.append(&progress_bar);

        // Control buttons
        let controls_row = Box::new(Orientation::Horizontal, 8);
        controls_row.set_halign(gtk4::Align::Center);

        let play_pause_button = Button::from_icon_name("media-playback-start-symbolic");
        play_pause_button.set_tooltip_text(Some("Play/Pause"));
        play_pause_button.add_css_class("circular");
        controls_row.append(&play_pause_button);

        let stop_button = Button::from_icon_name("media-playback-stop-symbolic");
        stop_button.set_tooltip_text(Some("Stop"));
        stop_button.add_css_class("circular");
        controls_row.append(&stop_button);

        playback_controls.append(&controls_row);

        // Speed control
        let speed_row = Box::new(Orientation::Horizontal, 8);

        let speed_label = Label::new(Some("Speed:"));
        speed_row.append(&speed_label);

        let speed_scale = Scale::with_range(Orientation::Horizontal, 0.25, 4.0, 0.25);
        speed_scale.set_value(1.0);
        speed_scale.set_hexpand(true);
        speed_scale.add_mark(1.0, gtk4::PositionType::Bottom, Some("1x"));
        speed_scale.add_mark(2.0, gtk4::PositionType::Bottom, Some("2x"));
        speed_row.append(&speed_scale);

        playback_controls.append(&speed_row);

        container.append(&playback_controls);

        // Help text
        let help = Label::new(Some("Record terminal sessions for training and playback"));
        help.add_css_class("dim-label");
        help.add_css_class("caption");
        help.set_margin_top(8);
        container.append(&help);

        let panel = Self {
            container,
            record_button,
            recording_list,
            playback_controls,
            progress_bar,
            speed_scale,
            play_pause_button,
            state,
            on_playback_event: Rc::new(RefCell::new(None)),
        };

        panel.setup_signals();
        panel.refresh_list();

        panel
    }

    fn setup_signals(&self) {
        let state = self.state.clone();

        // Record button toggle
        self.record_button.connect_toggled({
            let state = state.clone();
            move |button| {
                let mut state = state.write().unwrap();
                if button.is_active() {
                    // Start recording
                    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
                    let title = format!("Recording {}", Local::now().format("%Y-%m-%d %H:%M"));
                    state.start_recording(&title, cwd, 24, 80);
                    button.set_icon_name("media-playback-stop-symbolic");
                } else {
                    // Stop recording
                    state.stop_recording();
                    button.set_icon_name("media-record-symbolic");
                }
            }
        });

        // List row activation (start playback)
        let state_for_list = self.state.clone();
        let playback_controls = self.playback_controls.clone();

        self.recording_list.connect_row_activated({
            move |_list, row| {
                // Get recording ID from row data
                if let Some(id_str) = row.widget_name().strip_prefix("recording_") {
                    if let Some(id) = RecordingId::parse(id_str) {
                        let mut state = state_for_list.write().unwrap();
                        if state.start_playback(id) {
                            playback_controls.set_visible(true);
                            tracing::info!("Started playback of recording {}", id_str);
                        }
                    }
                }
            }
        });

        // Speed scale change
        let state_for_speed = self.state.clone();
        self.speed_scale.connect_value_changed({
            move |scale| {
                let speed = scale.value();
                let mut state = state_for_speed.write().unwrap();
                if let Some(ref mut playback) = state.playback {
                    playback.set_speed(speed);
                }
            }
        });

        // Play/Pause button
        let state_for_play = self.state.clone();
        let play_pause_button = self.play_pause_button.clone();
        self.play_pause_button.connect_clicked({
            move |_| {
                let mut state = state_for_play.write().unwrap();
                if let Some(ref mut playback) = state.playback {
                    playback.toggle_pause();
                    let icon = if playback.paused {
                        "media-playback-start-symbolic"
                    } else {
                        "media-playback-pause-symbolic"
                    };
                    play_pause_button.set_icon_name(icon);
                }
            }
        });
    }

    /// Refresh the recording list from store
    pub fn refresh_list(&self) {
        // Clear existing rows
        while let Some(row) = self.recording_list.first_child() {
            self.recording_list.remove(&row);
        }

        let state = self.state.read().unwrap();
        for meta in state.store.list() {
            let row_box = Box::new(Orientation::Vertical, 4);
            row_box.set_margin_top(8);
            row_box.set_margin_bottom(8);
            row_box.set_margin_start(8);
            row_box.set_margin_end(8);

            // Title
            let title_label = Label::new(Some(&meta.title));
            title_label.add_css_class("heading");
            title_label.set_xalign(0.0);
            row_box.append(&title_label);

            // Info line
            let duration = format_duration(meta.duration_ms);
            let date = meta.started_at.format("%Y-%m-%d %H:%M").to_string();
            let info = format!("{} | {} | {} events", date, duration, meta.event_count);
            let info_label = Label::new(Some(&info));
            info_label.add_css_class("dim-label");
            info_label.add_css_class("caption");
            info_label.set_xalign(0.0);
            row_box.append(&info_label);

            let row = ListBoxRow::new();
            row.set_child(Some(&row_box));
            // Store recording ID in widget name for retrieval
            row.set_widget_name(&format!("recording_{}", meta.id));

            self.recording_list.append(&row);
        }
    }

    /// Get the container widget
    pub fn widget(&self) -> &Box {
        &self.container
    }

    /// Get shared state for terminal integration
    pub fn state(&self) -> Arc<RwLock<RecordingState>> {
        self.state.clone()
    }

    /// Set callback for playback events
    pub fn set_on_playback_event<F>(&self, callback: F)
    where
        F: Fn(&RecordingEvent) + 'static,
    {
        *self.on_playback_event.borrow_mut() = Some(std::boxed::Box::new(callback));
    }

    /// Check if currently recording
    pub fn is_recording(&self) -> bool {
        self.state.read().unwrap().is_recording
    }

    /// Update progress bar during playback
    pub fn update_playback_progress(&self) {
        let state = self.state.read().unwrap();
        if let Some(ref playback) = state.playback {
            let progress = playback.progress();
            self.progress_bar.set_fraction(progress);

            let current = format_duration(playback.position_ms);
            let total = format_duration(playback.recording.meta.duration_ms);
            self.progress_bar
                .set_text(Some(&format!("{} / {}", current, total)));
        }
    }
}

impl Default for RecordingPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Format duration in milliseconds to human-readable string
fn format_duration(ms: u64) -> String {
    let secs = ms / 1000;
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{}:{:02}", mins, secs)
}

/// Show the session recording dialog
pub fn show_recording_dialog(parent: &impl gtk4::prelude::IsA<gtk4::Widget>) {
    let panel = RecordingPanel::new();

    // Create popup window
    let window = gtk4::Window::builder()
        .title("Session Recording")
        .modal(true)
        .default_width(450)
        .default_height(550)
        .child(panel.widget())
        .build();

    // Set transient for parent window
    if let Some(parent_window) = parent
        .root()
        .and_then(|r| r.downcast::<gtk4::Window>().ok())
    {
        window.set_transient_for(Some(&parent_window));
    }

    // Handle Escape to close
    let key_controller = gtk4::EventControllerKey::new();
    key_controller.connect_key_pressed({
        let window = window.clone();
        move |_, key, _keycode, _state| {
            if key == gtk4::gdk::Key::Escape {
                window.close();
                gtk4::glib::Propagation::Stop
            } else {
                gtk4::glib::Propagation::Proceed
            }
        }
    });
    window.add_controller(key_controller);

    window.present();
}
