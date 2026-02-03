//! Node HTTP Handlers
//!
//! HTTP handlers for node operations.

use std::sync::Arc;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;


use crate::application::use_cases::node_use_cases::*;
use crate::application::mappers::NodeMapper;

use crate::interfaces::http::AppState;

#[derive(Debug, Deserialize)]
pub struct ListNodesQuery {
    pub status: Option<String>,
}

/// List all nodes
#[get("/api/nodes")]
pub async fn list_nodes(
    data: web::Data<AppState>,
    _query: web::Query<ListNodesQuery>,
) -> impl Responder {
    let use_case = GetNodesUseCase::new(Arc::clone(&data.k8s_repo));

    match use_case.execute().await {
        Ok(nodes) => {
            let dtos: Vec<_> = nodes.into_iter().map(NodeMapper::to_dto).collect();
            HttpResponse::Ok().json(dtos)
        }
        Err(e) => e.error_response(),
    }
}

/// Get node status summary
#[get("/api/nodes/status")]
pub async fn get_nodes_status(
    data: web::Data<AppState>,
) -> impl Responder {
    let use_case = GetNodesStatusUseCase::new(Arc::clone(&data.k8s_repo));

    match use_case.execute().await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => e.error_response(),
    }
}

/// Get node details by name
#[derive(Deserialize)]
pub struct GetNodePath {
    pub name: String,
}

#[get("/api/nodes/{name}")]
pub async fn get_node_details(
    data: web::Data<AppState>,
    path: web::Path<GetNodePath>,
) -> impl Responder {
    let use_case = GetNodeDetailsUseCase::new(Arc::clone(&data.k8s_repo));

    match use_case.execute(&path.name).await {
        Ok(node) => HttpResponse::Ok().json(NodeMapper::to_dto(node)),
        Err(e) => e.error_response(),
    }
}

/// Check if a specific node is ready
#[get("/api/nodes/{name}/ready")]
pub async fn is_node_ready(
    data: web::Data<AppState>,
    path: web::Path<GetNodePath>,
) -> impl Responder {
    let use_case = IsNodeReadyUseCase::new(Arc::clone(&data.k8s_repo));

    match use_case.execute(&path.name).await {
        Ok(is_ready) => HttpResponse::Ok().json(serde_json::json!({
            "name": path.name,
            "ready": is_ready
        })),
        Err(e) => e.error_response(),
    }
}
