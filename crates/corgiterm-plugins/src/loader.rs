//! Plugin loading utilities

use crate::{Plugin, PluginManifest, PluginType};
use std::path::Path;

/// Load a plugin manifest from a directory
pub fn load_manifest(plugin_dir: &Path) -> anyhow::Result<PluginManifest> {
    let manifest_path = plugin_dir.join("plugin.toml");
    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest: PluginManifest = toml::from_str(&content)?;
    Ok(manifest)
}

/// Validate a plugin manifest
pub fn validate_manifest(manifest: &PluginManifest) -> Result<(), String> {
    if manifest.name.is_empty() {
        return Err("Plugin name is required".to_string());
    }

    if manifest.version.is_empty() {
        return Err("Plugin version is required".to_string());
    }

    if manifest.entry.is_empty() {
        return Err("Plugin entry point is required".to_string());
    }

    // Validate version is semver
    if semver::Version::parse(&manifest.version).is_err() {
        return Err("Plugin version must be valid semver".to_string());
    }

    Ok(())
}
