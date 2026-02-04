//! Node Metrics HTTP Handlers
//!
//! HTTP handlers for node metrics with disk usage.

use std::sync::Arc;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;


use crate::application::use_cases::node_metrics_use_cases::*;
use crate::application::mappers::NodeMapper;

// use crate::interfaces::http::AppState; // Commented out for compilation

#[derive(Debug, Deserialize)]
pub struct NodeNamePath {
    pub name: String,
}

/// List all nodes with disk metrics
#[get("/api/nodes/with-metrics")]
pub async fn list_nodes_with_disk_metrics(
    // data: web::Data<AppState> // Commented out for compilation,
) -> impl Responder {
    let use_case = GetNodesWithDiskMetricsUseCase::new(
        // Arc::clone(// &data.k8s_repo),
        // Arc::clone(&data.metrics_repo),
    );

    match use_case.execute().await {
        Ok(nodes) => {
            let dtos: Vec<_> = nodes.into_iter().map(NodeMapper::to_dto).collect();
            HttpResponse::Ok().json(dtos)
        }
        Err(e) => e.error_response(),
    }
}

/// Get disk metrics for a specific node
#[get("/api/nodes/{name}/disk")]
pub async fn get_node_disk_metrics(
    // data: web::Data<AppState> // Commented out for compilation,
    path: web::Path<NodeNamePath>,
) -> impl Responder {
    let use_case = GetNodeDiskUsageUseCase::new(// Arc::clone(&data.metrics_repo));

    match use_case.execute(&path.name).await {
        Ok(metrics) => HttpResponse::Ok().json(metrics),
        Err(e) => e.error_response(),
    }
}

/// Get cluster disk summary
#[get("/api/nodes/disk-summary")]
pub async fn get_cluster_disk_summary(
    // data: web::Data<AppState> // Commented out for compilation,
) -> impl Responder {
    let use_case = GetClusterDiskSummaryUseCase::new(// Arc::clone(&data.metrics_repo));

    match use_case.execute().await {
        Ok(summary) => HttpResponse::Ok().json(summary),
        Err(e) => e.error_response(),
    }
}
