use async_trait::async_trait;
use crate::error::Result;

#[async_trait]
pub trait McpRepository: Send + Sync {
    async fn get_k8s_resources(&self) -> Result<K8sResourceSummary>;
    async fn get_cilium_policies(&self) -> Result<CiliumPolicySummary>;
    async fn get_trivy_vulnerabilities(&self) -> Result<TrivyVulnerabilitySummary>;
    async fn query_steampipe(&self, query: &str) -> Result<SteampipeResult>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct K8sResourceSummary {
    pub deployments: i32,
    pub statefulsets: i32,
    pub daemonsets: i32,
    pub services: i32,
    pub configmaps: i32,
    pub secrets: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CiliumPolicySummary {
    pub total_policies: i32,
    pub allow_policies: i32,
    pub deny_policies: i32,
    pub namespaces_covered: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrivyVulnerabilitySummary {
    pub total_vulnerabilities: i32,
    pub critical: i32,
    pub high: i32,
    pub medium: i32,
    pub low: i32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SteampipeResult {
    pub query: String,
    pub rows: Vec<serde_json::Value>,
    pub row_count: usize,
}
