//! AI Provider implementations

use crate::{AiError, AiProvider, AiResponse, Message, Result, Role, TokenUsage};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Claude (Anthropic) provider
pub struct ClaudeProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl ClaudeProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
        }
    }
}

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    system: Option<String>,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
    usage: ClaudeUsage,
}

#[derive(Deserialize)]
struct ClaudeContent {
    text: String,
}

#[derive(Deserialize)]
struct ClaudeUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[async_trait]
impl AiProvider for ClaudeProvider {
    fn name(&self) -> &str {
        "claude"
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn complete(&self, messages: &[Message]) -> Result<AiResponse> {
        let start = Instant::now();

        // Extract system message if present
        let system = messages
            .iter()
            .find(|m| m.role == Role::System)
            .map(|m| m.content.clone());

        let claude_messages: Vec<ClaudeMessage> = messages
            .iter()
            .filter(|m| m.role != Role::System)
            .map(|m| ClaudeMessage {
                role: match m.role {
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::System => unreachable!(),
                },
                content: m.content.clone(),
            })
            .collect();

        let request = ClaudeRequest {
            model: self.model.clone(),
            max_tokens: 4096,
            messages: claude_messages,
            system,
        };

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AiError::ApiError(format!("{}: {}", status, text)));
        }

        let claude_response: ClaudeResponse = response.json().await?;

        let content = claude_response
            .content
            .first()
            .map(|c| c.text.clone())
            .unwrap_or_default();

        Ok(AiResponse {
            content,
            provider: "claude".to_string(),
            model: self.model.clone(),
            tokens_used: Some(TokenUsage {
                prompt: claude_response.usage.input_tokens,
                completion: claude_response.usage.output_tokens,
                total: claude_response.usage.input_tokens + claude_response.usage.output_tokens,
            }),
            latency_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn complete_stream(
        &self,
        messages: &[Message],
        _callback: Box<dyn Fn(&str) + Send>,
    ) -> Result<AiResponse> {
        // For now, fall back to non-streaming
        // TODO: Implement proper SSE streaming
        self.complete(messages).await
    }
}

/// OpenAI provider (GPT-4, Codex)
pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: model.unwrap_or_else(|| "gpt-4o".to_string()),
        }
    }
}

#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    fn name(&self) -> &str {
        "openai"
    }

    async fn is_available(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn complete(&self, messages: &[Message]) -> Result<AiResponse> {
        let start = Instant::now();

        let openai_messages: Vec<OpenAiMessage> = messages
            .iter()
            .map(|m| OpenAiMessage {
                role: match m.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                },
                content: m.content.clone(),
            })
            .collect();

        let request = OpenAiRequest {
            model: self.model.clone(),
            messages: openai_messages,
            max_tokens: 4096,
        };

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AiError::ApiError(format!("{}: {}", status, text)));
        }

        let openai_response: OpenAiResponse = response.json().await?;

        let content = openai_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        Ok(AiResponse {
            content,
            provider: "openai".to_string(),
            model: self.model.clone(),
            tokens_used: Some(TokenUsage {
                prompt: openai_response.usage.prompt_tokens,
                completion: openai_response.usage.completion_tokens,
                total: openai_response.usage.total_tokens,
            }),
            latency_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn complete_stream(
        &self,
        messages: &[Message],
        _callback: Box<dyn Fn(&str) + Send>,
    ) -> Result<AiResponse> {
        self.complete(messages).await
    }
}

/// Local LLM provider (Ollama)
pub struct OllamaProvider {
    client: Client,
    endpoint: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(endpoint: String, model: String) -> Self {
        Self {
            client: Client::new(),
            endpoint,
            model,
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[async_trait]
impl AiProvider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn is_available(&self) -> bool {
        let url = format!("{}/api/tags", self.endpoint);
        self.client.get(&url).send().await.is_ok()
    }

    async fn complete(&self, messages: &[Message]) -> Result<AiResponse> {
        let start = Instant::now();

        // Convert messages to a single prompt
        let prompt = messages
            .iter()
            .map(|m| match m.role {
                Role::System => format!("System: {}\n", m.content),
                Role::User => format!("User: {}\n", m.content),
                Role::Assistant => format!("Assistant: {}\n", m.content),
            })
            .collect::<String>()
            + "Assistant: ";

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
        };

        let url = format!("{}/api/generate", self.endpoint);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AiError::ApiError(format!("{}: {}", status, text)));
        }

        let ollama_response: OllamaResponse = response.json().await?;

        Ok(AiResponse {
            content: ollama_response.response,
            provider: "ollama".to_string(),
            model: self.model.clone(),
            tokens_used: None,
            latency_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn complete_stream(
        &self,
        messages: &[Message],
        _callback: Box<dyn Fn(&str) + Send>,
    ) -> Result<AiResponse> {
        self.complete(messages).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_provider_name() {
        let provider = ClaudeProvider::new("test".to_string(), None);
        assert_eq!(provider.name(), "claude");
    }

    #[test]
    fn test_openai_provider_name() {
        let provider = OpenAiProvider::new("test".to_string(), None);
        assert_eq!(provider.name(), "openai");
    }

    #[test]
    fn test_ollama_provider_name() {
        let provider = OllamaProvider::new("http://localhost:11434".to_string(), "llama3".to_string());
        assert_eq!(provider.name(), "ollama");
    }
}
