use crate::domain::entities::business::BusinessOverview;
use crate::domain::ports::business::CloudflareRepository;
use crate::error::Result;
use std::sync::Arc;

pub struct GetBusinessOverviewUseCase {
    cloudflare_repository: Arc<dyn CloudflareRepository>,
}

impl GetBusinessOverviewUseCase {
    pub fn new(cloudflare_repository: Arc<dyn CloudflareRepository>) -> Self {
        Self {
            cloudflare_repository,
        }
    }

    pub async fn execute(&self) -> Result<BusinessOverview> {
        self.cloudflare_repository.get_analytics_overview().await
    }
}
