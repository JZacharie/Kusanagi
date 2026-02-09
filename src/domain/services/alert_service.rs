//! Alert Domain Service
//!
//! Core business logic for alert operations.
//! This service is independent of infrastructure concerns.

use crate::domain::entities::{Alert, AlertsResponse};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Service for alert domain operations
pub struct AlertDomainService;

impl AlertDomainService {
    /// Create a new instance
    pub fn new() -> Self {
        Self
    }

    /// Create a mock alerts response for local mode
    pub fn create_mock_alerts(&self) -> AlertsResponse {
        AlertsResponse {
            critical: vec![],
            warning: vec![],
            info: vec![],
            total: 0,
            firing: 0,
            pending: 0,
        }
    }

    /// Parse severity level from string
    pub fn parse_severity(&self, severity: Option<&str>) -> &'static str {
        match severity {
            Some("critical") => "critical",
            Some("warning") => "warning",
            _ => "info",
        }
    }

    /// Parse alert state from string
    pub fn parse_state(&self, state: &str) -> &'static str {
        match state {
            "firing" => "firing",
            _ => "pending",
        }
    }

    /// Build an Alert from raw data
    pub fn build_alert(
        &self,
        name: String,
        severity: String,
        state: String,
        summary: String,
        description: Option<String>,
        namespace: Option<String>,
        pod: Option<String>,
        started_at: DateTime<Utc>,
        fingerprint: String,
    ) -> Alert {
        Alert {
            name,
            severity,
            state,
            summary,
            description,
            namespace,
            pod,
            started_at,
            fingerprint,
        }
    }

    /// Categorize and sort alerts by severity
    pub fn categorize_alerts(&self, alerts: Vec<Alert>) -> AlertsResponse {
        let mut critical = Vec::new();
        let mut warning = Vec::new();
        let mut info = Vec::new();
        let mut firing = 0i32;
        let mut pending = 0i32;

        for alert in alerts {
            // Count firing/pending
            if alert.state == "firing" {
                firing += 1;
            } else {
                pending += 1;
            }

            // Categorize by severity
            match alert.severity.as_str() {
                "critical" => critical.push(alert),
                "warning" => warning.push(alert),
                _ => info.push(alert),
            }
        }

        // Sort by started_at (most recent first)
        critical.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        warning.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        info.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        let total = (critical.len() + warning.len() + info.len()) as i32;

        AlertsResponse {
            critical,
            warning,
            info,
            total,
            firing,
            pending,
        }
    }

    /// Extract alert name from labels
    pub fn extract_alert_name(&self, labels: &HashMap<String, String>) -> String {
        labels
            .get("alertname")
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Extract namespace from labels
    pub fn extract_namespace(&self, labels: &HashMap<String, String>) -> Option<String> {
        labels.get("namespace").cloned()
    }

    /// Extract pod name from labels
    pub fn extract_pod(&self, labels: &HashMap<String, String>) -> Option<String> {
        labels.get("pod").cloned()
    }

    /// Extract summary from annotations
    pub fn extract_summary(&self, annotations: &HashMap<String, String>) -> String {
        annotations
            .get("summary")
            .cloned()
            .unwrap_or_else(|| "No summary".to_string())
    }

    /// Extract description from annotations
    pub fn extract_description(&self, annotations: &HashMap<String, String>) -> Option<String> {
        annotations.get("description").cloned()
    }

    /// Parse datetime from string
    pub fn parse_datetime(&self, datetime_str: &str) -> DateTime<Utc> {
        datetime_str
            .parse::<DateTime<Utc>>()
            .unwrap_or_else(|_| Utc::now())
    }
}

impl Default for AlertDomainService {
    fn default() -> Self {
        Self::new()
    }
}
