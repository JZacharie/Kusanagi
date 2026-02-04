use async_trait::async_trait;
use crate::domain::ports::{SecurityRepository, SecurityReport, EnrichmentData};
use crate::error::{Result, KusanagiError};
use crate::legacy;

pub struct LegacySecurityRepository;

#[async_trait]
impl SecurityRepository for LegacySecurityRepository {
    async fn list_reports(&self) -> Result<Vec<SecurityReport>> {
        legacy::security::list_enriched_reports().await
            .map_err(|e| KusanagiError::external_api("Security", &e.to_string()))
    }

    async fn get_report(&self, report_id: &str) -> Result<SecurityReport> {
        legacy::security::get_enriched_report(report_id).await
            .map_err(|e| KusanagiError::external_api("Security", &e.to_string()))
    }

    async fn enrich_report(&self, report_id: &str) -> Result<EnrichmentData> {
        legacy::security::enrich_with_ollama(report_id).await
            .map_err(|e| KusanagiError::external_api("Security", &e.to_string()))
    }
}
