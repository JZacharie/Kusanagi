// Legacy pods module - minimal
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PodInfo {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub node: String,
    pub restarts: u32,
}

pub async fn get_pods() -> Result<Vec<PodInfo>, Box<dyn std::error::Error>> {
    Ok(vec![
        PodInfo {
            name: "legacy-app-123".to_string(),
            namespace: "legacy-system".to_string(),
            status: "Running".to_string(),
            node: "legacy-worker-01".to_string(),
            restarts: 0,
        },
        PodInfo {
            name: "legacy-db-456".to_string(),
            namespace: "legacy-system".to_string(),
            status: "Running".to_string(),
            node: "legacy-worker-01".to_string(),
            restarts: 1,
        },
    ])
}
