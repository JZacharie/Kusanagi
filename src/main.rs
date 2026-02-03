use actix_web::{get, post, web, App, HttpServer, Responder, HttpResponse, HttpRequest, ResponseError};
use actix_files::Files;
use serde::Deserialize;
use tracing::{info, error};

pub mod error;
pub use error::{KusanagiError, Result};

pub mod config;
pub mod cache;
pub mod resilience;
pub mod event_bus;
pub mod response;
pub mod middleware;
pub mod metrics;
pub mod validation;
pub mod jobs;
pub mod features;

// Hexagonal Architecture layers
pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod interfaces;

// Legacy modules - being refactored to hexagonal architecture
pub mod legacy;

// Re-export legacy modules for backward compatibility
pub use legacy::notifications;

// Re-export key modules
pub use metrics::custom as metrics_custom;
pub use validation::*;
pub use features::*;

// Re-export middleware for convenience
pub use middleware::{StructuredLogging, RateLimiter, CorrelationId, get_correlation_id};

// Re-export legacy modules for backward compatibility
pub use legacy::{health, llm, doctor};

/// Shared application state
pub struct AppState {
    pub client: kube::Client,
}

#[derive(Deserialize)]
#[allow(hidden_glob_reexports)]
struct SyncRequest {
    app_name: String,
}

#[derive(Deserialize)]
struct EventsQuery {
    event_type: Option<String>,
    page: Option<usize>,
    per_page: Option<usize>,
}

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Kusanagi Agent Controller is healthy")
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

async fn preload_data(client: kube::Client) {
    info!("🚀 Kusanagi Preloading initiated...");
    
    let c = client.clone();
    tokio::spawn(async move {
        if let Err(e) = legacy::argocd::get_argocd_status(&c).await {
            tracing::error!("Preload ArgoCD failed: {}", e);
        } else {
            info!("✅ ArgoCD data preloaded");
        }
    });

    let c = client.clone();
    tokio::spawn(async move {
        if let Err(e) = legacy::nodes::get_nodes_status(&c).await {
            tracing::error!("Preload Nodes failed: {}", e);
        } else {
            info!("✅ Nodes status preloaded");
        }
    });

    let c = client.clone();
    tokio::spawn(async move {
        if let Err(e) = legacy::cluster::get_cluster_overview(&c).await {
            tracing::error!("Preload Cluster Overview failed: {}", e);
        } else {
            info!("✅ Cluster overview preloaded");
        }
    });

    let c = client.clone();
    tokio::spawn(async move {
        if let Err(e) = legacy::storage::get_storage_status(&c).await {
            tracing::error!("Preload Storage failed: {}", e);
        } else {
            info!("✅ Storage status preloaded");
        }
    });
}

#[get("/api/argocd/status")]
async fn argocd_status(data: web::Data<AppState>) -> impl Responder {
    match legacy::argocd::get_argocd_status(&data.client).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => {
            tracing::error!("Failed to get ArgoCD status: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[post("/api/argocd/sync")]
async fn argocd_sync(data: web::Data<AppState>, body: web::Json<SyncRequest>) -> impl Responder {
    info!("Sync requested for application: {}", body.app_name);
    
    match legacy::argocd::sync_application(&data.client, &body.app_name).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            tracing::error!("Failed to sync application {}: {}", body.app_name, e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": e
            }))
        }
    }
}

#[get("/api/nodes/status")]
async fn nodes_status(data: web::Data<AppState>) -> impl Responder {
    match legacy::nodes::get_nodes_status(&data.client).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => {
            tracing::warn!("Failed to get nodes status: {}", e);
            // Return empty nodes response instead of 500 error
            HttpResponse::Ok().json(serde_json::json!({
                "total_nodes": 0,
                "ready_nodes": 0,
                "not_ready_nodes": 0,
                "nodes": [],
                "_warning": format!("Kubernetes nodes unavailable: {}", e)
            }))
        }
    }
}

#[get("/api/debug/nodes")]
async fn nodes_debug(data: web::Data<AppState>) -> impl Responder {
    let diag = legacy::nodes::get_nodes_diagnostics(&data.client).await;
    HttpResponse::Ok().json(diag)
}

/// Helper trait for converting module results to HTTP responses
/// This bridges the gap between old String-based errors and new KusanagiError
#[allow(dead_code)]
async fn handle_result<T>(result: std::result::Result<T, String>) -> HttpResponse 
where
    T: serde::Serialize,
{
    match result {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(e) => {
            tracing::error!("Request failed: {}", e);
            // Convert String error to KusanagiError for consistent response format
            let error = KusanagiError::from(e);
            error.error_response()
        }
    }
}

#[get("/api/events")]
async fn k8s_events(data: web::Data<AppState>, query: web::Query<EventsQuery>) -> impl Responder {
    // events module now uses KusanagiError directly
    match legacy::events::get_events(&data.client, query.event_type.clone(), query.page, query.per_page).await {
        Ok(events) => HttpResponse::Ok().json(events),
        Err(e) => {
            tracing::warn!("Failed to get events: {}", e);
            // Return empty events response instead of 500 error
            HttpResponse::Ok().json(serde_json::json!({
                "total_events": 0,
                "warning_count": 0,
                "normal_count": 0,
                "page": query.page.unwrap_or(1),
                "per_page": query.per_page.unwrap_or(20),
                "total_pages": 0,
                "events": [],
                "_warning": format!("Kubernetes events unavailable: {}", e)
            }))
        }
    }
}

#[get("/api/apps")]
async fn apps_with_resources(data: web::Data<AppState>) -> impl Responder {
    match legacy::apps::get_apps_with_resources(&data.client).await {
        Ok(apps) => HttpResponse::Ok().json(apps),
        Err(e) => {
            tracing::error!("Failed to get apps with resources: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[post("/api/chat")]
async fn chat_endpoint(data: web::Data<AppState>, body: web::Json<legacy::chat::ChatRequest>) -> impl Responder {
    info!("Chat message: {}", body.message);
    let response = legacy::chat::process_message(&data.client, body.into_inner()).await;
    HttpResponse::Ok().json(response)
}

#[get("/api/backups")]
async fn backups_status(data: web::Data<AppState>) -> impl Responder {
    match legacy::backups::get_backups_status(&data.client).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => {
            tracing::warn!("Failed to get backups status: {}", e);
            // Return empty backups response instead of 500 error
            HttpResponse::Ok().json(serde_json::json!({
                "total_cronjobs": 0,
                "cronjobs": [],
                "_warning": format!("Backups data unavailable: {}", e)
            }))
        }
    }
}

#[get("/api/storage")]
async fn storage_status(data: web::Data<AppState>) -> impl Responder {
    match legacy::storage::get_storage_status(&data.client).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => {
            tracing::warn!("Failed to get storage status: {}", e);
            // Return empty storage response instead of 500 error
            HttpResponse::Ok().json(serde_json::json!({
                "pvc_count": 0,
                "pvcs": [],
                "pvc_total_capacity": "0 Gi",
                "_warning": format!("Storage data unavailable: {}", e)
            }))
        }
    }
}

#[get("/api/services")]
async fn services_status(data: web::Data<AppState>) -> impl Responder {
    match legacy::services::get_services(&data.client).await {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(e) => {
            tracing::warn!("Failed to get services info: {}", e);
            // Return empty services response instead of 500 error
            HttpResponse::Ok().json(serde_json::json!({
                "services": [],
                "total": 0,
                "namespaces": [],
                "_warning": format!("Services data unavailable: {}", e)
            }))
        }
    }
}

#[get("/api/ingress")]
async fn ingress_status(data: web::Data<AppState>) -> impl Responder {
    match legacy::ingress::get_ingresses(&data.client).await {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(e) => {
            tracing::warn!("Failed to get ingress info: {}", e);
            // Return empty ingress response instead of 500 error
            HttpResponse::Ok().json(serde_json::json!({
                "ingresses": [],
                "total": 0,
                "namespaces": [],
                "_warning": format!("Ingress data unavailable: {}", e)
            }))
        }
    }
}

#[get("/api/pods/status")]
async fn pods_status(data: web::Data<AppState>) -> impl Responder {
    match legacy::pods::get_pods_status(&data.client).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(e) => {
            tracing::warn!("Failed to get pods status: {}", e);
            // Return empty pods response instead of 500 error
            HttpResponse::Ok().json(serde_json::json!({
                "pods_in_error": [],
                "total_pods": 0,
                "error_count": 0,
                "_warning": format!("Pods data unavailable: {}", e)
            }))
        }
    }
}

#[post("/api/pods/force-delete")]
async fn force_delete_pod(data: web::Data<AppState>, body: web::Json<legacy::pods::ForceDeleteRequest>) -> impl Responder {
    info!("Force delete requested for pod: {}/{}", body.namespace, body.pod_name);
    
    match legacy::pods::force_delete_pod(&data.client, &body.namespace, &body.pod_name).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            tracing::error!("Failed to force delete pod: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "message": e
            }))
        }
    }
}

#[derive(Deserialize)]
struct RangeQuery {
    query: String,
    start: i64,
    end: i64,
    step: String,
}

#[get("/api/prometheus/range")]
async fn prometheus_range(_data: web::Data<AppState>, query: web::Query<RangeQuery>) -> impl Responder {
    match legacy::prometheus::query_range(&query.query, query.start, query.end, &query.step).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => {
            tracing::error!("Failed to query Prometheus range: {}", e);
            e.error_response()
        }
    }
}

#[get("/api/ws/notifications")]
async fn ws_route(req: HttpRequest, stream: web::Payload, data: web::Data<AppState>) -> std::result::Result<HttpResponse, actix_web::Error> {
    legacy::ws::ws_notifications(req, stream, data.get_ref().client.clone()).await
}

#[derive(Deserialize)]
struct CiliumQuery {
    namespace: Option<String>,
    limit: Option<usize>,
    format: Option<String>,
}

#[get("/api/cilium/namespaces")]
async fn cilium_namespaces() -> impl Responder {
    match legacy::cilium::get_namespaces().await {
        Ok(namespaces) => HttpResponse::Ok().json(namespaces),
        Err(e) => {
            tracing::error!("Failed to get namespaces: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/cilium/flows")]
async fn cilium_flows(query: web::Query<CiliumQuery>) -> impl Responder {
    let namespace = query.namespace.as_deref();
    let limit = query.limit.unwrap_or(100);
    
    match legacy::cilium::get_hubble_flows(namespace, limit).await {
        Ok(flows) => HttpResponse::Ok().json(flows),
        Err(e) => {
            tracing::error!("Failed to get Cilium flows: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/cilium/matrix")]
async fn cilium_matrix(query: web::Query<CiliumQuery>) -> impl Responder {
    let namespace = query.namespace.as_deref();
    
    match legacy::cilium::get_flow_matrix(namespace).await {
        Ok(matrix) => HttpResponse::Ok().json(matrix),
        Err(e) => {
            tracing::error!("Failed to get flow matrix: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/cilium/metrics")]
async fn cilium_metrics(query: web::Query<CiliumQuery>) -> impl Responder {
    let namespace = query.namespace.as_deref();
    
    match legacy::cilium::get_bandwidth_metrics(namespace).await {
        Ok(metrics) => HttpResponse::Ok().json(metrics),
        Err(e) => {
            tracing::error!("Failed to get bandwidth metrics: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/cilium/anomalies")]
async fn cilium_anomalies(query: web::Query<CiliumQuery>) -> impl Responder {
    let namespace = query.namespace.as_deref();
    
    match legacy::cilium::detect_anomalies(namespace).await {
        Ok(anomalies) => HttpResponse::Ok().json(anomalies),
        Err(e) => {
            tracing::error!("Failed to detect anomalies: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/cilium/export")]
async fn cilium_export(query: web::Query<CiliumQuery>) -> impl Responder {
    let namespace = query.namespace.as_deref();
    let limit = query.limit.unwrap_or(1000);
    let format = query.format.as_deref().unwrap_or("json");
    
    match legacy::cilium::get_hubble_flows(namespace, limit).await {
        Ok(flows) => {
            match format {
                "csv" => HttpResponse::Ok()
                    .content_type("text/csv")
                    .insert_header(("Content-Disposition", "attachment; filename=flows.csv"))
                    .body(legacy::cilium::export_flows_csv(&flows)),
                _ => HttpResponse::Ok()
                    .content_type("application/json")
                    .insert_header(("Content-Disposition", "attachment; filename=flows.json"))
                    .body(legacy::cilium::export_flows_json(&flows)),
            }
        }
        Err(e) => {
            tracing::error!("Failed to export flows: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }))
        }
    }
}

#[derive(Deserialize)]
struct PrometheusQuery {
    query: String,
}

#[get("/api/prometheus/metrics")]
async fn prometheus_metrics() -> impl Responder {
    // prometheus module now uses KusanagiError directly
    match legacy::prometheus::get_cached_metrics().await {
        Ok(metrics) => HttpResponse::Ok().json(metrics),
        Err(e) => {
            tracing::error!("Failed to get Prometheus metrics: {}", e);
            e.error_response()
        }
    }
}

#[get("/api/prometheus/query")]
async fn prometheus_query(query: web::Query<PrometheusQuery>) -> impl Responder {
    match legacy::prometheus::query_raw(&query.query).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => {
            tracing::error!("Failed to execute Prometheus query: {}", e);
            e.error_response()
        }
    }
}

#[get("/api/alerts")]
async fn alerts_status() -> impl Responder {
    match legacy::alertmanager::get_cached_active_alerts().await {
        Ok(alerts) => HttpResponse::Ok().json(alerts),
        Err(e) => {
            tracing::warn!("Alertmanager not available: {}", e);
            // Return empty alerts response instead of 500 error
            HttpResponse::Ok().json(serde_json::json!({
                "critical": [],
                "warning": [],
                "info": [],
                "total": 0,
                "firing": 0,
                "pending": 0,
                "_warning": format!("Alertmanager unavailable: {}", e)
            }))
        }
    }
}

#[derive(Deserialize)]
struct ExportQuery {
    format: Option<String>,
    lang: Option<String>,
}

#[get("/api/export/report")]
async fn export_report(data: web::Data<AppState>, query: web::Query<ExportQuery>) -> impl Responder {
    match legacy::export::generate_report(&data.client).await {
        Ok(report) => {
            let format = query.format.as_deref().unwrap_or("json");
            match format {
                "csv" => {
                    match legacy::export::export_csv(&report) {
                        Ok(csv) => HttpResponse::Ok()
                            .content_type("text/csv")
                            .insert_header(("Content-Disposition", "attachment; filename=kusanagi-report.csv"))
                            .body(csv),
                        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
                    }
                },
                "markdown" | "md" => {
                    match legacy::export::export_markdown(&report) {
                        Ok(md) => HttpResponse::Ok()
                            .content_type("text/markdown")
                            .insert_header(("Content-Disposition", "attachment; filename=kusanagi-report.md"))
                            .body(md),
                        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
                    }
                },
                _ => {
                    match legacy::export::export_json(&report) {
                        Ok(json) => HttpResponse::Ok()
                            .content_type("application/json")
                            .insert_header(("Content-Disposition", "attachment; filename=kusanagi-report.json"))
                            .body(json),
                        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
                    }
                }
            }
        },
        Err(e) => {
            tracing::error!("Failed to generate report: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e
            }))
        }
    }
}

#[get("/api/export/alerts")]
async fn export_alerts_endpoint(data: web::Data<AppState>, query: web::Query<ExportQuery>) -> impl Responder {
    match legacy::alertmanager::get_active_alerts().await {
        Ok(alerts) => {
            let lang = query.lang.as_deref().unwrap_or("en");
            match legacy::export::export_alerts_for_agent(&data.client, &alerts, lang).await {
                Ok(md) => HttpResponse::Ok()
                    .content_type("text/markdown")
                    .insert_header(("Content-Disposition", "attachment; filename=agent-remediation-context.md"))
                    .body(md),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
            }
        },
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e}))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    
    // Load configuration
    if let Err(e) = config::init() {
        eprintln!("Failed to load configuration: {}", e);
        std::process::exit(1);
    }
    let cfg = config::get();
    
    // Setup graceful shutdown handler
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
    let shutdown_tx_ctrlc = shutdown_tx.clone();
    
    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C, initiating graceful shutdown...");
                let _ = shutdown_tx_ctrlc.send(()).await;
            }
            Err(err) => {
                error!("Unable to listen for shutdown signal: {}", err);
            }
        }
    });
    
    // Configure logging based on config
    let log_level = &cfg.log.level;
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(log_level));
        
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    info!("Starting Kusanagi server on port {}", cfg.server.port);
    info!("Access the cyberpunk interface at http://localhost:{}", cfg.server.port);
    
    if cfg.is_dev_mode() {
        info!("Running in development mode");
    }

    let client = if cfg.is_dev_mode() {
        tracing::warn!("Running in development mode - Kubernetes features disabled");
        match kube::Client::try_default().await {
            Ok(client) => {
                tracing::info!("Connected to Kubernetes cluster in development mode");
                client
            },
            Err(e) => {
                tracing::warn!("No Kubernetes cluster available: {} - running in standalone mode", e);
                tracing::info!("Starting Kusanagi in standalone mode - only basic features available");
                // Créer un client avec une configuration vide pour éviter le panic
                // On utilisera une URL factice qui ne sera jamais appelée
                let config = kube::Config {
                    cluster_url: "https://localhost:6443".parse().unwrap(),
                    default_namespace: "default".to_string(),
                    root_cert: None,
                    auth_info: kube::config::AuthInfo::default(),
                    proxy_url: None,
                    accept_invalid_certs: false,
                    connect_timeout: None,
                    read_timeout: None,
                    write_timeout: None,
                    tls_server_name: None,
                };
                kube::Client::try_from(config).unwrap_or_else(|_| {
                    panic!("Failed to create development client")
                })
            }
        }
    } else {
        kube::Client::try_default()
            .await
            .expect("Failed to create Kubernetes client")
    };
    
    // Start data preloading
    preload_data(client.clone()).await;
    
    let app_state = web::Data::new(AppState { client: client.clone() });
    
    // Initialize news feed cache
    let news_cache = web::Data::new(legacy::newsfeed::NewsCache::new());
    legacy::newsfeed::start_news_refresh_task(news_cache.get_ref().clone()).await;

    // Initialize system manager and auto-update task
    let system_manager = web::Data::new(legacy::system::SystemManager::new());
    tokio::spawn(legacy::system::start_auto_update_task(client.clone(), system_manager.last_image_digest.clone()));

    // Start Slack alert monitoring
    legacy::slack::start_alert_monitoring_task(client.clone()).await;

    // Initialize database pool
    if let Err(e) = legacy::database::init_pool(&client).await {
        tracing::warn!("Failed to initialize database pool: {}", e);
        tracing::info!("Continuing without database connection");
    }

    // Initialize telemetry (OpenObserve)
    legacy::telemetry::init_telemetry(&client).await;

    // Initialize MQTT
    legacy::mqtt::init_mqtt().await;

    // Start Cilium background refresh task
    tokio::spawn(legacy::cilium::start_background_refresh(client.clone()));

    // Start Security enrichment worker
    tokio::spawn(legacy::security::start_security_worker());

    // Start Prometheus background refresh task
    tokio::spawn(legacy::prometheus::start_background_refresh());

    // Start Alertmanager background refresh task
    tokio::spawn(legacy::alertmanager::start_background_refresh());

    // Initialize rate limiter
    let rate_limiter = middleware::RateLimiter::per_minute(1000);

    let server = HttpServer::new(move || {
        App::new()
            // Add middleware
            .wrap(middleware::StructuredLogging::new())
            .wrap(rate_limiter.clone())
            // App data
            .app_data(app_state.clone())
            .app_data(news_cache.clone())
            .app_data(system_manager.clone())
            // Configure routes
            .service(metrics::metrics_handler)
            .configure(legacy::proxmox::configure_routes)
            .configure(legacy::homeassistant::configure_routes)
            .configure(legacy::weather::configure_routes)
            .configure(legacy::calendar::configure_routes)
            .configure(legacy::mcp::configure_routes)
            .configure(legacy::setup::configure_routes)
            .configure(legacy::system::configure_routes)
            .configure(legacy::mqtt::configure_routes)
            .configure(legacy::slack::configure_routes)
            .configure(legacy::security::configure_routes)
            .configure(legacy::database::configure_routes)
            .configure(legacy::health::configure_routes)
            .configure(interfaces::http::configure_routes)
            .configure(doctor::configure_routes)
            .service(health_check)

            .service(index)
            .service(argocd_status)
            .service(argocd_sync)
            .service(nodes_status)
            .service(nodes_debug)
            .service(k8s_events)
            .service(apps_with_resources)
            .service(chat_endpoint)
            .service(backups_status)
            .service(storage_status)
            .service(services_status)
            .service(ingress_status)
            .service(pods_status)
            .service(legacy::pods::get_pod_logs_handler)
            .service(legacy::pods::scale_resource_handler)
            .service(legacy::pods::delete_error_pods_handler)
            .service(force_delete_pod)
            .service(ws_route)
            .service(cilium_namespaces)
            .service(cilium_flows)
            .service(cilium_matrix)
            .service(cilium_metrics)
            .service(cilium_anomalies)
            .service(cilium_export)
            .service(prometheus_metrics)
            .service(prometheus_query)
            .service(prometheus_range)
            .service(alerts_status)
            .service(export_report)
            .service(export_alerts_endpoint)
            .route("/api/quotas", web::get().to(legacy::quota::get_quotas))
            .route("/api/news", web::get().to(legacy::newsfeed::get_news))
            .service(Files::new("/static", "./static").show_files_listing())
    })
    .bind(cfg.server_addr())?
    .run();

    info!("🚀 Kusanagi server started successfully");
    
    // Wait for either the server to complete or shutdown signal
    tokio::select! {
        result = server => {
            info!("Server stopped");
            result
        }
        _ = shutdown_rx.recv() => {
            info!("🛑 Graceful shutdown initiated...");
            // Give ongoing requests time to complete
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            info!("👋 Kusanagi shutdown complete");
            Ok(())
        }
    }
}
