//! Service HTTP Handlers
//!
//! HTTP handlers for service operations.

use std::sync::Arc;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;


use crate::application::use_cases::service_use_cases::*;
use crate::application::mappers::ServiceMapper;

// use crate::interfaces::http::AppState; // Commented out for compilation

#[derive(Debug, Deserialize)]
pub struct ListServicesQuery {
    pub namespace: Option<String>,
}

/// List all services
#[get("/api/services")]
pub async fn list_services(
    // data: web::Data<AppState> // Commented out for compilation,
    query: web::Query<ListServicesQuery>,
) -> impl Responder {
    let use_case = ListServicesUseCase::new(// Arc::clone(// &data.k8s_repo));

    match use_case.execute(query.namespace.as_deref()).await {
        Ok(services) => {
            let dtos = ServiceMapper::to_dto_list(services);
            HttpResponse::Ok().json(dtos)
        }
        Err(e) => e.error_response(),
    }
}

/// Get service statistics
#[get("/api/services/stats")]
pub async fn get_service_stats(
    // data: web::Data<AppState> // Commented out for compilation,
    query: web::Query<ListServicesQuery>,
) -> impl Responder {
    let use_case = GetServiceStatsUseCase::new(// Arc::clone(// &data.k8s_repo));

    match use_case.execute(query.namespace.as_deref()).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => e.error_response(),
    }
}

/// Get service details by namespace and name
#[derive(Deserialize)]
pub struct GetServicePath {
    pub namespace: String,
    pub name: String,
}

#[get("/api/services/{namespace}/{name}")]
pub async fn get_service_details(
    // data: web::Data<AppState> // Commented out for compilation,
    path: web::Path<GetServicePath>,
) -> impl Responder {
    let use_case = GetServiceDetailsUseCase::new(// Arc::clone(// &data.k8s_repo));

    match use_case.execute(&path.namespace, &path.name).await {
        Ok(service) => HttpResponse::Ok().json(ServiceMapper::to_dto(service)),
        Err(e) => e.error_response(),
    }
}
