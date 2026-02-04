use crate::domain::ports::{SecurityRepository, SecurityReport, EnrichmentData};
use crate::error::Result;
use std::sync::Arc;

pub struct GetSecurityReportsUseCase {
    security_repo: Arc<dyn SecurityRepository>,
}

impl GetSecurityReportsUseCase {
    pub fn new(security_repo: Arc<dyn SecurityRepository>) -> Self {
        Self { security_repo }
    }

    pub async fn execute(&self) -> Result<Vec<SecurityReport>> {
        self.security_repo.list_reports().await
    }
}

pub struct EnrichSecurityReportUseCase {
    security_repo: Arc<dyn SecurityRepository>,
}

impl EnrichSecurityReportUseCase {
    pub fn new(security_repo: Arc<dyn SecurityRepository>) -> Self {
        Self { security_repo }
    }

    pub async fn execute(&self, report_id: &str) -> Result<EnrichmentData> {
        self.security_repo.enrich_report(report_id).await
    }
}
