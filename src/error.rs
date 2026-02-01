//! Centralized error handling for Kusanagi
//!
//! This module provides a unified error type (`KusanagiError`) that encompasses
//! all possible errors in the application. It uses `thiserror` for ergonomic
//! error derivation and automatic conversions.
//!
//! # Usage
//!
//! ```rust
//! use crate::error::{KusanagiError, Result};
//!
//! fn may_fail() -> Result<String> {
//!     // Automatically converted from kube::Error
//!     let pods = client.list_pods().await?;
//!     
//!     // Custom error with context
//!     if pods.is_empty() {
//!         return Err(KusanagiError::cluster("No pods found in namespace"));
//!     }
//!     
//!     Ok(pods)
//! }
//! ```

use thiserror::Error;

/// Centralized Result type alias
pub type Result<T> = std::result::Result<T, KusanagiError>;

/// Main error type for Kusanagi application
///
/// Each variant represents a specific error domain with relevant context.
/// Variants use `#[from]` for automatic conversion from underlying errors.
#[derive(Error, Debug, Clone)]
pub enum KusanagiError {
    // ==================== Infrastructure Errors ====================
    /// Kubernetes API errors (kube-rs)
    #[error("Kubernetes API error: {0}")]
    K8s(String),

    /// Configuration errors (missing/invalid env vars)
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// Cache operation errors
    #[error("Cache error: {message}")]
    Cache { message: String },

    // ==================== External Service Errors ====================
    /// Prometheus query/connection errors
    #[error("Prometheus error: {message}")]
    Prometheus { message: String },

    /// Alertmanager errors
    #[error("Alertmanager error: {message}")]
    Alertmanager { message: String },

    /// MCP (Model Context Protocol) server errors
    #[error("MCP server error ({server}): {message}")]
    Mcp {
        server: String,
        message: String,
    },

    /// S3/MinIO storage errors
    #[error("Storage error: {message}")]
    Storage { message: String },

    /// Proxmox VE API errors
    #[error("Proxmox error: {message}")]
    Proxmox { message: String },

    /// Home Assistant API errors
    #[error("Home Assistant error: {message}")]
    HomeAssistant { message: String },

    /// MQTT broker errors
    #[error("MQTT error: {message}")]
    Mqtt { message: String },

    /// Slack API errors
    #[error("Slack error: {message}")]
    Slack { message: String },

    /// Calendar/Weather external API errors
    #[error("External API error ({provider}): {message}")]
    ExternalApi {
        provider: String,
        message: String,
    },

    // ==================== Network/HTTP Errors ====================
    /// HTTP client errors (reqwest)
    #[error("HTTP error: {message}")]
    Http { message: String },

    /// Network connectivity errors
    #[error("Network error: {message}")]
    Network { message: String },

    /// Timeout errors
    #[error("Timeout after {duration_secs}s: {operation}")]
    Timeout {
        duration_secs: u64,
        operation: String,
    },

    // ==================== Data/Serialization Errors ====================
    /// JSON serialization/deserialization errors
    #[error("JSON error: {message}")]
    Json { message: String },

    /// CSV export errors
    #[error("CSV error: {message}")]
    Csv { message: String },

    /// Data validation errors
    #[error("Validation error: {message}")]
    Validation { message: String },

    // ==================== Domain Errors ====================
    /// Resource not found (pod, node, etc.)
    #[error("Resource not found: {resource_type}/{name}")]
    NotFound {
        resource_type: String,
        name: String,
    },

    /// Resource already exists
    #[error("Resource already exists: {resource_type}/{name}")]
    AlreadyExists {
        resource_type: String,
        name: String,
    },

    /// Permission denied (RBAC)
    #[error("Permission denied: {action} on {resource}")]
    PermissionDenied { action: String, resource: String },

    /// Operation not supported
    #[error("Operation not supported: {operation}")]
    NotSupported { operation: String },

    // ==================== Internal Errors ====================
    /// Internal/unknown errors
    #[error("Internal error: {message}")]
    Internal { message: String },

    /// Feature not implemented
    #[error("Not implemented: {feature}")]
    NotImplemented { feature: String },
}

// ==================== Constructor Helpers ====================

impl KusanagiError {
    /// Create a K8s error with a message
    pub fn k8s<S: Into<String>>(msg: S) -> Self {
        Self::K8s(msg.into())
    }

    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Config {
            message: msg.into(),
        }
    }

    /// Create a Prometheus error
    pub fn prometheus<S: Into<String>>(msg: S) -> Self {
        Self::Prometheus {
            message: msg.into(),
        }
    }

    /// Create an Alertmanager error
    pub fn alertmanager<S: Into<String>>(msg: S) -> Self {
        Self::Alertmanager {
            message: msg.into(),
        }
    }

    /// Create an MCP error
    pub fn mcp<S: Into<String>>(server: S, msg: S) -> Self {
        Self::Mcp {
            server: server.into(),
            message: msg.into(),
        }
    }

    /// Create a storage error
    pub fn storage<S: Into<String>>(msg: S) -> Self {
        Self::Storage {
            message: msg.into(),
        }
    }

    /// Create a Proxmox error
    pub fn proxmox<S: Into<String>>(msg: S) -> Self {
        Self::Proxmox {
            message: msg.into(),
        }
    }

    /// Create a Home Assistant error
    pub fn home_assistant<S: Into<String>>(msg: S) -> Self {
        Self::HomeAssistant {
            message: msg.into(),
        }
    }

    /// Create an MQTT error
    pub fn mqtt<S: Into<String>>(msg: S) -> Self {
        Self::Mqtt {
            message: msg.into(),
        }
    }

    /// Create a Slack error
    pub fn slack<S: Into<String>>(msg: S) -> Self {
        Self::Slack {
            message: msg.into(),
        }
    }

    /// Create an external API error
    pub fn external_api<S: Into<String>>(provider: S, msg: S) -> Self {
        Self::ExternalApi {
            provider: provider.into(),
            message: msg.into(),
        }
    }

    /// Create an HTTP error
    pub fn http<S: Into<String>>(msg: S) -> Self {
        Self::Http {
            message: msg.into(),
        }
    }

    /// Create a network error
    pub fn network<S: Into<String>>(msg: S) -> Self {
        Self::Network {
            message: msg.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout<S: Into<String>>(secs: u64, operation: S) -> Self {
        Self::Timeout {
            duration_secs: secs,
            operation: operation.into(),
        }
    }

    /// Create a JSON error
    pub fn json<S: Into<String>>(msg: S) -> Self {
        Self::Json {
            message: msg.into(),
        }
    }

    /// Create a CSV error
    pub fn csv<S: Into<String>>(msg: S) -> Self {
        Self::Csv {
            message: msg.into(),
        }
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation {
            message: msg.into(),
        }
    }

    /// Create a not found error
    pub fn not_found<S: Into<String>>(resource_type: S, name: S) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            name: name.into(),
        }
    }

    /// Create an already exists error
    pub fn already_exists<S: Into<String>>(resource_type: S, name: S) -> Self {
        Self::AlreadyExists {
            resource_type: resource_type.into(),
            name: name.into(),
        }
    }

    /// Create a permission denied error
    pub fn permission_denied<S: Into<String>>(action: S, resource: S) -> Self {
        Self::PermissionDenied {
            action: action.into(),
            resource: resource.into(),
        }
    }

    /// Create a not supported error
    pub fn not_supported<S: Into<String>>(operation: S) -> Self {
        Self::NotSupported {
            operation: operation.into(),
        }
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal {
            message: msg.into(),
        }
    }

    /// Create a not implemented error
    pub fn not_implemented<S: Into<String>>(feature: S) -> Self {
        Self::NotImplemented {
            feature: feature.into(),
        }
    }

    /// Create a cache error
    pub fn cache<S: Into<String>>(msg: S) -> Self {
        Self::Cache {
            message: msg.into(),
        }
    }

    // ==================== Utility Methods ====================

    /// Returns true if this is a transient error that might succeed on retry
    pub fn is_transient(&self) -> bool {
        matches!(
            self,
            Self::Timeout { .. }
                | Self::Network { .. }
                | Self::K8s(_) // K8s API can be temporarily unavailable
                | Self::Prometheus { .. }
                | Self::Mcp { .. }
        )
    }

    /// Returns true if this is a client error (4xx equivalent)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::NotFound { .. }
                | Self::AlreadyExists { .. }
                | Self::PermissionDenied { .. }
                | Self::Validation { .. }
                | Self::Config { .. }
        )
    }

    /// Get HTTP status code equivalent for this error
    pub fn http_status(&self) -> actix_web::http::StatusCode {
        use actix_web::http::StatusCode;

        match self {
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::AlreadyExists { .. } => StatusCode::CONFLICT,
            Self::PermissionDenied { .. } => StatusCode::FORBIDDEN,
            Self::Validation { .. } | Self::Config { .. } => StatusCode::BAD_REQUEST,
            Self::Timeout { .. } => StatusCode::REQUEST_TIMEOUT,
            Self::NotSupported { .. } | Self::NotImplemented { .. } => StatusCode::NOT_IMPLEMENTED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Convert to a user-friendly message (without technical details)
    pub fn user_message(&self) -> String {
        match self {
            Self::NotFound { resource_type, name } => {
                format!("{} '{}' not found", resource_type, name)
            }
            Self::PermissionDenied { action, .. } => {
                format!("You don't have permission to {}", action)
            }
            Self::Timeout { .. } => "The operation timed out. Please try again.".to_string(),
            Self::Validation { message } => format!("Invalid input: {}", message),
            Self::Config { .. } => "Server configuration error. Contact administrator.".to_string(),
            _ => "An unexpected error occurred. Please try again later.".to_string(),
        }
    }
}

// ==================== Standard Error Conversions ====================

/// Convert from kube::Error
impl From<kube::Error> for KusanagiError {
    fn from(err: kube::Error) -> Self {
        use kube::Error as KubeErr;

        match &err {
            KubeErr::Api(resp) if resp.code == 404 => Self::NotFound {
                resource_type: "resource".to_string(),
                name: resp.message.clone(),
            },
            KubeErr::Api(resp) if resp.code == 403 => Self::PermissionDenied {
                action: "access".to_string(),
                resource: resp.message.clone(),
            },
            KubeErr::Api(resp) if resp.code == 409 => Self::AlreadyExists {
                resource_type: "resource".to_string(),
                name: resp.message.clone(),
            },
            _ => Self::K8s(err.to_string()),
        }
    }
}

/// Convert from reqwest::Error
impl From<reqwest::Error> for KusanagiError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout {
                duration_secs: 30, // Default assumption
                operation: "HTTP request".to_string(),
            }
        } else if err.is_connect() {
            Self::Network {
                message: err.to_string(),
            }
        } else {
            Self::Http {
                message: err.to_string(),
            }
        }
    }
}

/// Convert from serde_json::Error
impl From<serde_json::Error> for KusanagiError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json {
            message: err.to_string(),
        }
    }
}

/// Convert from csv::Error
impl From<csv::Error> for KusanagiError {
    fn from(err: csv::Error) -> Self {
        Self::Csv {
            message: err.to_string(),
        }
    }
}

/// Convert from std::env::VarError
impl From<std::env::VarError> for KusanagiError {
    fn from(err: std::env::VarError) -> Self {
        Self::Config {
            message: format!("Environment variable error: {}", err),
        }
    }
}

/// Convert from std::io::Error
impl From<std::io::Error> for KusanagiError {
    fn from(err: std::io::Error) -> Self {
        Self::Internal {
            message: format!("IO error: {}", err),
        }
    }
}

/// Convert from aws_sdk_s3::Error (simplified)
impl From<aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::put_object::PutObjectError>>
    for KusanagiError
{
    fn from(err: aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::put_object::PutObjectError>) -> Self {
        Self::Storage {
            message: format!("S3 put error: {}", err),
        }
    }
}

impl From<aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>>
    for KusanagiError
{
    fn from(err: aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>) -> Self {
        Self::Storage {
            message: format!("S3 get error: {}", err),
        }
    }
}

/// Convert from rumqttc::ClientError
impl From<rumqttc::ClientError> for KusanagiError {
    fn from(err: rumqttc::ClientError) -> Self {
        Self::Mqtt {
            message: err.to_string(),
        }
    }
}

// ==================== String Conversions (for gradual migration) ====================

impl From<String> for KusanagiError {
    fn from(msg: String) -> Self {
        Self::Internal { message: msg }
    }
}

impl From<&str> for KusanagiError {
    fn from(msg: &str) -> Self {
        Self::Internal {
            message: msg.to_string(),
        }
    }
}

// ==================== Actix-web Response Helper ====================

use actix_web::{HttpResponse, ResponseError};

impl ResponseError for KusanagiError {
    fn error_response(&self) -> HttpResponse {
        let status = self.http_status();
        let body = serde_json::json!({
            "error": self.to_string(),
            "message": self.user_message(),
            "transient": self.is_transient(),
        });

        HttpResponse::build(status).json(body)
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        self.http_status()
    }
}

// ==================== Macros for Easy Error Creation ====================

/// Macro to return an error with format! syntax
#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return Err($err.into());
    };
    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::error::KusanagiError::internal(format!($fmt, $($arg)*)));
    };
}

/// Macro to ensure a condition, returning an error if false
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err.into());
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return Err($crate::error::KusanagiError::internal(format!($fmt, $($arg)*)));
        }
    };
}

// Re-export for convenience
pub use crate::{bail, ensure};

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;

    // ==================== Error Creation Tests ====================

    #[test]
    fn test_k8s_error_creation() {
        let err = KusanagiError::k8s("connection refused");
        assert_eq!(err.to_string(), "Kubernetes API error: connection refused");
    }

    #[test]
    fn test_config_error_creation() {
        let err = KusanagiError::config("MISSING_VAR not set");
        assert!(matches!(err, KusanagiError::Config { ref message } if message == "MISSING_VAR not set"));
        assert_eq!(err.to_string(), "Configuration error: MISSING_VAR not set");
    }

    #[test]
    fn test_prometheus_error_creation() {
        let err = KusanagiError::prometheus("query timeout");
        assert!(matches!(err, KusanagiError::Prometheus { message } if message == "query timeout"));
    }

    #[test]
    fn test_mcp_error_creation() {
        let err = KusanagiError::mcp("kubernetes", "connection failed");
        assert!(matches!(err, KusanagiError::Mcp { ref server, ref message } 
            if server == "kubernetes" && message == "connection failed"
        ));
        assert_eq!(err.to_string(), "MCP server error (kubernetes): connection failed");
    }

    #[test]
    fn test_storage_error_creation() {
        let err = KusanagiError::storage("S3 bucket not found");
        assert!(matches!(err, KusanagiError::Storage { message } if message == "S3 bucket not found"));
    }

    #[test]
    fn test_external_api_error_creation() {
        let err = KusanagiError::external_api("Proxmox", "API rate limit exceeded");
        assert!(matches!(err, KusanagiError::ExternalApi { provider, message } 
            if provider == "Proxmox" && message == "API rate limit exceeded"
        ));
    }

    #[test]
    fn test_not_found_error_creation() {
        let err = KusanagiError::not_found("Pod", "nginx-abc123");
        assert!(matches!(err, KusanagiError::NotFound { ref resource_type, ref name } 
            if resource_type == "Pod" && name == "nginx-abc123"
        ));
        assert_eq!(err.to_string(), "Resource not found: Pod/nginx-abc123");
    }

    #[test]
    fn test_already_exists_error_creation() {
        let err = KusanagiError::already_exists("Namespace", "production");
        assert!(matches!(err, KusanagiError::AlreadyExists { resource_type, name } 
            if resource_type == "Namespace" && name == "production"
        ));
    }

    #[test]
    fn test_permission_denied_error_creation() {
        let err = KusanagiError::permission_denied("delete", "Pod/nginx");
        assert!(matches!(err, KusanagiError::PermissionDenied { ref action, ref resource } 
            if action == "delete" && resource == "Pod/nginx"
        ));
        assert_eq!(err.to_string(), "Permission denied: delete on Pod/nginx");
    }

    #[test]
    fn test_timeout_error_creation() {
        let err = KusanagiError::timeout(30, "Prometheus query");
        assert!(matches!(err, KusanagiError::Timeout { duration_secs, ref operation } 
            if duration_secs == 30 && operation == "Prometheus query"
        ));
        assert_eq!(err.to_string(), "Timeout after 30s: Prometheus query");
    }

    #[test]
    fn test_validation_error_creation() {
        let err = KusanagiError::validation("field 'name' is required");
        assert!(matches!(err, KusanagiError::Validation { message } 
            if message == "field 'name' is required"
        ));
    }

    #[test]
    fn test_not_implemented_error_creation() {
        let err = KusanagiError::not_implemented("multi-cluster support");
        assert!(matches!(err, KusanagiError::NotImplemented { ref feature } 
            if feature == "multi-cluster support"
        ));
        assert_eq!(err.to_string(), "Not implemented: multi-cluster support");
    }

    #[test]
    fn test_internal_error_creation() {
        let err = KusanagiError::internal("unexpected panic");
        assert!(matches!(err, KusanagiError::Internal { message } 
            if message == "unexpected panic"
        ));
    }

    // ==================== Classification Tests ====================

    #[test]
    fn test_is_transient_true() {
        // Timeout should be transient
        let err = KusanagiError::timeout(30, "test");
        assert!(err.is_transient());

        // Network errors should be transient
        let err = KusanagiError::network("connection refused");
        assert!(err.is_transient());

        // K8s errors should be transient
        let err = KusanagiError::k8s("server unavailable");
        assert!(err.is_transient());

        // Prometheus errors should be transient
        let err = KusanagiError::prometheus("service unavailable");
        assert!(err.is_transient());

        // MCP errors should be transient
        let err = KusanagiError::mcp("test", "error");
        assert!(err.is_transient());
    }

    #[test]
    fn test_is_transient_false() {
        // Not found should NOT be transient
        let err = KusanagiError::not_found("Pod", "test");
        assert!(!err.is_transient());

        // Validation errors should NOT be transient
        let err = KusanagiError::validation("invalid input");
        assert!(!err.is_transient());

        // Permission denied should NOT be transient
        let err = KusanagiError::permission_denied("delete", "resource");
        assert!(!err.is_transient());

        // Config errors should NOT be transient
        let err = KusanagiError::config("missing");
        assert!(!err.is_transient());
    }

    #[test]
    fn test_is_client_error_true() {
        assert!(KusanagiError::not_found("Pod", "test").is_client_error());
        assert!(KusanagiError::already_exists("Pod", "test").is_client_error());
        assert!(KusanagiError::permission_denied("get", "secret").is_client_error());
        assert!(KusanagiError::validation("invalid").is_client_error());
        assert!(KusanagiError::config("missing").is_client_error());
    }

    #[test]
    fn test_is_client_error_false() {
        assert!(!KusanagiError::k8s("error").is_client_error());
        assert!(!KusanagiError::prometheus("error").is_client_error());
        assert!(!KusanagiError::timeout(30, "test").is_client_error());
        assert!(!KusanagiError::internal("error").is_client_error());
    }

    // ==================== HTTP Status Tests ====================

    #[test]
    fn test_http_status_not_found() {
        let err = KusanagiError::not_found("Pod", "nginx");
        assert_eq!(err.http_status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_http_status_already_exists() {
        let err = KusanagiError::already_exists("Namespace", "prod");
        assert_eq!(err.http_status(), StatusCode::CONFLICT);
    }

    #[test]
    fn test_http_status_permission_denied() {
        let err = KusanagiError::permission_denied("delete", "secret");
        assert_eq!(err.http_status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn test_http_status_validation() {
        let err = KusanagiError::validation("invalid");
        assert_eq!(err.http_status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_http_status_config() {
        let err = KusanagiError::config("missing");
        assert_eq!(err.http_status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_http_status_timeout() {
        let err = KusanagiError::timeout(30, "test");
        assert_eq!(err.http_status(), StatusCode::REQUEST_TIMEOUT);
    }

    #[test]
    fn test_http_status_not_implemented() {
        let err = KusanagiError::not_implemented("feature");
        assert_eq!(err.http_status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[test]
    fn test_http_status_internal_server_error() {
        assert_eq!(KusanagiError::k8s("error").http_status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(KusanagiError::prometheus("error").http_status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(KusanagiError::internal("error").http_status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // ==================== User Message Tests ====================

    #[test]
    fn test_user_message_not_found() {
        let err = KusanagiError::not_found("Pod", "nginx-123");
        assert_eq!(err.user_message(), "Pod 'nginx-123' not found");
    }

    #[test]
    fn test_user_message_permission_denied() {
        let err = KusanagiError::permission_denied("delete pods", "default namespace");
        assert_eq!(err.user_message(), "You don't have permission to delete pods");
    }

    #[test]
    fn test_user_message_timeout() {
        let err = KusanagiError::timeout(30, "test");
        assert_eq!(err.user_message(), "The operation timed out. Please try again.");
    }

    #[test]
    fn test_user_message_validation() {
        let err = KusanagiError::validation("name is required");
        assert_eq!(err.user_message(), "Invalid input: name is required");
    }

    #[test]
    fn test_user_message_config() {
        let err = KusanagiError::config("API_KEY missing");
        assert_eq!(err.user_message(), "Server configuration error. Contact administrator.");
    }

    #[test]
    fn test_user_message_generic() {
        let err = KusanagiError::k8s("connection refused");
        assert_eq!(err.user_message(), "An unexpected error occurred. Please try again later.");
    }

    // ==================== String Conversion Tests ====================

    #[test]
    fn test_from_string() {
        let err: KusanagiError = "some error message".to_string().into();
        assert!(matches!(err, KusanagiError::Internal { message } 
            if message == "some error message"
        ));
    }

    #[test]
    fn test_from_str() {
        let err: KusanagiError = "some error message".into();
        assert!(matches!(err, KusanagiError::Internal { message } 
            if message == "some error message"
        ));
    }

    // ==================== Serde Tests ====================

    #[test]
    fn test_serde_json_conversion() {
        // Create a JSON parsing error
        let json_err: std::result::Result<serde_json::Value, _> = serde_json::from_str("invalid json");
        let err: KusanagiError = json_err.unwrap_err().into();
        
        assert!(matches!(err, KusanagiError::Json { ref message } 
            if message.contains("expected value")
        ));
    }

    // ==================== Helper Function Tests ====================

    #[test]
    fn test_cache_error_creation() {
        let err = KusanagiError::cache("lock poisoned");
        assert!(matches!(err, KusanagiError::Cache { message } 
            if message == "lock poisoned"
        ));
    }

    #[test]
    fn test_http_error_creation() {
        let err = KusanagiError::http("404 Not Found");
        assert!(matches!(err, KusanagiError::Http { message } 
            if message == "404 Not Found"
        ));
    }

    #[test]
    fn test_network_error_creation() {
        let err = KusanagiError::network("DNS resolution failed");
        assert!(matches!(err, KusanagiError::Network { message } 
            if message == "DNS resolution failed"
        ));
    }

    #[test]
    fn test_json_error_creation() {
        let err = KusanagiError::json("unexpected token");
        assert!(matches!(err, KusanagiError::Json { message } 
            if message == "unexpected token"
        ));
    }

    #[test]
    fn test_csv_error_creation() {
        let err = KusanagiError::csv("invalid column");
        assert!(matches!(err, KusanagiError::Csv { message } 
            if message == "invalid column"
        ));
    }

    #[test]
    fn test_slack_error_creation() {
        let err = KusanagiError::slack("rate limited");
        assert!(matches!(err, KusanagiError::Slack { message } 
            if message == "rate limited"
        ));
    }

    #[test]
    fn test_mqtt_error_creation() {
        let err = KusanagiError::mqtt("broker unreachable");
        assert!(matches!(err, KusanagiError::Mqtt { message } 
            if message == "broker unreachable"
        ));
    }

    #[test]
    fn test_proxmox_error_creation() {
        let err = KusanagiError::proxmox("API error");
        assert!(matches!(err, KusanagiError::Proxmox { message } 
            if message == "API error"
        ));
    }

    #[test]
    fn test_home_assistant_error_creation() {
        let err = KusanagiError::home_assistant("token expired");
        assert!(matches!(err, KusanagiError::HomeAssistant { message } 
            if message == "token expired"
        ));
    }

    #[test]
    fn test_not_supported_error_creation() {
        let err = KusanagiError::not_supported("websocket compression");
        assert!(matches!(err, KusanagiError::NotSupported { ref operation } 
            if operation == "websocket compression"
        ));
        assert_eq!(err.to_string(), "Operation not supported: websocket compression");
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_error_chain_compatibility() {
        // Test that errors work well with the ? operator
        fn inner_function() -> Result<String> {
            Err(KusanagiError::not_found("ConfigMap", "settings"))
        }

        fn outer_function() -> Result<String> {
            let result = inner_function()?;
            Ok(result)
        }

        let result = outer_function();
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(matches!(e, KusanagiError::NotFound { resource_type, name } 
                if resource_type == "ConfigMap" && name == "settings"
            ));
        }
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        fn returns_error() -> Result<i32> {
            Err(KusanagiError::validation("test"))
        }

        assert!(returns_result().is_ok());
        assert!(returns_error().is_err());
    }

    // ==================== ResponseError Trait Tests ====================

    #[test]
    fn test_response_error_status_code() {
        let err = KusanagiError::not_found("Pod", "test");
        assert_eq!(err.status_code(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_response_error_error_response() {
        let err = KusanagiError::not_found("Pod", "nginx-123");
        let response = err.error_response();
        
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_empty_strings() {
        let err = KusanagiError::internal("");
        assert_eq!(err.to_string(), "Internal error: ");
        
        let err = KusanagiError::k8s("");
        assert_eq!(err.to_string(), "Kubernetes API error: ");
    }

    #[test]
    fn test_special_characters_in_messages() {
        let msg = "Error: <script>alert('xss')</script>";
        let err = KusanagiError::validation(msg);
        assert_eq!(err.user_message(), format!("Invalid input: {}", msg));
    }

    #[test]
    fn test_unicode_in_messages() {
        let msg = "配置错误 🚨";
        let err = KusanagiError::config(msg);
        assert!(matches!(err, KusanagiError::Config { message } 
            if message == msg
        ));
    }

    #[test]
    fn test_long_messages() {
        let long_msg = "a".repeat(10000);
        let err = KusanagiError::internal(&long_msg);
        assert!(matches!(err, KusanagiError::Internal { message } 
            if message == long_msg
        ));
    }
}
