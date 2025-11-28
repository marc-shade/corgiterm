//! JSON Schema generation for configuration
//!
//! Generates schemas for editor autocomplete and validation.

use serde_json::json;

/// Generate JSON schema for configuration
/// TODO: Add full JsonSchema derive to all config types
pub fn generate_schema() -> serde_json::Value {
    json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "CorgiTerm Configuration",
        "description": "Configuration schema for CorgiTerm terminal emulator",
        "type": "object",
        "properties": {
            "general": { "type": "object", "description": "General settings" },
            "appearance": { "type": "object", "description": "Appearance settings" },
            "terminal": { "type": "object", "description": "Terminal behavior" },
            "keybindings": { "type": "object", "description": "Keyboard shortcuts" },
            "ai": { "type": "object", "description": "AI integration" },
            "safe_mode": { "type": "object", "description": "Safe Mode settings" },
            "sessions": { "type": "object", "description": "Session management" },
            "performance": { "type": "object", "description": "Performance settings" },
            "accessibility": { "type": "object", "description": "Accessibility settings" },
            "advanced": { "type": "object", "description": "Advanced settings" }
        }
    })
}

/// Save schema to file
pub fn save_schema(path: &std::path::Path) -> anyhow::Result<()> {
    let schema = generate_schema();
    let json = serde_json::to_string_pretty(&schema)?;
    std::fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_generation() {
        let schema = generate_schema();
        assert!(schema.get("$schema").is_some());
    }
}
