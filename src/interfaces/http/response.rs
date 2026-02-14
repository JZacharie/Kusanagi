//! Standardized API response helpers.
//!
//! Every JSON endpoint should use these helpers to produce a consistent
//! envelope: `{ "success": true, "data": ... }` or `{ "success": false, "error": "..." }`.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;

/// Standard API response envelope
#[derive(Debug, serde::Serialize)]
pub struct ApiResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Wrap a successful payload in the standard envelope.
///
/// Returns HTTP 200 with body `{ "success": true, "data": <value> }`.
pub fn api_success(data: Value) -> Response {
    Json(ApiResponse {
        success: true,
        data: Some(data),
        error: None,
    })
    .into_response()
}

/// Wrap an error in the standard envelope.
///
/// Returns the given HTTP status code with body `{ "success": false, "error": "<message>" }`.
pub fn api_error(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(ApiResponse {
            success: false,
            data: None,
            error: Some(message.into()),
        }),
    )
        .into_response()
}
