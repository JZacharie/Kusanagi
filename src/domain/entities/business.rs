use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareAnalytics {
    pub requests: u64,
    pub bandwidth: u64,
    pub threats: u64,
    pub page_views: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessOverview {
    pub cloudflare: CloudflareAnalytics,
}
