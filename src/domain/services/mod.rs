//! Domain services
//!
//! Domain services contain business logic that doesn't naturally fit within
//! an entity. They orchestrate entities and ports to perform business operations.
//!
//! # Characteristics
//!
//! - Stateless (or minimal state)
//! - Pure business logic
//! - No dependencies on infrastructure

use crate::domain::entities::*;
use crate::domain::ports::*;
use crate::error::Result;
use std::sync::Arc;

/// Service for cluster operations
pub struct ClusterService {
    k8s_repo: Arc<dyn KubernetesRepository>,
    metrics_repo: Arc<dyn MetricsRepository>,
}

impl ClusterService {
    pub fn new(
        k8s_repo: Arc<dyn KubernetesRepository>,
        metrics_repo: Arc<dyn MetricsRepository>,
    ) -> Self {
        Self {
            k8s_repo,
            metrics_repo,
        }
    }

    /// Get comprehensive cluster status
    pub async fn get_cluster_status(&self) -> Result<ClusterStatus> {
        let _overview = self.k8s_repo.get_cluster_overview().await?;
        let metrics = self.metrics_repo.get_cluster_metrics().await?;
        
        // Business logic: determine cluster status based on multiple factors
        let status = if overview.health.nodes_not_ready > 0 || metrics.cpu_percent > 90.0 {
            ClusterStatus::Degraded
        } else if overview.health.pods_crashing > 10 {
            ClusterStatus::Critical
        } else {
            ClusterStatus::Healthy
        };
        
        Ok(status)
    }

    /// Get cluster capacity assessment
    pub async fn assess_capacity(&self) -> Result<CapacityAssessment> {
        let _overview = self.k8s_repo.get_cluster_overview().await?;
        let metrics = self.metrics_repo.get_cluster_metrics().await?;
        
        let cpu_utilization = metrics.cpu_percent / 100.0;
        let memory_utilization = metrics.memory_percent / 100.0;
        
        let assessment = CapacityAssessment {
            cpu_status: if cpu_utilization > 0.9 {
                ResourceStatus::Critical
            } else if cpu_utilization > 0.7 {
                ResourceStatus::Warning
            } else {
                ResourceStatus::Healthy
            },
            memory_status: if memory_utilization > 0.9 {
                ResourceStatus::Critical
            } else if memory_utilization > 0.8 {
                ResourceStatus::Warning
            } else {
                ResourceStatus::Healthy
            },
            recommendation: if cpu_utilization > 0.8 || memory_utilization > 0.8 {
                Some("Consider scaling cluster resources".to_string())
            } else {
                None
            },
        };
        
        Ok(assessment)
    }

    /// Get top resource-consuming namespaces
    pub async fn get_top_namespaces(&self, limit: usize) -> Result<Vec<NamespaceUsage>> {
        let namespaces = self.k8s_repo.list_namespaces().await?;
        let pod_usage = self.metrics_repo.get_pod_resource_usage().await?;
        
        let mut usage_map: std::collections::HashMap<String, (f64, i64)> = std::collections::HashMap::new();
        
        // Aggregate usage by namespace
        for ((ns, _), (cpu, memory)) in pod_usage {
            let entry = usage_map.entry(ns).or_insert((0.0, 0));
            entry.0 += cpu;
            entry.1 += memory;
        }
        
        // Convert to sorted list
        let mut result: Vec<NamespaceUsage> = namespaces
            .into_iter()
            .filter_map(|ns| {
                usage_map.get(&ns.name).map(|(cpu, memory)| NamespaceUsage {
                    name: ns.name,
                    cpu_cores: *cpu,
                    memory_bytes: *memory,
                    pod_count: ns.pod_count,
                })
            })
            .collect();
        
        // Sort by CPU usage descending
        result.sort_by(|a, b| b.cpu_cores.partial_cmp(&a.cpu_cores).unwrap());
        result.truncate(limit);
        
        Ok(result)
    }
}

/// Capacity assessment result
#[derive(Debug, Clone)]
pub struct CapacityAssessment {
    pub cpu_status: ResourceStatus,
    pub memory_status: ResourceStatus,
    pub recommendation: Option<String>,
}

/// Resource status
#[derive(Debug, Clone)]
pub enum ResourceStatus {
    Healthy,
    Warning,
    Critical,
}

/// Namespace resource usage
#[derive(Debug, Clone)]
pub struct NamespaceUsage {
    pub name: String,
    pub cpu_cores: f64,
    pub memory_bytes: i64,
    pub pod_count: usize,
}

/// Service for pod operations
pub struct PodService {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl PodService {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    /// Get pods with issues
    pub async fn get_pods_with_issues(&self, namespace: Option<&str>) -> Result<Vec<PodIssue>> {
        let pods = self.k8s_repo.list_pods(namespace).await?;
        
        let issues: Vec<PodIssue> = pods
            .into_iter()
            .filter_map(|pod| {
                if pod.status.is_error() {
                    Some(PodIssue {
                        namespace: pod.namespace,
                        name: pod.name,
                        status: pod.status,
                        restart_count: pod.restart_count,
                        severity: if pod.restart_count > 5 {
                            IssueSeverity::Critical
                        } else {
                            IssueSeverity::Warning
                        },
                    })
                } else {
                    None
                }
            })
            .collect();
        
        Ok(issues)
    }

    /// Restart a pod by deleting it
    pub async fn restart_pod(&self, namespace: &str, name: &str) -> Result<()> {
        // Business rule: verify pod exists before deleting
        let _pod = self.k8s_repo.get_pod(namespace, name).await?;
        
        self.k8s_repo.delete_pod(namespace, name).await?;
        Ok(())
    }

    /// Get pod diagnostics
    pub async fn diagnose_pod(&self, namespace: &str, name: &str) -> Result<PodDiagnostics> {
        let pod = self.k8s_repo.get_pod(namespace, name).await?;
        let logs = self.k8s_repo.get_pod_logs(namespace, name, None, 100).await.ok();
        
        // Generate recommendations before moving pod fields
        let recommendations = Self::generate_recommendations(&pod);
        
        let diagnostics = PodDiagnostics {
            pod_name: pod.name,
            namespace: pod.namespace,
            status: pod.status,
            container_states: pod.containers.iter().map(|c| ContainerStateInfo {
                name: c.name.clone(),
                state: format!("{:?}", c.state),
                restarts: c.restart_count,
            }).collect(),
            recent_logs: logs,
            recommendations,
        };
        
        Ok(diagnostics)
    }

    fn generate_recommendations(pod: &Pod) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if pod.status == PodStatus::CrashLoopBackOff {
            recommendations.push("Check application logs for startup errors".to_string());
            recommendations.push("Verify resource limits are sufficient".to_string());
        }
        
        if pod.status == PodStatus::ImagePullBackOff {
            recommendations.push("Verify image name and tag are correct".to_string());
            recommendations.push("Check if image registry is accessible".to_string());
        }
        
        for container in &pod.containers {
            if container.restart_count > 10 {
                recommendations.push(format!(
                    "Container '{}' has high restart count ({}). Check liveness/readiness probes.",
                    container.name, container.restart_count
                ));
            }
        }
        
        recommendations
    }
}

/// Pod issue information
#[derive(Debug, Clone)]
pub struct PodIssue {
    pub namespace: String,
    pub name: String,
    pub status: PodStatus,
    pub restart_count: i32,
    pub severity: IssueSeverity,
}

/// Issue severity
#[derive(Debug, Clone)]
pub enum IssueSeverity {
    Warning,
    Critical,
}

/// Pod diagnostics
#[derive(Debug, Clone)]
pub struct PodDiagnostics {
    pub pod_name: String,
    pub namespace: String,
    pub status: PodStatus,
    pub container_states: Vec<ContainerStateInfo>,
    pub recent_logs: Option<String>,
    pub recommendations: Vec<String>,
}

/// Container state information
#[derive(Debug, Clone)]
pub struct ContainerStateInfo {
    pub name: String,
    pub state: String,
    pub restarts: i32,
}

/// Service for event analysis
pub struct EventService {
    k8s_repo: Arc<dyn KubernetesRepository>,
}

impl EventService {
    pub fn new(k8s_repo: Arc<dyn KubernetesRepository>) -> Self {
        Self { k8s_repo }
    }

    /// Get recent warnings with analysis
    pub async fn get_warning_analysis(&self, limit: usize) -> Result<EventAnalysis> {
        let events = self.k8s_repo.list_events(None, Some("Warning")).await?;
        
        let total_warnings = events.len();
        let event_types = Self::categorize_events(&events);
        
        let top_issues: Vec<String> = event_types
            .iter()
            .take(5)
            .map(|(reason, count)| format!("{}: {} occurrences", reason, count))
            .collect();
        
        let recent_events: Vec<ClusterEvent> = events
            .into_iter()
            .take(limit)
            .collect();
        
        Ok(EventAnalysis {
            total_warnings,
            recent_events,
            top_issues,
            severity: if total_warnings > 50 {
                EventSeverity::High
            } else if total_warnings > 10 {
                EventSeverity::Medium
            } else {
                EventSeverity::Low
            },
        })
    }

    fn categorize_events(events: &[ClusterEvent]) -> Vec<(String, usize)> {
        let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        
        for event in events {
            *counts.entry(event.reason.clone()).or_insert(0) += 1;
        }
        
        let mut result: Vec<(String, usize)> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result
    }
}

/// Event analysis result
#[derive(Debug, Clone)]
pub struct EventAnalysis {
    pub total_warnings: usize,
    pub recent_events: Vec<ClusterEvent>,
    pub top_issues: Vec<String>,
    pub severity: EventSeverity,
}

/// Event severity
#[derive(Debug, Clone)]
pub enum EventSeverity {
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::ports::*;
    use std::sync::Arc;

    // Mock implementations
    struct MockK8sRepo;
    
    #[async_trait::async_trait]
    impl KubernetesRepository for MockK8sRepo {
        async fn get_cluster_overview(&self) -> Result<ClusterOverview> {
            Ok(ClusterOverview {
                health: ClusterHealth {
                    nodes_not_ready: 1,
                    ..Default::default()
                },
                ..Default::default()
            })
        }
        
        async fn list_nodes(&self) -> Result<Vec<Node>> { Ok(vec![]) }
        async fn get_node(&self, _name: &str) -> Result<Node> {
            Err(crate::error::KusanagiError::not_found("Node", _name))
        }
        async fn list_pods(&self, _namespace: Option<&str>) -> Result<Vec<Pod>> {
            Ok(vec![Pod {
                namespace: "default".to_string(),
                name: "test-pod".to_string(),
                status: PodStatus::CrashLoopBackOff,
                containers: vec![Container {
                    name: "app".to_string(),
                    image: "test:latest".to_string(),
                    ready: false,
                    restart_count: 15,
                    state: ContainerState::Waiting,
                }],
                node_name: Some("node-1".to_string()),
                restart_count: 15,
                age: Some("10m".to_string()),
                age_seconds: 600,
                labels: Default::default(),
                reason: None,
                message: None,
                cpu_usage: None,
                memory_usage: None,
                cpu_limit: None,
                memory_limit: None,
                cpu_request: None,
                memory_request: None,
            }])
        }
        async fn get_pod(&self, _namespace: &str, _name: &str) -> Result<Pod> {
            Ok(Pod {
                namespace: "default".to_string(),
                name: "test".to_string(),
                status: PodStatus::Running,
                containers: vec![],
                node_name: None,
                restart_count: 0,
                age: None,
                age_seconds: 0,
                labels: Default::default(),
                reason: None,
                message: None,
                cpu_usage: None,
                memory_usage: None,
                cpu_limit: None,
                memory_limit: None,
                cpu_request: None,
                memory_request: None,
            })
        }
        async fn get_pod_logs(&self, _ns: &str, _name: &str, _c: Option<&str>, _t: i64) -> Result<String> {
            Ok("logs".to_string())
        }
        async fn delete_pod(&self, _ns: &str, _name: &str) -> Result<()> { Ok(()) }
        async fn force_delete_pod(&self, _ns: &str, _name: &str) -> Result<()> { Ok(()) }
        async fn get_pods_status(&self) -> Result< PodsStatus> {
            Ok(PodsStatus {
                total_pods: 0,
                running_pods: 0,
                pending_pods: 0,
                succeeded_pods: 0,
                failed_pods: 0,
                error_pods: 0,
                pods_in_error: vec![],
                fetch_duration_ms: 0,
            })
        }
        async fn delete_error_pods(&self) -> Result<(usize, usize)> { Ok((0, 0)) }
        async fn scale_deployment(&self, _ns: &str, _name: &str, _replicas: i32) -> Result<()> { Ok(()) }
        async fn scale_statefulset(&self, _ns: &str, _name: &str, _replicas: i32) -> Result<()> { Ok(()) }
        async fn list_events(&self, _ns: Option<&str>, _t: Option<&str>) -> Result<Vec<ClusterEvent>> {
            Ok(vec![])
        }
        async fn list_services(&self, _ns: Option<&str>) -> Result<Vec<Service>> { Ok(vec![]) }
        async fn list_ingresses(&self, _ns: Option<&str>) -> Result<Vec<Ingress>> { Ok(vec![]) }
        async fn list_namespaces(&self) -> Result<Vec<Namespace>> {
            Ok(vec![Namespace {
                name: "default".to_string(),
                status: "Active".to_string(),
                pod_count: 10,
                resource_quota: None,
                age: "1d".to_string(),
            }])
        }
        async fn get_storage_info(&self) -> Result<StorageInfo> { Ok(StorageInfo::default()) }
    }

    struct MockMetricsRepo;
    
    #[async_trait::async_trait]
    impl MetricsRepository for MockMetricsRepo {
        async fn query(&self, _q: &str) -> Result<f64> { Ok(0.0) }
        async fn query_raw(&self, _q: &str) -> Result<serde_json::Value> {
            Ok(serde_json::Value::Null)
        }
        async fn query_range(&self, _q: &str, _s: i64, _e: i64, _st: &str) -> Result<serde_json::Value> {
            Ok(serde_json::Value::Null)
        }
        async fn get_cluster_metrics(&self) -> Result<ClusterResources> {
            Ok(ClusterResources {
                cpu_percent: 95.0,
                memory_percent: 60.0,
                ..Default::default()
            })
        }
        async fn get_pod_resource_usage(&self) -> Result<std::collections::HashMap<(String, String), (f64, i64)>> {
            Ok(Default::default())
        }
    }

    #[tokio::test]
    async fn test_cluster_service_status() {
        let k8s = Arc::new(MockK8sRepo);
        let metrics = Arc::new(MockMetricsRepo);
        let service = ClusterService::new(k8s, metrics);
        
        let status = service.get_cluster_status().await.unwrap();
        // Should be Degraded due to high CPU (>90%) and node not ready
        assert!(matches!(status, ClusterStatus::Degraded));
    }

    #[tokio::test]
    async fn test_pod_service_get_issues() {
        let k8s = Arc::new(MockK8sRepo);
        let service = PodService::new(k8s);
        
        let issues = service.get_pods_with_issues(None).await.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].name, "test-pod");
        assert!(matches!(issues[0].severity, IssueSeverity::Critical));
    }

    #[test]
    fn test_resource_status() {
        let healthy = super::ResourceStatus::Healthy;
        let warning = super::ResourceStatus::Warning;
        let critical = super::ResourceStatus::Critical;
        
        // Just verify they exist and can be compared
        assert!(matches!(healthy, super::ResourceStatus::Healthy));
        assert!(matches!(warning, super::ResourceStatus::Warning));
        assert!(matches!(critical, super::ResourceStatus::Critical));
    }
}
