//! # CorgiTerm AI Integration
//!
//! Unified AI layer supporting multiple providers:
//! - Claude (Anthropic)
//! - OpenAI Codex / GPT-4
//! - Google Gemini
//! - Local LLMs (Ollama, llama.cpp)
//!
//! Features:
//! - Natural language to command translation
//! - Command explanations and documentation
//! - Smart auto-completion
//! - Safe Mode AI explanations
//! - MCP (Model Context Protocol) support

pub mod providers;
pub mod natural_language;
pub mod completions;
pub mod mcp;
pub mod learning;
pub mod history;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// AI-related errors
#[derive(Error, Debug)]
pub enum AiError {
    #[error("Provider not configured: {0}")]
    NotConfigured(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Provider not available: {0}")]
    Unavailable(String),
}

pub type Result<T> = std::result::Result<T, AiError>;

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// Message role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

/// AI response
#[derive(Debug, Clone)]
pub struct AiResponse {
    /// The generated content
    pub content: String,
    /// Provider that generated this
    pub provider: String,
    /// Model used
    pub model: String,
    /// Tokens used (if available)
    pub tokens_used: Option<TokenUsage>,
    /// Time taken in milliseconds
    pub latency_ms: u64,
}

/// Token usage stats
#[derive(Debug, Clone, Copy)]
pub struct TokenUsage {
    pub prompt: u32,
    pub completion: u32,
    pub total: u32,
}

/// AI provider trait
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Check if provider is available
    async fn is_available(&self) -> bool;

    /// Send a completion request
    async fn complete(&self, messages: &[Message]) -> Result<AiResponse>;

    /// Stream a completion request with incremental callback
    /// The callback receives each chunk as an owned String
    async fn complete_stream(
        &self,
        messages: &[Message],
        callback: Box<dyn Fn(String) + Send>,
    ) -> Result<AiResponse>;
}

/// Natural language command translation
pub struct NaturalLanguage {
    provider: Box<dyn AiProvider>,
}

impl NaturalLanguage {
    pub fn new(provider: Box<dyn AiProvider>) -> Self {
        Self { provider }
    }

    /// Translate natural language to shell command
    pub async fn to_command(&self, input: &str, context: &CommandContext) -> Result<CommandSuggestion> {
        let system_prompt = format!(
            r##"You are a shell command expert. Convert natural language requests into shell commands.

Current directory: {}
Shell: {}
OS: {}

Rules:
1. Output ONLY the command, no explanation
2. Use safe, standard commands
3. Prefer interactive flags when appropriate
4. If dangerous, prefix with "# WARNING: "

Examples:
"show files bigger than 1GB" -> find . -size +1G -type f
"delete node modules" -> rm -rf node_modules
"what's using port 3000" -> lsof -i :3000
"##,
            context.cwd.display(),
            context.shell,
            context.os
        );

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
            explanation: None,
            confidence: 0.9,
            is_dangerous: response.content.contains("# WARNING"),
        })
    }

    /// Explain what a command does
    pub async fn explain_command(&self, command: &str) -> Result<String> {
        let messages = vec![
            Message {
                role: Role::System,
                content: "Explain shell commands in simple terms. Be concise.".to_string(),
            },
            Message {
                role: Role::User,
                content: format!("What does this command do? {}", command),
            },
        ];

        let response = self.provider.complete(&messages).await?;
        Ok(response.content)
    }
}

/// Context for command translation
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub cwd: std::path::PathBuf,
    pub shell: String,
    pub os: String,
    pub recent_commands: Vec<String>,
}

impl Default for CommandContext {
    fn default() -> Self {
        Self {
            cwd: std::env::current_dir().unwrap_or_default(),
            shell: std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()),
            os: std::env::consts::OS.to_string(),
            recent_commands: Vec::new(),
        }
    }
}

/// A suggested command
#[derive(Debug, Clone)]
pub struct CommandSuggestion {
    pub command: String,
    pub explanation: Option<String>,
    pub confidence: f32,
    pub is_dangerous: bool,
}

/// AI panel for the UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiPanelMode {
    /// Chat mode - conversational AI
    Chat,
    /// Command mode - natural language to command
    Command,
    /// Explain mode - explain selected text/command
    Explain,
    /// Generate mode - generate code/scripts
    Generate,
}

/// Manages AI providers and routing
pub struct AiManager {
    providers: Vec<Box<dyn AiProvider>>,
    default_provider: usize,
}

impl AiManager {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            default_provider: 0,
        }
    }

    /// Add a provider
    pub fn add_provider(&mut self, provider: Box<dyn AiProvider>) {
        self.providers.push(provider);
    }

    /// Set default provider by name
    pub fn set_default(&mut self, name: &str) -> bool {
        if let Some(idx) = self.providers.iter().position(|p| p.name() == name) {
            self.default_provider = idx;
            true
        } else {
            false
        }
    }

    /// Get default provider
    pub fn default_provider(&self) -> Option<&dyn AiProvider> {
        self.providers.get(self.default_provider).map(|p| p.as_ref())
    }

    /// Get provider by name
    pub fn get_provider(&self, name: &str) -> Option<&dyn AiProvider> {
        self.providers.iter().find(|p| p.name() == name).map(|p| p.as_ref())
    }

    /// List available providers
    pub fn list_providers(&self) -> Vec<&str> {
        self.providers.iter().map(|p| p.name()).collect()
    }

    /// Check which providers are available
    pub async fn check_availability(&self) -> Vec<(String, bool)> {
        let mut results = Vec::new();
        for provider in &self.providers {
            let available = provider.is_available().await;
            results.push((provider.name().to_string(), available));
        }
        results
    }
}

impl Default for AiManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_context_default() {
        let ctx = CommandContext::default();
        assert!(!ctx.shell.is_empty());
        assert!(!ctx.os.is_empty());
    }
}
