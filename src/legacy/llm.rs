//! LLM Integration Module
//!
//! Support multi-provider LLM via LiteLLM ou direct:
//! - Ollama (local)
//! - OpenAI
//! - Anthropic (Claude)
//! - Azure OpenAI
//! - et tout autre provider supporté par LiteLLM
//!
//! Configuration via Kusanagi config ou variables d'environnement

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{warn, debug};

// ============================================================================
// Configuration
// ============================================================================

/// LLM Provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    /// LiteLLM proxy (recommended for multi-provider)
    Litellm,
    /// Ollama direct (local)
    Ollama,
    /// OpenAI direct
    Openai,
    /// Anthropic Claude
    Anthropic,
}

impl Default for LlmProvider {
    fn default() -> Self {
        // Default to LiteLLM for Kubernetes deployments
        LlmProvider::Litellm
    }
}

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
                        LlmProvider::Litellm => "http://litellm.default.svc.cluster.local:4000".to_string(),
                        LlmProvider::Ollama => "http://localhost:11434".to_string(),
                        LlmProvider::Openai => "https://api.openai.com/v1".to_string(),
                        LlmProvider::Anthropic => "https://api.anthropic.com/v1".to_string(),
                    }
                }
            }
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
                }
            }
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

// ============================================================================
// LLM Client
// ============================================================================

/// LLM Client for making requests
pub struct LlmClient {
    config: LlmConfig,
    http_client: reqwest::Client,
}

impl LlmClient {
    /// Create new client with default config
    pub fn new() -> Self {
        Self::with_config(LlmConfig::from_env())
    }

    /// Create new client with specific config
    pub fn with_config(config: LlmConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
        }
    }

    /// Generate completion
    pub async fn complete(&self, prompt: &str) -> Result<String, LlmError> {
        let mut last_error = None;

        for attempt in 0..self.config.max_retries {
            match self.try_complete(prompt).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    warn!("LLM request failed (attempt {}): {}", attempt + 1, e);
                    last_error = Some(e);
                    
                    if attempt < self.config.max_retries - 1 {
                        let delay = Duration::from_millis(500 * (attempt + 1) as u64);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or(LlmError::Unknown("Max retries exceeded".to_string())))
    }

    async fn try_complete(&self, prompt: &str) -> Result<String, LlmError> {
        match self.config.provider {
            LlmProvider::Litellm => self.call_litellm(prompt).await,
            LlmProvider::Ollama => self.call_ollama(prompt).await,
            LlmProvider::Openai => self.call_openai(prompt).await,
            LlmProvider::Anthropic => self.call_anthropic(prompt).await,
        }
    }

    /// Call LiteLLM proxy
    async fn call_litellm(&self, prompt: &str) -> Result<String, LlmError> {
        let url = format!("{}/chat/completions", self.config.base_url);
        
        let body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        });

        let mut request = self.http_client.post(&url).json(&body);
        
        if let Some(ref key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        debug!("Calling LiteLLM at {}", url);

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                LlmError::Timeout(e.to_string())
            } else {
                LlmError::RequestFailed(e.to_string())
            }
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(format!("HTTP {}: {}", status, text)));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| LlmError::ParseError(e.to_string()))?;
        
        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| LlmError::ParseError("No content in response".to_string()))
    }

    /// Call Ollama directly
    async fn call_ollama(&self, prompt: &str) -> Result<String, LlmError> {
        let url = format!("{}/api/generate", self.config.base_url);
        
        let body = serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": self.config.temperature,
                "num_predict": self.config.max_tokens,
            }
        });

        debug!("Calling Ollama at {} with model {}", url, self.config.model);

        let response = self.http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmError::Timeout(format!("Ollama request timed out after {}s", self.config.timeout_secs))
                } else {
                    LlmError::RequestFailed(format!("Ollama error: {}", e))
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(format!("Ollama HTTP {}: {}", status, text)));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| LlmError::ParseError(e.to_string()))?;
        
        json["response"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| LlmError::ParseError("No response field in Ollama output".to_string()))
    }

    /// Call OpenAI API
    async fn call_openai(&self, prompt: &str) -> Result<String, LlmError> {
        let url = format!("{}/chat/completions", self.config.base_url);
        
        let body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": self.config.temperature,
            "max_tokens": self.config.max_tokens,
        });

        let api_key = self.config.api_key.as_ref()
            .ok_or(LlmError::ConfigError("OpenAI API key not configured".to_string()))?;

        let response = self.http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(format!("OpenAI HTTP {}: {}", status, text)));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| LlmError::ParseError(e.to_string()))?;
        
        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| LlmError::ParseError("No content in response".to_string()))
    }

    /// Call Anthropic API
    async fn call_anthropic(&self, prompt: &str) -> Result<String, LlmError> {
        let url = format!("{}/messages", self.config.base_url);
        
        let body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "messages": [
                {"role": "user", "content": prompt}
            ]
        });

        let api_key = self.config.api_key.as_ref()
            .ok_or(LlmError::ConfigError("Anthropic API key not configured".to_string()))?;

        let response = self.http_client
            .post(&url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(format!("Anthropic HTTP {}: {}", status, text)));
        }

        let json: serde_json::Value = response.json().await.map_err(|e| LlmError::ParseError(e.to_string()))?;
        
        json["content"][0]["text"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| LlmError::ParseError("No content in response".to_string()))
    }

    /// Get current config
    pub fn config(&self) -> &LlmConfig {
        &self.config
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Error Types
// ============================================================================

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

// ============================================================================
// Convenience Functions
// ============================================================================

/// Quick completion with default client
pub async fn complete(prompt: &str) -> Result<String, LlmError> {
    let client = LlmClient::new();
    client.complete(prompt).await
}

/// Health check for LLM service
pub async fn health_check() -> Result<bool, LlmError> {
    let client = LlmClient::new();
    let result = client.complete("Hi").await;
    match result {
        Ok(_) => Ok(true),
        Err(e) => Err(e),
    }
}

/// Get current configuration info (for debugging)
pub fn get_config_info() -> serde_json::Value {
    let config = LlmConfig::from_env();
    serde_json::json!({
        "provider": format!("{:?}", config.provider),
        "base_url": config.base_url,
        "model": config.model,
        "timeout_secs": config.timeout_secs,
        "max_retries": config.max_retries,
        "has_api_key": config.api_key.is_some(),
        "is_valid": config.is_valid(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_from_env_defaults() {
        // Note: This test depends on env vars not being set
        // In practice, you'd want to use a test isolation mechanism
        let config = LlmConfig::from_env();
        assert!(config.timeout_secs > 0);
        assert!(config.max_retries > 0);
    }

    #[test]
    fn test_llm_error_display() {
        let err = LlmError::Timeout("test".to_string());
        assert!(err.to_string().contains("timeout"));
    }
}
