// Legacy events module - minimal
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct EventInfo {
    pub name: String,
    pub namespace: String,
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub timestamp: String,
}

pub async fn get_events() -> Result<Vec<EventInfo>, Box<dyn std::error::Error>> {
    Ok(vec![
        EventInfo {
            name: "legacy-event-1".to_string(),
            namespace: "legacy-system".to_string(),
            event_type: "Normal".to_string(),
            reason: "Scheduled".to_string(),
            message: "Pod scheduled successfully".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
        EventInfo {
            name: "legacy-event-2".to_string(),
            namespace: "legacy-system".to_string(),
            event_type: "Warning".to_string(),
            reason: "FailedMount".to_string(),
            message: "Volume mount failed".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        },
    ])
}
