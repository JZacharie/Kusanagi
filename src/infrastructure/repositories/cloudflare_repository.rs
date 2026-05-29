use crate::domain::entities::business::{BusinessOverview, CloudflareAnalytics};
use crate::domain::ports::CloudflareRepository;
use crate::error::{KusanagiError, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::env;
use tracing::{error, info};

pub struct CloudflareRepositoryImpl {
    account_id: String,
    api_token: String,
    http_client: reqwest::Client,
}

impl Default for CloudflareRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl CloudflareRepositoryImpl {
    pub fn new() -> Self {
        let account_id = env::var("CLOUDFLARE_ACCOUNT_ID")
            .unwrap_or_else(|_| "f9c73ac5f7a1b7bcd0958aaf219779f0".to_string());
        let api_token = env::var("CLOUDFLARE_API_TOKEN").unwrap_or_else(|_| {
            "cfat_GVE8PTnT6dDAQdyJn4NKgc5J0bt6FIIwbjxdEcwVd3611488".to_string()
        });

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            account_id,
            api_token,
            http_client,
        }
    }
}

#[async_trait]
impl CloudflareRepository for CloudflareRepositoryImpl {
    async fn get_analytics_overview(&self) -> Result<BusinessOverview> {
        info!(
            "Fetching Cloudflare analytics for account {}",
            self.account_id
        );

        let url = "https://api.cloudflare.com/client/v4/graphql";

        // Query last 24 hours of data
        let now = chrono::Utc::now();
        let yesterday = now - chrono::Duration::hours(24);
        let date_str = yesterday.format("%Y-%m-%dT%H:%M:%SZ").to_string();

        let query = json!({
            "query": r#"
                query GetAccountAnalytics($accountTag: String!, $datetimeGte: String!) {
                  viewer {
                    accounts(filter: {accountTag: $accountTag}) {
                      httpRequestsAdaptiveGroups(
                        limit: 100,
                        filter: {datetime_gte: $datetimeGte}
                      ) {
                        sum {
                          requests
                          bytes
                          pageViews
                        }
                      }
                      firewallEventsAdaptiveGroups(
                        limit: 100,
                        filter: {datetime_gte: $datetimeGte}
                      ) {
                        count
                      }
                    }
                  }
                }
            "#,
            "variables": {
                "accountTag": self.account_id,
                "datetimeGte": date_str
            }
        });

        let response = self
            .http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&query)
            .send()
            .await
            .map_err(|e| KusanagiError::external_service(format!("Cloudflare API error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Cloudflare API error {}: {}", status, text);
            return Err(KusanagiError::external_service(format!(
                "Cloudflare API error {}: {}",
                status, text
            )));
        }

        let resp_body: Value = response
            .json()
            .await
            .map_err(|e| KusanagiError::serialization(e.to_string()))?;

        // Extract data
        let account_data = &resp_body["data"]["viewer"]["accounts"][0];

        let http_sum = &account_data["httpRequestsAdaptiveGroups"];
        let mut total_requests = 0;
        let mut total_bytes = 0;
        let mut total_page_views = 0;

        if let Some(groups) = http_sum.as_array() {
            for group in groups {
                total_requests += group["sum"]["requests"].as_u64().unwrap_or(0);
                total_bytes += group["sum"]["bytes"].as_u64().unwrap_or(0);
                total_page_views += group["sum"]["pageViews"].as_u64().unwrap_or(0);
            }
        }

        let total_threats = account_data["firewallEventsAdaptiveGroups"]
            .as_array()
            .map(|arr| arr.iter().map(|g| g["count"].as_u64().unwrap_or(0)).sum())
            .unwrap_or(0);

        Ok(BusinessOverview {
            cloudflare: CloudflareAnalytics {
                requests: total_requests,
                bandwidth: total_bytes,
                threats: total_threats,
                page_views: total_page_views,
            },
        })
    }
}
