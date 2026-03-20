use crate::domain::entities::business::BusinessOverview;
use crate::error::Result;
use async_trait::async_trait;

#[async_trait]
pub trait CloudflareRepository: Send + Sync {
    /// Get account-level analytics overview
    async fn get_analytics_overview(&self) -> Result<BusinessOverview>;
}
