//! Centralized configuration management for Kusanagi
//!
//! This module provides structured, validated configuration for the entire application.
//! It uses the `config` crate to support multiple sources (environment variables,
//! configuration files, etc.) with environment variables taking precedence.
//!
//! # Environment Variable Prefix
//!
//! All environment variables are prefixed with `KUSANAGI_` by default.
//! For example, `KUSANAGI_SERVER_PORT` maps to `config.server.port`.
//!
//! # Configuration Sources (in order of precedence)
//!
//! 1. Environment variables (highest priority)
//! 2. `kusanagi.toml` file in the current directory
//! 3. `kusanagi.toml` in `$HOME/.config/kusanagi/`
//! 4. Default values (lowest priority)
//!
//! # Example
//!
//! ```rust,no_run
//! use kusanagi::config::Config;
//!
//! let config = Config::load().unwrap();
//! println!("Server will bind to {}:{}", config.server.host, config.server.port);
//! ```

use crate::error::{KusanagiError, Result};
use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Duration;

/// Application configuration
///
/// This is the root configuration structure that contains all settings
/// for the Kusanagi application.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,

    /// Kubernetes configuration
    pub kubernetes: KubernetesConfig,

    /// Prometheus configuration
    pub prometheus: PrometheusConfig,

    /// Alertmanager configuration
    pub alertmanager: AlertmanagerConfig,

    /// External integrations
    pub integrations: IntegrationsConfig,

    /// Storage configuration (S3/MinIO)
    pub storage: StorageConfig,

    /// Cache configuration
    pub cache: CacheConfig,

    /// Security settings
    pub security: SecurityConfig,

    /// Logging configuration
    pub log: LogConfig,

    /// Development mode
    pub dev_mode: bool,
}

/// Server configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    /// Host address to bind to
    pub host: String,

    /// Port to listen on
    pub port: u16,

    /// Number of worker threads (None = auto-detect)
    pub workers: Option<usize>,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Keep-alive timeout in seconds
    pub keep_alive_secs: u64,
}

/// Kubernetes configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct KubernetesConfig {
    /// Whether to enable Kubernetes integration
    pub enabled: bool,

    /// Path to kubeconfig file (None = use in-cluster config)
    pub kubeconfig: Option<String>,

    /// Default namespace
    pub namespace: String,

    /// ArgoCD URL (optional)
    pub argocd_url: Option<String>,

    /// Request timeout for K8s API calls
    pub timeout_secs: u64,
}

/// Prometheus configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PrometheusConfig {
    /// Primary Prometheus URL
    pub url: String,

    /// Home Assistant Prometheus URL (optional)
    pub url_ha: Option<String>,

    /// Authentication username (optional)
    pub username: Option<String>,

    /// Authentication password (optional)
    pub password: Option<String>,

    /// Query timeout in seconds
    pub timeout_secs: u64,

    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}

/// Alertmanager configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AlertmanagerConfig {
    /// Alertmanager URL
    pub url: String,

    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}

/// External integrations configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct IntegrationsConfig {
    /// MCP (Model Context Protocol) servers
    pub mcp: McpConfig,

    /// OpenObserve telemetry
    pub openobserve: OpenObserveConfig,

    /// Home Assistant
    pub home_assistant: HomeAssistantConfig,

    /// Proxmox VE
    pub proxmox: ProxmoxConfig,

    /// MQTT broker
    pub mqtt: MqttConfig,

    /// Weather API
    pub weather: WeatherConfig,

    /// Calendar integration
    pub calendar: CalendarConfig,

    /// Slack integration
    pub slack: SlackConfig,

    /// Ollama AI (deprecated, use llm instead)
    pub ollama: OllamaConfig,
    
    /// LLM configuration (multi-provider)
    pub llm: LlmConfig,
}

/// MCP servers configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct McpConfig {
    /// Kubernetes MCP URL
    pub kubernetes_url: String,

    /// Cilium MCP URL
    pub cilium_url: String,

    /// Steampipe MCP URL
    pub steampipe_url: String,

    /// Trivy MCP URL
    pub trivy_url: String,
}

/// OpenObserve configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OpenObserveConfig {
    /// Endpoint URL
    pub endpoint: Option<String>,

    /// Authentication token
    pub auth: Option<String>,

    /// Sample rate (0.0 - 1.0)
    pub sample_rate: f32,
}

/// Home Assistant configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct HomeAssistantConfig {
    /// Home Assistant URL
    pub url: Option<String>,

    /// Access token
    pub token: Option<String>,

    /// Username (for legacy auth)
    pub username: Option<String>,

    /// Password (for legacy auth)
    pub password: Option<String>,
}

/// Proxmox configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct ProxmoxConfig {
    /// Proxmox VE URL(s) - comma-separated for multiple
    pub urls: Option<String>,

    /// API user (e.g., "root@pam")
    pub user: Option<String>,

    /// Password (if not using token)
    pub password: Option<String>,

    /// Token ID
    pub token_id: Option<String>,

    /// Token secret
    pub token_secret: Option<String>,
}

/// MQTT broker configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct MqttConfig {
    /// MQTT broker host:port
    pub host: Option<String>,

    /// Client ID
    pub client_id: String,

    /// Username
    pub username: Option<String>,

    /// Password
    pub password: Option<String>,
}

/// Weather API configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WeatherConfig {
    /// OpenWeatherMap API key
    pub api_key: Option<String>,

    /// Default cities to display (comma-separated)
    pub cities: String,

    /// Update interval in minutes
    pub update_interval_mins: u64,
}

/// Calendar configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct CalendarConfig {
    /// Google Calendar API key
    pub google_api_key: Option<String>,

    /// Google Client Secret
    pub google_client_secret: Option<String>,

    /// Google OAuth redirect URL
    pub google_redirect_url: Option<String>,
}

/// Slack configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct SlackConfig {
    /// Bot token
    pub bot_token: Option<String>,

    /// Bot user ID
    pub bot_user_id: Option<String>,

    /// Channel ID for notifications
    pub channel_id: Option<String>,

    /// Signing secret for webhooks
    pub signing_secret: Option<String>,
}

/// Ollama AI configuration (deprecated)
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct OllamaConfig {
    /// Ollama API URL
    pub url: String,

    /// Default model to use
    pub model: String,
}

/// LLM Configuration (multi-provider)
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LlmConfig {
    /// Provider type: litellm, ollama, openai, anthropic
    pub provider: String,
    
    /// Base URL for the API
    pub base_url: String,
    
    /// API key (for cloud providers)
    pub api_key: Option<String>,
    
    /// Model name
    pub model: String,
    
    /// Request timeout in seconds
    pub timeout_secs: u64,
    
    /// Maximum retries
    pub max_retries: u32,
    
    /// Temperature for generation
    pub temperature: f32,
    
    /// Maximum tokens to generate
    pub max_tokens: u32,
    
    /// Enable fallback on failure
    pub enable_fallback: bool,
}

/// S3/MinIO storage configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    /// S3 endpoint URL
    pub endpoint: Option<String>,

    /// S3 bucket name
    pub bucket: Option<String>,

    /// Access key
    pub access_key: Option<String>,

    /// Secret key
    pub secret_key: Option<String>,

    /// Region
    pub region: String,
}

/// Cache configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// Default cache TTL in seconds
    pub default_ttl_secs: u64,

    /// News feed cache TTL in minutes
    pub news_ttl_mins: u64,

    /// Prometheus metrics cache TTL in seconds
    pub prometheus_ttl_secs: u64,

    /// Cilium flows cache TTL in seconds
    pub cilium_ttl_secs: u64,
}

/// Security configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SecurityConfig {
    /// Enable authentication
    pub auth_enabled: bool,

    /// JWT secret key
    pub jwt_secret: Option<String>,

    /// Session timeout in hours
    pub session_timeout_hours: u64,

    /// Allowed CORS origins (comma-separated)
    pub cors_origins: String,
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,

    /// Log format (json, pretty)
    pub format: String,

    /// Enable OpenTelemetry
    pub opentelemetry: bool,
}

// ==================== Default Implementations ====================

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: None,
            timeout_secs: 30,
            keep_alive_secs: 5,
        }
    }
}

impl Default for KubernetesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            kubeconfig: None,
            namespace: "default".to_string(),
            argocd_url: None,
            timeout_secs: 30,
        }
    }
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            url: "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string(),
            url_ha: None,
            username: None,
            password: None,
            timeout_secs: 10,
            cache_ttl_secs: 60,
        }
    }
}

impl Default for AlertmanagerConfig {
    fn default() -> Self {
        Self {
            url: "http://kube-prometheus-stack-alertmanager.kube-prometheus-stack.svc:9093".to_string(),
            cache_ttl_secs: 60,
        }
    }
}


impl Default for McpConfig {
    fn default() -> Self {
        Self {
            kubernetes_url: "http://localhost:3000/mcp/kubernetes".to_string(),
            cilium_url: "http://localhost:3000/mcp/cilium".to_string(),
            steampipe_url: "http://localhost:3000/mcp/steampipe".to_string(),
            trivy_url: "http://localhost:3000/mcp/trivy".to_string(),
        }
    }
}

impl Default for OpenObserveConfig {
    fn default() -> Self {
        Self {
            endpoint: None,
            auth: None,
            sample_rate: 1.0,
        }
    }
}



impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            host: None,
            client_id: "kusanagi".to_string(),
            username: None,
            password: None,
        }
    }
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            cities: "Lyon,Mexico City,New York".to_string(),
            update_interval_mins: 30,
        }
    }
}



impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            url: "http://192.168.0.52:11434/api/generate".to_string(),
            model: "ministral-3:14b".to_string(),
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: "litellm".to_string(),
            base_url: "http://litellm.default.svc.cluster.local:4000".to_string(),
            api_key: None,
            model: "gpt-3.5-turbo".to_string(),
            timeout_secs: 60,
            max_retries: 3,
            temperature: 0.7,
            max_tokens: 2048,
            enable_fallback: true,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            endpoint: None,
            bucket: None,
            access_key: None,
            secret_key: None,
            region: "us-east-1".to_string(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            default_ttl_secs: 300,
            news_ttl_mins: 30,
            prometheus_ttl_secs: 60,
            cilium_ttl_secs: 60,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            auth_enabled: false,
            jwt_secret: None,
            session_timeout_hours: 24,
            cors_origins: "*".to_string(),
        }
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            opentelemetry: false,
        }
    }
}

// ==================== Implementation ====================

impl Config {
    /// Load configuration from all sources
    ///
    /// Configuration is loaded from (in order of precedence):
    /// 1. Environment variables with `KUSANAGI_` prefix
    /// 2. `kusanagi.toml` in current directory
    /// 3. `kusanagi.toml` in `$HOME/.config/kusanagi/`
    /// 4. Default values
    pub fn load() -> Result<Self> {
        let config = config::Config::builder()
            // Start with defaults
            .set_default("default", true)?
            // Add config file from home directory (lowest priority)
            .add_source(
                config::File::with_name("$HOME/.config/kusanagi/kusanagi")
                    .required(false)
            )
            // Add local config file
            .add_source(config::File::with_name("kusanagi").required(false))
            // Add environment variables (highest priority)
            .add_source(
                config::Environment::with_prefix("KUSANAGI")
                    .separator("_")
                    .try_parsing(true)
            )
            .build()
            .map_err(|e| KusanagiError::config(format!("Failed to build config: {}", e)))?;

        let mut config: Config = config
            .try_deserialize()
            .map_err(|e| KusanagiError::config(format!("Failed to deserialize config: {}", e)))?;

        // Handle KUSANAGI_DEV_MODE specially (checked directly as it's a common pattern)
        if std::env::var("KUSANAGI_DEV_MODE").is_ok() || std::env::var("DEV_MODE").is_ok() {
            config.dev_mode = true;
        }

        // Validate the configuration
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration
    ///
    /// Returns Ok(()) if configuration is valid, otherwise returns an error
    /// describing what's wrong.
    fn validate(&self) -> Result<()> {
        // Validate server port
        if self.server.port == 0 {
            return Err(KusanagiError::config(
                "Server port cannot be 0"
            ));
        }

        // Validate Prometheus URL format
        if !self.prometheus.url.starts_with("http://") && !self.prometheus.url.starts_with("https://") {
            return Err(KusanagiError::config(
                "Prometheus URL must start with http:// or https://"
            ));
        }

        // Validate timeout values
        if self.server.timeout_secs == 0 {
            return Err(KusanagiError::config(
                "Server timeout must be greater than 0"
            ));
        }

        // Validate cache TTLs
        if self.cache.default_ttl_secs == 0 {
            return Err(KusanagiError::config(
                "Cache TTL must be greater than 0"
            ));
        }

        // Validate log level
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.log.level.as_str()) {
            return Err(KusanagiError::config(format!(
                "Invalid log level: {}. Must be one of: {:?}",
                self.log.level, valid_log_levels
            )));
        }

        Ok(())
    }

    /// Get server socket address
    pub fn server_addr(&self) -> SocketAddr {
        format!("{}:{}", self.server.host, self.server.port)
            .parse()
            .expect("Invalid server address")
    }

    /// Get server timeout as Duration
    pub fn server_timeout(&self) -> Duration {
        Duration::from_secs(self.server.timeout_secs)
    }

    /// Check if development mode is enabled
    pub fn is_dev_mode(&self) -> bool {
        self.dev_mode
    }

    /// Get the Prometheus URL for Home Assistant metrics
    /// Falls back to main Prometheus URL if not configured
    pub fn prometheus_url_ha(&self) -> &str {
        self.prometheus.url_ha.as_deref()
            .unwrap_or(&self.prometheus.url)
    }

    /// Check if authentication is enabled
    pub fn is_auth_enabled(&self) -> bool {
        self.security.auth_enabled
    }

    /// Get CORS origins as a vector
    pub fn cors_origins(&self) -> Vec<&str> {
        self.security.cors_origins.split(',').collect()
    }
}

// ==================== Global Configuration ====================

use std::sync::OnceLock;

static CONFIG: OnceLock<Config> = OnceLock::new();

/// Initialize global configuration
///
/// Must be called once at application startup before using `config()`.
/// Panics if called twice or if configuration loading fails.
pub fn init() -> Result<()> {
    let config = Config::load()?;
    CONFIG.set(config)
        .map_err(|_| KusanagiError::internal("Configuration already initialized"))?;
    Ok(())
}

/// Get the global configuration
///
/// Panics if `init()` hasn't been called.
pub fn get() -> &'static Config {
    CONFIG.get()
        .expect("Configuration not initialized. Call config::init() first.")
}

/// Get configuration, initializing if necessary
///
/// This is a convenience function that initializes the config if needed.
/// In production, prefer explicit `init()` at startup.
pub fn get_or_init() -> &'static Config {
    CONFIG.get_or_init(|| {
        Config::load().expect("Failed to load configuration")
    })
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.host, "0.0.0.0");
        assert!(!config.dev_mode);
    }

    #[test]
    fn test_server_addr() {
        let config = Config::default();
        let addr = config.server_addr();
        assert_eq!(addr.to_string(), "0.0.0.0:8080");
    }

    #[test]
    fn test_prometheus_url_ha_fallback() {
        let config = Config::default();
        assert_eq!(config.prometheus_url_ha(), config.prometheus.url);
    }

    #[test]
    fn test_validate_invalid_port() {
        let mut config = Config::default();
        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_prometheus_url() {
        let mut config = Config::default();
        config.prometheus.url = "invalid://url".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_invalid_log_level() {
        let mut config = Config::default();
        config.log.level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_valid_config() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cors_origins() {
        let mut config = Config::default();
        config.security.cors_origins = "http://localhost:3000,http://localhost:8080".to_string();
        let origins = config.cors_origins();
        assert_eq!(origins, vec!["http://localhost:3000", "http://localhost:8080"]);
    }

    #[test]
    fn test_cache_durations() {
        let config = Config::default();
        assert_eq!(config.cache.default_ttl_secs, 300);
        assert_eq!(config.cache.prometheus_ttl_secs, 60);
    }

    #[test]
    fn test_integration_config_defaults() {
        let config = Config::default();
        assert_eq!(config.integrations.ollama.model, "ministral-3:14b");
        assert_eq!(config.integrations.mqtt.client_id, "kusanagi");
        assert_eq!(config.integrations.weather.cities, "Lyon,Mexico City,New York");
    }

    #[test]
    fn test_prometheus_config_defaults() {
        let config = PrometheusConfig::default();
        assert!(config.url.contains("kube-prometheus-stack"));
        assert_eq!(config.timeout_secs, 10);
        assert_eq!(config.cache_ttl_secs, 60);
    }

    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig::default();
        assert!(!config.auth_enabled);
        assert_eq!(config.cors_origins, "*");
        assert_eq!(config.session_timeout_hours, 24);
    }

    #[test]
    fn test_storage_config_defaults() {
        let config = StorageConfig::default();
        assert_eq!(config.region, "us-east-1");
        assert!(config.endpoint.is_none());
    }

    #[test]
    fn test_is_dev_mode() {
        let mut config = Config::default();
        assert!(!config.is_dev_mode());
        config.dev_mode = true;
        assert!(config.is_dev_mode());
    }

    #[test]
    fn test_server_timeout() {
        let config = Config::default();
        assert_eq!(config.server_timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_mcp_config_defaults() {
        let config = McpConfig::default();
        assert!(config.kubernetes_url.contains("localhost"));
        assert!(config.cilium_url.contains("localhost"));
    }
}