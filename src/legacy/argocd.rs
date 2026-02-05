// Legacy ArgoCD module - minimal
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ArgoApplication {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub health: String,
    pub sync_status: String,
}

pub async fn get_applications() -> Result<Vec<ArgoApplication>, Box<dyn std::error::Error>> {
    Ok(vec![
        ArgoApplication {
            name: "legacy-app".to_string(),
            namespace: "argocd".to_string(),
            status: "Healthy".to_string(),
            health: "Healthy".to_string(),
            sync_status: "Synced".to_string(),
        },
    ])
}
