//! ArgoCD HTTP Handlers
//!
//! HTTP handlers for ArgoCD operations.

use std::sync::Arc;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;


use crate::application::use_cases::argocd_use_cases::*;

use crate::interfaces::http::AppState;

#[derive(Debug, Deserialize)]
pub struct SyncRequest {
    pub name: String,
}

/// List all ArgoCD applications
pub async fn list_applications(
    data: web::Data<AppState>,
) -> impl Responder {
    let repo = match data.get_argocd_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "ArgoCD repository not available"
            })),
    };

    let use_case = GetArgoCdApplicationsUseCase::new(repo);
    
    match use_case.execute().await {
        Ok(apps) => HttpResponse::Ok().json(apps),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

/// Get application status by name
#[derive(Deserialize)]
pub struct AppStatusPath {
    pub name: String,
}

pub async fn get_application_status(
    data: web::Data<AppState>,
    path: web::Path<AppStatusPath>,
) -> impl Responder {
    let repo = match data.get_argocd_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "ArgoCD repository not available"
            })),
    };

    let use_case = GetApplicationStatusUseCase::new(repo);
    
    match use_case.execute(&path.name).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

/// Sync an application
pub async fn sync_application(
    data: web::Data<AppState>,
    path: web::Path<AppStatusPath>,
) -> impl Responder {
    let repo = match data.get_argocd_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "ArgoCD repository not available"
            })),
    };

    let use_case = SyncApplicationUseCase::new(repo);
    
    match use_case.execute(&path.name).await {
        Ok(()) => HttpResponse::Ok().json(serde_json::json!({
            "message": format!("Sync triggered for application {}", path.name)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

/// Get application details
pub async fn get_application_details(
    data: web::Data<AppState>,
    path: web::Path<AppStatusPath>,
) -> impl Responder {
    let repo = match data.get_argocd_repo() {
        Some(repo) => repo,
        None => return HttpResponse::ServiceUnavailable()
            .json(serde_json::json!({
                "error": "ArgoCD repository not available"
            })),
    };

    let use_case = GetApplicationDetailsUseCase::new(repo);
    
    match use_case.execute(&path.name).await {
        Ok(details) => HttpResponse::Ok().json(details),
        Err(e) => HttpResponse::NotFound().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}
