// HTTP Routes configuration
// Extracted from main.rs for better organization

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, services::ServeDir, trace::TraceLayer,
};

use crate::state::AppState;

// Import handlers from the new structure
use crate::interfaces::http::handlers::{business::*, core::*, k8s::*, monitoring::*};

// Import helpers
use super::helpers::{api_info, index_handler, log_request};

// Import other handlers
use crate::domain::services::fusion_service::fusion_handler;
use crate::interfaces::http::docs::ApiDoc;
use crate::interfaces::http::handlers::business::chat::post_chat_handler;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use std::sync::Arc;

/// Configure all application routes
pub fn configure_routes(state: AppState) -> Router {
    // Rate Limiting Configuration
    // Very high limits for SPA initial load - many requests fire simultaneously
    let rate_limit = std::env::var("KUSANAGI_RATE_LIMIT_PER_SECOND")
        .unwrap_or_else(|_| "10000".to_string())
        .parse::<u32>()
        .unwrap_or(10000);

    let burst_limit = std::env::var("KUSANAGI_RATE_LIMIT_BURST")
        .unwrap_or_else(|_| "50000".to_string())
        .parse::<u32>()
        .unwrap_or(50000);

    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(u64::from(rate_limit))
            .burst_size(burst_limit)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .unwrap(),
    );

    // API Routes which need rate limiting
    let api_routes = Router::new()
        .route("/api", get(api_info))
        .route("/api/config", get(get_config))
        .route("/api/cache/stats", get(cache_stats))
        .route("/api/metrics", get(core_metrics_handler)) // Alias
        .route("/api/slack/notify", post(send_slack_notification))
        // LLM routes
        .route("/api/llm/health", get(llm_health_check))
        .route("/api/llm/config", get(llm_config_info))
        // Doctor routes
        .route("/api/doctor", get(doctor_handler))
        .route("/api/doctor/quick", get(doctor_quick_handler))
        // MCP routes
        .route(
            "/api/security/vulnerabilities",
            get(mcp_vulnerabilities_handler),
        )
        .route("/api/security/policies", get(mcp_policies_handler))
        .route(
            "/api/security/policies/violations",
            get(mcp_policy_violations_handler),
        )
        .route("/api/security/fence", get(mcp_fence_handler))
        // Cilium routes
        .route("/api/cilium/flows", get(get_flows_handler))
        .route("/api/cilium/matrix", get(get_matrix_handler))
        .route("/api/cilium/metrics", get(get_metrics_handler))
        .route("/api/cilium/anomalies", get(get_anomalies_handler))
        .route("/api/cilium/namespaces", get(get_namespaces_handler))
        .route("/api/cilium/status", get(get_cilium_status_handler))
        // Hexagonal routes
        .route("/api/alerts", get(get_alerts_handler))
        .route("/api/backups", get(get_backups_handler))
        .route(
            "/api/backups/{namespace}/{name}/trigger",
            post(trigger_backup_handler),
        )
        .route("/api/ha/devices", get(get_devices_handler))
        .route("/api/ha/sensors", get(get_sensors_handler))
        .route("/api/ha/automations", get(get_automations_handler))
        .route("/api/security/summary", get(get_security_handler))
        .route("/api/security/reports", get(get_security_reports_handler))
        .route(
            "/api/security/reports/{category}/{name}",
            get(get_security_report_handler),
        )
        .route("/api/weather/current", get(get_weather_handler))
        // System routes
        .route("/api/system/status", get(system_status))
        .route("/api/system/logs", get(system_logs))
        // Kubernetes routes
        .route("/api/k8s/cluster", get(cluster_overview))
        .route("/api/k8s/nodes", get(nodes_status))
        .route("/api/k8s/pods", get(pods_status))
        .route("/api/k8s/pods/{namespace}/{name}/logs", get(pod_logs))
        .route(
            "/api/pods/delete-error-pods",
            post(delete_error_pods_handler),
        )
        .route("/api/storage", get(storage))
        .route("/api/ingress", get(ingress))
        .route("/api/services", get(services))
        .route("/api/argocd/status", get(argocd_status))
        .route("/api/argocd/sync", post(argocd_sync))
        .route("/api/news", get(news))
        // Kubernetes routes - canonical paths only
        // Use /api/k8s/* paths (not legacy /api/cluster/overview, /api/nodes/status, /api/pods/status)
        // Monitoring routes
        .route("/api/monitoring/alerts", get(alerts))
        .route("/api/monitoring/quotas", get(quotas))
        .route("/api/dashboard/metrics", get(metrics_handler)) // Dashboard metrics
        .route("/api/chat", post(post_chat_handler))
        .route("/api/prometheus/range", get(prometheus_range_handler))
        .route("/api/database/health", get(database_health_handler))
        .route("/api/fusion", get(fusion_handler))
        // Proxmox routes
        .route("/api/proxmox/vms", get(get_vms_handler))
        .route("/api/proxmox/containers", get(get_containers_handler))
        .route("/api/proxmox/nodes", get(get_nodes_handler))
        // Apply rate limiting specifically to these routes
        .layer(GovernorLayer::new(governor_conf))
        // Convert 429 errors to JSON
        .layer(middleware::from_fn(
            crate::interfaces::http::middleware::error_handler::error_handler,
        ));

    Router::new()
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Core routes (No Rate Limit)
        .route("/", get(index_handler))
        .route("/health", get(health_check))
        .route("/metrics", get(core_metrics_handler))
        .route("/docs", get(docs_handler))
        // WebSocket (No Rate Limit - handled separately)
        .route("/api/ws/notifications", get(ws_notifications_handler))
        // Merge API routes (With Rate Limit)
        .merge(api_routes)
        // Static files (must be after API routes)
        .nest_service("/static", ServeDir::new("./static"))
        // Layers (applied in reverse order - last one is executed first)
        .layer(middleware::from_fn(log_request))
        .layer(middleware::from_fn(
            crate::interfaces::http::middleware::metrics::track_metrics,
        ))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        // State (must be last)
        .with_state(state)
}
