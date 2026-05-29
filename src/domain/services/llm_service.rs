//! LLM Service - Domain Service for LLM interactions
//!
//! Handles communication with various LLM providers

use crate::domain::entities::llm::{AsrResult, LlmConfig, LlmError, LlmProvider};
use reqwest::multipart;
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, warn};

/// LLM Service for making requests to various providers
pub struct LlmService {
    config: LlmConfig,
    http_client: Client,
}

impl LlmService {
    /// Create new service with default config from environment
    pub fn new() -> Self {
        Self::with_config(LlmConfig::from_env())
    }

    /// Create new service with specific config
    pub fn with_config(config: LlmConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
        }
    }

    /// Generate completion with retry logic
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

    /// Perform ASR (Speech-to-Text) using LiteLLM
    pub async fn asr(&self, audio_data: Vec<u8>, filename: &str) -> Result<AsrResult, LlmError> {
        if self.config.provider != LlmProvider::Litellm {
            return Err(LlmError::ConfigError(
                "ASR only supported with LiteLLM provider".to_string(),
            ));
        }

        let url = format!("{}/audio/transcriptions", self.config.base_url);

        let part = multipart::Part::bytes(audio_data)
            .file_name(filename.to_string())
            .mime_str("video/mp4")
            .map_err(|e| LlmError::Unknown(format!("Failed to create multipart: {}", e)))?;

        let form = multipart::Form::new()
            .text("model", self.config.model.clone())
            .part("file", part);

        let mut request = self.http_client.post(&url).multipart(form);

        if let Some(ref key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        debug!("Calling LiteLLM ASR at {}", url);

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

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        let text = json["text"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| LlmError::ParseError("No text in ASR response".to_string()))?;

        Ok(AsrResult {
            text,
            duration: None,
            language: None,
        })
    }

    /// Single completion attempt
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

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

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

        let response = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmError::Timeout(format!(
                        "Ollama request timed out after {}s",
                        self.config.timeout_secs
                    ))
                } else {
                    LlmError::RequestFailed(format!("Ollama error: {}", e))
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(format!(
                "Ollama HTTP {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

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

        let api_key = self.config.api_key.as_ref().ok_or(LlmError::ConfigError(
            "OpenAI API key not configured".to_string(),
        ))?;

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(LlmError::ApiError(format!(
                "OpenAI HTTP {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

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

        let api_key = self.config.api_key.as_ref().ok_or(LlmError::ConfigError(
            "Anthropic API key not configured".to_string(),
        ))?;

        let response = self
            .http_client
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
            return Err(LlmError::ApiError(format!(
                "Anthropic HTTP {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| LlmError::ParseError(e.to_string()))?;

        json["content"][0]["text"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| LlmError::ParseError("No content in response".to_string()))
    }

    /// Health check for LLM service
    pub async fn health_check(&self) -> Result<bool, LlmError> {
        let result = self.complete("Hi").await;
        match result {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    }

    /// Get current config
    pub fn config(&self) -> &LlmConfig {
        &self.config
    }
}

impl Default for LlmService {
    fn default() -> Self {
        Self::new()
    }
}
