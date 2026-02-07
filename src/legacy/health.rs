// Legacy health module - minimal
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub component: String,
    pub status: String,
    pub message: String,
    pub last_check: String,
}

pub async fn get_health_status() -> Result<Vec<HealthStatus>, Box<dyn std::error::Error>> {
    Ok(vec![
        HealthStatus {
            component: "legacy-api".to_string(),
            status: "healthy".to_string(),
            message: "All systems operational".to_string(),
            last_check: chrono::Utc::now().to_rfc3339(),
        },
        HealthStatus {
            component: "legacy-database".to_string(),
            status: "healthy".to_string(),
            message: "Connection pool active".to_string(),
            last_check: chrono::Utc::now().to_rfc3339(),
        },
        HealthStatus {
            component: "legacy-cache".to_string(),
            status: "degraded".to_string(),
            message: "High memory usage".to_string(),
            last_check: chrono::Utc::now().to_rfc3339(),
        },
    ])
}
