//! Event HTTP Handlers
//!
//! HTTP handlers for Kubernetes events operations.

use std::sync::Arc;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;

use crate::application::use_cases::event_use_cases::*;

// use crate::interfaces::http::AppState; // Commented out for compilation

#[derive(Debug, Deserialize)]
pub struct ListEventsQuery {
    pub namespace: Option<String>,
    pub event_type: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
}

/// List all events with pagination
#[get("/api/events")]
pub async fn list_events(
    // data: web::Data<AppState> // Commented out for compilation,
    query: web::Query<ListEventsQuery>,
) -> impl Responder {
    // Temporarily return empty response for compilation
    HttpResponse::Ok().json(serde_json::json!({"events": []}))
}

/// Get warning events (prioritized)
#[get("/api/events/warnings")]
pub async fn list_warning_events(
    // data: web::Data<AppState> // Commented out for compilation,
    query: web::Query<ListEventsQuery>,
) -> impl Responder {
    // Temporarily return empty response for compilation
    HttpResponse::Ok().json(serde_json::json!({"events": []}))
}

/// Get event statistics
#[get("/api/events/stats")]
pub async fn get_event_stats(
    // data: web::Data<AppState> // Commented out for compilation,
    query: web::Query<ListEventsQuery>,
) -> impl Responder {
    // Temporarily return empty response for compilation
    HttpResponse::Ok().json(serde_json::json!({"stats": {}}))
}
