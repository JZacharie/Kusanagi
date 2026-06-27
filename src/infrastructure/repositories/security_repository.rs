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
use k8s_openapi::api::batch::v1::{CronJob, Job};
use kube::{
    api::{Api, PostParams},
    Client,
};
use serde::Deserialize;
use std::env;
use std::sync::Arc;
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
    kube_client: Option<Arc<Client>>,
    trivy_server_url: String,
    bucket_name: String,
}

impl SecurityRepositoryImpl {
    /// Create a new repository instance
    pub async fn new() -> Self {
        let trivy_server_url =
            env::var("TRIVY_SERVER_URL") // matches .env key
                .or_else(|_| env::var("TRIVY_JSON_SERVER")) // legacy fallback
                .unwrap_or_else(|_| DEFAULT_TRIVY_JSON_SERVER.to_string());

        let bucket_name =
            env::var("SECURITY_S3_BUCKET").unwrap_or_else(|_| DEFAULT_BUCKET_NAME.to_string());

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        let s3_client = Self::init_s3_client().await.ok();
        let kube_client = Client::try_default().await.ok().map(Arc::new);

        Self {
            http_client,
            s3_client,
            kube_client,
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

    /// Count vulnerabilities in a report — handles both real Trivy JSON (lowercase)
    /// and the legacy PascalCase format.
    fn count_vulnerabilities(
        &self,
        report: &serde_json::Value,
    ) -> (usize, usize, usize, usize, usize) {
        // Real Trivy JSON: { "report": { "vulnerabilities": [...] } }
        // Legacy format:   { "Report": { "Vulnerabilities": [...] } }
        let vulns = report["report"]["vulnerabilities"]
            .as_array()
            .or_else(|| report["Report"]["Vulnerabilities"].as_array());

        let mut total = 0;
        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;

        if let Some(vuln_list) = vulns {
            total = vuln_list.len();
            for vuln in vuln_list {
                // Trivy uses "severity", legacy uses "Severity"
                let severity = vuln["severity"]
                    .as_str()
                    .or_else(|| vuln["Severity"].as_str())
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
        } else {
            // Try summary block: { "report": { "summary": { "criticalCount": N, ... } } }
            let summary = &report["report"]["summary"];
            if !summary.is_null() {
                critical = summary["criticalCount"].as_u64().unwrap_or(0) as usize;
                high = summary["highCount"].as_u64().unwrap_or(0) as usize;
                medium = summary["mediumCount"].as_u64().unwrap_or(0) as usize;
                low = summary["lowCount"].as_u64().unwrap_or(0) as usize;
                total = critical + high + medium + low;
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

                    // Only use S3 if it actually has cached reports
                    if !keys.is_empty() {
                        let fetch_futures = keys.iter().map(|key| self.fetch_from_s3(key));
                        let reports = futures::future::join_all(fetch_futures).await;

                        let mut total_vulns = 0;
                        let mut critical = 0;
                        let mut high = 0;
                        let mut medium = 0;
                        let mut low = 0;

                        for report in reports.into_iter().flatten() {
                            let (t, c, h, m, l) = self.count_vulnerabilities(&report.original_data);
                            total_vulns += t;
                            critical += c;
                            high += h;
                            medium += m;
                            low += l;
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
                    // else: S3 empty, fall through to Trivy
                }
                Err(e) => {
                    warn!(
                        "Failed to list S3 objects: {}, falling back to Trivy server",
                        e
                    );
                }
            }
        }

        // Fallback: fetch from Trivy server — only process vulnerability_reports
        let report_list = match self.fetch_report_list().await {
            Ok(list) => list,
            Err(e) => {
                warn!("Trivy server unavailable: {}, returning empty summary", e);
                return Ok(SecuritySummary {
                    total_reports: 0,
                    total_vulnerabilities: 0,
                    critical_count: 0,
                    high_count: 0,
                    medium_count: 0,
                    low_count: 0,
                    reports: vec![],
                    last_updated: chrono::Utc::now().to_rfc3339(),
                });
            }
        };

        // Only vulnerability_reports have CVE data; other types (sbom, config_audit, rbac…) are skipped
        let vuln_reports: Vec<(String, Vec<String>)> = report_list
            .into_iter()
            .filter(|(cat, _)| cat == "vulnerability_reports")
            .collect();

        let total_reports: usize = vuln_reports.iter().map(|(_, files)| files.len()).sum();
        let report_keys: Vec<String> = vuln_reports
            .iter()
            .flat_map(|(cat, files)| files.iter().map(move |f| format!("{}/{}", cat, f)))
            .collect();

        // Fetch each vulnerability report and count
        let mut total_vulns = 0usize;
        let mut critical = 0usize;
        let mut high = 0usize;
        let mut medium = 0usize;
        let mut low = 0usize;

        for (cat, files) in &vuln_reports {
            for file in files.iter().take(100) {
                match self.fetch_raw_report(cat, file).await {
                    Ok(raw) => {
                        let (t, c, h, m, l) = self.count_vulnerabilities(&raw);
                        total_vulns += t;
                        critical += c;
                        high += h;
                        medium += m;
                        low += l;
                    }
                    Err(e) => warn!("Skipping report {}/{}: {}", cat, file, e),
                }
            }
        }

        Ok(SecuritySummary {
            total_reports,
            total_vulnerabilities: total_vulns,
            critical_count: critical,
            high_count: high,
            medium_count: medium,
            low_count: low,
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

        // Fallback to Trivy server — only vulnerability_reports have CVE data
        match self.fetch_report_list().await {
            Ok(report_list) => {
                let keys: Vec<String> = report_list
                    .into_iter()
                    .filter(|(cat, _)| cat == "vulnerability_reports")
                    .flat_map(|(cat, files)| files.into_iter().map(move |f| format!("{}/{}", cat, f)))
                    .collect();
                Ok(keys)
            }
            Err(e) => {
                warn!("Trivy server unavailable: {}, returning empty report list", e);
                Ok(vec![])
            }
        }
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
        let raw_report = match self.fetch_raw_report(category, name).await {
            Ok(report) => report,
            Err(e) => {
                warn!("Failed to fetch report {}/{}: {}, returning empty", category, name, e);
                serde_json::json!({})
            }
        };

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

    async fn trigger_scan(&self) -> Result<String> {
        let client = self
            .kube_client
            .as_ref()
            .ok_or_else(|| KusanagiError::configuration("Kubernetes client not available"))?;

        let namespace = env::var("TRIVY_NAMESPACE").unwrap_or_else(|_| "trivy-system".to_string());
        let cronjob_name = env::var("TRIVY_RESCAN_CRONJOB").unwrap_or_else(|_| "trivy-rescan-all".to_string());

        let cronjobs_api: Api<CronJob> = Api::namespaced(client.as_ref().clone(), &namespace);

        // Get the CronJob
        let cronjob = match cronjobs_api.get(&cronjob_name).await {
            Ok(cj) => cj,
            Err(e) => {
                let msg = format!("CronJob '{}' not found in namespace '{}': {}. Set TRIVY_NAMESPACE and TRIVY_RESCAN_CRONJOB env vars if deployed elsewhere.", cronjob_name, namespace, e);
                warn!("{}", msg);
                return Err(KusanagiError::external_service(msg));
            }
        };

        // Create a Job from the CronJob template
        let job_spec = cronjob
            .spec
            .job_template
            .spec
            .ok_or_else(|| KusanagiError::configuration("CronJob has no job template"))?;

        let job = Job {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some(format!(
                    "{}-manual-{}",
                    cronjob_name,
                    chrono::Utc::now().timestamp()
                )),
                namespace: Some(namespace.to_string()),
                ..Default::default()
            },
            spec: Some(job_spec),
            status: None,
        };

        let jobs_api: Api<Job> = Api::namespaced(client.as_ref().clone(), &namespace);
        jobs_api
            .create(&PostParams::default(), &job)
            .await
            .map_err(|e| KusanagiError::external_service(format!("Failed to create Job: {}", e)))?;

        info!("Triggered manual Trivy scan via CronJob {}", cronjob_name);
        Ok("Trivy scan triggered successfully".to_string())
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
