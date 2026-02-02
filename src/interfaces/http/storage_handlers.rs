//! Storage HTTP Handlers
//!
//! HTTP handlers for storage operations.

use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;
use std::sync::Arc;

use crate::application::use_cases::storage_use_cases::*;
use crate::application::mappers::StorageMapper;
use crate::domain::ports::KubernetesRepository;
use crate::interfaces::http::{AppState, ErrorResponse};

/// Get storage information
#[get("/api/storage")]
pub async fn get_storage_info(
    data: web::Data<AppState>,
) -> impl Responder {
    let use_case = GetStorageInfoUseCase::new(Arc::clone(&data.k8s_repo));

    match use_case.execute().await {
        Ok(info) => HttpResponse::Ok().json(StorageMapper::to_dto(info)),
        Err(e) => e.error_response(),
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
        Err(e) => e.error_response(),
    }
}
