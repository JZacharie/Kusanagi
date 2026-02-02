//! Security Entities
//!
//! Domain entities for security operations.

use serde::{Deserialize, Serialize};

/// Security report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub name: String,
    pub report_type: String,
    pub original_data: serde_json::Value,
    pub enrichment: Option<EnrichmentData>,
    pub timestamp: String,
}

/// Enrichment data from AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentData {
    pub summary: String,
    pub remediation_advice: String,
    pub criticality_score: f64,
}

/// Vulnerability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub title: String,
    pub severity: Severity,
    pub description: Option<String>,
    pub fixed_version: Option<String>,
}

/// Severity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Unknown,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "Critical"),
            Severity::High => write!(f, "High"),
            Severity::Medium => write!(f, "Medium"),
            Severity::Low => write!(f, "Low"),
            Severity::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Default for Severity {
    fn default() -> Self {
        Severity::Unknown
    }
}

/// Security scan summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanSummary {
    pub total_vulnerabilities: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub scanned_at: String,
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub key: String,
    pub category: String,
    pub name: String,
    pub size: i64,
    pub last_modified: String,
}
