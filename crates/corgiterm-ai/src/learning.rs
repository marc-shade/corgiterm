//! AI learning integration
//!
//! Enhances AI suggestions with learned command patterns and user preferences

use crate::{AiProvider, CommandContext, CommandSuggestion, Message, Result, Role};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context built from command history for AI suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningContext {
    /// Most frequent commands
    pub frequent_commands: Vec<FrequentCommand>,
    /// User preferences for alternative commands
    pub preferences: Vec<CommandPreference>,
    /// Common command patterns
    pub patterns: Vec<CommandPatternInfo>,
    /// Directory-specific commands
    pub directory_commands: HashMap<String, Vec<String>>,
}

impl Default for LearningContext {
    fn default() -> Self {
        Self {
            frequent_commands: Vec::new(),
            preferences: Vec::new(),
            patterns: Vec::new(),
            directory_commands: HashMap::new(),
        }
    }
}

/// A frequently used command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequentCommand {
    pub command: String,
    pub count: usize,
    pub success_rate: f32,
}

/// User's command preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPreference {
    pub standard: String,
    pub preferred: String,
    pub ratio: f32,
}

/// Information about a command pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandPatternInfo {
    pub sequence: Vec<String>,
    pub frequency: usize,
    pub confidence: f32,
}

/// AI with learning capabilities
pub struct LearningAi {
    provider: Box<dyn AiProvider>,
    learning_context: LearningContext,
}

impl LearningAi {
    pub fn new(provider: Box<dyn AiProvider>) -> Self {
        Self {
            provider,
            learning_context: LearningContext::default(),
        }
    }

    /// Update learning context
    pub fn update_context(&mut self, context: LearningContext) {
        self.learning_context = context;
    }

    /// Get learning context
    pub fn context(&self) -> &LearningContext {
        &self.learning_context
    }

    /// Generate command with learned context
    pub async fn generate_command_with_learning(
        &self,
        input: &str,
        cmd_context: &CommandContext,
    ) -> Result<CommandSuggestion> {
        let system_prompt = self.build_learning_prompt(cmd_context);

        let messages = vec![
            Message {
                role: Role::System,
                content: system_prompt,
            },
            Message {
                role: Role::User,
                content: input.to_string(),
            },
        ];

        let response = self.provider.complete(&messages).await?;

        Ok(CommandSuggestion {
            command: response.content.trim().to_string(),
            explanation: Some("Generated with learned patterns".to_string()),
            confidence: 0.85,
            is_dangerous: response.content.contains("rm ")
                || response.content.contains("# WARNING"),
        })
    }

    /// Suggest next command based on current command
    pub async fn suggest_next_command(
        &self,
        current_command: &str,
    ) -> Result<Vec<CommandSuggestion>> {
        // First, check learned patterns
        let mut suggestions = Vec::new();

        // Look for patterns
        for pattern in &self.learning_context.patterns {
            if let Some(first) = pattern.sequence.first() {
                if current_command.starts_with(first) {
                    if let Some(next) = pattern.sequence.get(1) {
                        suggestions.push(CommandSuggestion {
                            command: next.clone(),
                            explanation: Some(format!(
                                "Often follows {} ({}% of the time)",
                                first,
                                (pattern.confidence * 100.0) as u32
                            )),
                            confidence: pattern.confidence,
                            is_dangerous: false,
                        });
                    }
                }
            }
        }

        // If we have learned suggestions, return them
        if !suggestions.is_empty() {
            return Ok(suggestions);
        }

        // Otherwise, ask AI with context
        let prompt = format!(
            "User just executed: {}\n\nBased on common command sequences, what command might they want to run next? Output only the command.",
            current_command
        );

        let messages = vec![
            Message {
                role: Role::System,
                content: "You are a shell command expert. Suggest logical next commands."
                    .to_string(),
            },
            Message {
                role: Role::User,
                content: prompt,
            },
        ];

        let response = self.provider.complete(&messages).await?;

        Ok(vec![CommandSuggestion {
            command: response.content.trim().to_string(),
            explanation: Some("AI suggested next command".to_string()),
            confidence: 0.6,
            is_dangerous: false,
        }])
    }

    /// Get suggestions based on user's history in current directory
    pub fn directory_suggestions(&self, directory: &str, limit: usize) -> Vec<CommandSuggestion> {
        if let Some(commands) = self.learning_context.directory_commands.get(directory) {
            commands
                .iter()
                .take(limit)
                .map(|cmd| CommandSuggestion {
                    command: cmd.clone(),
                    explanation: Some(format!("Frequently used in {}", directory)),
                    confidence: 0.7,
                    is_dangerous: false,
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Build system prompt with learning context
    fn build_learning_prompt(&self, cmd_context: &CommandContext) -> String {
        let mut prompt = format!(
            r##"You are a shell command expert. Convert natural language requests into shell commands.

Current directory: {}
Shell: {}
OS: {}

"##,
            cmd_context.cwd.display(),
            cmd_context.shell,
            cmd_context.os
        );

        // Add user preferences
        if !self.learning_context.preferences.is_empty() {
            prompt.push_str("USER PREFERENCES (always use these when applicable):\n");
            for pref in &self.learning_context.preferences {
                prompt.push_str(&format!(
                    "- Prefers '{}' over '{}' ({}% of the time)\n",
                    pref.preferred,
                    pref.standard,
                    (pref.ratio * 100.0) as u32
                ));
            }
            prompt.push('\n');
        }

        // Add frequent commands
        if !self.learning_context.frequent_commands.is_empty() {
            prompt.push_str("USER'S MOST COMMON COMMANDS:\n");
            for cmd in self.learning_context.frequent_commands.iter().take(10) {
                prompt.push_str(&format!(
                    "- {} (used {} times, {}% success rate)\n",
                    cmd.command,
                    cmd.count,
                    (cmd.success_rate * 100.0) as u32
                ));
            }
            prompt.push('\n');
        }

        // Add directory-specific commands
        if let Some(dir_cmds) = self
            .learning_context
            .directory_commands
            .get(&cmd_context.cwd.display().to_string())
        {
            if !dir_cmds.is_empty() {
                prompt.push_str("COMMANDS COMMONLY USED IN THIS DIRECTORY:\n");
                for cmd in dir_cmds.iter().take(5) {
                    prompt.push_str(&format!("- {}\n", cmd));
                }
                prompt.push('\n');
            }
        }

        // Add patterns
        if !self.learning_context.patterns.is_empty() {
            prompt.push_str("LEARNED COMMAND PATTERNS:\n");
            for pattern in self.learning_context.patterns.iter().take(5) {
                let sequence_str = pattern.sequence.join(" → ");
                prompt.push_str(&format!(
                    "- {} (seen {} times)\n",
                    sequence_str, pattern.frequency
                ));
            }
            prompt.push('\n');
        }

        prompt.push_str(
            r##"Rules:
1. Output ONLY the command, no explanation
2. Use user's preferred commands when possible
3. Consider the directory context
4. Use safe, standard commands
5. If dangerous, prefix with "# WARNING: "

Examples:
"show files bigger than 1GB" -> find . -size +1G -type f
"delete node modules" -> rm -rf node_modules
"what's using port 3000" -> lsof -i :3000
"##,
        );

        prompt
    }

    /// Explain why a command was suggested
    pub async fn explain_suggestion(&self, command: &str, reason: &str) -> Result<String> {
        let messages = vec![
            Message {
                role: Role::System,
                content: "Explain shell command suggestions clearly and concisely.".to_string(),
            },
            Message {
                role: Role::User,
                content: format!(
                    "I was suggested the command '{}' because: {}\n\nProvide a brief explanation of what this command does and why it was suggested.",
                    command, reason
                ),
            },
        ];

        let response = self.provider.complete(&messages).await?;
        Ok(response.content)
    }

    /// Get frequently used commands for display
    pub fn frequent_commands(&self, limit: usize) -> Vec<&FrequentCommand> {
        self.learning_context
            .frequent_commands
            .iter()
            .take(limit)
            .collect()
    }

    /// Get user preferences for display
    pub fn preferences(&self) -> &[CommandPreference] {
        &self.learning_context.preferences
    }

    /// Get command patterns
    pub fn patterns(&self) -> &[CommandPatternInfo] {
        &self.learning_context.patterns
    }

    /// Generate insights about user's command usage
    pub async fn generate_usage_insights(&self) -> Result<String> {
        let mut insights = Vec::new();

        // Analyze preferences
        if !self.learning_context.preferences.is_empty() {
            let pref_list: Vec<String> = self
                .learning_context
                .preferences
                .iter()
                .map(|p| format!("{} (instead of {})", p.preferred, p.standard))
                .collect();
            insights.push(format!(
                "You prefer modern alternatives: {}",
                pref_list.join(", ")
            ));
        }

        // Analyze most used commands
        if let Some(top_cmd) = self.learning_context.frequent_commands.first() {
            insights.push(format!(
                "Your most used command is '{}' with {} executions",
                top_cmd.command, top_cmd.count
            ));
        }

        // Analyze patterns
        if !self.learning_context.patterns.is_empty() {
            let pattern_count = self.learning_context.patterns.len();
            insights.push(format!(
                "I've learned {} command patterns from your workflow",
                pattern_count
            ));
        }

        if insights.is_empty() {
            insights
                .push("Not enough data yet to generate insights. Keep using commands!".to_string());
        }

        Ok(insights.join("\n• "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_learning_context_default() {
        let ctx = LearningContext::default();
        assert!(ctx.frequent_commands.is_empty());
        assert!(ctx.preferences.is_empty());
    }

    #[test]
    fn test_directory_suggestions() {
        let mut context = LearningContext::default();
        context.directory_commands.insert(
            "/home/user/project".to_string(),
            vec!["cargo build".to_string(), "cargo test".to_string()],
        );

        // Mock provider would be needed for full test
        // This is just a structure test
        assert_eq!(context.directory_commands.len(), 1);
    }
}
