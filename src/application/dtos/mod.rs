//! Data Transfer Objects (DTOs)
//!
//! DTOs are used for:
//! - API request/response payloads
//! - Decoupling internal domain from external interfaces
//! - Validation of input data

use serde::{Deserialize, Serialize};

/// DTO for cluster overview response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterOverviewDto {
    pub name: String,
    pub version: String,
    pub status: String,
    pub node_count: usize,
    pub pod_count: usize,
    pub namespace_count: usize,
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub alerts_firing: i32,
    pub alerts_pending: i32,
}

/// DTO for node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDto {
    pub name: String,
    pub status: String,
    pub role: String,
    pub os: String,
    pub kernel_version: String,
    pub kubelet_version: String,
    pub cpu_capacity: String,
    pub memory_capacity: String,
    pub pod_count: usize,
    pub disk_usage_percent: Option<f64>,
    pub disk_capacity: Option<String>,
    pub ephemeral_storage_capacity: Option<String>,
}

/// DTO for pod information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodDto {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub node_name: Option<String>,
    pub restart_count: i32,
    pub age: String,
    pub containers: Vec<ContainerDto>,
}

/// DTO for container information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerDto {
    pub name: String,
    pub image: String,
    pub ready: bool,
    pub restart_count: i32,
    pub state: String,
}

/// DTO for event information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDto {
    pub name: String,
    pub namespace: String,
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub involved_object_kind: String,
    pub involved_object_name: String,
    pub count: i32,
    pub age: String,
}

/// DTO for paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub page: usize,
    pub per_page: usize,
    pub total: usize,
    pub total_pages: usize,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, page: usize, per_page: usize, total: usize) -> Self {
        let total_pages = if total == 0 { 1 } else { (total + per_page - 1) / per_page };
        Self {
            items,
            page,
            per_page,
            total,
            total_pages,
        }
    }
}

/// DTO for alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertDto {
    pub name: String,
    pub status: String,
    pub severity: String,
    pub summary: String,
    pub description: String,
    pub starts_at: String,
}

/// DTO for storage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDto {
    pub total_pvs: usize,
    pub available_pvs: usize,
    pub bound_pvs: usize,
    pub total_capacity: String,
    pub used_capacity: String,
    pub utilization_percent: f64,
}

/// DTO for service information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDto {
    pub name: String,
    pub namespace: String,
    pub service_type: String,
    pub cluster_ip: String,
    pub external_ips: Vec<String>,
    pub ports: Vec<ServicePortDto>,
    pub age: String,
}

/// DTO for service port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePortDto {
    pub name: String,
    pub port: i32,
    pub target_port: String,
    pub protocol: String,
}

/// DTO for namespace information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceDto {
    pub name: String,
    pub status: String,
    pub pod_count: usize,
    pub age: String,
}

/// DTO for health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckDto {
    pub status: String,
    pub message: String,
    pub version: String,
    pub uptime_seconds: u64,
}

/// DTO for metrics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsDto {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub pod_count: i32,
    pub node_count: i32,
    pub container_count: i32,
    pub alerts_firing: i32,
    pub alerts_pending: i32,
}

/// DTO for creating a sync request
#[derive(Debug, Clone, Deserialize)]
pub struct SyncRequestDto {
    pub app_name: String,
}

/// DTO for sync response
#[derive(Debug, Clone, Serialize)]
pub struct SyncResponseDto {
    pub success: bool,
    pub message: String,
}

/// DTO for pod logs request
#[derive(Debug, Clone, Deserialize)]
pub struct PodLogsRequestDto {
    pub namespace: String,
    pub pod_name: String,
    pub container: Option<String>,
    pub tail: Option<i64>,
}

/// DTO for event query parameters
#[derive(Debug, Clone, Deserialize)]
pub struct EventsQueryDto {
    pub event_type: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paginated_response() {
        let items = vec![1, 2, 3];
        let response: PaginatedResponse<i32> = PaginatedResponse::new(items, 1, 10, 3);
        
        assert_eq!(response.page, 1);
        assert_eq!(response.per_page, 10);
        assert_eq!(response.total, 3);
        assert_eq!(response.total_pages, 1);
    }

    #[test]
    fn test_paginated_multiple_pages() {
        let items: Vec<i32> = (1..=5).collect();
        let response = PaginatedResponse::new(items, 1, 2, 5);
        
        assert_eq!(response.total_pages, 3);
    }

    #[test]
    fn test_cluster_overview_dto() {
        let dto = ClusterOverviewDto {
            name: "prod-cluster".to_string(),
            version: "v1.28".to_string(),
            status: "Healthy".to_string(),
            node_count: 5,
            pod_count: 100,
            namespace_count: 10,
            cpu_percent: 45.5,
            memory_percent: 60.0,
            alerts_firing: 2,
            alerts_pending: 5,
        };
        
        assert_eq!(dto.node_count, 5);
        assert!(dto.cpu_percent > 0.0);
    }
}
