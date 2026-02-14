//! Standardized API response helpers.
//!
//! Every JSON endpoint should use these helpers to produce a consistent
//! envelope: `{ "success": true, "data": ... }` or `{ "success": false, "error": "..." }`.

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::{json, Value};

/// Wrap a successful payload in the standard envelope.
///
/// Returns HTTP 200 with body `{ "success": true, "data": <value> }`.
pub fn api_success(data: Value) -> impl IntoResponse {
    Json(json!({ "success": true, "data": data }))
}

/// Wrap an error in the standard envelope.
///
/// Returns the given HTTP status code with body `{ "success": false, "error": "<message>" }`.
pub fn api_error(status: StatusCode, message: impl Into<String>) -> impl IntoResponse {
    (
        status,
        Json(json!({ "success": false, "error": message.into() })),
    )
}
