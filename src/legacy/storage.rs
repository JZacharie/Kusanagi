// Legacy storage module - minimal
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageInfo {
    pub name: String,
    pub storage_class: String,
    pub capacity: String,
    pub status: String,
    pub access_modes: Vec<String>,
}

pub async fn get_storage() -> Result<Vec<StorageInfo>, Box<dyn std::error::Error>> {
    Ok(vec![
        StorageInfo {
            name: "legacy-pv-1".to_string(),
            storage_class: "fast-ssd".to_string(),
            capacity: "10Gi".to_string(),
            status: "Bound".to_string(),
            access_modes: vec!["ReadWriteOnce".to_string()],
        },
        StorageInfo {
            name: "legacy-pv-2".to_string(),
            storage_class: "standard".to_string(),
            capacity: "50Gi".to_string(),
            status: "Available".to_string(),
            access_modes: vec!["ReadWriteMany".to_string()],
        },
    ])
}
