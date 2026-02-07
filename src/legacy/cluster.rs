// Legacy cluster module - minimal
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub name: String,
    pub version: String,
    pub status: String,
    pub nodes: u32,
}

pub async fn get_cluster_info() -> Result<ClusterInfo, Box<dyn std::error::Error>> {
    Ok(ClusterInfo {
        name: "legacy-cluster".to_string(),
        version: "v1.28.0".to_string(),
        status: "healthy".to_string(),
        nodes: 3,
    })
}
