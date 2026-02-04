//! Cluster HTTP Handlers
//!
//! HTTP handlers for cluster operations.

use std::sync::Arc;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};


use crate::application::use_cases::cluster_use_cases::*;
use crate::application::mappers::ClusterMapper;

// use crate::interfaces::http::AppState; // Commented out for compilation

/// Get cluster overview
#[get("/api/cluster/overview")]
pub async fn get_cluster_overview(
    // data: web::Data<AppState> // Commented out for compilation,
) -> impl Responder {
    let use_case = GetClusterOverviewUseCase::new(// Arc::clone(// &data.k8s_repo));

    match use_case.execute().await {
        Ok(overview) => HttpResponse::Ok().json(ClusterMapper::to_overview_dto(overview)),
        Err(e) => e.error_response(),
    }
}

/// Get empty namespaces
#[get("/api/cluster/empty-namespaces")]
pub async fn get_empty_namespaces(
    // data: web::Data<AppState> // Commented out for compilation,
) -> impl Responder {
    let use_case = GetEmptyNamespacesUseCase::new(// Arc::clone(// &data.k8s_repo));

    match use_case.execute().await {
        Ok(namespaces) => HttpResponse::Ok().json(namespaces),
        Err(e) => e.error_response(),
    }
}

/// Get cluster statistics
#[get("/api/cluster/stats")]
pub async fn get_cluster_stats(
    // data: web::Data<AppState> // Commented out for compilation,
) -> impl Responder {
    let use_case = GetClusterStatsUseCase::new(// Arc::clone(// &data.k8s_repo));

    match use_case.execute().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => e.error_response(),
    }
}
