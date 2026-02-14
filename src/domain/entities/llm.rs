//! LLM (Large Language Model) Domain Entities
//!
//! Support multi-provider LLM integration:
//! - LiteLLM (proxy for multiple providers)
//! - Ollama (local models)
//! - OpenAI
//! - Anthropic (Claude)

use serde::{Deserialize, Serialize};

// ==================== LLM Provider ====================

/// LLM Provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    /// LiteLLM proxy (recommended for multi-provider)
    #[default]
    Litellm,
    /// Ollama direct (local)
    Ollama,
    /// OpenAI direct
    Openai,
    /// Anthropic Claude
    Anthropic,
}

// ==================== LLM Configuration ====================

/// LLM Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Provider type
    pub provider: LlmProvider,
    /// Base URL for the API
    pub base_url: String,
    /// API key (if required)
    pub api_key: Option<String>,
    /// Model name to use
    pub model: String,
    /// Timeout in seconds
    pub timeout_secs: u64,
    /// Maximum retries
    pub max_retries: u32,
    /// Temperature for generation
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: u32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: LlmProvider::default(),
            base_url: "http://litellm.litellm.svc.cluster.local:4000".to_string(),
            api_key: None,
            model: "gpt-3.5-turbo".to_string(),
            timeout_secs: 60,
            max_retries: 3,
            temperature: 0.7,
            max_tokens: 2048,
        }
    }
}

impl LlmConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let provider = std::env::var("LLM_PROVIDER")
            .ok()
            .and_then(|p| match p.to_lowercase().as_str() {
                "litellm" => Some(LlmProvider::Litellm),
                "ollama" => Some(LlmProvider::Ollama),
                "openai" => Some(LlmProvider::Openai),
                "anthropic" => Some(LlmProvider::Anthropic),
                _ => None,
            })
            .unwrap_or_default();

        let base_url = match std::env::var("LLM_BASE_URL") {
            Ok(url) => url,
            Err(_) => match std::env::var("OLLAMA_HOST") {
                Ok(url) => url,
                Err(_) => match std::env::var("LITELLM_URL") {
                    Ok(url) => url,
                    Err(_) => match provider {
                        LlmProvider::Litellm => {
                            "http://litellm.default.svc.cluster.local:4000".to_string()
                        }
                        LlmProvider::Ollama => "http://localhost:11434".to_string(),
                        LlmProvider::Openai => "https://api.openai.com/v1".to_string(),
                        LlmProvider::Anthropic => "https://api.anthropic.com/v1".to_string(),
                    },
                },
            },
        };

        let model = match std::env::var("LLM_MODEL") {
            Ok(m) => m,
            Err(_) => match std::env::var("OLLAMA_MODEL") {
                Ok(m) => m,
                Err(_) => match provider {
                    LlmProvider::Litellm => "gpt-3.5-turbo".to_string(),
                    LlmProvider::Ollama => "llama2".to_string(),
                    LlmProvider::Openai => "gpt-3.5-turbo".to_string(),
                    LlmProvider::Anthropic => "claude-3-haiku-20240307".to_string(),
                },
            },
        };

        Self {
            provider,
            base_url,
            api_key: std::env::var("LLM_API_KEY")
                .or_else(|_| std::env::var("OPENAI_API_KEY"))
                .or_else(|_| std::env::var("ANTHROPIC_API_KEY"))
                .ok(),
            model,
            timeout_secs: std::env::var("LLM_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            max_retries: std::env::var("LLM_MAX_RETRIES")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3),
            temperature: std::env::var("LLM_TEMPERATURE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.7),
            max_tokens: std::env::var("LLM_MAX_TOKENS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2048),
        }
    }

    /// Check if configuration is valid
    pub fn is_valid(&self) -> bool {
        match self.provider {
            LlmProvider::Ollama => !self.base_url.is_empty(),
            _ => !self.base_url.is_empty() && self.api_key.is_some(),
        }
    }
}

// ==================== LLM Error ====================

#[derive(Debug, Clone)]
pub enum LlmError {
    Timeout(String),
    RequestFailed(String),
    ApiError(String),
    ParseError(String),
    ConfigError(String),
    Unknown(String),
}

impl std::fmt::Display for LlmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmError::Timeout(msg) => write!(f, "LLM timeout: {}", msg),
            LlmError::RequestFailed(msg) => write!(f, "LLM request failed: {}", msg),
            LlmError::ApiError(msg) => write!(f, "LLM API error: {}", msg),
            LlmError::ParseError(msg) => write!(f, "LLM parse error: {}", msg),
            LlmError::ConfigError(msg) => write!(f, "LLM config error: {}", msg),
            LlmError::Unknown(msg) => write!(f, "LLM unknown error: {}", msg),
        }
    }
}

impl std::error::Error for LlmError {}

// ==================== LLM Response ====================

/// Response from LLM completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub provider: String,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmHealthResponse {
    pub healthy: bool,
    pub provider: String,
    pub model: String,
    pub error: Option<String>,
}

/// Configuration info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfigInfo {
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub has_api_key: bool,
    pub is_valid: bool,
}
