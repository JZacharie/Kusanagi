// HTTP Routes configuration
// Extracted from main.rs for better organization

use axum::{
    middleware,
    routing::{get, post, delete},
    Router,
};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, services::ServeDir, trace::TraceLayer,
};

use crate::state::AppState;

// Import handlers from the new structure
use crate::interfaces::http::handlers::{business::*, core::*, k8s::*, monitoring::*};
use crate::interfaces::http::handlers::business::proxmox_handlers::delete_volume_handler;

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
use crate::interfaces::http::a2ui_handlers::*;

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
            get(get_vulnerabilities_handler),
        )
        .route("/api/security/scan", post(post_security_scan_handler))
        .route("/api/security/policies", get(mcp_policies_handler))
        .route(
            "/api/security/policies/violations",
            get(mcp_policy_violations_handler),
        )
        .route("/api/security/fence", get(mcp_fence_handler))
        .route("/api/monitoring/logs", get(mcp_openobserve_logs_handler))
        .route("/api/monitoring/netbox", get(mcp_netbox_handler))
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
        .route("/api/business/cloudflare", get(get_cloudflare_analytics_handler))
        // System routes

        .route("/api/system/status", get(system_status))
        .route("/api/system/logs", get(system_logs))
        // Kubernetes routes
        .route("/api/k8s/cluster", get(cluster_overview))
        .route("/api/k8s/nodes", get(nodes_status))
        .route("/api/debug/nodes", get(nodes_debug))
        .route("/api/k8s/pods", get(pods_status))
        .route("/api/k8s/namespaces/metrics", get(get_namespace_metrics_handler))
        .route("/api/k8s/pods/{namespace}/{name}/logs", get(pod_logs))
        .route(
            "/api/pods/delete-error-pods",
            post(delete_error_pods_handler),
        )
        .route("/api/pods/force-delete", post(force_delete_pod_handler))
        .route("/api/storage", get(storage))
        .route("/api/storage/analysis", get(storage_analysis))
        .route("/api/ingress", get(ingress))
        .route("/api/services", get(services))
        .route("/api/pods/{namespace}/{name}/logs", get(pod_logs))
        .route("/api/argocd/status", get(argocd_status))
        .route("/api/argocd/sync", post(argocd_sync))
        .route("/api/news", get(news))
        .route("/api/news/refresh", post(news_refresh))
        .route("/api/streaming", get(streaming))
        .route("/api/streaming/refresh", post(streaming_refresh))
        .route("/api/streaming/poster/{hash}", get(get_poster))
        // Kubernetes routes - canonical paths only
        // Use /api/k8s/* paths (not legacy /api/cluster/overview, /api/nodes/status, /api/pods/status)
        // Monitoring routes
        .route("/api/monitoring/alerts", get(alerts))
        .route(
            "/api/monitoring/enphase/history",
            get(enphase_history_handler),
        )
        .route("/api/monitoring/enphase/debug", get(enphase_debug_handler))
        .route("/api/monitoring/gpu/debug", get(gpu_debug_handler))
        .route("/api/monitoring/trivy/debug", get(trivy_debug_handler))
        .route(
            "/api/monitoring/litellm/metrics",
            get(litellm_metrics_handler),
        )
        .route("/api/dashboard/metrics", get(metrics_handler)) // Dashboard metrics
        .route("/api/github/pipelines", get(github_pipelines_handler))
        .route("/api/chat", post(post_chat_handler))
        .route("/api/prometheus/range", get(prometheus_range_handler))
        .route("/api/database/health", get(database_health_handler))
        .route("/api/fusion", get(fusion_handler))
        // Proxmox routes
        .route("/api/proxmox/vms", get(get_vms_handler))
        .route("/api/proxmox/containers", get(get_containers_handler))
        .route("/api/proxmox/nodes", get(get_nodes_handler))
        .route("/api/proxmox/volume/{server}/{node}/{storage}/{volume}", delete(delete_volume_handler))
        .route("/api/proxmox/deploy-compose", post(deploy_compose_handler))
        // MQTT routes
        .route("/api/mqtt/devices", get(get_mqtt_devices_handler))
        .route("/api/mqtt/messages", get(get_mqtt_messages_handler))
        .route("/api/github/promote", post(promote_to_prod_handler))
        // A2UI routes
        .route("/api/a2ui/message", post(post_a2ui_message_handler))
        .route("/api/a2ui/surface/{id}", get(get_a2ui_surface_handler))
        .route("/api/a2ui/data/{id}", get(get_a2ui_data_handler))
        // Apply rate limiting specifically to these routes
        .layer(GovernorLayer::new(governor_conf))
        // Convert 429 errors to JSON
        .layer(middleware::from_fn(
            crate::interfaces::http::middleware::error_handler::error_handler,
        ));

    Router::new()
        // Swagger UI - FastAPI-like interactive API documentation
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Core routes (No Rate Limit)
        .route("/", get(index_handler))
        .route("/health", get(health_check))
        .route("/metrics", get(core_metrics_handler))
        // Redirect /docs to Swagger UI for interactive API testing
        .route(
            "/docs",
            get(|| async { axum::response::Redirect::permanent("/swagger-ui") }),
        )
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
