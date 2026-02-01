//! Domain entities
//!
//! Entities are objects that have a distinct identity and lifecycle.
//! They encapsulate business rules and invariants.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Cluster overview information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClusterOverview {
    pub name: String,
    pub version: String,
    pub node_count: usize,
    pub pod_count: usize,
    pub namespace_count: usize,
    pub status: ClusterStatus,
    pub resources: ClusterResources,
    pub health: ClusterHealth,
}

/// Cluster operational status
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum ClusterStatus {
    #[default]
    Healthy,
    Degraded,
    Critical,
    Unknown,
}

impl ClusterStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, ClusterStatus::Healthy)
    }
}

/// Cluster resource utilization
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClusterResources {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub storage_percent: f64,
    pub pod_capacity: i32,
    pub pod_used: i32,
}

/// Cluster health metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClusterHealth {
    pub alerts_firing: i32,
    pub alerts_pending: i32,
    pub nodes_not_ready: i32,
    pub pods_crashing: i32,
}

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub name: String,
    pub status: NodeStatus,
    pub role: NodeRole,
    pub resources: NodeResources,
    pub info: NodeInfo,
    pub conditions: Vec<NodeCondition>,
}

/// Node operational status
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum NodeStatus {
    #[default]
    Ready,
    NotReady,
    SchedulingDisabled,
    Unknown,
}

impl NodeStatus {
    pub fn is_ready(&self) -> bool {
        matches!(self, NodeStatus::Ready)
    }
}

/// Node role in the cluster
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum NodeRole {
    #[default]
    Worker,
    ControlPlane,
    Edge,
}

/// Node resource information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeResources {
    pub cpu_capacity: String,
    pub cpu_allocatable: String,
    pub memory_capacity: String,
    pub memory_allocatable: String,
    pub pod_capacity: i32,
    pub pod_count: i32,
}

/// Node system information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeInfo {
    pub architecture: String,
    pub os: String,
    pub kernel_version: String,
    pub kubelet_version: String,
    pub container_runtime: String,
}

/// Node condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCondition {
    pub condition_type: String,
    pub status: String,
    pub last_transition: Option<DateTime<Utc>>,
    pub reason: String,
    pub message: String,
}

/// Pod information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pod {
    pub name: String,
    pub namespace: String,
    pub status: PodStatus,
    pub containers: Vec<Container>,
    pub node_name: Option<String>,
    pub restart_count: i32,
    pub age: Option<String>,
    pub labels: HashMap<String, String>,
}

/// Pod status
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum PodStatus {
    #[default]
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
    CrashLoopBackOff,
    ImagePullBackOff,
}

impl PodStatus {
    pub fn is_running(&self) -> bool {
        matches!(self, PodStatus::Running)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, PodStatus::Failed | PodStatus::CrashLoopBackOff | PodStatus::ImagePullBackOff)
    }
}

/// Container information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub name: String,
    pub image: String,
    pub ready: bool,
    pub restart_count: i32,
    pub state: ContainerState,
}

/// Container state
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum ContainerState {
    #[default]
    Waiting,
    Running,
    Terminated,
}

/// Event in the cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterEvent {
    pub name: String,
    pub namespace: String,
    pub event_type: EventType,
    pub reason: String,
    pub message: String,
    pub involved_object: InvolvedObject,
    pub count: i32,
    pub first_timestamp: Option<DateTime<Utc>>,
    pub last_timestamp: Option<DateTime<Utc>>,
    pub age: Option<String>,
}

/// Event type
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum EventType {
    #[default]
    Normal,
    Warning,
}

/// Object involved in an event
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InvolvedObject {
    pub kind: String,
    pub name: String,
}

/// Storage information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageInfo {
    pub total_pvs: usize,
    pub available_pvs: usize,
    pub bound_pvs: usize,
    pub released_pvs: usize,
    pub total_capacity: String,
    pub used_capacity: String,
}

/// Service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub namespace: String,
    pub service_type: String,
    pub cluster_ip: String,
    pub external_ips: Vec<String>,
    pub ports: Vec<ServicePort>,
    pub selector: HashMap<String, String>,
    pub age: String,
}

/// Service port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePort {
    pub name: String,
    pub port: i32,
    pub target_port: String,
    pub protocol: String,
}

/// Ingress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingress {
    pub name: String,
    pub namespace: String,
    pub hosts: Vec<String>,
    pub paths: Vec<IngressPath>,
    pub tls: Vec<String>,
    pub age: String,
}

/// Ingress path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressPath {
    pub host: String,
    pub path: String,
    pub service_name: String,
    pub service_port: i32,
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub labels: HashMap<String, String>,
    pub timestamp: DateTime<Utc>,
}

/// Alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub name: String,
    pub status: AlertStatus,
    pub severity: AlertSeverity,
    pub summary: String,
    pub description: String,
    pub labels: HashMap<String, String>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: Option<DateTime<Utc>>,
}

/// Alert status
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum AlertStatus {
    #[default]
    Firing,
    Pending,
    Resolved,
}

/// Alert severity
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub enum AlertSeverity {
    #[default]
    Info,
    Warning,
    Critical,
}

/// Namespace information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Namespace {
    pub name: String,
    pub status: String,
    pub pod_count: usize,
    pub resource_quota: Option<ResourceQuota>,
    pub age: String,
}

/// Resource quota
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceQuota {
    pub hard: HashMap<String, String>,
    pub used: HashMap<String, String>,
}

/// Pagination information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Pagination {
    pub page: usize,
    pub per_page: usize,
    pub total: usize,
    pub total_pages: usize,
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub items: Vec<T>,
    pub pagination: Pagination,
}

impl<T> Paginated<T> {
    pub fn new(items: Vec<T>, page: usize, per_page: usize, total: usize) -> Self {
        let total_pages = if total == 0 { 1 } else { (total + per_page - 1) / per_page };
        Self {
            items,
            pagination: Pagination {
                page,
                per_page,
                total,
                total_pages,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_status_is_healthy() {
        assert!(ClusterStatus::Healthy.is_healthy());
        assert!(!ClusterStatus::Degraded.is_healthy());
        assert!(!ClusterStatus::Critical.is_healthy());
    }

    #[test]
    fn test_node_status_is_ready() {
        assert!(NodeStatus::Ready.is_ready());
        assert!(!NodeStatus::NotReady.is_ready());
    }

    #[test]
    fn test_pod_status_is_error() {
        assert!(PodStatus::Failed.is_error());
        assert!(PodStatus::CrashLoopBackOff.is_error());
        assert!(!PodStatus::Running.is_error());
        assert!(!PodStatus::Pending.is_error());
    }

    #[test]
    fn test_paginated_response() {
        let items = vec![1, 2, 3, 4, 5];
        let paginated = Paginated::new(items, 1, 10, 5);
        
        assert_eq!(paginated.pagination.page, 1);
        assert_eq!(paginated.pagination.per_page, 10);
        assert_eq!(paginated.pagination.total, 5);
        assert_eq!(paginated.pagination.total_pages, 1);
    }

    #[test]
    fn test_paginated_multiple_pages() {
        let items: Vec<i32> = (1..=5).collect();
        let paginated = Paginated::new(items, 1, 2, 5);
        
        assert_eq!(paginated.pagination.total_pages, 3);
    }

    #[test]
    fn test_event_type_default() {
        let event_type: EventType = Default::default();
        assert!(matches!(event_type, EventType::Normal));
    }

    #[test]
    fn test_cluster_overview_default() {
        let overview = ClusterOverview::default();
        assert_eq!(overview.node_count, 0);
        assert!(matches!(overview.status, ClusterStatus::Healthy));
    }
}
