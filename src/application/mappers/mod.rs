//! Mappers
//!
//! Mappers convert between domain entities and DTOs.
//! This decouples the internal domain model from external representations.

use crate::application::dtos::*;
use crate::domain::entities::*;

/// Mapper for cluster-related entities
pub struct ClusterMapper;

impl ClusterMapper {
    /// Convert domain entity to DTO
    pub fn to_overview_dto(entity: ClusterOverview) -> ClusterOverviewDto {
        ClusterOverviewDto {
            name: entity.name,
            version: entity.version,
            status: format!("{:?}", entity.status),
            node_count: entity.node_count,
            pod_count: entity.pod_count,
            namespace_count: entity.namespace_count,
            cpu_percent: entity.resources.cpu_percent,
            memory_percent: entity.resources.memory_percent,
            alerts_firing: entity.health.alerts_firing,
            alerts_pending: entity.health.alerts_pending,
        }
    }

    /// Convert status to string
    pub fn status_to_string(status: ClusterStatus) -> String {
        match status {
            ClusterStatus::Healthy => "Healthy".to_string(),
            ClusterStatus::Degraded => "Degraded".to_string(),
            ClusterStatus::Critical => "Critical".to_string(),
            ClusterStatus::Unknown => "Unknown".to_string(),
        }
    }
}

/// Mapper for node entities
pub struct NodeMapper;

impl NodeMapper {
    /// Convert domain entity to DTO
    pub fn to_dto(entity: Node) -> NodeDto {
        NodeDto {
            name: entity.name,
            status: format!("{:?}", entity.status),
            role: format!("{:?}", entity.role),
            os: entity.info.os,
            kernel_version: entity.info.kernel_version,
            kubelet_version: entity.info.kubelet_version,
            cpu_capacity: entity.resources.cpu_capacity,
            memory_capacity: entity.resources.memory_capacity,
            pod_count: entity.resources.pod_count as usize,
        }
    }

    /// Convert multiple entities to DTOs
    pub fn to_dto_list(entities: Vec<Node>) -> Vec<NodeDto> {
        entities.into_iter().map(Self::to_dto).collect()
    }
}

/// Mapper for pod entities
pub struct PodMapper;

impl PodMapper {
    /// Convert domain entity to DTO
    pub fn to_dto(entity: Pod) -> PodDto {
        PodDto {
            name: entity.name,
            namespace: entity.namespace,
            status: format!("{:?}", entity.status),
            node_name: entity.node_name,
            restart_count: entity.restart_count,
            age: entity.age.unwrap_or_default(),
            containers: entity.containers.into_iter().map(Self::container_to_dto).collect(),
        }
    }

    fn container_to_dto(entity: Container) -> ContainerDto {
        ContainerDto {
            name: entity.name,
            image: entity.image,
            ready: entity.ready,
            restart_count: entity.restart_count,
            state: format!("{:?}", entity.state),
        }
    }

    /// Convert multiple entities to DTOs
    pub fn to_dto_list(entities: Vec<Pod>) -> Vec<PodDto> {
        entities.into_iter().map(Self::to_dto).collect()
    }
}

/// Mapper for event entities
pub struct EventMapper;

impl EventMapper {
    /// Convert domain entity to DTO
    pub fn to_dto(entity: ClusterEvent) -> EventDto {
        EventDto {
            name: entity.name,
            namespace: entity.namespace,
            event_type: format!("{:?}", entity.event_type),
            reason: entity.reason,
            message: entity.message,
            involved_object_kind: entity.involved_object.kind,
            involved_object_name: entity.involved_object.name,
            count: entity.count,
            age: entity.age.unwrap_or_default(),
        }
    }

    /// Convert multiple entities to DTOs
    pub fn to_dto_list(entities: Vec<ClusterEvent>) -> Vec<EventDto> {
        entities.into_iter().map(Self::to_dto).collect()
    }
}

/// Mapper for alert entities
pub struct AlertMapper;

impl AlertMapper {
    /// Convert domain entity to DTO
    pub fn to_dto(entity: Alert) -> AlertDto {
        AlertDto {
            name: entity.name,
            status: format!("{:?}", entity.status),
            severity: format!("{:?}", entity.severity),
            summary: entity.summary,
            description: entity.description,
            starts_at: entity.starts_at.to_rfc3339(),
        }
    }

    /// Convert multiple entities to DTOs
    pub fn to_dto_list(entities: Vec<Alert>) -> Vec<AlertDto> {
        entities.into_iter().map(Self::to_dto).collect()
    }
}

/// Mapper for storage entities
pub struct StorageMapper;

impl StorageMapper {
    /// Convert domain entity to DTO
    pub fn to_dto(entity: StorageInfo) -> StorageDto {
        // Calculate utilization percentage
        let utilization = if entity.total_capacity != "0" {
            100.0 * entity.released_pvs as f64 / entity.total_pvs.max(1) as f64
        } else {
            0.0
        };

        StorageDto {
            total_pvs: entity.total_pvs,
            available_pvs: entity.available_pvs,
            bound_pvs: entity.bound_pvs,
            total_capacity: entity.total_capacity,
            used_capacity: entity.used_capacity,
            utilization_percent: utilization,
        }
    }
}

/// Mapper for service entities
pub struct ServiceMapper;

impl ServiceMapper {
    /// Convert domain entity to DTO
    pub fn to_dto(entity: Service) -> ServiceDto {
        ServiceDto {
            name: entity.name,
            namespace: entity.namespace,
            service_type: entity.service_type,
            cluster_ip: entity.cluster_ip,
            external_ips: entity.external_ips,
            ports: entity.ports.into_iter().map(Self::port_to_dto).collect(),
            age: entity.age,
        }
    }

    fn port_to_dto(entity: ServicePort) -> ServicePortDto {
        ServicePortDto {
            name: entity.name,
            port: entity.port,
            target_port: entity.target_port,
            protocol: entity.protocol,
        }
    }

    /// Convert multiple entities to DTOs
    pub fn to_dto_list(entities: Vec<Service>) -> Vec<ServiceDto> {
        entities.into_iter().map(Self::to_dto).collect()
    }
}

/// Mapper for namespace entities
pub struct NamespaceMapper;

impl NamespaceMapper {
    /// Convert domain entity to DTO
    pub fn to_dto(entity: Namespace) -> NamespaceDto {
        NamespaceDto {
            name: entity.name,
            status: entity.status,
            pod_count: entity.pod_count,
            age: entity.age,
        }
    }

    /// Convert multiple entities to DTOs
    pub fn to_dto_list(entities: Vec<Namespace>) -> Vec<NamespaceDto> {
        entities.into_iter().map(Self::to_dto).collect()
    }
}

/// Generic paginated mapper
pub struct PaginatedMapper;

impl PaginatedMapper {
    /// Convert domain paginated to DTO
    pub fn to_dto<T, U>(paginated: Paginated<T>, mapper: impl Fn(T) -> U) -> PaginatedResponse<U> {
        PaginatedResponse {
            items: paginated.items.into_iter().map(mapper).collect(),
            page: paginated.pagination.page,
            per_page: paginated.pagination.per_page,
            total: paginated.pagination.total,
            total_pages: paginated.pagination.total_pages,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_mapper() {
        let entity = ClusterOverview {
            name: "test".to_string(),
            version: "v1.28".to_string(),
            status: ClusterStatus::Healthy,
            node_count: 5,
            pod_count: 100,
            namespace_count: 10,
            resources: ClusterResources {
                cpu_percent: 45.0,
                memory_percent: 60.0,
                ..Default::default()
            },
            health: ClusterHealth {
                alerts_firing: 2,
                alerts_pending: 5,
                ..Default::default()
            },
        };

        let dto = ClusterMapper::to_overview_dto(entity);
        
        assert_eq!(dto.name, "test");
        assert_eq!(dto.node_count, 5);
        assert_eq!(dto.cpu_percent, 45.0);
        assert_eq!(dto.alerts_firing, 2);
    }

    #[test]
    fn test_node_mapper() {
        let entity = Node {
            name: "node-1".to_string(),
            status: NodeStatus::Ready,
            role: NodeRole::Worker,
            resources: NodeResources {
                cpu_capacity: "4".to_string(),
                memory_capacity: "16Gi".to_string(),
                pod_count: 10,
                ..Default::default()
            },
            info: NodeInfo {
                os: "Linux".to_string(),
                kernel_version: "5.4.0".to_string(),
                kubelet_version: "v1.28.0".to_string(),
                ..Default::default()
            },
            conditions: vec![],
        };

        let dto = NodeMapper::to_dto(entity);
        
        assert_eq!(dto.name, "node-1");
        assert_eq!(dto.status, "Ready");
        assert_eq!(dto.cpu_capacity, "4");
    }

    #[test]
    fn test_pod_mapper() {
        let entity = Pod {
            name: "pod-1".to_string(),
            namespace: "default".to_string(),
            status: PodStatus::Running,
            node_name: Some("node-1".to_string()),
            restart_count: 0,
            age: Some("10m".to_string()),
            containers: vec![Container {
                name: "app".to_string(),
                image: "nginx:latest".to_string(),
                ready: true,
                restart_count: 0,
                state: ContainerState::Running,
            }],
            labels: Default::default(),
        };

        let dto = PodMapper::to_dto(entity);
        
        assert_eq!(dto.name, "pod-1");
        assert_eq!(dto.status, "Running");
        assert_eq!(dto.containers.len(), 1);
    }

    #[test]
    fn test_event_mapper() {
        let entity = ClusterEvent {
            name: "event-1".to_string(),
            namespace: "default".to_string(),
            event_type: EventType::Warning,
            reason: "FailedScheduling".to_string(),
            message: "No nodes available".to_string(),
            involved_object: InvolvedObject {
                kind: "Pod".to_string(),
                name: "test-pod".to_string(),
            },
            count: 5,
            first_timestamp: None,
            last_timestamp: None,
            age: Some("5m".to_string()),
        };

        let dto = EventMapper::to_dto(entity);
        
        assert_eq!(dto.reason, "FailedScheduling");
        assert_eq!(dto.event_type, "Warning");
    }

    #[test]
    fn test_paginated_mapper() {
        let paginated = Paginated {
            items: vec![1, 2, 3],
            pagination: Pagination {
                page: 1,
                per_page: 10,
                total: 3,
                total_pages: 1,
            },
        };

        let dto = PaginatedMapper::to_dto(paginated, |i| i * 2);
        
        assert_eq!(dto.items, vec![2, 4, 6]);
        assert_eq!(dto.page, 1);
        assert_eq!(dto.total, 3);
    }
}
