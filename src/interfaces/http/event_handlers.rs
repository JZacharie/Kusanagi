//! Event HTTP Handlers
//!
//! HTTP handlers for Kubernetes events operations.

use std::sync::Arc;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;


use crate::application::use_cases::event_use_cases::*;

use crate::interfaces::http::AppState;

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
    data: web::Data<AppState>,
    query: web::Query<ListEventsQuery>,
) -> impl Responder {
    let use_case = GetRecentEventsUseCase::new(Arc::clone(&data.k8s_repo));

    match use_case.execute(
        query.namespace.as_deref(),
        query.event_type.as_deref(),
        query.page.unwrap_or(1),
        query.per_page.unwrap_or(20),
    ).await {
        Ok(paginated) => HttpResponse::Ok().json(paginated),
        Err(e) => e.error_response(),
    }
}

/// Get warning events (prioritized)
#[get("/api/events/warnings")]
pub async fn list_warning_events(
    data: web::Data<AppState>,
    query: web::Query<ListEventsQuery>,
) -> impl Responder {
    let use_case = GetWarningEventsUseCase::new(Arc::clone(&data.k8s_repo));

    match use_case.execute(
        query.namespace.as_deref(),
        query.per_page.unwrap_or(20),
    ).await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => e.error_response(),
    }
}

/// Get event statistics
#[get("/api/events/stats")]
pub async fn get_event_stats(
    data: web::Data<AppState>,
    query: web::Query<ListEventsQuery>,
) -> impl Responder {
    let use_case = GetEventStatsUseCase::new(Arc::clone(&data.k8s_repo));

    match use_case.execute(query.namespace.as_deref()).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => e.error_response(),
    }
}
