// HTTP Controllers - Interface Layer
use crate::application::use_cases::ClusterUseCase;
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

pub async fn get_cluster_info(cluster_use_case: web::Data<Arc<ClusterUseCase>>) -> impl Responder {
    match cluster_use_case.get_cluster_status().await {
        Ok(cluster) => HttpResponse::Ok().json(cluster),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_nodes_info(cluster_use_case: web::Data<Arc<ClusterUseCase>>) -> impl Responder {
    match cluster_use_case.list_nodes().await {
        Ok(nodes) => HttpResponse::Ok().json(nodes),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": e.to_string()
        })),
    }
}
