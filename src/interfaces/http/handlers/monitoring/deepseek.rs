use axum::{extract::State, response::IntoResponse};
use serde_json::json;
use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;

pub async fn deepseek_metrics_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .unwrap_or_default();

    if api_key.is_empty() {
        return api_error(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "DEEPSEEK_API_KEY not configured",
        );
    }

    let budget_threshold: f64 = std::env::var("DEEPSEEK_BALANCE_THRESHOLD")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10.0); // Default threshold for warning

    match fetch_deepseek_all_metrics(&state, &api_key, budget_threshold).await {
        Ok(metrics) => api_success(json!({
            "provider": "deepseek",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "deepseek": metrics,
        })),
        Err(e) => api_error(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            format!("DeepSeek metrics unavailable: {}", e),
        ),
    }
}

async fn fetch_deepseek_all_metrics(
    state: &AppState,
    api_key: &str,
    threshold: f64,
) -> Result<serde_json::Value, String> {
    let mut result = serde_json::Map::new();

    // 1. Fetch Balance
    let balance_data = fetch_deepseek_balance(state, api_key).await?;
    result.insert("balance".to_string(), balance_data.clone());

    // 2. Fetch Models (health check)
    let models = fetch_deepseek_models(state, api_key).await.unwrap_or(json!([]));
    result.insert("models".to_string(), models);

    // 3. Process Balance and Alerts
    if let Some(balance_infos) = balance_data.get("balance_infos").and_then(|b| b.as_array()) {
        if let Some(first_balance) = balance_infos.first() {
            let total_balance: f64 = first_balance.get("total_balance")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);
            
            let currency = first_balance.get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("CNY");

            result.insert("summary".to_string(), json!({
                "total_balance": total_balance,
                "currency": currency,
                "is_low": total_balance < threshold,
                "threshold": threshold,
            }));
        }
    }

    Ok(json!(result))
}

async fn fetch_deepseek_balance(
    state: &AppState,
    api_key: &str,
) -> Result<serde_json::Value, String> {
    let url = "https://api.deepseek.com/user/balance";
    
    let response = state.http_client
        .get(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("DeepSeek API returned status: {}", response.status()));
    }

    let data = response.json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(data)
}

async fn fetch_deepseek_models(
    state: &AppState,
    api_key: &str,
) -> Result<serde_json::Value, String> {
    let url = "https://api.deepseek.com/models";
    
    let response = state.http_client
        .get(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("DeepSeek API returned status: {}", response.status()));
    }

    let data = response.json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    Ok(data.get("data").cloned().unwrap_or(json!([])))
}
