// HTTP helpers - Utility functions for HTTP handlers

use axum::{extract::Request, middleware::Next, response::IntoResponse, Json};
use tracing::{info, warn};

// Static files
static INDEX_HTML_TEMPLATE: &str = include_str!("../../../static/index.html");

use std::sync::OnceLock;

static INDEX_HTML: OnceLock<String> = OnceLock::new();

/// Serve index.html
pub async fn index_handler() -> impl IntoResponse {
    let html = INDEX_HTML.get_or_init(|| {
        let version = env!("BUILD_TIMESTAMP");
        let full_features =
            std::env::var("KUSANAGI_FULL_FEATURES").unwrap_or_else(|_| "true".to_string());
        INDEX_HTML_TEMPLATE
            .replace("{{VERSION}}", version)
            .replace("{{FULL_FEATURES}}", &full_features)
    });
    axum::response::Html(html.as_str())
}

/// API information endpoint
pub async fn api_info() -> impl IntoResponse {
    use super::handlers::core::docs::get_routes;

    let routes = get_routes();
    let mut endpoints: std::collections::BTreeMap<String, Vec<String>> =
        std::collections::BTreeMap::new();

    for route in routes {
        endpoints
            .entry(route.category.to_lowercase())
            .or_default()
            .push(format!(
                "{} {} - {}",
                route.method, route.path, route.description
            ));
    }

    Json(serde_json::json!({
        "service": "Kusanagi",
        "version": env!("CARGO_PKG_VERSION"),
        "architecture": "axum-migration",
        "features": [
            "Axum Framework",
            "Hexagonal Architecture",
            "Kubernetes Integration",
            "Cache System",
            "WebSocket Notifications"
        ],
        "endpoints": endpoints
    }))
}

/// Middleware to log incoming requests
pub async fn log_request(request: Request, next: Next) -> impl IntoResponse {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path();

    // Skip logging for noisy monitoring endpoints
    let skip_log = path == "/api/metrics" || path == "/metrics" || path == "/health";

    if !skip_log {
        info!("📥 {} {}", method, uri);
    }

    let response = next.run(request).await;
    let status = response.status();

    if !skip_log {
        let status_code = status.as_u16();
        if status_code >= 400 {
            warn!(
                "📤 {} - Status: {} {}",
                uri,
                status_code,
                status.canonical_reason().unwrap_or("")
            );
        } else {
            info!("📤 {} - Status: {}", uri, status);
        }
    }

    response
}
