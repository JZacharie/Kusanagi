use async_trait::async_trait;
use crate::domain::ports::{McpRepository, K8sResourceSummary, CiliumPolicySummary, TrivyVulnerabilitySummary, SteampipeResult};
use crate::error::{Result, KusanagiError};
// use crate::legacy; // Disabled for core version

pub struct LegacyMcpRepository;

#[async_trait]
impl McpRepository for LegacyMcpRepository {
    async fn get_k8s_resources(&self) -> Result<K8sResourceSummary> {
        Ok(K8sResourceSummary {
            deployments: 0,
            statefulsets: 0,
            daemonsets: 0,
            services: 0,
            configmaps: 0,
            secrets: 0,
        })
    }

    async fn get_cilium_policies(&self) -> Result<CiliumPolicySummary> {
        Ok(CiliumPolicySummary {
            total_policies: 0,
            allow_policies: 0,
            deny_policies: 0,
            namespaces_covered: 0,
        })
    }

    async fn get_trivy_vulnerabilities(&self) -> Result<TrivyVulnerabilitySummary> {
        Ok(TrivyVulnerabilitySummary {
            total_vulnerabilities: 0,
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
        })
    }

    async fn query_steampipe(&self, query: &str) -> Result<SteampipeResult> {
        Ok(SteampipeResult {
            query: query.to_string(),
            rows: vec![],
            row_count: 0,
        })
    }
}
