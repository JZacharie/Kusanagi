use async_trait::async_trait;
use crate::domain::ports::{McpRepository, K8sResourceSummary, CiliumPolicySummary, TrivyVulnerabilitySummary, SteampipeResult};
use crate::error::{Result, KusanagiError};
use crate::legacy;

pub struct LegacyMcpRepository;

#[async_trait]
impl McpRepository for LegacyMcpRepository {
    async fn get_k8s_resources(&self) -> Result<K8sResourceSummary> {
        legacy::mcp::get_k8s_resources().await
            .map_err(|e| KusanagiError::external_api("MCP", &e.to_string()))
    }

    async fn get_cilium_policies(&self) -> Result<CiliumPolicySummary> {
        legacy::mcp::get_cilium_policies().await
            .map_err(|e| KusanagiError::external_api("MCP", &e.to_string()))
    }

    async fn get_trivy_vulnerabilities(&self) -> Result<TrivyVulnerabilitySummary> {
        legacy::mcp::get_trivy_vulnerabilities().await
            .map_err(|e| KusanagiError::external_api("MCP", &e.to_string()))
    }

    async fn query_steampipe(&self, query: &str) -> Result<SteampipeResult> {
        legacy::mcp::query_steampipe(query).await
            .map_err(|e| KusanagiError::external_api("MCP", &e.to_string()))
    }
}
