//! Alert Port
//!
//! Port defining the interface for alert operations.

use async_trait::async_trait;
use crate::domain::entities::{Alert, AlertsResponse, AlertStats};

/// Port for alert operations
#[async_trait]
pub trait AlertRepository: Send + Sync {
    /// Get all active alerts
    async fn get_active_alerts(&self) -> Result<AlertsResponse, String>;
    
    /// Get cached alerts (with caching logic)
    async fn get_cached_alerts(&self) -> Result<AlertsResponse, String>;
    
    /// Get alert by fingerprint
    async fn get_alert(&self, fingerprint: &str) -> Result<Alert, String>;
    
    /// Silence an alert
    async fn silence_alert(&self, fingerprint: &str, duration_secs: u64) -> Result<(), String>;
}
