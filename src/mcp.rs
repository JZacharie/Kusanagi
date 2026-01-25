//! MCP (Model Context Protocol) integrations for Kusanagi
//! Provides access to various infrastructure tools via MCP servers

use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

/// MCP Server endpoints (configurable via env vars)
fn get_mcp_kubernetes_url() -> String {
    std::env::var("MCP_KUBERNETES_URL").unwrap_or_else(|_| "http://localhost:3000/mcp/kubernetes".to_string())
}
fn get_mcp_cilium_url() -> String {
    std::env::var("MCP_CILIUM_URL").unwrap_or_else(|_| "http://localhost:3000/mcp/cilium".to_string())
}
fn get_mcp_steampipe_url() -> String {
    std::env::var("MCP_STEAMPIPE_URL").unwrap_or_else(|_| "http://localhost:3000/mcp/steampipe".to_string())
}
fn get_mcp_trivy_url() -> String {
    std::env::var("MCP_TRIVY_URL").unwrap_or_else(|_| "http://localhost:3000/mcp/trivy".to_string())
}

/// MCP Request structure
#[derive(Serialize)]
pub struct McpRequest {
    pub method: String,
    pub params: serde_json::Value,
}

/// MCP Response structure
#[derive(Deserialize, Debug)]
pub struct McpResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Kubernetes resource summary from MCP
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct K8sResourceSummary {
    pub deployments: i32,
    pub statefulsets: i32,
    pub daemonsets: i32,
    pub services: i32,
    pub configmaps: i32,
    pub secrets: i32,
}

/// Cilium network policy summary
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CiliumPolicySummary {
    pub total_policies: i32,
    pub policies: Vec<CiliumPolicy>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CiliumPolicy {
    pub name: String,
    pub namespace: String,
    pub endpoints_matched: i32,
    pub ingress_rules: i32,
    pub egress_rules: i32,
}

/// Steampipe query result
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SteampipeResult {
    pub query: String,
    pub rows: Vec<serde_json::Value>,
    pub columns: Vec<String>,
    pub row_count: i32,
}

/// Trivy vulnerability summary
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrivyVulnerabilitySummary {
    pub total_images: i32,
    pub critical: i32,
    pub high: i32,
    pub medium: i32,
    pub low: i32,
    pub images: Vec<TrivyImageReport>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrivyImageReport {
    pub image: String,
    pub namespace: String,
    pub critical_count: i32,
    pub high_count: i32,
    pub last_scan: String,
}

/// Kyverno Policy Report structs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolicyViolation {
    pub policy: String,
    pub rule: String,
    pub resource: String,
    pub namespace: String,
    pub message: String,
    pub severity: String,
    pub result: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolicySummary {
    pub pass: i32,
    pub fail: i32,
    pub warn: i32,
    pub error: i32,
    pub skip: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PolicyReportOverview {
    pub total_violations: i32,
    pub violations: Vec<PolicyViolation>,
    pub summary: PolicySummary,
}

/// HTTP client helper for MCP requests
async fn mcp_request(url: &str, method: &str, params: serde_json::Value) -> Result<McpResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let request = McpRequest {
        method: method.to_string(),
        params,
    };

    let response = client
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

// ============================================================================
// Kubernetes MCP Integration
// ============================================================================

/// Get Kubernetes resource summary via MCP
pub async fn get_k8s_resources(namespace: Option<&str>) -> Result<K8sResourceSummary, String> {
    info!("Fetching K8s resources via MCP");
    
    let params = serde_json::json!({
        "namespace": namespace.unwrap_or("all")
    });

    match mcp_request(&get_mcp_kubernetes_url(), "list_resources", params).await {
        Ok(response) => {
            if response.success {
                if let Some(data) = response.data {
                    serde_json::from_value(data)
                        .map_err(|e| format!("Failed to parse K8s resources: {}", e))
                } else {
                    Err("No data in MCP response".to_string())
                }
            } else {
                Err(response.error.unwrap_or_else(|| "Unknown MCP error".to_string()))
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

// ============================================================================
// Cilium MCP Integration
// ============================================================================

/// Get Cilium network policies via MCP
pub async fn get_cilium_policies(namespace: Option<&str>) -> Result<CiliumPolicySummary, String> {
    info!("Fetching Cilium policies via MCP");

    let params = serde_json::json!({
        "namespace": namespace.unwrap_or("all")
    });

    match mcp_request(&get_mcp_cilium_url(), "list_policies", params).await {
        Ok(response) => {
            if response.success {
                if let Some(data) = response.data {
                    serde_json::from_value(data)
                        .map_err(|e| format!("Failed to parse Cilium policies: {}", e))
                } else {
                    Err("No data in MCP response".to_string())
                }
            } else {
                Err(response.error.unwrap_or_else(|| "Unknown MCP error".to_string()))
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

// ============================================================================
// Steampipe MCP Integration
// ============================================================================

/// Execute Steampipe SQL query via MCP
pub async fn query_steampipe(sql: &str) -> Result<SteampipeResult, String> {
    info!("Executing Steampipe query via MCP: {}", sql);

    // Validate query is read-only (SELECT only)
    let sql_upper = sql.trim().to_uppercase();
    if !sql_upper.starts_with("SELECT") {
        return Err("Only SELECT queries are allowed".to_string());
    }

    let params = serde_json::json!({
        "query": sql
    });

    match mcp_request(&get_mcp_steampipe_url(), "query", params).await {
        Ok(response) => {
            if response.success {
                if let Some(data) = response.data {
                    serde_json::from_value(data)
                        .map_err(|e| format!("Failed to parse Steampipe result: {}", e))
                } else {
                    Err("No data in MCP response".to_string())
                }
            } else {
                Err(response.error.unwrap_or_else(|| "Unknown MCP error".to_string()))
            }
        }
        Err(e) => {
            warn!("MCP Steampipe unavailable: {}", e);
            Err(format!("Steampipe MCP server unavailable: {}", e))
        }
    }
}

// ============================================================================
// Trivy MCP Integration (S3 based)
// ============================================================================

/// Get Trivy vulnerability reports from S3 via MCP
pub async fn get_trivy_vulnerabilities() -> Result<TrivyVulnerabilitySummary, String> {
    info!("Fetching Trivy vulnerabilities via MCP");

    let params = serde_json::json!({});

    match mcp_request(&get_mcp_trivy_url(), "get_vulnerabilities", params).await {
        Ok(response) => {
            if response.success {
                if let Some(data) = response.data {
                    serde_json::from_value(data)
                        .map_err(|e| format!("Failed to parse Trivy report: {}", e))
                } else {
                    Err("No data in MCP response".to_string())
                }
            } else {
                Err(response.error.unwrap_or_else(|| "Unknown MCP error".to_string()))
            }
        }
        Err(e) => {
            warn!("MCP Trivy unavailable: {}", e);
            Ok(TrivyVulnerabilitySummary {
                total_images: 0,
                critical: 0,
                high: 0,
                medium: 0,
                low: 0,
                images: vec![],
            })
        }
    }
}


// ============================================================================
// Kyverno Policy Integration
// ============================================================================

pub async fn get_policy_violations(client: &kube::Client) -> Result<PolicyReportOverview, String> {
    info!("Fetching Kyverno Policy Reports");

    let mut violations = Vec::new();
    let mut total_summary = PolicySummary { pass: 0, fail: 0, warn: 0, error: 0, skip: 0 };

    // Fetch ClusterPolicyReports
    let dynamic_cpr = kube::api::ApiResource {
        group: "wgpolicyk8s.io".to_string(),
        version: "v1alpha2".to_string(),
        api_version: "wgpolicyk8s.io/v1alpha2".to_string(),
        kind: "ClusterPolicyReport".to_string(),
        plural: "clusterpolicyreports".to_string(),
    };

    let cpr_api: kube::Api<kube::api::DynamicObject> = kube::Api::all_with(client.clone(), &dynamic_cpr);

    match cpr_api.list(&kube::api::ListParams::default()).await {
        Ok(list) => {
            for item in list.items {
                process_report_results(&item, &mut violations, &mut total_summary, "cluster-wide");
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

    let pr_api: kube::Api<kube::api::DynamicObject> = kube::Api::all_with(client.clone(), &dynamic_pr);

    match pr_api.list(&kube::api::ListParams::default()).await {
        Ok(list) => {
            for item in list.items {
                let ns = item.metadata.namespace.clone().unwrap_or_else(|| "unknown".to_string());
                process_report_results(&item, &mut violations, &mut total_summary, &ns);
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

fn process_report_results(item: &kube::api::DynamicObject, violations: &mut Vec<PolicyViolation>, total_summary: &mut PolicySummary, namespace: &str) {
    if let Some(obj) = item.data.as_object() {
        if let Some(summary) = obj.get("summary").and_then(|s| serde_json::from_value::<PolicySummary>(s.clone()).ok()) {
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
                        timestamp: res["timestamp"]["seconds"].as_i64().map(|s| s.to_string()).unwrap_or_default(),
                    });
                }
            }
        }
    }
}

// ============================================================================
// Chat command handlers for MCP
// ============================================================================

/// Format K8s resources for chat response
pub fn format_k8s_resources(resources: &K8sResourceSummary) -> String {
    format!(
        r#"## 📦 Kubernetes Resources

| Resource | Count |
|----------|-------|
| Deployments | {} |
| StatefulSets | {} |
| DaemonSets | {} |
| Services | {} |
| ConfigMaps | {} |
| Secrets | {} |"#,
        resources.deployments,
        resources.statefulsets,
        resources.daemonsets,
        resources.services,
        resources.configmaps,
        resources.secrets
    )
}

/// Format Cilium policies for chat response
pub fn format_cilium_policies(summary: &CiliumPolicySummary) -> String {
    if summary.policies.is_empty() {
        return "## 🛡️ Cilium Policies\n\nNo network policies found.".to_string();
    }

    let mut lines = vec![format!(
        "## 🛡️ Cilium Network Policies\n\n**Total:** {} policies\n",
        summary.total_policies
    )];

    for policy in summary.policies.iter().take(10) {
        lines.push(format!(
            "- `{}` ({}) | {} endpoints | {} ingress, {} egress rules",
            policy.name,
            policy.namespace,
            policy.endpoints_matched,
            policy.ingress_rules,
            policy.egress_rules
        ));
    }

    lines.join("\n")
}

/// Format Trivy vulnerabilities for chat response
pub fn format_trivy_vulnerabilities(summary: &TrivyVulnerabilitySummary) -> String {
    let mut lines = vec![format!(
        r#"## 🔍 Security Vulnerabilities (Trivy)

**Images Scanned:** {}

| Severity | Count |
|----------|-------|
| 🔴 Critical | {} |
| 🟠 High | {} |
| 🟡 Medium | {} |
| 🟢 Low | {} |
"#,
        summary.total_images,
        summary.critical,
        summary.high,
        summary.medium,
        summary.low
    )];

    if !summary.images.is_empty() {
        lines.push("**Images with Critical Vulnerabilities:**\n".to_string());
        for img in summary.images.iter().filter(|i| i.critical_count > 0).take(5) {
            lines.push(format!(
                "- `{}` ({}) - {} critical, {} high",
                img.image.chars().take(40).collect::<String>(),
                img.namespace,
                img.critical_count,
                img.high_count
            ));
        }
    }

    lines.join("\n")
}

/// Format Steampipe result for chat response  
pub fn format_steampipe_result(result: &SteampipeResult) -> String {
    if result.rows.is_empty() {
        return format!("## 📊 Query Result\n\n```sql\n{}\n```\n\nNo results found.", result.query);
    }

    let mut lines = vec![format!(
        "## 📊 Query Result\n\n```sql\n{}\n```\n\n**Rows:** {}\n",
        result.query, result.row_count
    )];

    // Build table header
    let header = result.columns.join(" | ");
    let separator = result.columns.iter().map(|_| "---").collect::<Vec<_>>().join(" | ");
    lines.push(format!("| {} |", header));
    lines.push(format!("| {} |", separator));

    // Build table rows (first 10)
    for row in result.rows.iter().take(10) {
        if let Some(obj) = row.as_object() {
            let cells: Vec<String> = result.columns.iter()
                .map(|col| {
                    obj.get(col)
                        .map(|v| v.to_string().replace('"', "").chars().take(30).collect())
                        .unwrap_or_else(|| "-".to_string())
                })
                .collect();
            lines.push(format!("| {} |", cells.join(" | ")));
        }
    }

    if result.row_count > 10 {
        lines.push(format!("\n... and {} more rows", result.row_count - 10));
    }

    lines.join("\n")
}

// ============================================================================
// API Handlers
// ============================================================================

pub async fn get_vulnerabilities_handler() -> Result<HttpResponse> {
    match get_trivy_vulnerabilities().await {
        Ok(summary) => Ok(HttpResponse::Ok().json(summary)),
        Err(e) => {
            error!("Failed to get vulnerabilities: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })))
        }
    }
}

pub async fn get_policies_handler() -> Result<HttpResponse> {
    match get_cilium_policies(None).await {
        Ok(summary) => Ok(HttpResponse::Ok().json(summary)),
        Err(e) => {
            error!("Failed to get policies: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })))
        }
    }
}

pub async fn get_policy_violations_handler(client: web::Data<crate::AppState>) -> Result<HttpResponse> {
    match get_policy_violations(&client.client).await {
        Ok(overview) => Ok(HttpResponse::Ok().json(overview)),
        Err(e) => {
            error!("Failed to get policy violations: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({ "error": e })))
        }
    }
}

pub async fn get_fence_status_handler(client: web::Data<crate::AppState>) -> Result<HttpResponse> {
    // Check if fence pods are running in the security namespace
    match crate::pods::get_pods_status(&client.client).await {
        Ok(_) => {
            // Simplified: we'll just check if any pods are in the security namespace
            // A better way would be to specifically look for "fence" pods
            let pods = kube::Api::<k8s_openapi::api::core::v1::Pod>::namespaced(client.client.clone(), "security");
            match pods.list(&kube::api::ListParams::default().labels("app.kubernetes.io/name=fence")).await {
                Ok(list) => {
                    let running = list.items.iter().any(|p| {
                        p.status.as_ref().map(|s| s.phase.as_deref() == Some("Running")).unwrap_or(false)
                    });
                    Ok(HttpResponse::Ok().json(serde_json::json!({
                        "name": "Fence",
                        "status": if running { "healthy" } else { "unhealthy" },
                        "namespace": "security",
                        "pods": list.items.len()
                    })))
                }
                Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })))
            }
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({ "error": e.to_string() })))
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/security")
            .route("/vulnerabilities", web::get().to(get_vulnerabilities_handler))
            .route("/policies", web::get().to(get_policies_handler))
            .route("/policies/violations", web::get().to(get_policy_violations_handler))
            .route("/fence", web::get().to(get_fence_status_handler)),
    );
}
