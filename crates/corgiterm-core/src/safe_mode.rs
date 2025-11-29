//! Safe Mode - Command preview and risk assessment
//!
//! For users who are nervous about terminals, Safe Mode provides:
//! - Preview of what a command will do before execution
//! - Risk level assessment (safe, caution, danger)
//! - Undo suggestions for dangerous operations
//! - Natural language explanations
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────────┐
//! │ 🐕 Safe Mode Preview                                              │
//! ├──────────────────────────────────────────────────────────────────┤
//! │                                                                  │
//! │  Command: rm -rf ./node_modules                                  │
//! │                                                                  │
//! │  ⚠️  CAUTION - This will permanently delete files               │
//! │                                                                  │
//! │  What it does:                                                   │
//! │  • Recursively removes the 'node_modules' directory             │
//! │  • This will delete ~1,247 files (148 MB)                       │
//! │  • Files will be permanently deleted (not moved to trash)       │
//! │                                                                  │
//! │  To undo: npm install                                           │
//! │                                                                  │
//! │  ┌────────────┐  ┌─────────────────┐                            │
//! │  │  Execute   │  │  Cancel (Esc)   │                            │
//! │  └────────────┘  └─────────────────┘                            │
//! └──────────────────────────────────────────────────────────────────┘
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Risk level for a command
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Safe - read-only or easily reversible
    Safe,
    /// Caution - makes changes but can be undone
    Caution,
    /// Danger - destructive or hard to reverse
    Danger,
    /// Unknown - can't determine risk
    Unknown,
}

impl RiskLevel {
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Safe => "✅",
            Self::Caution => "⚠️",
            Self::Danger => "🚨",
            Self::Unknown => "❓",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Safe => "SAFE",
            Self::Caution => "CAUTION",
            Self::Danger => "DANGER",
            Self::Unknown => "UNKNOWN",
        }
    }
}

/// Preview information for a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPreview {
    /// The original command
    pub command: String,
    /// Risk assessment
    pub risk: RiskLevel,
    /// Human-readable explanation
    pub explanation: Vec<String>,
    /// Files that will be affected
    pub affected_files: Vec<PathBuf>,
    /// Estimated count of affected items
    pub affected_count: Option<usize>,
    /// Estimated size of affected data
    pub affected_size: Option<u64>,
    /// Suggested undo command
    pub undo_hint: Option<String>,
    /// Whether this needs sudo
    pub needs_sudo: bool,
    /// Network access required?
    pub network_access: bool,
    /// Similar safe alternatives
    pub alternatives: Vec<CommandAlternative>,
}

/// A safer alternative to a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandAlternative {
    pub command: String,
    pub description: String,
    pub risk: RiskLevel,
}

/// Safe Mode analyzer
pub struct SafeMode {
    /// Dangerous command patterns
    dangerous_patterns: Vec<DangerPattern>,
    /// Safe command patterns
    safe_patterns: Vec<&'static str>,
    /// Is Safe Mode enabled?
    pub enabled: bool,
    /// AI integration for smart explanations
    ai_enabled: bool,
}

#[derive(Debug)]
struct DangerPattern {
    pattern: regex::Regex,
    risk: RiskLevel,
    explanation: &'static str,
    undo_hint: Option<&'static str>,
}

impl SafeMode {
    /// Create a new SafeMode analyzer
    pub fn new() -> Self {
        Self {
            dangerous_patterns: Self::default_dangerous_patterns(),
            safe_patterns: Self::default_safe_patterns(),
            enabled: false,
            ai_enabled: false,
        }
    }

    /// Enable or disable Safe Mode
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Enable AI-powered explanations
    pub fn set_ai_enabled(&mut self, enabled: bool) {
        self.ai_enabled = enabled;
    }

    /// Analyze a command and generate preview
    pub fn analyze(&self, command: &str, cwd: &PathBuf) -> CommandPreview {
        let mut preview = CommandPreview {
            command: command.to_string(),
            risk: RiskLevel::Unknown,
            explanation: Vec::new(),
            affected_files: Vec::new(),
            affected_count: None,
            affected_size: None,
            undo_hint: None,
            needs_sudo: command.starts_with("sudo "),
            network_access: false,
            alternatives: Vec::new(),
        };

        // Check for dangerous patterns
        for pattern in &self.dangerous_patterns {
            if pattern.pattern.is_match(command) {
                preview.risk = pattern.risk;
                preview.explanation.push(pattern.explanation.to_string());
                if let Some(undo) = pattern.undo_hint {
                    preview.undo_hint = Some(undo.to_string());
                }
            }
        }

        // Check for safe patterns
        for safe in &self.safe_patterns {
            if command.starts_with(safe) {
                preview.risk = RiskLevel::Safe;
                preview.explanation.clear();
                preview.explanation.push(format!("This is a safe, read-only command"));
                break;
            }
        }

        // If still unknown, try to infer
        if preview.risk == RiskLevel::Unknown {
            preview.risk = self.infer_risk(command);
        }

        // Try to count affected files for rm/mv/cp
        if let Some(files) = self.estimate_affected_files(command, cwd) {
            preview.affected_files = files.clone();
            preview.affected_count = Some(files.len());
            preview.affected_size = self.estimate_size(&files);
        }

        // Network commands
        if command.contains("curl") || command.contains("wget") ||
           command.contains("ssh") || command.contains("scp") ||
           command.contains("npm") || command.contains("pip") {
            preview.network_access = true;
        }

        // Generate alternatives for dangerous commands
        if preview.risk == RiskLevel::Danger {
            preview.alternatives = self.suggest_alternatives(command);
        }

        preview
    }

    fn default_dangerous_patterns() -> Vec<DangerPattern> {
        vec![
            DangerPattern {
                pattern: regex::Regex::new(r"rm\s+-rf?\s+/").unwrap(),
                risk: RiskLevel::Danger,
                explanation: "Recursively removes files from the root directory - EXTREMELY DANGEROUS",
                undo_hint: None,
            },
            DangerPattern {
                pattern: regex::Regex::new(r"rm\s+-rf").unwrap(),
                risk: RiskLevel::Caution,
                explanation: "Recursively removes files and directories without confirmation",
                undo_hint: Some("Files cannot be recovered - consider using trash-cli instead"),
            },
            DangerPattern {
                pattern: regex::Regex::new(r"rm\s").unwrap(),
                risk: RiskLevel::Caution,
                explanation: "Removes files - they will be permanently deleted",
                undo_hint: Some("Consider: trash-put <file> (recoverable)"),
            },
            DangerPattern {
                pattern: regex::Regex::new(r"chmod\s+-R\s+777").unwrap(),
                risk: RiskLevel::Danger,
                explanation: "Makes all files world-readable/writable - security risk",
                undo_hint: None,
            },
            DangerPattern {
                pattern: regex::Regex::new(r":\(\)\s*\{\s*:\|:\s*&\s*\};:").unwrap(),
                risk: RiskLevel::Danger,
                explanation: "Fork bomb - will crash your system",
                undo_hint: None,
            },
            DangerPattern {
                pattern: regex::Regex::new(r"dd\s+if=").unwrap(),
                risk: RiskLevel::Danger,
                explanation: "Low-level disk write - can overwrite entire drives",
                undo_hint: None,
            },
            DangerPattern {
                pattern: regex::Regex::new(r"mkfs").unwrap(),
                risk: RiskLevel::Danger,
                explanation: "Formats a filesystem - all data will be lost",
                undo_hint: None,
            },
            DangerPattern {
                pattern: regex::Regex::new(r">\s*/dev/").unwrap(),
                risk: RiskLevel::Danger,
                explanation: "Writing directly to device - can corrupt data",
                undo_hint: None,
            },
            DangerPattern {
                pattern: regex::Regex::new(r"git\s+push\s+(-f|--force)").unwrap(),
                risk: RiskLevel::Caution,
                explanation: "Force pushing can overwrite remote history",
                undo_hint: Some("git reflog to find lost commits"),
            },
            DangerPattern {
                pattern: regex::Regex::new(r"git\s+reset\s+--hard").unwrap(),
                risk: RiskLevel::Caution,
                explanation: "Discards all uncommitted changes",
                undo_hint: Some("git reflog to recover"),
            },
        ]
    }

    fn default_safe_patterns() -> Vec<&'static str> {
        vec![
            "ls",
            "pwd",
            "echo",
            "cat",
            "less",
            "head",
            "tail",
            "grep",
            "find",
            "which",
            "whereis",
            "man",
            "help",
            "type",
            "file",
            "stat",
            "wc",
            "diff",
            "git status",
            "git log",
            "git diff",
            "git branch",
            "git show",
            "ps",
            "top",
            "htop",
            "df",
            "du",
            "free",
            "uptime",
            "whoami",
            "date",
            "cal",
            "env",
            "printenv",
        ]
    }

    fn infer_risk(&self, command: &str) -> RiskLevel {
        // Commands that modify things
        if command.contains("mv ") || command.contains("cp ") {
            return RiskLevel::Caution;
        }

        // Install commands
        if command.contains("install") || command.contains("apt") ||
           command.contains("dnf") || command.contains("yum") {
            return RiskLevel::Caution;
        }

        // Anything with sudo
        if command.starts_with("sudo") {
            return RiskLevel::Caution;
        }

        RiskLevel::Unknown
    }

    fn estimate_affected_files(&self, command: &str, cwd: &PathBuf) -> Option<Vec<PathBuf>> {
        // Simple parser for rm, mv, cp targets
        // In a real implementation, this would do proper shell expansion
        let parts: Vec<&str> = command.split_whitespace().collect();

        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "rm" | "mv" | "cp" => {
                let mut files = Vec::new();
                for part in parts.iter().skip(1) {
                    if !part.starts_with('-') {
                        let path = if part.starts_with('/') {
                            PathBuf::from(part)
                        } else {
                            cwd.join(part)
                        };
                        if path.exists() {
                            files.push(path);
                        }
                    }
                }
                if files.is_empty() { None } else { Some(files) }
            }
            _ => None,
        }
    }

    fn estimate_size(&self, files: &[PathBuf]) -> Option<u64> {
        let mut total = 0u64;
        for file in files {
            if let Ok(metadata) = file.metadata() {
                if metadata.is_dir() {
                    // Would need to walk directory for accurate size
                    total += 1024 * 1024; // Estimate 1MB per directory
                } else {
                    total += metadata.len();
                }
            }
        }
        if total > 0 { Some(total) } else { None }
    }

    fn suggest_alternatives(&self, command: &str) -> Vec<CommandAlternative> {
        let mut alternatives = Vec::new();

        if command.contains("rm ") {
            alternatives.push(CommandAlternative {
                command: command.replace("rm ", "trash-put "),
                description: "Move to trash instead (recoverable)".to_string(),
                risk: RiskLevel::Safe,
            });
        }

        if command.contains("rm -rf") {
            alternatives.push(CommandAlternative {
                command: command.replace("rm -rf", "rm -ri"),
                description: "Interactive mode - confirm each file".to_string(),
                risk: RiskLevel::Caution,
            });
        }

        alternatives
    }
}

impl Default for SafeMode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_command() {
        let safe_mode = SafeMode::new();
        let preview = safe_mode.analyze("ls -la", &PathBuf::from("/home"));
        assert_eq!(preview.risk, RiskLevel::Safe);
    }

    #[test]
    fn test_dangerous_command() {
        let safe_mode = SafeMode::new();
        let preview = safe_mode.analyze("rm -rf /", &PathBuf::from("/home"));
        assert_eq!(preview.risk, RiskLevel::Danger);
    }

    #[test]
    fn test_caution_command() {
        let safe_mode = SafeMode::new();
        let preview = safe_mode.analyze("rm -rf node_modules", &PathBuf::from("/home"));
        assert_eq!(preview.risk, RiskLevel::Caution);
        assert!(!preview.alternatives.is_empty());
    }
}
