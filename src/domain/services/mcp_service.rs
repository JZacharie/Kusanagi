//! MCP Service - Domain Service for Model Context Protocol integrations
//!
//! Handles interactions with various MCP servers and provides unified access
//! to infrastructure data (Kubernetes, Cilium, Steampipe, Trivy).

use crate::domain::entities::mcp::{
    CiliumPolicySummary, K8sResourceSummary, McpConfig, McpRequest, McpResponse,
    OpenObserveQueryResult, PolicyReportOverview, PolicySummary, PolicyViolation,
    SteampipeResult, TrivyVulnerabilitySummary,
};
use tracing::{info, warn};

/// Service for handling MCP integrations
pub struct McpService {
    config: McpConfig,
    http_client: reqwest::Client,
    kube_client: Option<kube::Client>,
    cache: std::sync::Arc<crate::AdvancedCache<String>>,
}

impl McpService {
    /// Create a new MCP service
    pub fn new(
        kube_client: Option<kube::Client>,
        cache: std::sync::Arc<crate::AdvancedCache<String>>,
    ) -> Self {
        let config = McpConfig::default();
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .unwrap_or_default();

        Self {
            config,
            http_client,
            kube_client,
            cache,
        }
    }

    /// Helper to make MCP requests
    async fn request(
        &self,
        url: &str,
        method: &str,
        params: serde_json::Value,
    ) -> Result<McpResponse, String> {
        let request = McpRequest {
            method: method.to_string(),
            params,
        };

        let response = self
            .http_client
            .post(url)
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("MCP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("MCP server returned status: {}", response.status()));
        }

        response
            .json::<McpResponse>()
            .await
            .map_err(|e| format!("Failed to parse MCP response: {}", e))
    }

    // ==================== Kubernetes MCP ====================

    /// Get Kubernetes resource summary via MCP
    pub async fn get_k8s_resources(
        &self,
        namespace: Option<&str>,
    ) -> Result<K8sResourceSummary, String> {
        info!("Fetching K8s resources via MCP");

        let params = serde_json::json!({
            "namespace": namespace.unwrap_or("all")
        });

        match self
            .request(&self.config.kubernetes_url, "list_resources", params)
            .await
        {
            Ok(response) => {
                if response.success {
                    if let Some(data) = response.data {
                        serde_json::from_value(data)
                            .map_err(|e| format!("Failed to parse K8s resources: {}", e))
                    } else {
                        Err("No data in MCP response".to_string())
                    }
                } else {
                    Err(response
                        .error
                        .unwrap_or_else(|| "Unknown MCP error".to_string()))
                }
            }
            Err(e) => {
                warn!("MCP Kubernetes unavailable, using fallback: {}", e);
                // Fallback: return placeholder data
                Ok(K8sResourceSummary {
                    deployments: -1,
                    statefulsets: -1,
                    daemonsets: -1,
                    services: -1,
                    configmaps: -1,
                    secrets: -1,
                })
            }
        }
    }

    // ==================== Cilium MCP ====================

    /// Get Cilium network policies via MCP
    pub async fn get_cilium_policies(
        &self,
        namespace: Option<&str>,
    ) -> Result<CiliumPolicySummary, String> {
        info!("Fetching Cilium policies via MCP");

        let params = serde_json::json!({
            "namespace": namespace.unwrap_or("all")
        });

        match self
            .request(&self.config.cilium_url, "list_policies", params)
            .await
        {
            Ok(response) => {
                if response.success {
                    if let Some(data) = response.data {
                        serde_json::from_value(data)
                            .map_err(|e| format!("Failed to parse Cilium policies: {}", e))
                    } else {
                        Err("No data in MCP response".to_string())
                    }
                } else {
                    Err(response
                        .error
                        .unwrap_or_else(|| "Unknown MCP error".to_string()))
                }
            }
            Err(e) => {
                warn!("MCP Cilium unavailable: {}", e);
                Ok(CiliumPolicySummary {
                    total_policies: 0,
                    policies: vec![],
                })
            }
        }
    }

    // ==================== Steampipe MCP ====================

    /// Execute Steampipe SQL query via MCP
    pub async fn query_steampipe(&self, sql: &str) -> Result<SteampipeResult, String> {
        info!("Executing Steampipe query via MCP: {}", sql);

        // Validate query is read-only (SELECT only)
        let sql_upper = sql.trim().to_uppercase();
        if !sql_upper.starts_with("SELECT") {
            return Err("Only SELECT queries are allowed".to_string());
        }

        let params = serde_json::json!({
            "query": sql
        });

        match self
            .request(&self.config.steampipe_url, "query", params)
            .await
        {
            Ok(response) => {
                if response.success {
                    if let Some(data) = response.data {
                        serde_json::from_value(data)
                            .map_err(|e| format!("Failed to parse Steampipe result: {}", e))
                    } else {
                        Err("No data in MCP response".to_string())
                    }
                } else {
                    Err(response
                        .error
                        .unwrap_or_else(|| "Unknown MCP error".to_string()))
                }
            }
            Err(e) => {
                warn!("MCP Steampipe unavailable: {}", e);
                Err(format!("Steampipe MCP server unavailable: {}", e))
            }
        }
    }

    // ==================== Trivy MCP ====================

    /// Get Trivy vulnerability reports from S3 via MCP
    pub async fn get_trivy_vulnerabilities(&self) -> Result<TrivyVulnerabilitySummary, String> {
        const CACHE_KEY: &str = "mcp_vulnerabilities";

        if let Some(cached) = self.cache.get(CACHE_KEY).await {
            if let Ok(value) = serde_json::from_str::<TrivyVulnerabilitySummary>(&cached) {
                return Ok(value);
            }
        }

        info!("Fetching Trivy vulnerabilities via MCP");

        let params = serde_json::json!({});

        let result = match self
            .request(&self.config.trivy_url, "get_vulnerabilities", params)
            .await
        {
            Ok(response) => {
                if response.success {
                    if let Some(data) = response.data {
                        serde_json::from_value(data)
                            .map_err(|e| format!("Failed to parse Trivy report: {}", e))
                    } else {
                        Err("No data in MCP response".to_string())
                    }
                } else {
                    Err(response
                        .error
                        .unwrap_or_else(|| "Unknown MCP error".to_string()))
                }
            }
            Err(e) => {
                warn!("MCP Trivy unavailable: {}", e);
                return Err(format!("MCP Trivy unavailable: {}", e));
            }
        };

        if let Ok(ref data) = result {
            if let Ok(serialized) = serde_json::to_string(data) {
                self.cache
                    .set(
                        CACHE_KEY.to_string(),
                        serialized,
                        Some(std::time::Duration::from_secs(300)),
                    )
                    .await;
            }
        }

        result
    }

    // ==================== OpenObserve MCP ====================

    /// Query logs from OpenObserve via MCP
    pub async fn query_logs(
        &self,
        query: &str,
        limit: Option<i32>,
    ) -> Result<OpenObserveQueryResult, String> {
        info!("Querying OpenObserve logs via MCP: {}", query);

        let params = serde_json::json!({
            "query": query,
            "limit": limit.unwrap_or(100)
        });

        match self
            .request(&self.config.openobserve_url, "query", params)
            .await
        {
            Ok(response) => {
                if response.success {
                    if let Some(data) = response.data {
                        serde_json::from_value(data)
                            .map_err(|e| format!("Failed to parse OpenObserve result: {}", e))
                    } else {
                        Err("No data in MCP response".to_string())
                    }
                } else {
                    Err(response
                        .error
                        .unwrap_or_else(|| "Unknown MCP error".to_string()))
                }
            }
            Err(e) => {
                warn!("MCP OpenObserve unavailable: {}", e);
                Err(format!("OpenObserve MCP server unavailable: {}", e))
            }
        }
    }

    // ==================== Kyverno Policy Reports ====================

    /// Get Kyverno Policy Reports (Direct K8s access, not URL based MCP)
    pub async fn get_policy_violations(&self) -> Result<PolicyReportOverview, String> {
        // If no kube client, return empty result or error
        let client = match &self.kube_client {
            Some(c) => c,
            None => return Err("Kubernetes client not available for policy checks".to_string()),
        };

        info!("Fetching Kyverno Policy Reports");

        let mut violations = Vec::new();
        let mut total_summary = PolicySummary {
            pass: 0,
            fail: 0,
            warn: 0,
            error: 0,
            skip: 0,
        };

        // Fetch ClusterPolicyReports
        let dynamic_cpr = kube::api::ApiResource {
            group: "wgpolicyk8s.io".to_string(),
            version: "v1alpha2".to_string(),
            api_version: "wgpolicyk8s.io/v1alpha2".to_string(),
            kind: "ClusterPolicyReport".to_string(),
            plural: "clusterpolicyreports".to_string(),
        };

        let cpr_api: kube::Api<kube::api::DynamicObject> =
            kube::Api::all_with(client.clone(), &dynamic_cpr);

        match cpr_api.list(&kube::api::ListParams::default()).await {
            Ok(list) => {
                for item in list.items {
                    self.process_report_results(
                        &item,
                        &mut violations,
                        &mut total_summary,
                        "cluster-wide",
                    );
                }
            }
            Err(e) => warn!("Failed to list ClusterPolicyReports: {}", e),
        }

        // Fetch namespaced PolicyReports
        let dynamic_pr = kube::api::ApiResource {
            group: "wgpolicyk8s.io".to_string(),
            version: "v1alpha2".to_string(),
            api_version: "wgpolicyk8s.io/v1alpha2".to_string(),
            kind: "PolicyReport".to_string(),
            plural: "policyreports".to_string(),
        };

        let pr_api: kube::Api<kube::api::DynamicObject> =
            kube::Api::all_with(client.clone(), &dynamic_pr);

        match pr_api.list(&kube::api::ListParams::default()).await {
            Ok(list) => {
                for item in list.items {
                    let ns = item
                        .metadata
                        .namespace
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string());
                    self.process_report_results(&item, &mut violations, &mut total_summary, &ns);
                }
            }
            Err(e) => warn!("Failed to list PolicyReports: {}", e),
        }

        Ok(PolicyReportOverview {
            total_violations: violations.len() as i32,
            violations,
            summary: total_summary,
        })
    }

    fn process_report_results(
        &self,
        item: &kube::api::DynamicObject,
        violations: &mut Vec<PolicyViolation>,
        total_summary: &mut PolicySummary,
        namespace: &str,
    ) {
        if let Some(obj) = item.data.as_object() {
            if let Some(summary) = obj
                .get("summary")
                .and_then(|s| serde_json::from_value::<PolicySummary>(s.clone()).ok())
            {
                total_summary.pass += summary.pass;
                total_summary.fail += summary.fail;
                total_summary.warn += summary.warn;
                total_summary.error += summary.error;
                total_summary.skip += summary.skip;
            }

            if let Some(results) = obj.get("results").and_then(|r| r.as_array()) {
                for res in results {
                    if res["result"] == "fail" || res["result"] == "warn" {
                        violations.push(PolicyViolation {
                            policy: res["policy"].as_str().unwrap_or("unknown").to_string(),
                            rule: res["rule"].as_str().unwrap_or("-").to_string(),
                            resource: item.metadata.name.clone().unwrap_or_default(),
                            namespace: namespace.to_string(),
                            message: res["message"].as_str().unwrap_or("-").to_string(),
                            severity: res["severity"].as_str().unwrap_or("medium").to_string(),
                            result: res["result"].as_str().unwrap_or("unknown").to_string(),
                            timestamp: res["timestamp"]["seconds"]
                                .as_i64()
                                .map(|s| s.to_string())
                                .unwrap_or_default(),
                        });
                    }
                }
            }
        }
    }

    // ==================== Fence Status ====================

    /// Check status of Fence pods in security namespace
    pub async fn get_fence_status(&self) -> Result<serde_json::Value, String> {
        let client = match &self.kube_client {
            Some(c) => c,
            None => return Err("Kubernetes client not available".to_string()),
        };

        let pods: kube::Api<k8s_openapi::api::core::v1::Pod> =
            kube::Api::namespaced(client.clone(), "security");

        match pods
            .list(&kube::api::ListParams::default().labels("app.kubernetes.io/name=fence"))
            .await
        {
            Ok(list) => {
                let running = list.items.iter().any(|p| {
                    p.status
                        .as_ref()
                        .map(|s| s.phase.as_deref() == Some("Running"))
                        .unwrap_or(false)
                });

                Ok(serde_json::json!({
                    "name": "Fence",
                    "status": if running { "healthy" } else { "unhealthy" },
                    "namespace": "security",
                    "pods": list.items.len()
                }))
            }
            Err(e) => Err(format!("Failed to list Fence pods: {}", e)),
        }
    }
}
