//! Low priority HTTP handlers - Part 2 (9 modules)

use actix_web::{web, HttpResponse, Result as ActixResult};
use crate::application::use_cases::low_priority_use_cases_part2::*;
use std::sync::Arc;

// Notifications Handlers
pub async fn send_notification(
    notifications_use_cases: web::Data<Arc<NotificationsUseCases>>,
    body: web::Json<serde_json::Value>,
) -> ActixResult<HttpResponse> {
    let notification = crate::domain::ports::low_priority_ports_part2::Notification {
        id: uuid::Uuid::new_v4().to_string(),
        title: body.get("title").and_then(|t| t.as_str()).unwrap_or("Notification").to_string(),
        message: body.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string(),
        severity: body.get("severity").and_then(|s| s.as_str()).unwrap_or("info").to_string(),
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        channels: vec!["default".to_string()],
    };
    
    match notifications_use_cases.send_notification(&notification).await {
        Ok(_) => Ok(HttpResponse::Ok().json("Notification sent")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to send notification")),
    }
}

pub async fn get_notification_history(
    notifications_use_cases: web::Data<Arc<NotificationsUseCases>>,
) -> ActixResult<HttpResponse> {
    match notifications_use_cases.get_notification_history().await {
        Ok(history) => Ok(HttpResponse::Ok().json(history)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get notification history")),
    }
}

// Telemetry Handlers
pub async fn collect_metrics(
    telemetry_use_cases: web::Data<Arc<TelemetryUseCases>>,
) -> ActixResult<HttpResponse> {
    match telemetry_use_cases.collect_metrics().await {
        Ok(data) => Ok(HttpResponse::Ok().json(data)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to collect metrics")),
    }
}

pub async fn send_telemetry(
    telemetry_use_cases: web::Data<Arc<TelemetryUseCases>>,
    _body: web::Json<serde_json::Value>,
) -> ActixResult<HttpResponse> {
    let telemetry_data = crate::domain::ports::low_priority_ports_part2::TelemetryData {
        metrics: std::collections::HashMap::new(),
        events: vec!["telemetry_sent".to_string()],
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
    };
    
    match telemetry_use_cases.send_telemetry(&telemetry_data).await {
        Ok(_) => Ok(HttpResponse::Ok().json("Telemetry sent")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to send telemetry")),
    }
}

// Translation Handlers
pub async fn translate_text(
    translation_use_cases: web::Data<Arc<TranslationUseCases>>,
    body: web::Json<serde_json::Value>,
) -> ActixResult<HttpResponse> {
    let text = body.get("text").and_then(|t| t.as_str()).unwrap_or("");
    let target_lang = body.get("target_lang").and_then(|l| l.as_str()).unwrap_or("en");
    
    match translation_use_cases.translate_text(text, target_lang).await {
        Ok(translated) => Ok(HttpResponse::Ok().json(serde_json::json!({"translated_text": translated}))),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to translate text")),
    }
}

pub async fn get_supported_languages(
    translation_use_cases: web::Data<Arc<TranslationUseCases>>,
) -> ActixResult<HttpResponse> {
    match translation_use_cases.get_supported_languages().await {
        Ok(languages) => Ok(HttpResponse::Ok().json(languages)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get supported languages")),
    }
}

// LLM Handlers
pub async fn generate_response(
    llm_use_cases: web::Data<Arc<LlmUseCases>>,
    body: web::Json<serde_json::Value>,
) -> ActixResult<HttpResponse> {
    let prompt = body.get("prompt").and_then(|p| p.as_str()).unwrap_or("");
    let model = body.get("model").and_then(|m| m.as_str()).unwrap_or("default");
    
    match llm_use_cases.generate_response(prompt, model).await {
        Ok(response) => Ok(HttpResponse::Ok().json(serde_json::json!({"response": response}))),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to generate response")),
    }
}

pub async fn get_available_models(
    llm_use_cases: web::Data<Arc<LlmUseCases>>,
) -> ActixResult<HttpResponse> {
    match llm_use_cases.get_available_models().await {
        Ok(models) => Ok(HttpResponse::Ok().json(models)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get available models")),
    }
}

// Events Handlers
pub async fn get_cluster_events(
    events_use_cases: web::Data<Arc<EventsUseCases>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> ActixResult<HttpResponse> {
    let namespace = query.get("namespace").map(|s| s.as_str());
    
    match events_use_cases.get_cluster_events(namespace).await {
        Ok(events) => Ok(HttpResponse::Ok().json(events)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get cluster events")),
    }
}

pub async fn watch_events(
    events_use_cases: web::Data<Arc<EventsUseCases>>,
) -> ActixResult<HttpResponse> {
    match events_use_cases.watch_events().await {
        Ok(_) => Ok(HttpResponse::Ok().json("Watching events")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to watch events")),
    }
}

// Cluster Handlers
pub async fn get_cluster_info(
    cluster_use_cases: web::Data<Arc<ClusterUseCases>>,
) -> ActixResult<HttpResponse> {
    match cluster_use_cases.get_cluster_info().await {
        Ok(info) => Ok(HttpResponse::Ok().json(info)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get cluster info")),
    }
}

pub async fn scale_cluster(
    cluster_use_cases: web::Data<Arc<ClusterUseCases>>,
    body: web::Json<serde_json::Value>,
) -> ActixResult<HttpResponse> {
    let node_count = body.get("node_count").and_then(|n| n.as_u64()).unwrap_or(3) as u32;
    
    match cluster_use_cases.scale_cluster(node_count).await {
        Ok(_) => Ok(HttpResponse::Ok().json("Cluster scaled")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to scale cluster")),
    }
}

// Storage Handlers
pub async fn list_storage_classes(
    storage_use_cases: web::Data<Arc<StorageUseCases>>,
) -> ActixResult<HttpResponse> {
    match storage_use_cases.list_storage_classes().await {
        Ok(classes) => Ok(HttpResponse::Ok().json(classes)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to list storage classes")),
    }
}

pub async fn get_persistent_volumes(
    storage_use_cases: web::Data<Arc<StorageUseCases>>,
) -> ActixResult<HttpResponse> {
    match storage_use_cases.get_persistent_volumes().await {
        Ok(volumes) => Ok(HttpResponse::Ok().json(volumes)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get persistent volumes")),
    }
}

// Doctor Handlers
pub async fn run_diagnostics(
    doctor_use_cases: web::Data<Arc<DoctorUseCases>>,
) -> ActixResult<HttpResponse> {
    match doctor_use_cases.run_diagnostics().await {
        Ok(report) => Ok(HttpResponse::Ok().json(report)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to run diagnostics")),
    }
}

pub async fn fix_issues(
    doctor_use_cases: web::Data<Arc<DoctorUseCases>>,
    body: web::Json<serde_json::Value>,
) -> ActixResult<HttpResponse> {
    let issues: Vec<String> = body.get("issues")
        .and_then(|i| i.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    
    match doctor_use_cases.fix_issues(&issues).await {
        Ok(_) => Ok(HttpResponse::Ok().json("Issues fixed")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to fix issues")),
    }
}

// ArgoCD Handlers
pub async fn list_argocd_applications(
    argocd_use_cases: web::Data<Arc<ArgoCdUseCases>>,
) -> ActixResult<HttpResponse> {
    match argocd_use_cases.list_applications().await {
        Ok(apps) => Ok(HttpResponse::Ok().json(apps)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to list ArgoCD applications")),
    }
}

pub async fn sync_argocd_application(
    argocd_use_cases: web::Data<Arc<ArgoCdUseCases>>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let app_name = path.into_inner();
    
    match argocd_use_cases.sync_application(&app_name).await {
        Ok(_) => Ok(HttpResponse::Ok().json("Application synced")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to sync application")),
    }
}
