//! Session and Project management
//!
//! CorgiTerm organizes terminals into Projects, where each Project:
//! - Is tied to a folder on the filesystem
//! - Contains multiple terminal sessions as tabs
//! - Remembers its state across restarts
//! - Can have project-specific settings
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │ CorgiTerm                                              _ □ ✕    │
//! ├────────────┬────────────────────────────────────────────────────┤
//! │ PROJECTS   │ [Tab 1: ~/src] [Tab 2: npm run dev] [Tab 3: git] + │
//! │            ├────────────────────────────────────────────────────┤
//! │ 📁 website │ ~/projects/website $ npm run build                 │
//! │   ├ dev    │ > Building...                                      │
//! │   ├ git    │ > Done in 2.3s                                     │
//! │   └ tests  │                                                    │
//! │            │ ~/projects/website $                               │
//! │ 📁 api     │                                                    │
//! │   ├ server │                                                    │
//! │   └ logs   │                                                    │
//! │            │                                                    │
//! │ + New      │                                                    │
//! └────────────┴────────────────────────────────────────────────────┘
//! ```

use crate::{CoreError, Result, Terminal, TerminalSize, Pty};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Unique identifier for a session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(Uuid);

impl ProjectId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ProjectId {
    fn default() -> Self {
        Self::new()
    }
}

/// A terminal session (a single tab within a project)
pub struct Session {
    /// Unique session ID
    pub id: SessionId,
    /// Display name for the tab
    pub name: String,
    /// The PTY for this session
    pty: Option<Pty>,
    /// Terminal emulator state
    terminal: Option<Terminal>,
    /// Working directory
    pub cwd: PathBuf,
    /// Shell command (if custom)
    pub shell: Option<String>,
    /// Creation time
    pub created_at: DateTime<Utc>,
    /// Last activity time
    pub last_activity: DateTime<Utc>,
    /// Is this session active/alive?
    pub is_alive: bool,
    /// Session icon (emoji or path)
    pub icon: Option<String>,
    /// Custom environment variables
    pub env: HashMap<String, String>,
    /// Thumbnail image data (for sidebar preview)
    thumbnail: Option<Vec<u8>>,
}

impl Session {
    /// Create a new session in the given directory
    pub fn new(name: impl Into<String>, cwd: PathBuf) -> Self {
        Self {
            id: SessionId::new(),
            name: name.into(),
            pty: None,
            terminal: None,
            cwd,
            shell: None,
            created_at: Utc::now(),
            last_activity: Utc::now(),
            is_alive: false,
            icon: None,
            env: HashMap::new(),
            thumbnail: None,
        }
    }

    /// Start the session (spawn shell)
    pub fn start(&mut self, size: TerminalSize) -> Result<()> {
        use crate::pty::PtySize;

        let pty_size = PtySize {
            rows: size.rows as u16,
            cols: size.cols as u16,
            pixel_width: 0,
            pixel_height: 0,
        };

        // Spawn PTY with working directory
        let pty = Pty::spawn(self.shell.as_deref(), pty_size, Some(&self.cwd), None)?;

        let (tx, _rx) = crossbeam_channel::unbounded();
        let terminal = Terminal::new(size, tx);

        self.pty = Some(pty);
        self.terminal = Some(terminal);
        self.is_alive = true;

        Ok(())
    }

    /// Write input to the session
    pub fn write(&self, data: &[u8]) -> Result<()> {
        if let Some(ref pty) = self.pty {
            pty.write(data)?;
        }
        Ok(())
    }

    /// Read output from the session
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if let Some(ref pty) = self.pty {
            let n = pty.read(buf)?;
            if n > 0 {
                if let Some(ref mut terminal) = self.terminal {
                    terminal.process(&buf[..n]);
                }
                self.last_activity = Utc::now();
            }
            Ok(n)
        } else {
            Ok(0)
        }
    }

    /// Resize the session
    pub fn resize(&mut self, size: TerminalSize) -> Result<()> {
        use crate::pty::PtySize;

        if let Some(ref mut pty) = self.pty {
            pty.resize(PtySize {
                rows: size.rows as u16,
                cols: size.cols as u16,
                pixel_width: 0,
                pixel_height: 0,
            })?;
        }

        if let Some(ref mut terminal) = self.terminal {
            terminal.resize(size);
        }

        Ok(())
    }

    /// Get terminal for rendering
    pub fn terminal(&self) -> Option<&Terminal> {
        self.terminal.as_ref()
    }

    /// Update thumbnail (for sidebar preview)
    pub fn update_thumbnail(&mut self, data: Vec<u8>) {
        self.thumbnail = Some(data);
    }

    /// Get thumbnail data
    pub fn thumbnail(&self) -> Option<&[u8]> {
        self.thumbnail.as_deref()
    }
}

/// A project containing multiple terminal sessions
#[derive(Serialize, Deserialize)]
pub struct Project {
    /// Unique project ID
    pub id: ProjectId,
    /// Project name (defaults to folder name)
    pub name: String,
    /// Root folder path
    pub path: PathBuf,
    /// Project icon (emoji or path)
    pub icon: String,
    /// Sessions in this project (runtime only, not serialized)
    #[serde(skip)]
    sessions: Vec<Session>,
    /// Active session index
    pub active_session: usize,
    /// Project-specific settings overrides
    pub settings: ProjectSettings,
    /// Is this project expanded in sidebar?
    pub expanded: bool,
    /// Last opened time
    pub last_opened: DateTime<Utc>,
    /// Sort order in sidebar
    pub sort_order: i32,
}

/// Project-specific settings that override global settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Override shell for this project
    pub shell: Option<String>,
    /// Override theme for this project
    pub theme: Option<String>,
    /// Custom environment variables
    pub env: HashMap<String, String>,
    /// Startup commands to run when opening project
    pub startup_commands: Vec<String>,
    /// Default terminal size
    pub default_size: Option<(usize, usize)>,
}

impl Project {
    /// Create a new project from a folder path
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string();

        Self {
            id: ProjectId::new(),
            name,
            path,
            icon: "📁".to_string(),
            sessions: Vec::new(),
            active_session: 0,
            settings: ProjectSettings::default(),
            expanded: true,
            last_opened: Utc::now(),
            sort_order: 0,
        }
    }

    /// Add a new session to this project
    pub fn add_session(&mut self, name: impl Into<String>) -> &mut Session {
        let session = Session::new(name, self.path.clone());
        self.sessions.push(session);
        self.active_session = self.sessions.len() - 1;
        self.sessions.last_mut().unwrap()
    }

    /// Get all sessions
    pub fn sessions(&self) -> &[Session] {
        &self.sessions
    }

    /// Get mutable sessions
    pub fn sessions_mut(&mut self) -> &mut [Session] {
        &mut self.sessions
    }

    /// Get active session
    pub fn active_session(&self) -> Option<&Session> {
        self.sessions.get(self.active_session)
    }

    /// Get mutable active session
    pub fn active_session_mut(&mut self) -> Option<&mut Session> {
        self.sessions.get_mut(self.active_session)
    }

    /// Set active session by index
    pub fn set_active(&mut self, index: usize) {
        if index < self.sessions.len() {
            self.active_session = index;
        }
    }

    /// Close a session by index
    pub fn close_session(&mut self, index: usize) {
        if index < self.sessions.len() {
            self.sessions.remove(index);
            if self.active_session >= self.sessions.len() && !self.sessions.is_empty() {
                self.active_session = self.sessions.len() - 1;
            }
        }
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

/// Manages all projects and their sessions
pub struct SessionManager {
    /// All projects
    projects: Vec<Project>,
    /// Active project index
    active_project: usize,
    /// Recently closed projects (for undo)
    recently_closed: Vec<Project>,
    /// Config directory for persistence
    config_dir: PathBuf,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            projects: Vec::new(),
            active_project: 0,
            recently_closed: Vec::new(),
            config_dir,
        }
    }

    /// Load saved projects from disk
    pub fn load(&mut self) -> Result<()> {
        let projects_file = self.config_dir.join("projects.json");
        if projects_file.exists() {
            let content = std::fs::read_to_string(&projects_file)
                .map_err(|e| CoreError::Session(format!("Failed to read projects: {}", e)))?;
            self.projects = serde_json::from_str(&content)
                .map_err(|e| CoreError::Session(format!("Failed to parse projects: {}", e)))?;
        }
        Ok(())
    }

    /// Save projects to disk
    pub fn save(&self) -> Result<()> {
        std::fs::create_dir_all(&self.config_dir)
            .map_err(|e| CoreError::Session(format!("Failed to create config dir: {}", e)))?;

        let projects_file = self.config_dir.join("projects.json");
        let content = serde_json::to_string_pretty(&self.projects)
            .map_err(|e| CoreError::Session(format!("Failed to serialize projects: {}", e)))?;
        std::fs::write(&projects_file, content)
            .map_err(|e| CoreError::Session(format!("Failed to write projects: {}", e)))?;

        Ok(())
    }

    /// Open or create a project for a folder
    pub fn open_project(&mut self, path: PathBuf) -> &mut Project {
        // Check if project already exists
        if let Some(idx) = self.projects.iter().position(|p| p.path == path) {
            self.active_project = idx;
            self.projects[idx].last_opened = Utc::now();
            return &mut self.projects[idx];
        }

        // Create new project
        let project = Project::new(path);
        self.projects.push(project);
        self.active_project = self.projects.len() - 1;
        &mut self.projects[self.active_project]
    }

    /// Get all projects
    pub fn projects(&self) -> &[Project] {
        &self.projects
    }

    /// Get mutable projects
    pub fn projects_mut(&mut self) -> &mut [Project] {
        &mut self.projects
    }

    /// Get active project
    pub fn active_project(&self) -> Option<&Project> {
        self.projects.get(self.active_project)
    }

    /// Get mutable active project
    pub fn active_project_mut(&mut self) -> Option<&mut Project> {
        self.projects.get_mut(self.active_project)
    }

    /// Set active project by index
    pub fn set_active_project(&mut self, index: usize) {
        if index < self.projects.len() {
            self.active_project = index;
        }
    }

    /// Close a project
    pub fn close_project(&mut self, index: usize) {
        if index < self.projects.len() {
            let project = self.projects.remove(index);
            self.recently_closed.push(project);

            // Keep only last 10 closed projects
            while self.recently_closed.len() > 10 {
                self.recently_closed.remove(0);
            }

            if self.active_project >= self.projects.len() && !self.projects.is_empty() {
                self.active_project = self.projects.len() - 1;
            }
        }
    }

    /// Reopen last closed project
    pub fn reopen_last_closed(&mut self) -> Option<&mut Project> {
        if let Some(project) = self.recently_closed.pop() {
            self.projects.push(project);
            self.active_project = self.projects.len() - 1;
            Some(&mut self.projects[self.active_project])
        } else {
            None
        }
    }

    /// Get quick switcher data (for Cmd+K style switching)
    pub fn quick_switch_data(&self) -> Vec<QuickSwitchItem> {
        let mut items = Vec::new();

        for (proj_idx, project) in self.projects.iter().enumerate() {
            for (sess_idx, session) in project.sessions.iter().enumerate() {
                items.push(QuickSwitchItem {
                    project_index: proj_idx,
                    session_index: sess_idx,
                    project_name: project.name.clone(),
                    session_name: session.name.clone(),
                    path: project.path.clone(),
                    last_activity: session.last_activity,
                });
            }
        }

        // Sort by last activity
        items.sort_by(|a, b| b.last_activity.cmp(&a.last_activity));
        items
    }
}

/// Item for quick switcher (Cmd+K)
#[derive(Debug)]
pub struct QuickSwitchItem {
    pub project_index: usize,
    pub session_index: usize,
    pub project_name: String,
    pub session_name: String,
    pub path: PathBuf,
    pub last_activity: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_project_creation() {
        let project = Project::new(PathBuf::from("/home/user/myproject"));
        assert_eq!(project.name, "myproject");
        assert_eq!(project.icon, "📁");
    }

    #[test]
    fn test_add_session() {
        let mut project = Project::new(PathBuf::from("/home/user/myproject"));
        project.add_session("dev server");
        project.add_session("git");

        assert_eq!(project.session_count(), 2);
        assert_eq!(project.sessions[0].name, "dev server");
        assert_eq!(project.sessions[1].name, "git");
    }

    #[test]
    fn test_session_id() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);
    }
}
