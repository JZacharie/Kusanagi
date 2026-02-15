//! Error response middleware
//! Converts non-JSON error responses (like 429) to proper JSON format

use axum::{
    body::Body,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// Middleware to ensure error responses are JSON
pub async fn error_handler(request: axum::extract::Request, next: Next) -> impl IntoResponse {
    let response = next.run(request).await;
    let status = response.status();

    // If it's a 429, convert to JSON
    if status == StatusCode::TOO_MANY_REQUESTS {
        // Check if response is already JSON
        let is_json = response
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|v| v.as_bytes().starts_with(b"application/json"))
            .unwrap_or(false);

        if !is_json {
            let json_response = serde_json::json!({
                "success": false,
                "error": "Too many requests. Please try again later.",
                "status": 429,
                "retry_after": 5
            });

            return Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json_response.to_string()))
                .unwrap();
        }
    }

    response
}
