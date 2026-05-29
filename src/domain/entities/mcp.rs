//! MCP (Model Context Protocol) Domain Entities
//!
//! Entities for MCP integrations with various infrastructure tools

use serde::{Deserialize, Serialize};

// ==================== MCP Request/Response ====================

/// MCP Request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRequest {
    pub method: String,
    pub params: serde_json::Value,
}

/// MCP Response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

// ==================== Kubernetes Resources ====================

/// Kubernetes resource summary from MCP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct K8sResourceSummary {
    pub deployments: i32,
    pub statefulsets: i32,
    pub daemonsets: i32,
    pub services: i32,
    pub configmaps: i32,
    pub secrets: i32,
}

// ==================== Cilium Network Policies ====================

/// Cilium network policy summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiliumPolicySummary {
    pub total_policies: i32,
    pub policies: Vec<CiliumPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiliumPolicy {
    pub name: String,
    pub namespace: String,
    pub endpoints_matched: i32,
    pub ingress_rules: i32,
    pub egress_rules: i32,
}

// ==================== Steampipe Query ====================

/// Steampipe query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteampipeResult {
    pub query: String,
    pub rows: Vec<serde_json::Value>,
    pub columns: Vec<String>,
    pub row_count: i32,
}

// ==================== Trivy Vulnerabilities ====================

/// Trivy vulnerability summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrivyVulnerabilitySummary {
    pub total_images: i32,
    pub critical: i32,
    pub high: i32,
    pub medium: i32,
    pub low: i32,
    pub images: Vec<TrivyImageReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrivyImageReport {
    pub image: String,
    pub namespace: String,
    pub critical_count: i32,
    pub high_count: i32,
    pub last_scan: String,
}

// ==================== Kyverno Policy Reports ====================

/// Policy violation from Kyverno
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Policy summary counts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySummary {
    pub pass: i32,
    pub fail: i32,
    pub warn: i32,
    pub error: i32,
    pub skip: i32,
}

/// Complete policy report overview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyReportOverview {
    pub total_violations: i32,
    pub violations: Vec<PolicyViolation>,
    pub summary: PolicySummary,
}

// ==================== OpenObserve Logs ====================

/// Single log entry from OpenObserve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenObserveLog {
    pub timestamp: String,
    pub log: String,
    pub stream: String,
    pub level: Option<String>,
    pub pod: Option<String>,
    pub namespace: Option<String>,
    pub container: Option<String>,
}

/// Result of an OpenObserve query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenObserveQueryResult {
    pub logs: Vec<OpenObserveLog>,
    pub total: i32,
    pub took_ms: i32,
    pub query: String,
}

// ==================== MCP Configuration ====================

/// MCP server URLs configuration
#[derive(Debug, Clone)]
pub struct McpConfig {
    pub kubernetes_url: String,
    pub cilium_url: String,
    pub steampipe_url: String,
    pub trivy_url: String,
    pub openobserve_url: String,
    pub netbox_url: String,
    pub timeout_secs: u64,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            kubernetes_url: std::env::var("KUSANAGI_INTEGRATIONS_MCP_KUBERNETES_URL")
                .or_else(|_| std::env::var("MCP_KUBERNETES_URL"))
                .unwrap_or_else(|_| "http://localhost:3000/mcp/kubernetes".to_string()),
            cilium_url: std::env::var("KUSANAGI_INTEGRATIONS_MCP_CILIUM_URL")
                .or_else(|_| std::env::var("MCP_CILIUM_URL"))
                .unwrap_or_else(|_| "http://localhost:3000/mcp/cilium".to_string()),
            steampipe_url: std::env::var("KUSANAGI_INTEGRATIONS_MCP_STEAMPIPE_URL")
                .or_else(|_| std::env::var("MCP_STEAMPIPE_URL"))
                .unwrap_or_else(|_| "http://localhost:3000/mcp/steampipe".to_string()),
            trivy_url: std::env::var("KUSANAGI_INTEGRATIONS_MCP_TRIVY_URL")
                .or_else(|_| std::env::var("MCP_TRIVY_URL"))
                .unwrap_or_else(|_| "http://localhost:3000/mcp/trivy".to_string()),
            openobserve_url: std::env::var("KUSANAGI_INTEGRATIONS_MCP_OPENOBSERVE_URL")
                .or_else(|_| std::env::var("MCP_OPENOBSERVE_URL"))
                .unwrap_or_else(|_| "http://localhost:3000/mcp/openobserve".to_string()),
            netbox_url: std::env::var("KUSANAGI_INTEGRATIONS_MCP_NETBOX_URL")
                .or_else(|_| std::env::var("MCP_NETBOX_URL"))
                .unwrap_or_else(|_| {
                    "http://netbox-mcp-server.netbox.svc.cluster.local".to_string()
                }),
            timeout_secs: std::env::var("KUSANAGI_INTEGRATIONS_MCP_TIMEOUT_SECS")
                .or_else(|_| std::env::var("MCP_TIMEOUT_SECS"))
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_config_defaults() {
        // Clear env vars to test defaults
        std::env::remove_var("KUSANAGI_INTEGRATIONS_MCP_TRIVY_URL");
        std::env::remove_var("MCP_TRIVY_URL");

        let config = McpConfig::default();
        assert_eq!(config.trivy_url, "http://localhost:3000/mcp/trivy");
    }

    #[test]
    fn test_mcp_config_prefixed_priority() {
        std::env::set_var(
            "KUSANAGI_INTEGRATIONS_MCP_TRIVY_URL",
            "http://prefixed:3000",
        );
        std::env::set_var("MCP_TRIVY_URL", "http://legacy:3000");

        let config = McpConfig::default();
        assert_eq!(config.trivy_url, "http://prefixed:3000");

        std::env::remove_var("KUSANAGI_INTEGRATIONS_MCP_TRIVY_URL");
        std::env::remove_var("MCP_TRIVY_URL");
    }

    #[test]
    fn test_mcp_config_legacy_fallback() {
        std::env::remove_var("KUSANAGI_INTEGRATIONS_MCP_TRIVY_URL");
        std::env::set_var("MCP_TRIVY_URL", "http://legacy:3000");

        let config = McpConfig::default();
        assert_eq!(config.trivy_url, "http://legacy:3000");

        std::env::remove_var("MCP_TRIVY_URL");
    }
}
