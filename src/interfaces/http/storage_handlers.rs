//! Storage HTTP Handlers
//!
//! HTTP handlers for storage operations.

use std::sync::Arc;
use actix_web::{get, web, HttpResponse, Responder};



use crate::application::use_cases::storage_use_cases::*;
use crate::application::mappers::StorageMapper;

use crate::interfaces::http::AppState;

/// Get storage information
#[get("/api/storage")]
pub async fn get_storage_info(
    data: web::Data<AppState>,
) -> impl Responder {
    let use_case = GetStorageInfoUseCase::new(Arc::clone(&data.k8s_repo));

    match use_case.execute().await {
        Ok(info) => HttpResponse::Ok().json(StorageMapper::to_dto(info)),
        Err(e) => HttpResponse::Ok().json(serde_json::json!({
            "pvc_count": 0,
            "pvcs": [],
            "pvc_total_capacity": "0 Gi",
            "_warning": format!("Storage error: {}", e)
        })),
    }
}

/// Get storage statistics
#[get("/api/storage/stats")]
pub async fn get_storage_stats(
    data: web::Data<AppState>,
) -> impl Responder {
    let use_case = GetStorageStatsUseCase::new(Arc::clone(&data.k8s_repo));

    match use_case.execute().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => HttpResponse::Ok().json(serde_json::json!({
            "total_pvcs": 0,
            "bound_pvcs": 0,
            "pending_pvcs": 0,
            "total_capacity_bytes": 0,
            "_warning": format!("Storage stats error: {}", e)
        })),
    }
}
