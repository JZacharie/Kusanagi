//! Security Use Cases
//!
//! Application layer use cases for security operations.

use crate::domain::entities::{SecurityReport, ReportMetadata, SecurityScanSummary};
use crate::domain::ports::{SecurityRepository, AiEnrichmentService, VulnerabilityScanner};
use crate::error::{KusanagiError, Result};
use std::sync::Arc;

/// List security reports use case
pub struct ListSecurityReportsUseCase {
    repository: Arc<dyn SecurityRepository>,
}

impl ListSecurityReportsUseCase {
    pub fn new(repository: Arc<dyn SecurityRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<Vec<ReportMetadata>> {
        self.repository.list_reports().await
            .map_err(|e| KusanagiError::internal(format!("Failed to list reports: {}", e)))
    }
}

/// Get security report use case
pub struct GetSecurityReportUseCase {
    repository: Arc<dyn SecurityRepository>,
}

impl GetSecurityReportUseCase {
    pub fn new(repository: Arc<dyn SecurityRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, category: &str, name: &str) -> Result<SecurityReport> {
        self.repository.get_report(category, name).await
            .map_err(|_e| KusanagiError::not_found("Security report", &format!("{}/{}", category, name)))
    }
}

/// Get security summary use case
pub struct GetSecuritySummaryUseCase {
    repository: Arc<dyn SecurityRepository>,
}

impl GetSecuritySummaryUseCase {
    pub fn new(repository: Arc<dyn SecurityRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self) -> Result<SecurityScanSummary> {
        self.repository.get_scan_summary().await
            .map_err(|e| KusanagiError::internal(format!("Failed to get security summary: {}", e)))
    }
}

/// Enrich security report use case
pub struct EnrichSecurityReportUseCase {
    scanner: Arc<dyn VulnerabilityScanner>,
    enrichment: Arc<dyn AiEnrichmentService>,
    repository: Arc<dyn SecurityRepository>,
}

impl EnrichSecurityReportUseCase {
    pub fn new(
        scanner: Arc<dyn VulnerabilityScanner>,
        enrichment: Arc<dyn AiEnrichmentService>,
        repository: Arc<dyn SecurityRepository>,
    ) -> Self {
        Self { scanner, enrichment, repository }
    }

    pub async fn execute(&self, category: &str, name: &str, language: &str) -> Result<SecurityReport> {
        // Fetch raw report
        let raw_report = self.scanner.fetch_report(category, name).await
            .map_err(|e| KusanagiError::internal(format!("Failed to fetch report: {}", e)))?;
        
        // Enrich with AI
        let enrichment = self.enrichment.enrich_report(&raw_report, language).await.ok();
        
        // Create enriched report
        let report = SecurityReport {
            name: name.to_string(),
            report_type: category.to_string(),
            original_data: raw_report,
            enrichment,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        
        // Store enriched report
        self.repository.store_report(&report).await
            .map_err(|e| KusanagiError::internal(format!("Failed to store report: {}", e)))?;
        
        Ok(report)
    }
}

/// Run security enrichment worker use case
pub struct RunSecurityEnrichmentUseCase {
    scanner: Arc<dyn VulnerabilityScanner>,
    enrichment: Arc<dyn AiEnrichmentService>,
    repository: Arc<dyn SecurityRepository>,
}

impl RunSecurityEnrichmentUseCase {
    pub fn new(
        scanner: Arc<dyn VulnerabilityScanner>,
        enrichment: Arc<dyn AiEnrichmentService>,
        repository: Arc<dyn SecurityRepository>,
    ) -> Self {
        Self { scanner, enrichment, repository }
    }

    pub async fn execute(&self, language: &str) -> Result<usize> {
        // Fetch all reports
        let reports = self.scanner.fetch_reports().await
            .map_err(|e| KusanagiError::internal(format!("Failed to fetch reports: {}", e)))?;
        
        let mut processed = 0;
        
        for (category, name, raw_report) in reports {
            // Enrich with AI
            let enrichment = self.enrichment.enrich_report(&raw_report, language).await.ok();
            
            // Create enriched report
            let report = SecurityReport {
                name: name.clone(),
                report_type: category.clone(),
                original_data: raw_report,
                enrichment,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
            
            // Store enriched report
            if self.repository.store_report(&report).await.is_ok() {
                processed += 1;
            }
        }
        
        Ok(processed)
    }
}

/// Security service - aggregates all security use cases
pub struct SecurityUseCaseService {
    pub list_reports: ListSecurityReportsUseCase,
    pub get_report: GetSecurityReportUseCase,
    pub get_summary: GetSecuritySummaryUseCase,
    pub enrich_report: EnrichSecurityReportUseCase,
    pub run_enrichment: RunSecurityEnrichmentUseCase,
}

impl SecurityUseCaseService {
    pub fn new(
        repository: Arc<dyn SecurityRepository>,
        scanner: Arc<dyn VulnerabilityScanner>,
        enrichment: Arc<dyn AiEnrichmentService>,
    ) -> Self {
        Self {
            list_reports: ListSecurityReportsUseCase::new(repository.clone()),
            get_report: GetSecurityReportUseCase::new(repository.clone()),
            get_summary: GetSecuritySummaryUseCase::new(repository.clone()),
            enrich_report: EnrichSecurityReportUseCase::new(scanner.clone(), enrichment.clone(), repository.clone()),
            run_enrichment: RunSecurityEnrichmentUseCase::new(scanner, enrichment, repository),
        }
    }
}
