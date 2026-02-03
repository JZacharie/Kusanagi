//! HTTP handlers (REST API)
//!
//! These handlers use application services and use cases
//! to fulfill HTTP requests.

use actix_web::{get, web, HttpResponse, Responder, ResponseError};
use serde::Deserialize;


use crate::application::use_cases::*;

use crate::domain::ports::*;

use std::sync::Arc;

mod event_handlers;
mod node_handlers;
mod argocd_handlers;
mod storage_handlers;
mod service_handlers;
mod ingress_handlers;
mod cluster_handlers;
mod prometheus_handlers;
mod backup_handlers;
mod security_handlers;
mod alert_handlers;
mod chat_handlers;
mod node_metrics_handlers;

pub use event_handlers::*;
pub use node_handlers::*;
pub use argocd_handlers::*;
pub use storage_handlers::*;
pub use service_handlers::*;
pub use ingress_handlers::*;
pub use cluster_handlers::*;
pub use prometheus_handlers::*;
pub use backup_handlers::*;
pub use security_handlers::*;
pub use alert_handlers::*;
pub use chat_handlers::*;
pub use node_metrics_handlers::*;

/// Application state shared across handlers
pub struct AppState {
    pub k8s_repo: Arc<dyn KubernetesRepository>,
    pub metrics_repo: Arc<dyn MetricsRepository>,
    pub argocd_repo: Option<Arc<dyn crate::domain::ports::argocd_port::ArgoCdRepository>>,
    pub prometheus_repo: Option<Arc<dyn crate::domain::ports::prometheus_port::PrometheusRepository>>,
    pub backup_repo: Option<Arc<dyn crate::domain::ports::backup_port::BackupRepository>>,
    pub security_repo: Option<Arc<dyn crate::domain::ports::security_port::SecurityRepository>>,
    pub vulnerability_scanner: Option<Arc<dyn crate::domain::ports::security_port::VulnerabilityScanner>>,
    pub ai_enrichment: Option<Arc<dyn crate::domain::ports::security_port::AiEnrichmentService>>,
    pub alert_repo: Option<Arc<dyn crate::domain::ports::alert_port::AlertRepository>>,
    pub chat_service: Option<Arc<dyn crate::domain::ports::chat_port::ChatService>>,
    pub chat_history_repo: Option<Arc<dyn crate::domain::ports::chat_port::ChatHistoryRepository>>,
}

impl AppState {
    /// Get ArgoCD repository if available
    pub fn get_argocd_repo(&self) -> Option<Arc<dyn crate::domain::ports::argocd_port::ArgoCdRepository>> {
        self.argocd_repo.clone()
    }

    /// Get Prometheus repository if available
    pub fn get_prometheus_repo(&self) -> Option<Arc<dyn crate::domain::ports::prometheus_port::PrometheusRepository>> {
        self.prometheus_repo.clone()
    }

    /// Get Backup repository if available
    pub fn get_backup_repo(&self) -> Option<Arc<dyn crate::domain::ports::backup_port::BackupRepository>> {
        self.backup_repo.clone()
    }

    /// Get Security repository if available
    pub fn get_security_repo(&self) -> Option<Arc<dyn crate::domain::ports::security_port::SecurityRepository>> {
        self.security_repo.clone()
    }

    /// Get Vulnerability scanner if available
    pub fn get_vulnerability_scanner(&self) -> Option<Arc<dyn crate::domain::ports::security_port::VulnerabilityScanner>> {
        self.vulnerability_scanner.clone()
    }

    /// Get AI enrichment service if available
    pub fn get_ai_enrichment_service(&self) -> Option<Arc<dyn crate::domain::ports::security_port::AiEnrichmentService>> {
        self.ai_enrichment.clone()
    }

    /// Get Alert repository if available
    pub fn get_alert_repo(&self) -> Option<Arc<dyn crate::domain::ports::alert_port::AlertRepository>> {
        self.alert_repo.clone()
    }

    /// Get Chat service if available
    pub fn get_chat_service(&self) -> Option<Arc<dyn crate::domain::ports::chat_port::ChatService>> {
        self.chat_service.clone()
    }

    /// Get Chat history repository if available
    pub fn get_chat_history_repo(&self) -> Option<Arc<dyn crate::domain::ports::chat_port::ChatHistoryRepository>> {
        self.chat_history_repo.clone()
    }
}

/// Configure HTTP routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(health_check)
        // Cluster
        .service(get_cluster_overview)
        .service(get_empty_namespaces)
        .service(get_cluster_stats)
        // Nodes
        .service(list_nodes)
        .service(get_nodes_status)
        .service(get_node_details)
        .service(is_node_ready)
        // Node Metrics (with disk usage)
        .service(list_nodes_with_disk_metrics)
        .service(get_node_disk_metrics)
        .service(get_cluster_disk_summary)
        // Pods
        .service(list_pods)
        .service(get_pod_details)
        // Events
        .service(list_events)
        .service(list_warning_events)
        .service(get_event_stats)
        // Namespaces
        .service(list_namespaces)
        // Services
        .service(list_services)
        .service(get_service_stats)
        .service(get_service_details)
        // Ingresses
        .service(list_ingresses)
        .service(get_ingress_stats)
        .service(get_ingress_details)
        // Storage
        .service(get_storage_info)
        .service(get_storage_stats)
        // Prometheus
        .service(get_cluster_metrics)
        .service(query_metric)
        .service(query_raw)
        .service(query_range)
        // Backups
        .service(get_backup_status)
        .service(get_backup_stats)
        .service(list_cronjobs)
        .service(trigger_backup)
        // Security
        .service(list_security_reports)
        .service(get_security_summary)
        .service(get_enriched_report)
        .service(enrich_security_report)
        .service(run_security_enrichment)
        // Alerts
        .service(get_active_alerts)
        .service(get_cached_alerts)
        .service(get_alert_stats)
        .service(get_alert)
        .service(silence_alert)
        // Chat
        .service(process_chat_message)
        .service(handle_chat_command)
        .service(query_ai)
        .service(get_chat_history)
        .service(clear_chat_history);
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

#[get("/api/cluster/overview")]
async fn get_cluster_overview(data: web::Data<AppState>) -> impl Responder {
    let use_case = ClusterApplicationService::new(
        Arc::clone(&data.k8s_repo),
        Arc::clone(&data.metrics_repo),
    );

    match use_case.get_cluster_overview().await {
        Ok(dto) => HttpResponse::Ok().json(dto),
        Err(e) => e.error_response(),
    }
}

#[get("/api/nodes")]
async fn list_nodes(data: web::Data<AppState>) -> impl Responder {
    let use_case = NodeApplicationService::new(Arc::clone(&data.k8s_repo));

    match use_case.list_nodes().await {
        Ok(dtos) => HttpResponse::Ok().json(dtos),
        Err(e) => e.error_response(),
    }
}

#[derive(Deserialize)]
struct ListPodsQuery {
    namespace: Option<String>,
}

#[get("/api/pods")]
async fn list_pods(
    data: web::Data<AppState>,
    query: web::Query<ListPodsQuery>,
) -> impl Responder {
    let use_case = ListPodsUseCase::new(Arc::clone(&data.k8s_repo));

    match use_case.execute(query.namespace.as_deref()).await {
        Ok(dtos) => HttpResponse::Ok().json(dtos),
        Err(e) => e.error_response(),
    }
}

#[derive(Deserialize)]
struct GetPodPath {
    namespace: String,
    name: String,
}

#[get("/api/pods/{namespace}/{name}")]
async fn get_pod_details(
    data: web::Data<AppState>,
    path: web::Path<GetPodPath>,
) -> impl Responder {
    let use_case = GetPodDetailsUseCase::new(Arc::clone(&data.k8s_repo));

    match use_case.execute(&path.namespace, &path.name).await {
        Ok(dto) => HttpResponse::Ok().json(dto),
        Err(e) => e.error_response(),
    }
}

#[derive(Deserialize)]
struct ListEventsQuery {
    namespace: Option<String>,
    event_type: Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
}

#[get("/api/events")]
async fn list_events(
    data: web::Data<AppState>,
    query: web::Query<ListEventsQuery>,
) -> impl Responder {
    let use_case = GetRecentEventsUseCase::new(Arc::clone(&data.k8s_repo));

    match use_case.execute(
        query.namespace.as_deref(),
        query.event_type.as_deref(),
        query.page.unwrap_or(1),
        query.per_page.unwrap_or(20),
    ).await {
        Ok(dto) => HttpResponse::Ok().json(dto),
        Err(e) => e.error_response(),
    }
}

#[get("/api/namespaces")]
async fn list_namespaces(data: web::Data<AppState>) -> impl Responder {
    let use_case = NamespaceApplicationService::new(Arc::clone(&data.k8s_repo));

    match use_case.list_namespaces().await {
        Ok(dtos) => HttpResponse::Ok().json(dtos),
        Err(e) => e.error_response(),
    }
}

#[get("/api/services")]
async fn list_services(data: web::Data<AppState>) -> impl Responder {
    let use_case = ServiceApplicationService::new(Arc::clone(&data.k8s_repo));

    match use_case.list_services(None).await {
        Ok(dtos) => HttpResponse::Ok().json(dtos),
        Err(e) => e.error_response(),
    }
}

#[get("/api/storage")]
async fn get_storage_info(data: web::Data<AppState>) -> impl Responder {
    let use_case = StorageApplicationService::new(Arc::clone(&data.k8s_repo));

    match use_case.get_storage_info().await {
        Ok(dto) => HttpResponse::Ok().json(dto),
        Err(e) => e.error_response(),
    }
}

/// Error response structure
#[derive(serde::Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl From<KusanagiError> for ErrorResponse {
    fn from(err: KusanagiError) -> Self {
        Self {
            error: err.to_string(),
            message: err.user_message(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(
            actix_web::App::new().configure(configure_routes)
        ).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }
}
