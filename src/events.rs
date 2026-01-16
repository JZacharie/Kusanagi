use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::Event;
use kube::{
    api::{Api, ListParams},
    Client,
};
use serde::Serialize;
use tracing::info;

/// Events response
#[derive(Clone, Debug, Serialize)]
pub struct EventsResponse {
    pub total_events: usize,
    pub warning_count: usize,
    pub normal_count: usize,
    pub events: Vec<EventInfo>,
}

#[derive(Clone, Debug, Serialize)]
pub struct EventInfo {
    pub name: String,
    pub namespace: String,
    pub event_type: String,
    pub reason: String,
    pub message: String,
    pub involved_object_kind: String,
    pub involved_object_name: String,
    pub count: i32,
    pub first_timestamp: Option<String>,
    pub last_timestamp: Option<String>,
    pub age: Option<String>,
}

/// Get recent Kubernetes events (last 1 hour, warnings prioritized)
pub async fn get_events() -> Result<EventsResponse, String> {
    let client = Client::try_default()
        .await
        .map_err(|e| format!("Failed to create Kubernetes client: {}", e))?;

    let events_api: Api<Event> = Api::all(client);

    let events = events_api
        .list(&ListParams::default())
        .await
        .map_err(|e| format!("Failed to list events: {}", e))?;

    let now = Utc::now();
    let one_hour_ago = now - chrono::Duration::hours(1);

    let mut event_infos: Vec<EventInfo> = events
        .items
        .iter()
        .filter_map(|event| {
            // Get the most recent timestamp
            let last_ts = event
                .last_timestamp
                .as_ref()
                .and_then(|t| DateTime::parse_from_rfc3339(&t.0.to_rfc3339()).ok())
                .map(|t| t.with_timezone(&Utc));

            // Filter events from last hour
            if let Some(ts) = last_ts {
                if ts < one_hour_ago {
                    return None;
                }
            }

            let name = event.metadata.name.clone().unwrap_or_default();
            let namespace = event.metadata.namespace.clone().unwrap_or_else(|| "default".to_string());
            
            let event_type = event.type_.clone().unwrap_or_else(|| "Normal".to_string());
            let reason = event.reason.clone().unwrap_or_default();
            let message = event.message.clone().unwrap_or_default();
            
            let involved_object_kind = event
                .involved_object
                .kind
                .clone()
                .unwrap_or_default();
            let involved_object_name = event
                .involved_object
                .name
                .clone()
                .unwrap_or_default();
            
            let count = event.count.unwrap_or(1);
            
            let first_timestamp = event
                .first_timestamp
                .as_ref()
                .map(|t| t.0.to_rfc3339());
            let last_timestamp = event
                .last_timestamp
                .as_ref()
                .map(|t| t.0.to_rfc3339());

            // Calculate age from last timestamp
            let age = last_ts.map(|ts| {
                let duration = now.signed_duration_since(ts);
                format_duration(duration)
            });

            Some(EventInfo {
                name,
                namespace,
                event_type,
                reason,
                message,
                involved_object_kind,
                involved_object_name,
                count,
                first_timestamp,
                last_timestamp,
                age,
            })
        })
        .collect();

    // Sort: Warnings first, then by timestamp (newest first)
    event_infos.sort_by(|a, b| {
        let a_warning = a.event_type == "Warning";
        let b_warning = b.event_type == "Warning";
        
        if a_warning != b_warning {
            return b_warning.cmp(&a_warning);
        }
        
        // Sort by last timestamp (newest first)
        b.last_timestamp.cmp(&a.last_timestamp)
    });

    let warning_count = event_infos.iter().filter(|e| e.event_type == "Warning").count();
    let normal_count = event_infos.iter().filter(|e| e.event_type == "Normal").count();

    info!(
        "Events: {} total ({} warnings, {} normal)",
        event_infos.len(),
        warning_count,
        normal_count
    );

    Ok(EventsResponse {
        total_events: event_infos.len(),
        warning_count,
        normal_count,
        events: event_infos,
    })
}

fn format_duration(duration: chrono::Duration) -> String {
    let total_seconds = duration.num_seconds();

    if total_seconds < 0 {
        return "just now".to_string();
    }

    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m ago", hours, minutes)
    } else if minutes > 0 {
        format!("{}m ago", minutes)
    } else {
        format!("{}s ago", seconds)
    }
}
