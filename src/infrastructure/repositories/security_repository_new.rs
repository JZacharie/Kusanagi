use async_trait::async_trait;
use crate::domain::ports::SecurityRepository;
use crate::domain::entities::{SecurityReport, EnrichmentData, SecurityScanSummary, ReportMetadata};
use crate::error::{Result, KusanagiError};
// use crate::legacy; // Disabled for core version

pub struct LegacySecurityRepository;

#[async_trait]
impl SecurityRepository for LegacySecurityRepository {
    async fn list_reports(&self) -> Result<Vec<SecurityReport>> {
        Ok(vec![])
    }

    async fn get_report(&self, _category: &str, _name: &str) -> Result<SecurityReport> {
        Ok(SecurityReport::default())
    }

    async fn store_report(&self, _report: &SecurityReport) -> Result<()> {
        Ok(())
    }

    async fn get_scan_summary(&self) -> Result<SecurityScanSummary> {
        Ok(SecurityScanSummary::default())
    }
}
