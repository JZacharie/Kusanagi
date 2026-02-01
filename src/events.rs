use chrono::{DateTime, Utc};
use k8s_openapi::api::core::v1::Event;
use kube::{
    api::{Api, ListParams},
    Client,
};
use serde::Serialize;
use tracing::info;

use crate::error::Result;

/// Events response
#[derive(Clone, Debug, Serialize)]
pub struct EventsResponse {
    pub total_events: usize,
    pub warning_count: usize,
    pub normal_count: usize,
    pub page: usize,
    pub per_page: usize,
    pub total_pages: usize,
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
/// Optionally filter by event type (e.g., "Warning" or "Normal")
pub async fn get_events(
    client: &Client, 
    event_type_filter: Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
) -> Result<EventsResponse> {
    let events_api: Api<Event> = Api::all(client.clone());

    // kube::Error is automatically converted to KusanagiError::K8s
    let events = events_api
        .list(&ListParams::default())
        .await?;

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

    let warning_count = event_infos.iter().filter(|e| e.event_type == "Warning").count();
    let normal_count = event_infos.iter().filter(|e| e.event_type == "Normal").count();
    let total_count = event_infos.len();

    // Apply event type filter if specified
    if let Some(filter) = event_type_filter {
        event_infos.retain(|e| e.event_type.eq_ignore_ascii_case(&filter));
    }

    // Sort by type (Warning first) then last timestamp (newest first)
    event_infos.sort_by(|a, b| {
        // First compare by event type (Warning < Normal lexicographically but we want Warning first)
        let type_cmp = if a.event_type == "Warning" && b.event_type != "Warning" {
            std::cmp::Ordering::Less
        } else if a.event_type != "Warning" && b.event_type == "Warning" {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        };

        if type_cmp != std::cmp::Ordering::Equal {
            type_cmp
        } else {
            b.last_timestamp.cmp(&a.last_timestamp)
        }
    });

    let filtered_total = event_infos.len();
    let page = page.unwrap_or(1);
    let per_page = per_page.unwrap_or(20);
    let total_pages = filtered_total.div_ceil(per_page);
    
    // Slice for pagination
    let start = (page.max(1) - 1) * per_page;
    let paginated_events = if start < event_infos.len() {
        let end = (start + per_page).min(event_infos.len());
        event_infos[start..end].to_vec()
    } else {
        Vec::new()
    };

    info!(
        "Events: {} paginated from {} filtered ({} total)",
        paginated_events.len(),
        filtered_total,
        total_count
    );

    Ok(EventsResponse {
        total_events: total_count,
        warning_count,
        normal_count,
        page,
        per_page,
        total_pages,
        events: paginated_events,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(chrono::Duration::seconds(30)), "30s ago");
        assert_eq!(format_duration(chrono::Duration::minutes(5)), "5m ago");
        assert_eq!(format_duration(chrono::Duration::minutes(90)), "1h 30m ago");
        assert_eq!(format_duration(chrono::Duration::seconds(-5)), "just now");
    }
}
