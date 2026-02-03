//! Prometheus HTTP Handlers
//!
//! HTTP handlers for Prometheus metrics operations.

use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;


use crate::application::use_cases::prometheus_use_cases::*;

use crate::interfaces::http::AppState;

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub query: String,
}

#[derive(Debug, Deserialize)]
pub struct RangeQueryRequest {
    pub query: String,
    pub start: i64,
    pub end: i64,
    pub step: String,
}

/// Get cluster metrics
#[get("/api/metrics")]
pub async fn get_cluster_metrics(
    data: web::Data<AppState>,
) -> impl Responder {
    let repo = match data.get_prometheus_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Prometheus repository not available"
            })),
    };

    let use_case = GetClusterMetricsUseCase::new(repo);

    match use_case.execute().await {
        Ok(metrics) => HttpResponse::Ok().json(metrics),
        Err(e) => e.error_response(),
    }
}

/// Query Prometheus metric
#[get("/api/prometheus/query")]
pub async fn query_metric(
    data: web::Data<AppState>,
    query: web::Query<QueryRequest>,
) -> impl Responder {
    let repo = match data.get_prometheus_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Prometheus repository not available"
            })),
    };

    let use_case = QueryMetricUseCase::new(repo);

    match use_case.execute(&query.query).await {
        Ok(value) => HttpResponse::Ok().json(serde_json::json!({ "value": value })),
        Err(e) => e.error_response(),
    }
}

/// Query raw Prometheus data
#[get("/api/prometheus/query_raw")]
pub async fn query_raw(
    data: web::Data<AppState>,
    query: web::Query<QueryRequest>,
) -> impl Responder {
    let repo = match data.get_prometheus_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Prometheus repository not available"
            })),
    };

    let use_case = QueryRawUseCase::new(repo);

    match use_case.execute(&query.query).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => e.error_response(),
    }
}

/// Query Prometheus range
#[get("/api/prometheus/range")]
pub async fn query_range(
    data: web::Data<AppState>,
    query: web::Query<RangeQueryRequest>,
) -> impl Responder {
    let repo = match data.get_prometheus_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "Prometheus repository not available"
            })),
    };

    let use_case = QueryRangeUseCase::new(repo);

    match use_case.execute(&query.query, query.start, query.end, &query.step).await {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => e.error_response(),
    }
}
