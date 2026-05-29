//! Tests for domain entities

use kusanagi::domain::entities::{ClusterInfo, NodeInfo};

#[test]
fn test_cluster_info_creation() {
    let cluster = ClusterInfo {
        name: "production".to_string(),
        version: "v1.28.0".to_string(),
        status: "Active".to_string(),
        nodes: 5,
    };

    assert_eq!(cluster.name, "production");
    assert_eq!(cluster.version, "v1.28.0");
    assert_eq!(cluster.status, "Active");
    assert_eq!(cluster.nodes, 5);
}

#[test]
fn test_cluster_info_clone() {
    let cluster = ClusterInfo {
        name: "staging".to_string(),
        version: "v1.27.0".to_string(),
        status: "Active".to_string(),
        nodes: 3,
    };

    let cloned = cluster.clone();
    assert_eq!(cloned.name, cluster.name);
    assert_eq!(cloned.version, cluster.version);
}

#[test]
fn test_cluster_info_serialization() {
    let cluster = ClusterInfo {
        name: "test-cluster".to_string(),
        version: "v1.28.0".to_string(),
        status: "Active".to_string(),
        nodes: 3,
    };

    let json = serde_json::to_string(&cluster).expect("Failed to serialize");
    assert!(json.contains("test-cluster"));
    assert!(json.contains("v1.28.0"));

    let deserialized: ClusterInfo = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(deserialized.name, cluster.name);
    assert_eq!(deserialized.nodes, cluster.nodes);
}

#[test]
fn test_node_info_creation() {
    let node = NodeInfo {
        name: "worker-1".to_string(),
        status: "Ready".to_string(),
        role: "worker".to_string(),
        disk_usage: None,
    };

    assert_eq!(node.name, "worker-1");
    assert_eq!(node.status, "Ready");
    assert_eq!(node.role, "worker");
}

#[test]
fn test_node_info_variants() {
    let nodes = [
        NodeInfo {
            name: "master-1".to_string(),
            status: "Ready".to_string(),
            role: "master".to_string(),
            disk_usage: None,
        },
        NodeInfo {
            name: "worker-1".to_string(),
            status: "NotReady".to_string(),
            role: "worker".to_string(),
            disk_usage: None,
        },
        NodeInfo {
            name: "worker-2".to_string(),
            status: "Ready".to_string(),
            role: "worker".to_string(),
            disk_usage: None,
        },
    ];

    let ready_count = nodes.iter().filter(|n| n.status == "Ready").count();
    assert_eq!(ready_count, 2);

    let master_count = nodes.iter().filter(|n| n.role == "master").count();
    assert_eq!(master_count, 1);
}

#[test]
fn test_node_info_serialization() {
    let node = NodeInfo {
        name: "test-node".to_string(),
        status: "Ready".to_string(),
        role: "worker".to_string(),
        disk_usage: None,
    };

    let json = serde_json::to_string(&node).expect("Failed to serialize");
    let deserialized: NodeInfo = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.name, node.name);
    assert_eq!(deserialized.status, node.status);
    assert_eq!(deserialized.role, node.role);
}
