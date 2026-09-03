use crate::domain::entities::business::{BusinessOverview, CloudflareAnalytics};
use crate::domain::ports::CloudflareRepository;
use crate::error::{KusanagiError, Result};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::env;
use tracing::{error, info};

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct CloudflareRepositoryImpl {
    account_id: String,
    api_token: String,
    http_client: reqwest::Client,
    cache: Arc<RwLock<Option<(Instant, BusinessOverview)>>>,
    cooldown_until: Arc<RwLock<Option<Instant>>>,
    cache_ttl: Duration,
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

        let cache_ttl_secs = env::var("CLOUDFLARE_CACHE_TTL_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(1800); // 30 minutes default cache

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            account_id,
            api_token,
            http_client,
            cache: Arc::new(RwLock::new(None)),
            cooldown_until: Arc::new(RwLock::new(None)),
            cache_ttl: Duration::from_secs(cache_ttl_secs),
        }
    }
}

#[async_trait]
impl CloudflareRepository for CloudflareRepositoryImpl {
    async fn get_analytics_overview(&self) -> Result<BusinessOverview> {
        // 1. Check in-memory cache
        {
            let cache_read = self.cache.read().await;
            if let Some((cached_at, ref data)) = *cache_read {
                if cached_at.elapsed() < self.cache_ttl {
                    info!(
                        "Serving Cloudflare analytics from cache (age: {}s)",
                        cached_at.elapsed().as_secs()
                    );
                    return Ok(data.clone());
                }
            }
        }

        // 2. Check if we are in a rate-limit cooldown
        {
            let cooldown = self.cooldown_until.read().await;
            if let Some(until) = *cooldown {
                if Instant::now() < until {
                    info!("Cloudflare API is in cooldown after 429/rate-limit, serving cached/default data");
                    let cache_read = self.cache.read().await;
                    if let Some((_, ref data)) = *cache_read {
                        return Ok(data.clone());
                    }
                    return Ok(BusinessOverview {
                        cloudflare: CloudflareAnalytics {
                            requests: 0,
                            bandwidth: 0,
                            threats: 0,
                            page_views: 0,
                        },
                    });
                }
            }
        }

        info!(
            "Fetching Cloudflare analytics for account {}",
            if self.account_id.len() > 8 {
                format!(
                    "{}***{}",
                    &self.account_id[..4],
                    &self.account_id[self.account_id.len() - 4..]
                )
            } else {
                "***".to_string()
            }
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

        let response_result = self
            .http_client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&query)
            .send()
            .await;

        let response = match response_result {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to reach Cloudflare API: {}", e);
                // Return cached data if available
                let cache_read = self.cache.read().await;
                if let Some((_, ref data)) = *cache_read {
                    info!("Serving stale cached analytics after connection error");
                    return Ok(data.clone());
                }
                return Err(KusanagiError::external_service(format!("Cloudflare API error: {}", e)));
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            error!("Cloudflare API error {}: {}", status, text);

            // If 429 Too Many Requests or 5xx, set a 15-minute cooldown to avoid hammering Cloudflare
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
                let mut cooldown_write = self.cooldown_until.write().await;
                *cooldown_write = Some(Instant::now() + Duration::from_secs(900));
                info!("Entering 15-minute Cloudflare cooldown due to HTTP {}", status);
            }

            // If we have stale cache, serve it gracefully
            let cache_read = self.cache.read().await;
            if let Some((_, ref data)) = *cache_read {
                info!("Serving stale cached analytics after Cloudflare HTTP {}", status);
                return Ok(data.clone());
            }

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

        let overview = BusinessOverview {
            cloudflare: CloudflareAnalytics {
                requests: total_requests,
                bandwidth: total_bytes,
                threats: total_threats,
                page_views: total_page_views,
            },
        };

        // Cache the successful result
        {
            let mut cache_write = self.cache.write().await;
            *cache_write = Some((Instant::now(), overview.clone()));
        }

        Ok(overview)
    }
}
