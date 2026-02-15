use axum::{extract::Query, response::IntoResponse, Json};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct RangeQuery {
    pub query: String,
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub step: Option<String>,
}

pub async fn prometheus_range_handler(Query(params): Query<RangeQuery>) -> impl IntoResponse {
    // Proxy the request to Prometheus
    let prometheus_url = std::env::var("PROMETHEUS_URL")
        .unwrap_or_else(|_| "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string());
    
    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build() {
        Ok(c) => c,
        Err(_) => {
            return Json(json!({
                "status": "error",
                "data": {
                    "resultType": "matrix",
                    "result": []
                }
            }));
        }
    };
    
    let url = format!("{}/api/v1/query_range", prometheus_url);
    
    // Build query parameters
    let mut query_params = vec![("query", params.query)];
    
    if let Some(start) = params.start {
        query_params.push(("start", start.to_string()));
    }
    if let Some(end) = params.end {
        query_params.push(("end", end.to_string()));
    }
    if let Some(step) = params.step {
        query_params.push(("step", step));
    }
    
    match client.get(&url).query(&query_params).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(data) => Json(data),
                    Err(_) => Json(json!({
                        "status": "error",
                        "data": { "resultType": "matrix", "result": [] }
                    })),
                }
            } else {
                Json(json!({
                    "status": "error",
                    "data": { "resultType": "matrix", "result": [] }
                }))
            }
        }
        Err(_) => Json(json!({
            "status": "error",
            "data": { "resultType": "matrix", "result": [] }
        })),
    }
}
