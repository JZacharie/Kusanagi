//! Security Port
//!
//! Port defining the interface for security operations.

use async_trait::async_trait;
use crate::error::Result;
use crate::domain::entities::{SecurityReport, SecurityScanSummary, EnrichmentData};

/// Port for security report operations
#[async_trait]
pub trait SecurityRepository: Send + Sync {
    /// List available reports
    async fn list_reports(&self) -> Result<Vec<SecurityReport>>;
    
    /// Get a specific report
    async fn get_report(&self, category: &str, name: &str) -> Result<SecurityReport>;
    
    /// Store a report
    async fn store_report(&self, report: &SecurityReport) -> Result<()>;
    
    /// Get security scan summary
    async fn get_scan_summary(&self) -> Result<SecurityScanSummary>;
}

/// Port for AI enrichment operations
#[async_trait]
pub trait AiEnrichmentService: Send + Sync {
    /// Enrich a security report with AI analysis
    async fn enrich_report(&self, report: &serde_json::Value, language: &str) -> Result<crate::domain::entities::EnrichmentData, String>;
}

/// Port for vulnerability scanner
#[async_trait]
pub trait VulnerabilityScanner: Send + Sync {
    /// Fetch raw reports from scanner
    async fn fetch_reports(&self) -> Result<Vec<(String, String, serde_json::Value)>, String>;
    
    /// Fetch a specific report
    async fn fetch_report(&self, category: &str, name: &str) -> Result<serde_json::Value, String>;
}
