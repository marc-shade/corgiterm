//! Dynamic model discovery and caching for AI providers
//!
//! This module provides:
//! - Automatic model list fetching from each provider's API
//! - Intelligent caching with configurable TTL
//! - Background refresh to keep model lists current

use crate::{AiError, Result};
use chrono::{DateTime, Duration, Utc};
use parking_lot::RwLock;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Information about an AI model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model identifier (e.g., "gpt-4o", "claude-sonnet-4-20250514")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Provider this model belongs to
    pub provider: String,
    /// Model description
    pub description: Option<String>,
    /// Context window size (tokens)
    pub context_window: Option<u32>,
    /// Maximum output tokens
    pub max_output: Option<u32>,
    /// Whether the model supports vision/images
    pub supports_vision: bool,
    /// Whether the model supports tool use/function calling
    pub supports_tools: bool,
    /// Model size in bytes (for local models)
    pub size_bytes: Option<u64>,
    /// Model family/series (e.g., "claude-3", "gpt-4")
    pub family: Option<String>,
    /// Release/modified date
    pub created_at: Option<DateTime<Utc>>,
    /// Whether this is a recommended/featured model
    pub recommended: bool,
}

impl ModelInfo {
    /// Create a new ModelInfo with minimal fields
    pub fn new(id: impl Into<String>, provider: impl Into<String>) -> Self {
        let id = id.into();
        Self {
            name: id.clone(),
            id,
            provider: provider.into(),
            description: None,
            context_window: None,
            max_output: None,
            supports_vision: false,
            supports_tools: false,
            size_bytes: None,
            family: None,
            created_at: None,
            recommended: false,
        }
    }

    /// Human-readable size (e.g., "4.7 GB")
    pub fn size_display(&self) -> Option<String> {
        self.size_bytes.map(|bytes| {
            if bytes >= 1_000_000_000 {
                format!("{:.1} GB", bytes as f64 / 1_000_000_000.0)
            } else if bytes >= 1_000_000 {
                format!("{:.1} MB", bytes as f64 / 1_000_000.0)
            } else {
                format!("{} bytes", bytes)
            }
        })
    }
}

/// Cached model list for a provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedModelList {
    pub models: Vec<ModelInfo>,
    pub fetched_at: DateTime<Utc>,
    pub provider: String,
}

impl CachedModelList {
    /// Check if cache is still valid
    pub fn is_valid(&self, ttl: Duration) -> bool {
        Utc::now() - self.fetched_at < ttl
    }
}

/// Configuration for model registry
#[derive(Debug, Clone)]
pub struct ModelRegistryConfig {
    /// Cache time-to-live for API providers (default: 1 hour)
    pub api_cache_ttl: Duration,
    /// Cache TTL for local providers like Ollama (default: 5 minutes)
    pub local_cache_ttl: Duration,
    /// Path to cache file
    pub cache_path: PathBuf,
    /// OpenAI API key
    pub openai_api_key: Option<String>,
    /// Anthropic API key
    pub anthropic_api_key: Option<String>,
    /// Google API key
    pub google_api_key: Option<String>,
    /// Ollama endpoint
    pub ollama_endpoint: String,
}

impl Default for ModelRegistryConfig {
    fn default() -> Self {
        let cache_path = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("corgiterm")
            .join("model_cache.json");

        Self {
            api_cache_ttl: Duration::hours(1),
            local_cache_ttl: Duration::minutes(5),
            cache_path,
            openai_api_key: std::env::var("OPENAI_API_KEY").ok(),
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
            google_api_key: std::env::var("GOOGLE_API_KEY")
                .or_else(|_| std::env::var("GEMINI_API_KEY"))
                .ok(),
            ollama_endpoint: std::env::var("OLLAMA_HOST")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
        }
    }
}

/// Central registry for managing model lists across all providers
pub struct ModelRegistry {
    config: ModelRegistryConfig,
    client: Client,
    cache: Arc<RwLock<HashMap<String, CachedModelList>>>,
}

impl ModelRegistry {
    /// Create a new ModelRegistry with default configuration
    pub fn new() -> Self {
        Self::with_config(ModelRegistryConfig::default())
    }

    /// Create a new ModelRegistry with custom configuration
    pub fn with_config(config: ModelRegistryConfig) -> Self {
        Self {
            config,
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load cache from disk (synchronous)
    pub fn load_cache(&self) -> Result<()> {
        if self.config.cache_path.exists() {
            match std::fs::read_to_string(&self.config.cache_path) {
                Ok(data) => {
                    if let Ok(cached) = serde_json::from_str::<HashMap<String, CachedModelList>>(&data) {
                        let mut cache = self.cache.write();
                        *cache = cached;
                        info!("Loaded model cache from {:?}", self.config.cache_path);
                    }
                }
                Err(e) => {
                    warn!("Failed to load model cache: {}", e);
                }
            }
        }
        Ok(())
    }

    /// Save cache to disk (synchronous)
    pub fn save_cache(&self) -> Result<()> {
        let cache = self.cache.read();
        if let Some(parent) = self.config.cache_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let data = serde_json::to_string_pretty(&*cache)
            .map_err(|e| AiError::Parse(format!("Failed to serialize cache: {}", e)))?;
        std::fs::write(&self.config.cache_path, &data)
            .map_err(|e| AiError::ApiError(format!("Failed to write cache: {}", e)))?;
        debug!("Saved model cache to {:?}", self.config.cache_path);
        Ok(())
    }

    /// Get cached models for a provider without fetching (synchronous)
    pub fn get_cached(&self, provider: &str) -> Option<Vec<ModelInfo>> {
        let cache = self.cache.read();
        cache.get(provider).map(|cached| cached.models.clone())
    }

    /// Check if cache is valid for a provider (synchronous)
    pub fn is_cache_valid(&self, provider: &str) -> bool {
        let cache = self.cache.read();
        if let Some(cached) = cache.get(provider) {
            let ttl = if provider == "ollama" {
                self.config.local_cache_ttl
            } else {
                self.config.api_cache_ttl
            };
            cached.is_valid(ttl)
        } else {
            false
        }
    }

    /// Get models for a specific provider, using cache if valid
    pub async fn get_models(&self, provider: &str) -> Result<Vec<ModelInfo>> {
        // Check cache first (sync lock access)
        {
            let cache = self.cache.read();
            if let Some(cached) = cache.get(provider) {
                let ttl = if provider == "ollama" {
                    self.config.local_cache_ttl
                } else {
                    self.config.api_cache_ttl
                };
                if cached.is_valid(ttl) {
                    debug!("Using cached models for {}", provider);
                    return Ok(cached.models.clone());
                }
            }
        }

        // Fetch fresh models
        let models = self.fetch_models(provider).await?;

        // Update cache (sync lock access)
        {
            let mut cache = self.cache.write();
            cache.insert(
                provider.to_string(),
                CachedModelList {
                    models: models.clone(),
                    fetched_at: Utc::now(),
                    provider: provider.to_string(),
                },
            );
        }

        // Save cache (sync operation now)
        let _ = self.save_cache();

        Ok(models)
    }

    /// Get all models from all configured providers
    pub async fn get_all_models(&self) -> HashMap<String, Vec<ModelInfo>> {
        let mut results = HashMap::new();
        let providers = ["openai", "anthropic", "gemini", "ollama", "claude-cli", "gemini-cli"];

        for provider in providers {
            match self.get_models(provider).await {
                Ok(models) if !models.is_empty() => {
                    results.insert(provider.to_string(), models);
                }
                Ok(_) => {
                    debug!("No models found for {}", provider);
                }
                Err(e) => {
                    warn!("Failed to fetch models for {}: {}", provider, e);
                }
            }
        }

        results
    }

    /// Refresh all provider caches
    pub async fn refresh_all(&self) -> HashMap<String, Result<usize>> {
        let mut results = HashMap::new();
        let providers = ["openai", "anthropic", "gemini", "ollama"];

        for provider in providers {
            // Clear cache entry to force refresh (sync lock)
            {
                let mut cache = self.cache.write();
                cache.remove(provider);
            }

            match self.get_models(provider).await {
                Ok(models) => {
                    results.insert(provider.to_string(), Ok(models.len()));
                }
                Err(e) => {
                    results.insert(provider.to_string(), Err(e));
                }
            }
        }

        results
    }

    /// Public method to fetch models for a specific provider (async)
    /// This fetches from the API without caching
    pub async fn fetch_provider_models(&self, provider: &str) -> Result<Vec<ModelInfo>> {
        self.fetch_models(provider).await
    }

    /// Fetch models from provider API
    async fn fetch_models(&self, provider: &str) -> Result<Vec<ModelInfo>> {
        match provider {
            "openai" => self.fetch_openai_models().await,
            "anthropic" => self.fetch_anthropic_models().await,
            "gemini" => self.fetch_gemini_models().await,
            "ollama" => self.fetch_ollama_models().await,
            "claude-cli" => Ok(self.get_claude_cli_models()),
            "gemini-cli" => Ok(self.get_gemini_cli_models()),
            _ => Err(AiError::ApiError(format!("Unknown provider: {}", provider))),
        }
    }

    /// Fetch models from OpenAI API
    async fn fetch_openai_models(&self) -> Result<Vec<ModelInfo>> {
        let api_key = match self.config.openai_api_key.as_ref() {
            Some(key) if !key.is_empty() => key,
            _ => {
                // No API key - use known models fallback
                warn!("OpenAI API key not set, using known models");
                return Ok(self.get_openai_known_models());
            }
        };

        let response = match self
            .client
            .get("https://api.openai.com/v1/models")
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                warn!("OpenAI API request failed: {}, using known models", e);
                return Ok(self.get_openai_known_models());
            }
        };

        if !response.status().is_success() {
            warn!("OpenAI API error: {}, using known models", response.status());
            return Ok(self.get_openai_known_models());
        }

        #[derive(Deserialize)]
        struct OpenAiModelList {
            data: Vec<OpenAiModel>,
        }

        #[derive(Deserialize)]
        struct OpenAiModel {
            id: String,
            created: Option<i64>,
            #[serde(rename = "owned_by")]
            _owned_by: Option<String>,
        }

        let model_list: OpenAiModelList = response.json().await?;

        // Filter and categorize models
        let mut models: Vec<ModelInfo> = model_list
            .data
            .into_iter()
            .filter(|m| {
                // Include chat models, exclude embeddings, whisper, dall-e, tts
                let id = m.id.to_lowercase();
                (id.contains("gpt") || id.contains("o1") || id.contains("o3"))
                    && !id.contains("realtime")
                    && !id.contains("audio")
            })
            .map(|m| {
                let mut info = ModelInfo::new(&m.id, "openai");
                info.name = format_openai_name(&m.id);
                info.created_at = m.created.map(|ts| {
                    DateTime::from_timestamp(ts, 0).unwrap_or_else(Utc::now)
                });
                info.family = extract_model_family(&m.id);

                // Set capabilities based on model name
                if m.id.contains("gpt-4") || m.id.contains("o1") || m.id.contains("o3") {
                    info.supports_tools = true;
                }
                if m.id.contains("vision") || m.id.contains("gpt-4o") || m.id.contains("gpt-4-turbo") {
                    info.supports_vision = true;
                }

                // Mark recommended models
                info.recommended = m.id == "gpt-4o" || m.id == "gpt-4o-mini" || m.id == "o1" || m.id == "o3-mini";

                // Set context windows for known models
                info.context_window = match m.id.as_str() {
                    s if s.contains("gpt-4o") => Some(128_000),
                    s if s.contains("gpt-4-turbo") => Some(128_000),
                    s if s.contains("gpt-4-32k") => Some(32_768),
                    s if s.contains("gpt-4") => Some(8_192),
                    s if s.contains("gpt-3.5-turbo-16k") => Some(16_384),
                    s if s.contains("gpt-3.5") => Some(4_096),
                    s if s.contains("o1") => Some(128_000),
                    s if s.contains("o3") => Some(128_000),
                    _ => None,
                };

                info
            })
            .collect();

        // Sort by recommended first, then by name
        models.sort_by(|a, b| {
            b.recommended.cmp(&a.recommended).then(a.name.cmp(&b.name))
        });

        info!("Fetched {} OpenAI models", models.len());
        Ok(models)
    }

    /// Fetch models from Anthropic API
    async fn fetch_anthropic_models(&self) -> Result<Vec<ModelInfo>> {
        let api_key = self.config.anthropic_api_key.as_ref().ok_or_else(|| {
            AiError::NotConfigured("Anthropic API key not set".to_string())
        })?;

        let response = self
            .client
            .get("https://api.anthropic.com/v1/models")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await?;

        if !response.status().is_success() {
            // Anthropic may not have /models endpoint, use known models
            warn!("Anthropic models API not available, using known models");
            return Ok(self.get_anthropic_known_models());
        }

        #[derive(Deserialize)]
        struct AnthropicModelList {
            data: Vec<AnthropicModel>,
        }

        #[derive(Deserialize)]
        struct AnthropicModel {
            id: String,
            display_name: Option<String>,
            #[serde(rename = "created_at")]
            _created_at: Option<String>,
        }

        match response.json::<AnthropicModelList>().await {
            Ok(model_list) => {
                let models: Vec<ModelInfo> = model_list
                    .data
                    .into_iter()
                    .map(|m| {
                        let mut info = ModelInfo::new(&m.id, "anthropic");
                        info.name = m.display_name.unwrap_or_else(|| format_claude_name(&m.id));
                        info.supports_tools = true;
                        info.supports_vision = m.id.contains("claude-3") || m.id.contains("claude-4");
                        info.family = Some("Claude".to_string());

                        // Set context windows
                        info.context_window = Some(200_000);
                        info.max_output = Some(8_192);

                        // Mark recommended
                        info.recommended = m.id.contains("sonnet") || m.id.contains("opus");

                        info
                    })
                    .collect();

                info!("Fetched {} Anthropic models", models.len());
                Ok(models)
            }
            Err(_) => {
                // Fallback to known models
                Ok(self.get_anthropic_known_models())
            }
        }
    }

    /// Get known Anthropic models (fallback)
    fn get_anthropic_known_models(&self) -> Vec<ModelInfo> {
        vec![
            create_claude_model("claude-opus-4-20250514", "Claude Opus 4", true),
            create_claude_model("claude-sonnet-4-20250514", "Claude Sonnet 4", true),
            create_claude_model("claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet", true),
            create_claude_model("claude-3-5-haiku-20241022", "Claude 3.5 Haiku", false),
            create_claude_model("claude-3-opus-20240229", "Claude 3 Opus", false),
            create_claude_model("claude-3-sonnet-20240229", "Claude 3 Sonnet", false),
            create_claude_model("claude-3-haiku-20240307", "Claude 3 Haiku", false),
        ]
    }

    /// Get known OpenAI models (fallback when API key not set)
    fn get_openai_known_models(&self) -> Vec<ModelInfo> {
        vec![
            create_openai_model("gpt-4o", "GPT-4o", 128_000, true),
            create_openai_model("gpt-4o-mini", "GPT-4o Mini", 128_000, true),
            create_openai_model("o1", "o1", 128_000, true),
            create_openai_model("o1-mini", "o1 Mini", 128_000, false),
            create_openai_model("o3-mini", "o3 Mini", 128_000, true),
            create_openai_model("gpt-4-turbo", "GPT-4 Turbo", 128_000, false),
            create_openai_model("gpt-4", "GPT-4", 8_192, false),
            create_openai_model("gpt-3.5-turbo", "GPT-3.5 Turbo", 16_384, false),
        ]
    }

    /// Get known Gemini models (fallback when API key not set)
    fn get_gemini_known_models(&self) -> Vec<ModelInfo> {
        vec![
            create_gemini_model("gemini-2.0-flash-exp", "Gemini 2.0 Flash", 1_000_000, true),
            create_gemini_model("gemini-2.0-flash-thinking-exp", "Gemini 2.0 Flash Thinking", 1_000_000, true),
            create_gemini_model("gemini-1.5-pro", "Gemini 1.5 Pro", 2_000_000, true),
            create_gemini_model("gemini-1.5-flash", "Gemini 1.5 Flash", 1_000_000, false),
            create_gemini_model("gemini-1.5-flash-8b", "Gemini 1.5 Flash 8B", 1_000_000, false),
        ]
    }

    /// Fetch models from Google Gemini API
    async fn fetch_gemini_models(&self) -> Result<Vec<ModelInfo>> {
        let api_key = match self.config.google_api_key.as_ref() {
            Some(key) if !key.is_empty() => key,
            _ => {
                warn!("Google API key not set, using known models");
                return Ok(self.get_gemini_known_models());
            }
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            api_key
        );

        let response = match self.client.get(&url).send().await {
            Ok(resp) => resp,
            Err(e) => {
                warn!("Gemini API request failed: {}, using known models", e);
                return Ok(self.get_gemini_known_models());
            }
        };

        if !response.status().is_success() {
            warn!("Gemini API error: {}, using known models", response.status());
            return Ok(self.get_gemini_known_models());
        }

        #[derive(Deserialize)]
        struct GeminiModelList {
            models: Vec<GeminiModel>,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct GeminiModel {
            name: String,
            display_name: Option<String>,
            description: Option<String>,
            input_token_limit: Option<u32>,
            output_token_limit: Option<u32>,
            supported_generation_methods: Option<Vec<String>>,
        }

        let model_list: GeminiModelList = response.json().await?;

        let models: Vec<ModelInfo> = model_list
            .models
            .into_iter()
            .filter(|m| {
                // Filter for generateContent-capable models
                m.supported_generation_methods
                    .as_ref()
                    .map(|methods| methods.iter().any(|m| m == "generateContent"))
                    .unwrap_or(false)
            })
            .map(|m| {
                // Extract model ID from "models/gemini-xxx" format
                let id = m.name.strip_prefix("models/").unwrap_or(&m.name).to_string();

                let mut info = ModelInfo::new(&id, "gemini");
                info.name = m.display_name.unwrap_or_else(|| format_gemini_name(&id));
                info.description = m.description;
                info.context_window = m.input_token_limit;
                info.max_output = m.output_token_limit;
                info.supports_tools = true;
                info.supports_vision = id.contains("vision") || id.contains("pro") || id.contains("flash");
                info.family = Some("Gemini".to_string());

                // Mark recommended
                info.recommended = id == "gemini-2.0-flash" || id == "gemini-1.5-pro";

                info
            })
            .collect();

        info!("Fetched {} Gemini models", models.len());
        Ok(models)
    }

    /// Fetch models from local Ollama
    async fn fetch_ollama_models(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/api/tags", self.config.ollama_endpoint);

        let response = self.client.get(&url).send().await.map_err(|e| {
            AiError::ApiError(format!("Ollama not reachable: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(AiError::ApiError(format!(
                "Ollama API error: {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct OllamaTagList {
            models: Vec<OllamaModel>,
        }

        #[derive(Deserialize)]
        struct OllamaModel {
            name: String,
            size: Option<u64>,
            modified_at: Option<String>,
            details: Option<OllamaModelDetails>,
        }

        #[derive(Deserialize)]
        struct OllamaModelDetails {
            family: Option<String>,
            parameter_size: Option<String>,
            quantization_level: Option<String>,
        }

        let tag_list: OllamaTagList = response.json().await?;

        let models: Vec<ModelInfo> = tag_list
            .models
            .into_iter()
            // Filter out embedding models - they don't support text generation
            .filter(|m| {
                let name_lower = m.name.to_lowercase();
                !name_lower.contains("embed")
                    && !name_lower.contains("nomic")
                    && !name_lower.contains("bge")
                    && !name_lower.contains("mxbai")
                    && !name_lower.contains("all-minilm")
            })
            .map(|m| {
                let mut info = ModelInfo::new(&m.name, "ollama");
                info.name = format_ollama_name(&m.name);
                info.size_bytes = m.size;

                if let Some(details) = m.details {
                    info.family = details.family;
                    if let Some(param_size) = details.parameter_size {
                        info.description = Some(format!(
                            "{} parameters{}",
                            param_size,
                            details.quantization_level
                                .map(|q| format!(", {}", q))
                                .unwrap_or_default()
                        ));
                    }
                }

                // Detect vision models
                info.supports_vision = m.name.contains("vision")
                    || m.name.contains("llava")
                    || m.name.contains("bakllava");

                // Parse modified_at
                if let Some(modified) = m.modified_at {
                    info.created_at = DateTime::parse_from_rfc3339(&modified)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok();
                }

                info
            })
            .collect();

        info!("Fetched {} Ollama models", models.len());
        Ok(models)
    }

    /// Get known Claude CLI models
    fn get_claude_cli_models(&self) -> Vec<ModelInfo> {
        vec![
            create_cli_model("claude-opus-4-20250514", "Claude Opus 4", "claude-cli", true),
            create_cli_model("claude-sonnet-4-20250514", "Claude Sonnet 4", "claude-cli", true),
            create_cli_model("claude-3-5-sonnet-20241022", "Claude 3.5 Sonnet", "claude-cli", false),
            create_cli_model("claude-3-5-haiku-20241022", "Claude 3.5 Haiku", "claude-cli", false),
        ]
    }

    /// Get known Gemini CLI models
    fn get_gemini_cli_models(&self) -> Vec<ModelInfo> {
        vec![
            create_cli_model("gemini-2.0-flash", "Gemini 2.0 Flash", "gemini-cli", true),
            create_cli_model("gemini-2.0-flash-thinking", "Gemini 2.0 Flash Thinking", "gemini-cli", true),
            create_cli_model("gemini-1.5-pro", "Gemini 1.5 Pro", "gemini-cli", false),
            create_cli_model("gemini-1.5-flash", "Gemini 1.5 Flash", "gemini-cli", false),
        ]
    }

}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions

fn format_openai_name(id: &str) -> String {
    match id {
        "gpt-4o" => "GPT-4o".to_string(),
        "gpt-4o-mini" => "GPT-4o Mini".to_string(),
        "gpt-4-turbo" => "GPT-4 Turbo".to_string(),
        "gpt-4-turbo-preview" => "GPT-4 Turbo Preview".to_string(),
        "gpt-4" => "GPT-4".to_string(),
        "gpt-3.5-turbo" => "GPT-3.5 Turbo".to_string(),
        "o1" => "o1".to_string(),
        "o1-mini" => "o1 Mini".to_string(),
        "o1-preview" => "o1 Preview".to_string(),
        "o3-mini" => "o3 Mini".to_string(),
        _ => id.to_string(),
    }
}

fn format_claude_name(id: &str) -> String {
    if id.contains("opus-4") {
        "Claude Opus 4".to_string()
    } else if id.contains("sonnet-4") {
        "Claude Sonnet 4".to_string()
    } else if id.contains("3-5-sonnet") {
        "Claude 3.5 Sonnet".to_string()
    } else if id.contains("3-5-haiku") {
        "Claude 3.5 Haiku".to_string()
    } else if id.contains("3-opus") {
        "Claude 3 Opus".to_string()
    } else if id.contains("3-sonnet") {
        "Claude 3 Sonnet".to_string()
    } else if id.contains("3-haiku") {
        "Claude 3 Haiku".to_string()
    } else {
        id.to_string()
    }
}

fn format_gemini_name(id: &str) -> String {
    match id {
        "gemini-2.0-flash" => "Gemini 2.0 Flash".to_string(),
        "gemini-2.0-flash-thinking" => "Gemini 2.0 Flash Thinking".to_string(),
        "gemini-1.5-pro" => "Gemini 1.5 Pro".to_string(),
        "gemini-1.5-flash" => "Gemini 1.5 Flash".to_string(),
        "gemini-1.5-flash-8b" => "Gemini 1.5 Flash 8B".to_string(),
        "gemini-pro" => "Gemini Pro".to_string(),
        _ => id.to_string(),
    }
}

fn format_ollama_name(id: &str) -> String {
    // Convert "llama3.2:3b" to "Llama 3.2 (3B)"
    let parts: Vec<&str> = id.split(':').collect();
    let base = parts[0];
    let tag = parts.get(1);

    let formatted_base = base
        .replace("llama", "Llama ")
        .replace("mistral", "Mistral ")
        .replace("qwen", "Qwen ")
        .replace("gemma", "Gemma ")
        .replace("deepseek", "DeepSeek ")
        .replace("codellama", "CodeLlama ")
        .replace("phi", "Phi ");

    match tag {
        Some(t) => format!("{} ({})", formatted_base.trim(), t.to_uppercase()),
        None => formatted_base.trim().to_string(),
    }
}

fn extract_model_family(id: &str) -> Option<String> {
    if id.starts_with("gpt-4") {
        Some("GPT-4".to_string())
    } else if id.starts_with("gpt-3") {
        Some("GPT-3.5".to_string())
    } else if id.starts_with("o1") {
        Some("o1".to_string())
    } else if id.starts_with("o3") {
        Some("o3".to_string())
    } else {
        None
    }
}

fn create_claude_model(id: &str, name: &str, recommended: bool) -> ModelInfo {
    let mut info = ModelInfo::new(id, "anthropic");
    info.name = name.to_string();
    info.family = Some("Claude".to_string());
    info.context_window = Some(200_000);
    info.max_output = Some(8_192);
    info.supports_tools = true;
    info.supports_vision = true;
    info.recommended = recommended;
    info
}

fn create_openai_model(id: &str, name: &str, context_window: u32, recommended: bool) -> ModelInfo {
    let mut info = ModelInfo::new(id, "openai");
    info.name = name.to_string();
    info.family = extract_model_family(id);
    info.context_window = Some(context_window);
    info.supports_tools = true;
    info.supports_vision = id.contains("gpt-4o") || id.contains("gpt-4-turbo");
    info.recommended = recommended;
    info
}

fn create_gemini_model(id: &str, name: &str, context_window: u32, recommended: bool) -> ModelInfo {
    let mut info = ModelInfo::new(id, "gemini");
    info.name = name.to_string();
    info.family = Some("Gemini".to_string());
    info.context_window = Some(context_window);
    info.supports_tools = true;
    info.supports_vision = true;
    info.recommended = recommended;
    info
}

fn create_cli_model(id: &str, name: &str, provider: &str, recommended: bool) -> ModelInfo {
    let mut info = ModelInfo::new(id, provider);
    info.name = name.to_string();
    info.supports_tools = true;
    info.supports_vision = true;
    info.recommended = recommended;
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info_new() {
        let model = ModelInfo::new("gpt-4o", "openai");
        assert_eq!(model.id, "gpt-4o");
        assert_eq!(model.provider, "openai");
    }

    #[test]
    fn test_size_display() {
        let mut model = ModelInfo::new("test", "test");
        model.size_bytes = Some(4_700_000_000);
        assert_eq!(model.size_display(), Some("4.7 GB".to_string()));

        model.size_bytes = Some(500_000_000);
        assert_eq!(model.size_display(), Some("500.0 MB".to_string()));
    }

    #[test]
    fn test_format_ollama_name() {
        assert_eq!(format_ollama_name("llama3.2:3b"), "Llama 3.2 (3B)");
        assert_eq!(format_ollama_name("mistral:latest"), "Mistral (LATEST)");
    }

    #[test]
    fn test_format_openai_name() {
        assert_eq!(format_openai_name("gpt-4o"), "GPT-4o");
        assert_eq!(format_openai_name("gpt-4o-mini"), "GPT-4o Mini");
    }
}
