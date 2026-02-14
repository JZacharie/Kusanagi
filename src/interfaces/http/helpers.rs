// HTTP helpers - Utility functions for HTTP handlers

use axum::{extract::Request, middleware::Next, response::IntoResponse, Json};
use tracing::info;

// Static files
static INDEX_HTML: &str = include_str!("../../../static/index.html");

/// Serve index.html
pub async fn index_handler() -> impl IntoResponse {
    axum::response::Html(INDEX_HTML)
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

    info!("📥 {} {}", method, uri);

    let response = next.run(request).await;

    info!("📤 {} - Status: {}", uri, response.status());

    response
}
