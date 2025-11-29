//! # CorgiTerm Plugin System
//!
//! Extensible plugin architecture supporting:
//! - WASM plugins (sandboxed, cross-platform)
//! - Lua scripts (lightweight, easy to write)
//!
//! Plugins can:
//! - Add custom commands
//! - Modify terminal behavior
//! - Add UI elements
//! - Integrate with external services

pub mod api;
pub mod loader;
pub mod lua_runtime;
pub mod wasm_runtime;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Plugin manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin name
    pub name: String,
    /// Version (semver)
    pub version: String,
    /// Description
    pub description: Option<String>,
    /// Author
    pub author: Option<String>,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Plugin type
    pub plugin_type: PluginType,
    /// Entry point file
    pub entry: String,
    /// Required CorgiTerm version
    pub corgiterm_version: Option<String>,
    /// Permissions required
    pub permissions: Vec<Permission>,
}

/// Plugin types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    Wasm,
    Lua,
}

/// Plugin permissions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Permission {
    /// Read terminal output
    ReadTerminal,
    /// Write to terminal
    WriteTerminal,
    /// Execute commands
    ExecuteCommands,
    /// Access filesystem
    Filesystem,
    /// Network access
    Network,
    /// Access clipboard
    Clipboard,
    /// Show notifications
    Notifications,
    /// Access configuration
    Configuration,
}

/// A loaded plugin
pub struct Plugin {
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Plugin directory
    pub path: PathBuf,
    /// Is plugin enabled?
    pub enabled: bool,
}

/// Plugin manager
pub struct PluginManager {
    plugins: Vec<Plugin>,
    plugin_dir: PathBuf,
}

impl PluginManager {
    pub fn new(plugin_dir: PathBuf) -> Self {
        Self {
            plugins: Vec::new(),
            plugin_dir,
        }
    }

    /// Discover and load all plugins
    pub fn discover(&mut self) -> anyhow::Result<usize> {
        let mut count = 0;

        if !self.plugin_dir.exists() {
            std::fs::create_dir_all(&self.plugin_dir)?;
            return Ok(0);
        }

        for entry in std::fs::read_dir(&self.plugin_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let manifest_path = path.join("plugin.toml");
                if manifest_path.exists() {
                    match self.load_plugin(&path) {
                        Ok(plugin) => {
                            self.plugins.push(plugin);
                            count += 1;
                        }
                        Err(e) => {
                            tracing::warn!("Failed to load plugin at {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    fn load_plugin(&self, path: &PathBuf) -> anyhow::Result<Plugin> {
        let manifest_path = path.join("plugin.toml");
        let content = std::fs::read_to_string(&manifest_path)?;
        let manifest: PluginManifest = toml::from_str(&content)?;

        Ok(Plugin {
            manifest,
            path: path.clone(),
            enabled: true,
        })
    }

    /// Get all loaded plugins
    pub fn plugins(&self) -> &[Plugin] {
        &self.plugins
    }

    /// Enable a plugin by name
    pub fn enable(&mut self, name: &str) -> bool {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.manifest.name == name) {
            plugin.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable a plugin by name
    pub fn disable(&mut self, name: &str) -> bool {
        if let Some(plugin) = self.plugins.iter_mut().find(|p| p.manifest.name == name) {
            plugin.enabled = false;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_type_serialization() {
        let wasm = PluginType::Wasm;
        let lua = PluginType::Lua;
        assert_eq!(serde_json::to_string(&wasm).unwrap(), "\"wasm\"");
        assert_eq!(serde_json::to_string(&lua).unwrap(), "\"lua\"");
    }
}
