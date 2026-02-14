use axum::{extract::Query, response::IntoResponse};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct RangeQuery {
    pub query: String,
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub step: Option<String>,
}

use crate::interfaces::http::response::api_success;

pub async fn prometheus_range_handler(Query(_params): Query<RangeQuery>) -> impl IntoResponse {
    // Mock response or proxy logic could go here.
    // For now, return empty data to prevent 404s and keep frontend happy.
    api_success(json!({
        "status": "success",
        "data": {
            "resultType": "matrix",
            "result": []
        }
    }))
}
