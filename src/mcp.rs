//! MCP (Model Context Protocol) integrations for Kusanagi
//! Provides access to various infrastructure tools via MCP servers

use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

/// MCP Server endpoints (configurable via env vars)
const MCP_KUBERNETES_URL: &str = "http://localhost:3000/mcp/kubernetes";
const MCP_CILIUM_URL: &str = "http://localhost:3000/mcp/cilium";
const MCP_STEAMPIPE_URL: &str = "http://localhost:3000/mcp/steampipe";
const MCP_TRIVY_URL: &str = "http://localhost:3000/mcp/trivy";

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

    match mcp_request(MCP_KUBERNETES_URL, "list_resources", params).await {
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

    match mcp_request(MCP_CILIUM_URL, "list_policies", params).await {
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

    match mcp_request(MCP_STEAMPIPE_URL, "query", params).await {
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

    match mcp_request(MCP_TRIVY_URL, "get_vulnerabilities", params).await {
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

/// Get critical vulnerabilities only
pub async fn get_critical_vulnerabilities() -> Result<Vec<TrivyImageReport>, String> {
    let summary = get_trivy_vulnerabilities().await?;
    Ok(summary.images.into_iter()
        .filter(|img| img.critical_count > 0)
        .collect())
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
