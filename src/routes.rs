//! API Routes - Axum Router configuration
//!
//! This module configures all API routes for the Axum framework.

use axum::{
    routing::get,
    Router,
};

use crate::handlers;
use crate::state::AppState;

/// Create the main application router
pub fn create_router() -> Router<AppState> {
    Router::new()
        // Health & Info
        .route("/health", get(handlers::health::health_check))
        .route("/api", get(handlers::health::service_info))
        // System
        .route("/api/system/status", get(handlers::system::system_status))
        .route("/api/system/logs", get(handlers::system::system_logs))
        // Cache
        .route("/api/cache/stats", get(handlers::cache::cache_stats))
        // Kubernetes
        .route("/api/k8s/cluster", get(handlers::k8s::cluster_overview))
        .route("/api/k8s/nodes", get(handlers::k8s::nodes_status))
        .route("/api/k8s/pods", get(handlers::k8s::pods_status))
        // Monitoring
        .route("/api/alerts", get(handlers::monitoring::alerts))
        .route("/api/monitoring/quotas", get(handlers::monitoring::quotas))
}

/// Configure all routes (legacy function for compatibility)
pub fn configure_routes() -> Router<AppState> {
    create_router()
}
