//! AI Provider implementations

use crate::{AiError, AiProvider, AiResponse, Message, Result, Role, TokenUsage};
use async_trait::async_trait;
use futures::StreamExt;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
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

/// SSE streaming event for Claude
#[derive(Deserialize)]
struct ClaudeStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    delta: Option<ClaudeStreamDelta>,
    #[serde(default)]
    usage: Option<ClaudeUsage>,
}

#[derive(Deserialize, Default)]
struct ClaudeStreamDelta {
    #[serde(default)]
    text: String,
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
            stream: None,
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
        callback: Box<dyn Fn(String) + Send>,
    ) -> Result<AiResponse> {
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
            stream: Some(true),
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

        let mut full_content = String::new();
        let mut usage: Option<ClaudeUsage> = None;

        // Process SSE stream
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| AiError::ApiError(format!("Stream error: {}", e)))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete SSE events
            while let Some(event_end) = buffer.find("\n\n") {
                let event_data = buffer[..event_end].to_string();
                buffer = buffer[event_end + 2..].to_string();

                // Parse SSE event
                for line in event_data.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            continue;
                        }
                        if let Ok(event) = serde_json::from_str::<ClaudeStreamEvent>(data) {
                            match event.event_type.as_str() {
                                "content_block_delta" => {
                                    if let Some(delta) = event.delta {
                                        if !delta.text.is_empty() {
                                            // Pass owned string to callback
                                            callback(delta.text.clone());
                                            full_content.push_str(&delta.text);
                                        }
                                    }
                                }
                                "message_delta" => {
                                    if let Some(u) = event.usage {
                                        usage = Some(u);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        let final_content = full_content;

        Ok(AiResponse {
            content: final_content,
            provider: "claude".to_string(),
            model: self.model.clone(),
            tokens_used: usage.map(|u| TokenUsage {
                prompt: u.input_tokens,
                completion: u.output_tokens,
                total: u.input_tokens + u.output_tokens,
            }),
            latency_ms: start.elapsed().as_millis() as u64,
        })
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
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
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

/// Streaming response chunk from OpenAI
#[derive(Deserialize)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiStreamChoice>,
}

#[derive(Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiStreamDelta,
}

#[derive(Deserialize, Default)]
struct OpenAiStreamDelta {
    #[serde(default)]
    content: Option<String>,
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
            stream: None,
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
        callback: Box<dyn Fn(String) + Send>,
    ) -> Result<AiResponse> {
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
            stream: Some(true),
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

        let mut full_content = String::new();

        // Process SSE stream
        let mut stream = response.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| AiError::ApiError(format!("Stream error: {}", e)))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            // Process complete SSE events
            while let Some(event_end) = buffer.find("\n\n") {
                let event_data = buffer[..event_end].to_string();
                buffer = buffer[event_end + 2..].to_string();

                // Parse SSE event
                for line in event_data.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            continue;
                        }
                        if let Ok(parsed) = serde_json::from_str::<OpenAiStreamChunk>(data) {
                            for choice in parsed.choices {
                                if let Some(content) = choice.delta.content {
                                    // Pass owned string to callback
                                    callback(content.clone());
                                    full_content.push_str(&content);
                                }
                            }
                        }
                    }
                }
            }
        }

        let final_content = full_content;

        Ok(AiResponse {
            content: final_content,
            provider: "openai".to_string(),
            model: self.model.clone(),
            tokens_used: None, // Token usage not available in streaming mode
            latency_ms: start.elapsed().as_millis() as u64,
        })
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
        _callback: Box<dyn Fn(String) + Send>,
    ) -> Result<AiResponse> {
        // Ollama doesn't support streaming in this implementation - fall back to complete
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
