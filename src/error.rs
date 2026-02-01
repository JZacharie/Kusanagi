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
