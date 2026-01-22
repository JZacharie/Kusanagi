use actix_web::{web, HttpResponse, HttpRequest, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tracing::{error, warn, info};
use chrono::{Local, DateTime, Utc};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl,
    Scope, TokenResponse, TokenUrl, basic::BasicClient, reqwest::async_http_client,
};

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

#[derive(Debug, Clone)]
pub struct TokenStore {
    tokens: Arc<Mutex<HashMap<String, String>>>, // user_id -> access_token
}

impl TokenStore {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn store_token(&self, user_id: String, token: String) {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.insert(user_id, token);
    }

    pub fn get_token(&self, user_id: &str) -> Option<String> {
        let tokens = self.tokens.lock().unwrap();
        tokens.get(user_id).cloned()
    }
}

lazy_static::lazy_static! {
    static ref TOKEN_STORE: TokenStore = TokenStore::new();
}

pub struct CalendarClient {
    api_key: String,
    oauth_client: Option<BasicClient>,
}

impl CalendarClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let api_key = env::var("GOOGLE_CALENDAR_API_KEY")
            .unwrap_or_else(|_| "".to_string());
        
        let oauth_client = Self::build_oauth_client().ok();

        if api_key.is_empty() && oauth_client.is_none() {
            warn!("Neither GOOGLE_CALENDAR_API_KEY nor OAuth2 credentials set, using mock calendar data");
        }

        Ok(Self {
            api_key,
            oauth_client,
        })
    }

    fn build_oauth_client() -> Result<BasicClient, Box<dyn std::error::Error>> {
        let client_id = env::var("GOOGLE_CLIENT_ID")?;
        let client_secret = env::var("GOOGLE_CLIENT_SECRET")?;
        let redirect_url = env::var("GOOGLE_REDIRECT_URL")
            .unwrap_or_else(|_| "http://localhost:8085/api/calendar/oauth/callback".to_string());

        let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())?;
        let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())?;

        Ok(BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url)?))
    }

    pub async fn get_upcoming_events(&self, token: Option<String>) -> Result<CalendarResponse, Box<dyn std::error::Error>> {
        if let Some(access_token) = token {
            return self.fetch_google_calendar_events(&access_token).await;
        }

        if self.api_key.is_empty() {
            return Ok(self.get_mock_events());
        }

        Ok(self.get_mock_events())
    }

    async fn fetch_google_calendar_events(&self, access_token: &str) -> Result<CalendarResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let url = "https://www.googleapis.com/calendar/v3/calendars/primary/events";
        
        let now = Utc::now();
        let time_min = now.to_rfc3339();
        
        let response = client
            .get(url)
            .bearer_auth(access_token)
            .query(&[
                ("timeMin", time_min.as_str()),
                ("maxResults", "10"),
                ("singleEvents", "true"),
                ("orderBy", "startTime"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            error!("Google Calendar API error: {} - {}", status, error_text);
            return Err(format!("Google Calendar API error: {}", status).into());
        }

        let data: serde_json::Value = response.json().await?;
        
        let mut events = Vec::new();
        if let Some(items) = data["items"].as_array() {
            for item in items {
                events.push(CalendarEvent {
                    id: item["id"].as_str().unwrap_or("").to_string(),
                    summary: item["summary"].as_str().unwrap_or("Untitled Event").to_string(),
                    description: item["description"].as_str().map(|s| s.to_string()),
                    start_time: item["start"]["dateTime"]
                        .as_str()
                        .or(item["start"]["date"].as_str())
                        .unwrap_or("")
                        .to_string(),
                    end_time: item["end"]["dateTime"]
                        .as_str()
                        .or(item["end"]["date"].as_str())
                        .unwrap_or("")
                        .to_string(),
                    location: item["location"].as_str().map(|s| s.to_string()),
                    status: item["status"].as_str().unwrap_or("confirmed").to_string(),
                });
            }
        }

        Ok(CalendarResponse {
            calendar_name: "Google Calendar".to_string(),
            events,
            last_updated: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        })
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
pub async fn get_events_handler(req: HttpRequest) -> Result<HttpResponse> {
    let token = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    match CalendarClient::new() {
        Ok(client) => match client.get_upcoming_events(token).await {
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

pub async fn oauth_authorize_handler() -> Result<HttpResponse> {
    match CalendarClient::build_oauth_client() {
        Ok(client) => {
            let (auth_url, _csrf_token) = client
                .authorize_url(CsrfToken::new_random)
                .add_scope(Scope::new("https://www.googleapis.com/auth/calendar.readonly".to_string()))
                .url();

            Ok(HttpResponse::Found()
                .append_header(("Location", auth_url.to_string()))
                .finish())
        }
        Err(e) => {
            error!("OAuth client setup failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "OAuth not configured. Please set GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET"
            })))
        }
    }
}

#[derive(Deserialize)]
pub struct OAuthCallbackQuery {
    code: String,
    state: Option<String>,
}

pub async fn oauth_callback_handler(query: web::Query<OAuthCallbackQuery>) -> Result<HttpResponse> {
    match CalendarClient::build_oauth_client() {
        Ok(client) => {
            let code = AuthorizationCode::new(query.code.clone());
            
            match client.exchange_code(code).request_async(async_http_client).await {
                Ok(token_response) => {
                    let access_token = token_response.access_token().secret().to_string();
                    
                    // Store token (using a default user ID for now)
                    TOKEN_STORE.store_token("default".to_string(), access_token.clone());
                    
                    info!("OAuth token obtained successfully");
                    
                    // Redirect to frontend with success
                    Ok(HttpResponse::Found()
                        .append_header(("Location", format!("/?calendar_auth=success#access_token={}", access_token)))
                        .finish())
                }
                Err(e) => {
                    error!("Token exchange failed: {}", e);
                    Ok(HttpResponse::Found()
                        .append_header(("Location", "/?calendar_auth=error"))
                        .finish())
                }
            }
        }
        Err(e) => {
            error!("OAuth client setup failed: {}", e);
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "OAuth not configured"
            })))
        }
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/calendar")
            .route("/events", web::get().to(get_events_handler))
            .route("/oauth/authorize", web::get().to(oauth_authorize_handler))
            .route("/oauth/callback", web::get().to(oauth_callback_handler)),
    );
}
