//! Alert HTTP Handlers
//!
//! HTTP handlers for alert operations.

use std::sync::Arc;
use actix_web::{get, post, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;


use crate::application::use_cases::alert_use_cases::*;

use crate::interfaces::http::AppState;

#[derive(Debug, Deserialize)]
pub struct SilenceAlertRequest {
    pub fingerprint: String,
    pub duration_secs: u64,
}

/// Get active alerts
#[get("/api/alerts")]
pub async fn get_active_alerts(
    data: web::Data<AppState>,
) -> impl Responder {
    let repo = match data.get_alert_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Alert repository not available"
            })),
    };

    let use_case = GetActiveAlertsUseCase::new(repo);

    match use_case.execute().await {
        Ok(alerts) => HttpResponse::Ok().json(alerts),
        Err(e) => e.error_response(),
    }
}

/// Get cached alerts
#[get("/api/alerts/cached")]
pub async fn get_cached_alerts(
    data: web::Data<AppState>,
) -> impl Responder {
    let repo = match data.get_alert_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Alert repository not available"
            })),
    };

    let use_case = GetCachedAlertsUseCase::new(repo);

    match use_case.execute().await {
        Ok(alerts) => HttpResponse::Ok().json(alerts),
        Err(e) => e.error_response(),
    }
}

/// Get alert statistics
#[get("/api/alerts/stats")]
pub async fn get_alert_stats(
    data: web::Data<AppState>,
) -> impl Responder {
    let repo = match data.get_alert_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Alert repository not available"
            })),
    };

    let use_case = GetAlertStatsUseCase::new(repo);

    match use_case.execute().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => e.error_response(),
    }
}

/// Get alert by fingerprint
#[get("/api/alerts/{fingerprint}")]
pub async fn get_alert(
    data: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let repo = match data.get_alert_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Alert repository not available"
            })),
    };

    let use_case = GetAlertUseCase::new(repo);

    match use_case.execute(&path).await {
        Ok(alert) => HttpResponse::Ok().json(alert),
        Err(e) => e.error_response(),
    }
}

/// Silence an alert
#[post("/api/alerts/silence")]
pub async fn silence_alert(
    data: web::Data<AppState>,
    body: web::Json<SilenceAlertRequest>,
) -> impl Responder {
    let repo = match data.get_alert_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Alert repository not available"
            })),
    };

    let use_case = SilenceAlertUseCase::new(repo);

    match use_case.execute(&body.fingerprint, body.duration_secs).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Alert {} silenced for {} seconds", body.fingerprint, body.duration_secs)
        })),
        Err(e) => e.error_response(),
    }
}
