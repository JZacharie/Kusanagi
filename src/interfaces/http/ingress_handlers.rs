//! Ingress HTTP Handlers
//!
//! HTTP handlers for ingress operations.

use std::sync::Arc;
use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;


use crate::application::use_cases::ingress_use_cases::*;
use crate::application::mappers::IngressMapper;

// use crate::interfaces::http::AppState; // Commented out for compilation

#[derive(Debug, Deserialize)]
pub struct ListIngressesQuery {
    pub namespace: Option<String>,
}

/// List all ingresses
#[get("/api/ingresses")]
pub async fn list_ingresses(
    // data: web::Data<AppState> // Commented out for compilation,
    query: web::Query<ListIngressesQuery>,
) -> impl Responder {
    let use_case = ListIngressesUseCase::new(// Arc::clone(// &data.k8s_repo));

    match use_case.execute(query.namespace.as_deref()).await {
        Ok(ingresses) => {
            let dtos = IngressMapper::to_dto_list(ingresses);
            HttpResponse::Ok().json(dtos)
        }
        Err(e) => e.error_response(),
    }
}

/// Get ingress statistics
#[get("/api/ingresses/stats")]
pub async fn get_ingress_stats(
    // data: web::Data<AppState> // Commented out for compilation,
    query: web::Query<ListIngressesQuery>,
) -> impl Responder {
    let use_case = GetIngressStatsUseCase::new(// Arc::clone(// &data.k8s_repo));

    match use_case.execute(query.namespace.as_deref()).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(e) => e.error_response(),
    }
}

/// Get ingress details by namespace and name
#[derive(Deserialize)]
pub struct GetIngressPath {
    pub namespace: String,
    pub name: String,
}

#[get("/api/ingresses/{namespace}/{name}")]
pub async fn get_ingress_details(
    // data: web::Data<AppState> // Commented out for compilation,
    path: web::Path<GetIngressPath>,
) -> impl Responder {
    let use_case = GetIngressDetailsUseCase::new(// Arc::clone(// &data.k8s_repo));

    match use_case.execute(&path.namespace, &path.name).await {
        Ok(ingress) => HttpResponse::Ok().json(IngressMapper::to_dto(ingress)),
        Err(e) => e.error_response(),
    }
}
