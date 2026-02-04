use crate::domain::ports::{McpRepository, K8sResourceSummary, CiliumPolicySummary, TrivyVulnerabilitySummary, SteampipeResult};
use crate::error::Result;
use std::sync::Arc;

pub struct GetK8sResourcesUseCase {
    mcp_repo: Arc<dyn McpRepository>,
}

impl GetK8sResourcesUseCase {
    pub fn new(mcp_repo: Arc<dyn McpRepository>) -> Self {
        Self { mcp_repo }
    }

    pub async fn execute(&self) -> Result<K8sResourceSummary> {
        self.mcp_repo.get_k8s_resources().await
    }
}

pub struct GetCiliumPoliciesUseCase {
    mcp_repo: Arc<dyn McpRepository>,
}

impl GetCiliumPoliciesUseCase {
    pub fn new(mcp_repo: Arc<dyn McpRepository>) -> Self {
        Self { mcp_repo }
    }

    pub async fn execute(&self) -> Result<CiliumPolicySummary> {
        self.mcp_repo.get_cilium_policies().await
    }
}

pub struct GetTrivyVulnerabilitiesUseCase {
    mcp_repo: Arc<dyn McpRepository>,
}

impl GetTrivyVulnerabilitiesUseCase {
    pub fn new(mcp_repo: Arc<dyn McpRepository>) -> Self {
        Self { mcp_repo }
    }

    pub async fn execute(&self) -> Result<TrivyVulnerabilitySummary> {
        self.mcp_repo.get_trivy_vulnerabilities().await
    }
}

pub struct QuerySteampipeUseCase {
    mcp_repo: Arc<dyn McpRepository>,
}

impl QuerySteampipeUseCase {
    pub fn new(mcp_repo: Arc<dyn McpRepository>) -> Self {
        Self { mcp_repo }
    }

    pub async fn execute(&self, query: &str) -> Result<SteampipeResult> {
        self.mcp_repo.query_steampipe(query).await
    }
}
