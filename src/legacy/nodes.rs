// Legacy nodes module - minimal
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub status: String,
    pub role: String,
    pub cpu_usage: String,
    pub memory_usage: String,
}

pub async fn get_nodes() -> Result<Vec<NodeInfo>, Box<dyn std::error::Error>> {
    Ok(vec![
        NodeInfo {
            name: "legacy-master-01".to_string(),
            status: "Ready".to_string(),
            role: "control-plane".to_string(),
            cpu_usage: "15%".to_string(),
            memory_usage: "45%".to_string(),
        },
        NodeInfo {
            name: "legacy-worker-01".to_string(),
            status: "Ready".to_string(),
            role: "worker".to_string(),
            cpu_usage: "32%".to_string(),
            memory_usage: "67%".to_string(),
        },
    ])
}
