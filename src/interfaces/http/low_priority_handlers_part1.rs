//! Low priority HTTP handlers - Part 1 (9 modules)

use actix_web::{web, HttpResponse, Result as ActixResult};
use crate::application::use_cases::low_priority_use_cases_part1::*;
use std::sync::Arc;

// Services Handlers
pub async fn list_services(
    services_use_cases: web::Data<Arc<ServicesUseCases>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> ActixResult<HttpResponse> {
    let namespace = query.get("namespace").map(|s| s.as_str());
    
    match services_use_cases.list_services(namespace).await {
        Ok(services) => Ok(HttpResponse::Ok().json(services)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to list services")),
    }
}

pub async fn get_service_details(
    services_use_cases: web::Data<Arc<ServicesUseCases>>,
    path: web::Path<(String, String)>,
) -> ActixResult<HttpResponse> {
    let (namespace, name) = path.into_inner();
    
    match services_use_cases.get_service_details(&namespace, &name).await {
        Ok(details) => Ok(HttpResponse::Ok().json(details)),
        Err(_) => Ok(HttpResponse::NotFound().json("Service not found")),
    }
}

// Ingress Handlers
pub async fn list_ingresses(
    ingress_use_cases: web::Data<Arc<IngressUseCases>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> ActixResult<HttpResponse> {
    let namespace = query.get("namespace").map(|s| s.as_str());
    
    match ingress_use_cases.list_ingresses(namespace).await {
        Ok(ingresses) => Ok(HttpResponse::Ok().json(ingresses)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to list ingresses")),
    }
}

pub async fn get_ingress_rules(
    ingress_use_cases: web::Data<Arc<IngressUseCases>>,
    path: web::Path<(String, String)>,
) -> ActixResult<HttpResponse> {
    let (namespace, name) = path.into_inner();
    
    match ingress_use_cases.get_ingress_rules(&namespace, &name).await {
        Ok(rules) => Ok(HttpResponse::Ok().json(rules)),
        Err(_) => Ok(HttpResponse::NotFound().json("Ingress not found")),
    }
}

// Alertmanager Handlers
pub async fn get_alerts(
    alertmanager_use_cases: web::Data<Arc<AlertmanagerUseCases>>,
) -> ActixResult<HttpResponse> {
    match alertmanager_use_cases.get_alerts().await {
        Ok(alerts) => Ok(HttpResponse::Ok().json(alerts)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get alerts")),
    }
}

pub async fn silence_alert(
    alertmanager_use_cases: web::Data<Arc<AlertmanagerUseCases>>,
    path: web::Path<String>,
    body: web::Json<serde_json::Value>,
) -> ActixResult<HttpResponse> {
    let alert_id = path.into_inner();
    let duration = body.get("duration").and_then(|d| d.as_u64()).unwrap_or(3600);
    
    match alertmanager_use_cases.silence_alert(&alert_id, duration).await {
        Ok(_) => Ok(HttpResponse::Ok().json("Alert silenced")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to silence alert")),
    }
}

// Quota Handlers
pub async fn get_resource_quotas(
    quota_use_cases: web::Data<Arc<QuotaUseCases>>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let namespace = path.into_inner();
    
    match quota_use_cases.get_resource_quotas(&namespace).await {
        Ok(quotas) => Ok(HttpResponse::Ok().json(quotas)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get quotas")),
    }
}

pub async fn get_quota_usage(
    quota_use_cases: web::Data<Arc<QuotaUseCases>>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let namespace = path.into_inner();
    
    match quota_use_cases.get_quota_usage(&namespace).await {
        Ok(usage) => Ok(HttpResponse::Ok().json(usage)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get quota usage")),
    }
}

// Setup Handlers
pub async fn initialize_cluster(
    setup_use_cases: web::Data<Arc<SetupUseCases>>,
    body: web::Json<serde_json::Value>,
) -> ActixResult<HttpResponse> {
    // Simplified config parsing
    let config = crate::domain::ports::low_priority_ports_part1::ClusterConfig {
        name: body.get("name").and_then(|n| n.as_str()).unwrap_or("default").to_string(),
        version: body.get("version").and_then(|v| v.as_str()).unwrap_or("1.0").to_string(),
        node_count: body.get("node_count").and_then(|n| n.as_u64()).unwrap_or(3) as u32,
        features: vec!["basic".to_string()],
    };
    
    match setup_use_cases.initialize_cluster(&config).await {
        Ok(_) => Ok(HttpResponse::Ok().json("Cluster initialized")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to initialize cluster")),
    }
}

pub async fn get_setup_status(
    setup_use_cases: web::Data<Arc<SetupUseCases>>,
) -> ActixResult<HttpResponse> {
    match setup_use_cases.get_setup_status().await {
        Ok(status) => Ok(HttpResponse::Ok().json(status)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get setup status")),
    }
}

// WebSocket Handlers
pub async fn broadcast_message(
    websocket_use_cases: web::Data<Arc<WebSocketUseCases>>,
    body: web::Json<serde_json::Value>,
) -> ActixResult<HttpResponse> {
    let message = body.get("message").and_then(|m| m.as_str()).unwrap_or("");
    
    match websocket_use_cases.broadcast_message(message).await {
        Ok(_) => Ok(HttpResponse::Ok().json("Message broadcasted")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to broadcast message")),
    }
}

pub async fn get_active_connections(
    websocket_use_cases: web::Data<Arc<WebSocketUseCases>>,
) -> ActixResult<HttpResponse> {
    match websocket_use_cases.get_active_connections().await {
        Ok(count) => Ok(HttpResponse::Ok().json(serde_json::json!({"active_connections": count}))),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get connections")),
    }
}

// Chat Storage Handlers
pub async fn store_conversation(
    chat_storage_use_cases: web::Data<Arc<ChatStorageUseCases>>,
    body: web::Json<serde_json::Value>,
) -> ActixResult<HttpResponse> {
    // Simplified conversation creation
    let conversation = crate::domain::ports::low_priority_ports_part1::Conversation {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: body.get("user_id").and_then(|u| u.as_str()).unwrap_or("anonymous").to_string(),
        messages: vec![],
        timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
    };
    
    match chat_storage_use_cases.store_conversation(&conversation).await {
        Ok(_) => Ok(HttpResponse::Ok().json("Conversation stored")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to store conversation")),
    }
}

pub async fn get_conversation_history(
    chat_storage_use_cases: web::Data<Arc<ChatStorageUseCases>>,
    path: web::Path<String>,
) -> ActixResult<HttpResponse> {
    let user_id = path.into_inner();
    
    match chat_storage_use_cases.get_conversation_history(&user_id).await {
        Ok(history) => Ok(HttpResponse::Ok().json(history)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to get history")),
    }
}

// Export Handlers
pub async fn export_cluster_config(
    export_use_cases: web::Data<Arc<ExportUseCases>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> ActixResult<HttpResponse> {
    let format = query.get("format").map(|s| s.as_str()).unwrap_or("yaml");
    
    match export_use_cases.export_cluster_config(format).await {
        Ok(config) => Ok(HttpResponse::Ok().json(serde_json::json!({"config": config}))),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to export config")),
    }
}

pub async fn export_metrics(
    export_use_cases: web::Data<Arc<ExportUseCases>>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> ActixResult<HttpResponse> {
    let start_time = query.get("start").and_then(|s| s.parse().ok()).unwrap_or(0);
    let end_time = query.get("end").and_then(|s| s.parse().ok()).unwrap_or(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
    
    match export_use_cases.export_metrics(start_time, end_time).await {
        Ok(metrics) => Ok(HttpResponse::Ok().json(serde_json::json!({"metrics": metrics}))),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to export metrics")),
    }
}

// Apps Handlers
pub async fn list_applications(
    apps_use_cases: web::Data<Arc<AppsUseCases>>,
) -> ActixResult<HttpResponse> {
    match apps_use_cases.list_applications().await {
        Ok(apps) => Ok(HttpResponse::Ok().json(apps)),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to list applications")),
    }
}

pub async fn deploy_application(
    apps_use_cases: web::Data<Arc<AppsUseCases>>,
    body: web::Json<serde_json::Value>,
) -> ActixResult<HttpResponse> {
    let app_config = crate::domain::ports::low_priority_ports_part1::ApplicationConfig {
        name: body.get("name").and_then(|n| n.as_str()).unwrap_or("app").to_string(),
        image: body.get("image").and_then(|i| i.as_str()).unwrap_or("nginx").to_string(),
        replicas: body.get("replicas").and_then(|r| r.as_u64()).unwrap_or(1) as u32,
        namespace: body.get("namespace").and_then(|n| n.as_str()).unwrap_or("default").to_string(),
    };
    
    match apps_use_cases.deploy_application(&app_config).await {
        Ok(_) => Ok(HttpResponse::Ok().json("Application deployed")),
        Err(_) => Ok(HttpResponse::InternalServerError().json("Failed to deploy application")),
    }
}
