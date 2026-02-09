//! Security Repository Implementation
//!
//! Infrastructure adapter implementing the SecurityRepository port.
//! Handles Trivy JSON server API calls and S3 caching for security reports.

use crate::domain::entities::{EnrichmentData, SecurityReport, SecuritySummary};
use crate::domain::ports::SecurityRepository;
use crate::error::{KusanagiError, Result};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::Client as S3Client;
use serde::Deserialize;
use std::env;
use tracing::{debug, error, info, warn};

const DEFAULT_TRIVY_JSON_SERVER: &str = "http://trivy-json-server.trivy-system.svc:8080";
const DEFAULT_S3_ENDPOINT: &str = "http://192.168.0.170:9010";
const DEFAULT_BUCKET_NAME: &str = "kusanagi-security-reports";
const S3_REGION: &str = "us-east-1";

/// Index entry from Trivy JSON server
#[derive(Debug, Deserialize)]
struct IndexEntry {
    name: String,
    #[serde(rename = "type")]
    entry_type: String,
}

/// Security repository implementation
pub struct SecurityRepositoryImpl {
    http_client: reqwest::Client,
    s3_client: Option<S3Client>,
    trivy_server_url: String,
    bucket_name: String,
}

impl SecurityRepositoryImpl {
    /// Create a new repository instance
    pub async fn new() -> Self {
        let trivy_server_url =
            env::var("TRIVY_JSON_SERVER").unwrap_or_else(|_| DEFAULT_TRIVY_JSON_SERVER.to_string());

        let bucket_name =
            env::var("SECURITY_S3_BUCKET").unwrap_or_else(|_| DEFAULT_BUCKET_NAME.to_string());

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let s3_client = Self::init_s3_client().await.ok();

        Self {
            http_client,
            s3_client,
            trivy_server_url,
            bucket_name,
        }
    }

    /// Initialize S3 client for caching
    async fn init_s3_client() -> Result<S3Client> {
        let endpoint = env::var("S3_ENDPOINT").unwrap_or_else(|_| DEFAULT_S3_ENDPOINT.to_string());

        let config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(S3_REGION))
            .endpoint_url(&endpoint)
            .load()
            .await;

        let s3_config = aws_sdk_s3::config::Builder::from(&config)
            .force_path_style(true)
            .build();

        Ok(S3Client::from_conf(s3_config))
    }

    /// Check if running in local mode
    fn check_local_mode(&self) -> bool {
        env::var("KUSANAGI_MODE").unwrap_or_default() == "local"
    }

    /// Fetch report list from Trivy JSON server
    async fn fetch_report_list(&self) -> Result<Vec<(String, Vec<String>)>> {
        let url = format!("{}/", self.trivy_server_url);

        debug!("Fetching security report list from Trivy server");

        let categories: Vec<IndexEntry> = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| KusanagiError::external_service(format!("Trivy server error: {}", e)))?
            .json()
            .await
            .map_err(|e| KusanagiError::serialization(format!("Failed to parse index: {}", e)))?;

        let mut result = Vec::new();

        for cat in categories.iter().filter(|c| c.entry_type == "directory") {
            let cat_url = format!("{}/{}", self.trivy_server_url, cat.name);

            let files: Vec<IndexEntry> = self
                .http_client
                .get(&cat_url)
                .send()
                .await
                .map_err(|e| {
                    KusanagiError::external_service(format!(
                        "Trivy server error for {}: {}",
                        cat.name, e
                    ))
                })?
                .json()
                .await
                .map_err(|e| {
                    KusanagiError::serialization(format!(
                        "Failed to parse {} index: {}",
                        cat.name, e
                    ))
                })?;

            let report_files: Vec<String> = files
                .iter()
                .filter(|f| f.name.ends_with(".json"))
                .map(|f| f.name.clone())
                .collect();

            if !report_files.is_empty() {
                result.push((cat.name.clone(), report_files));
            }
        }

        Ok(result)
    }

    /// Fetch raw report from Trivy JSON server
    async fn fetch_raw_report(&self, category: &str, name: &str) -> Result<serde_json::Value> {
        let url = format!("{}/{}/{}", self.trivy_server_url, category, name);

        debug!("Fetching security report: {}/{}", category, name);

        let report: serde_json::Value = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| {
                KusanagiError::external_service(format!(
                    "Failed to fetch report {}/{}: {}",
                    category, name, e
                ))
            })?
            .json()
            .await
            .map_err(|e| {
                KusanagiError::serialization(format!(
                    "Failed to parse report {}/{}: {}",
                    category, name, e
                ))
            })?;

        Ok(report)
    }

    /// Count vulnerabilities in a report
    fn count_vulnerabilities(
        &self,
        report: &serde_json::Value,
    ) -> (usize, usize, usize, usize, usize) {
        let vulns = report["Report"]["Vulnerabilities"].as_array();

        let mut total = 0;
        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        if let Some(vuln_list) = vulns {
            total = vuln_list.len();
            for vuln in vuln_list {
                let severity = vuln["Severity"]
                    .as_str()
                    .or_else(|| vuln["severity"].as_str())
                    .unwrap_or("UNKNOWN")
                    .to_lowercase();

                match severity.as_str() {
                    "critical" => critical += 1,
                    "high" => high += 1,
                    "medium" => medium += 1,
                    "low" => low += 1,
                    _ => {}
                }
            }
        }

        (total, critical, high, medium, low)
    }

    /// Fetch report from S3 cache
    async fn fetch_from_s3(&self, key: &str) -> Option<SecurityReport> {
        let client = self.s3_client.as_ref()?;

        match client
            .get_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await
        {
            Ok(resp) => {
                let data = resp.body.collect().await.ok()?.into_bytes();
                match serde_json::from_slice::<SecurityReport>(&data) {
                    Ok(report) => {
                        debug!("Successfully loaded report from S3: {}", key);
                        Some(report)
                    }
                    Err(e) => {
                        error!("Failed to parse S3 report {}: {}", key, e);
                        None
                    }
                }
            }
            Err(e) => {
                debug!("Failed to fetch report from S3 {}: {}", key, e);
                None
            }
        }
    }

    /// Create mock security summary for local mode
    fn create_mock_summary(&self) -> SecuritySummary {
        SecuritySummary {
            total_reports: 3,
            total_vulnerabilities: 5,
            critical_count: 0,
            high_count: 1,
            medium_count: 2,
            low_count: 2,
            reports: vec![
                "cluster/report1.json".to_string(),
                "cluster/report2.json".to_string(),
                "apps/app-report.json".to_string(),
            ],
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create mock report for local mode
    fn create_mock_report(&self, category: &str, name: &str) -> SecurityReport {
        let mock_data = serde_json::json!({
            "Report": {
                "Vulnerabilities": [
                    {
                        "VulnerabilityID": "CVE-2024-0001",
                        "Title": "Mock vulnerability",
                        "Severity": "MEDIUM"
                    }
                ]
            }
        });

        SecurityReport {
            name: name.to_string(),
            report_type: category.to_string(),
            original_data: mock_data,
            enrichment: Some(EnrichmentData {
                summary: "Mock security report (local mode)".to_string(),
                remediation_advice: "No action required in local mode".to_string(),
                criticality_score: 3.0,
            }),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

#[async_trait]
impl SecurityRepository for SecurityRepositoryImpl {
    async fn get_security_summary(&self) -> Result<SecuritySummary> {
        // Check local mode
        if self.check_local_mode() {
            info!("Security repository running in local mode, returning mock summary");
            return Ok(self.create_mock_summary());
        }

        // Try to get from S3 first (cached enriched reports)
        if let Some(s3_client) = &self.s3_client {
            match s3_client
                .list_objects_v2()
                .bucket(&self.bucket_name)
                .send()
                .await
            {
                Ok(output) => {
                    let keys: Vec<String> = output
                        .contents
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|o| o.key)
                        .collect();

                    let mut total_vulns = 0;
                    let mut critical = 0;
                    let mut high = 0;
                    let mut medium = 0;
                    let mut low = 0;

                    for key in &keys {
                        if let Some(report) = self.fetch_from_s3(key).await {
                            let (t, c, h, m, l) = self.count_vulnerabilities(&report.original_data);
                            total_vulns += t;
                            critical += c;
                            high += h;
                            medium += m;
                            low += l;
                        }
                    }

                    return Ok(SecuritySummary {
                        total_reports: keys.len(),
                        total_vulnerabilities: total_vulns,
                        critical_count: critical,
                        high_count: high,
                        medium_count: medium,
                        low_count: low,
                        reports: keys,
                        last_updated: chrono::Utc::now().to_rfc3339(),
                    });
                }
                Err(e) => {
                    warn!(
                        "Failed to list S3 objects: {}, falling back to Trivy server",
                        e
                    );
                }
            }
        }

        // Fallback: fetch from Trivy server
        let report_list = self.fetch_report_list().await?;
        let total_reports: usize = report_list.iter().map(|(_, files)| files.len()).sum();
        let report_keys: Vec<String> = report_list
            .iter()
            .flat_map(|(cat, files)| files.iter().map(move |f| format!("{}/{}", cat, f)))
            .collect();

        Ok(SecuritySummary {
            total_reports,
            total_vulnerabilities: 0,
            critical_count: 0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            reports: report_keys,
            last_updated: chrono::Utc::now().to_rfc3339(),
        })
    }

    async fn get_security_reports(&self) -> Result<Vec<String>> {
        // Check local mode
        if self.check_local_mode() {
            return Ok(vec![
                "cluster/report1.json".to_string(),
                "cluster/report2.json".to_string(),
                "apps/app-report.json".to_string(),
            ]);
        }

        // Try S3 first
        if let Some(s3_client) = &self.s3_client {
            match s3_client
                .list_objects_v2()
                .bucket(&self.bucket_name)
                .send()
                .await
            {
                Ok(output) => {
                    let keys: Vec<String> = output
                        .contents
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|o| o.key)
                        .collect();
                    if !keys.is_empty() {
                        return Ok(keys);
                    }
                }
                Err(e) => {
                    debug!("Failed to list S3 objects: {}", e);
                }
            }
        }

        // Fallback to Trivy server
        let report_list = self.fetch_report_list().await?;
        let keys: Vec<String> = report_list
            .iter()
            .flat_map(|(cat, files)| files.iter().map(move |f| format!("{}/{}", cat, f)))
            .collect();

        Ok(keys)
    }

    async fn get_security_report(&self, category: &str, name: &str) -> Result<SecurityReport> {
        // Check local mode
        if self.check_local_mode() {
            return Ok(self.create_mock_report(category, name));
        }

        let key = format!("{}/{}", category, name);

        // Try S3 first (enriched reports)
        if let Some(report) = self.fetch_from_s3(&key).await {
            return Ok(report);
        }

        // Fallback: fetch from Trivy server
        let raw_report = self.fetch_raw_report(category, name).await?;

        Ok(SecurityReport {
            name: name.to_string(),
            report_type: category.to_string(),
            original_data: raw_report,
            enrichment: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        })
    }

    fn is_local_mode(&self) -> bool {
        self.check_local_mode()
    }
}

impl Default for SecurityRepositoryImpl {
    fn default() -> Self {
        // Note: This will panic if called outside of tokio runtime
        // Use SecurityRepositoryImpl::new().await instead
        panic!("Use SecurityRepositoryImpl::new().await instead of default()");
    }
}

/// Factory function for creating security repository
pub async fn create_security_repository() -> Box<dyn SecurityRepository> {
    Box::new(SecurityRepositoryImpl::new().await)
}
