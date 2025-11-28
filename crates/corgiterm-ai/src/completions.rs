//! Smart command completions powered by AI

use crate::{AiProvider, Message, Result, Role};

/// AI-powered completion engine
pub struct CompletionEngine {
    provider: Box<dyn AiProvider>,
    context_lines: usize,
}

impl CompletionEngine {
    pub fn new(provider: Box<dyn AiProvider>) -> Self {
        Self {
            provider,
            context_lines: 10,
        }
    }

    /// Get completions for partial input
    pub async fn complete(&self, input: &str, context: &CompletionContext) -> Result<Vec<Completion>> {
        let prompt = format!(
            r#"Complete this shell command. Output only the completed command(s), one per line.

Working directory: {}
Shell: {}
Recent commands:
{}

Partial input: {}"#,
            context.cwd.display(),
            context.shell,
            context.recent_output.join("\n"),
            input
        );

        let messages = vec![
            Message {
                role: Role::System,
                content: "You are a shell completion engine. Output only valid shell commands.".to_string(),
            },
            Message {
                role: Role::User,
                content: prompt,
            },
        ];

        let response = self.provider.complete(&messages).await?;

        let completions = response
            .content
            .lines()
            .filter(|l| !l.is_empty())
            .take(5)
            .map(|line| Completion {
                text: line.to_string(),
                description: None,
                kind: CompletionKind::Command,
            })
            .collect();

        Ok(completions)
    }
}

/// Context for completion
#[derive(Debug, Clone)]
pub struct CompletionContext {
    pub cwd: std::path::PathBuf,
    pub shell: String,
    pub recent_output: Vec<String>,
    pub env: std::collections::HashMap<String, String>,
}

impl Default for CompletionContext {
    fn default() -> Self {
        Self {
            cwd: std::env::current_dir().unwrap_or_default(),
            shell: std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string()),
            recent_output: Vec::new(),
            env: std::env::vars().collect(),
        }
    }
}

/// A completion suggestion
#[derive(Debug, Clone)]
pub struct Completion {
    /// The completion text
    pub text: String,
    /// Optional description
    pub description: Option<String>,
    /// Type of completion
    pub kind: CompletionKind,
}

/// Types of completions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Command,
    Path,
    Argument,
    Option,
    Variable,
}
