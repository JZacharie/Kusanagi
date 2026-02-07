// Legacy services module - minimal
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub namespace: String,
    pub service_type: String,
    pub cluster_ip: String,
    pub ports: Vec<u16>,
}

pub async fn get_services() -> Result<Vec<ServiceInfo>, Box<dyn std::error::Error>> {
    Ok(vec![
        ServiceInfo {
            name: "legacy-api-service".to_string(),
            namespace: "legacy-system".to_string(),
            service_type: "ClusterIP".to_string(),
            cluster_ip: "10.96.1.100".to_string(),
            ports: vec![80, 443],
        },
        ServiceInfo {
            name: "legacy-db-service".to_string(),
            namespace: "legacy-system".to_string(),
            service_type: "ClusterIP".to_string(),
            cluster_ip: "10.96.1.101".to_string(),
            ports: vec![5432],
        },
    ])
}
