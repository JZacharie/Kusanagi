use crate::interfaces::http::response::api_success;
use crate::state::AppState;
use axum::{extract::State, response::IntoResponse};
use serde_json::json;

/// Pylos Metrics proxy handler
pub async fn pylos_metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let pylos_url =
        std::env::var("PYLOS_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    // 1. Fetch Health
    let health_url = format!("{}/health", pylos_url);
    let health_req = state
        .http_client
        .get(&health_url)
        .timeout(std::time::Duration::from_secs(5))
        .send();

    // 2. Fetch Models Info
    let models_url = format!("{}/v1/models", pylos_url);
    let models_req = state
        .http_client
        .get(&models_url)
        .timeout(std::time::Duration::from_secs(5))
        .send();

    // 3. Fetch logs stats for the last 24h
    let stats_url = format!("{}/api/logs/stats?period=24h", pylos_url);
    let stats_req = state
        .http_client
        .get(&stats_url)
        .timeout(std::time::Duration::from_secs(5))
        .send();

    // Execute in parallel
    let (health_res, models_res, stats_res) = tokio::join!(health_req, models_req, stats_req);

    let mut response_data = serde_json::Map::new();
    response_data.insert("pylos_url".to_string(), json!(pylos_url));

    // Process Health
    match health_res {
        Ok(resp) if resp.status().is_success() => {
            response_data.insert("healthy".to_string(), json!(true));
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                response_data.insert("health_status".to_string(), data);
            }
        }
        _ => {
            response_data.insert("healthy".to_string(), json!(false));
        }
    }

    // Process Models
    if let Ok(resp) = models_res {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                response_data.insert("models".to_string(), data.clone());
                if let Some(models_list) = data.as_array() {
                    response_data.insert("model_count".to_string(), json!(models_list.len()));
                } else if let Some(data_obj) = data.get("data") {
                    if let Some(models_list) = data_obj.as_array() {
                        response_data.insert("model_count".to_string(), json!(models_list.len()));
                    }
                }
            }
        }
    }

    // Process Stats
    if let Ok(resp) = stats_res {
        if resp.status().is_success() {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                response_data.insert("stats".to_string(), data);
            }
        }
    }

    api_success(json!(response_data))
}
