//! Alert Entities
//!
//! Domain entities for alert operations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Single alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub name: String,
    pub severity: AlertSeverity,
    pub state: AlertState,
    pub summary: String,
    pub description: Option<String>,
    pub namespace: Option<String>,
    pub pod: Option<String>,
    pub started_at: DateTime<Utc>,
    pub fingerprint: String,
}

/// Alert severity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
    Unknown,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Critical => write!(f, "critical"),
            AlertSeverity::Warning => write!(f, "warning"),
            AlertSeverity::Info => write!(f, "info"),
            AlertSeverity::Unknown => write!(f, "unknown"),
        }
    }
}

impl Default for AlertSeverity {
    fn default() -> Self {
        AlertSeverity::Unknown
    }
}

/// Alert state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertState {
    Firing,
    Pending,
    Resolved,
}

impl std::fmt::Display for AlertState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertState::Firing => write!(f, "firing"),
            AlertState::Pending => write!(f, "pending"),
            AlertState::Resolved => write!(f, "resolved"),
        }
    }
}

impl Default for AlertState {
    fn default() -> Self {
        AlertState::Pending
    }
}

/// Grouped alerts response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertsResponse {
    pub critical: Vec<Alert>,
    pub warning: Vec<Alert>,
    pub info: Vec<Alert>,
    pub total: i32,
    pub firing: i32,
    pub pending: i32,
}

/// Alert statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertStats {
    pub total: i32,
    pub critical: i32,
    pub warning: i32,
    pub info: i32,
    pub firing: i32,
    pub pending: i32,
}
