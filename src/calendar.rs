use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, warn};
use chrono::{Local};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarEvent {
    pub id: String,
    pub summary: String,
    pub description: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub location: Option<String>,
    pub status: String, // confirmed, tentative, cancelled
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarResponse {
    pub events: Vec<CalendarEvent>,
    pub calendar_name: String,
    pub last_updated: String,
}

pub struct CalendarClient {
    // Future: Add OAuth2 client fields here
    api_key: String,
}

impl CalendarClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = env::var("GOOGLE_CALENDAR_API_KEY")
            .unwrap_or_else(|_| "".to_string());
        
        if api_key.is_empty() {
            warn!("GOOGLE_CALENDAR_API_KEY not set, using mock calendar data");
        }

        Ok(Self {
            api_key,
        })
    }

    pub async fn get_upcoming_events(&self) -> Result<CalendarResponse, Box<dyn std::error::Error>> {
        if self.api_key.is_empty() {
            return Ok(self.get_mock_events());
        }

        // Implementation for real Google API would go here
        // For now, fallback to mock
        Ok(self.get_mock_events())
    }

    fn get_mock_events(&self) -> CalendarResponse {
        let now = Local::now();
        
        CalendarResponse {
            calendar_name: "Personal (Mocked)".to_string(),
            events: vec![
                CalendarEvent {
                    id: "evt1".to_string(),
                    summary: "Daily Standup".to_string(),
                    description: Some("Discussion on current sprints".to_string()),
                    start_time: (now + chrono::Duration::hours(1)).format("%Y-%m-%dT%H:%M:00Z").to_string(),
                    end_time: (now + chrono::Duration::hours(1) + chrono::Duration::minutes(30)).format("%Y-%m-%dT%H:%M:00Z").to_string(),
                    location: Some("Zoom / Discord".to_string()),
                    status: "confirmed".to_string(),
                },
                CalendarEvent {
                    id: "evt2".to_string(),
                    summary: "Project Kusanagi Sync".to_string(),
                    description: Some("Reviewing v1.0.0 roadmap progress".to_string()),
                    start_time: (now + chrono::Duration::hours(4)).format("%Y-%m-%dT%H:%M:00Z").to_string(),
                    end_time: (now + chrono::Duration::hours(5)).format("%Y-%m-%dT%H:%M:00Z").to_string(),
                    location: Some("Berlin Office / Remote".to_string()),
                    status: "tentative".to_string(),
                },
                CalendarEvent {
                    id: "evt3".to_string(),
                    summary: "Design Review".to_string(),
                    description: Some("New cyberpunk aesthetics".to_string()),
                    start_time: (now + chrono::Duration::days(1)).format("%Y-%m-%dT10:00:00Z").to_string(),
                    end_time: (now + chrono::Duration::days(1)).format("%Y-%m-%dT11:30:00Z").to_string(),
                    location: None,
                    status: "confirmed".to_string(),
                },
            ],
            last_updated: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

// API Handlers
pub async fn get_events_handler() -> Result<HttpResponse> {
    match CalendarClient::new() {
        Ok(client) => match client.get_upcoming_events().await {
            Ok(data) => Ok(HttpResponse::Ok().json(data)),
            Err(e) => {
                error!("Calendar error: {}", e);
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})))
            }
        },
        Err(e) => {
            error!("Failed to create calendar client: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})))
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/calendar")
            .route("/events", web::get().to(get_events_handler)),
    );
}
