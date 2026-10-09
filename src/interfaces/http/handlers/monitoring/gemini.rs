use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;
use axum::{extract::State, response::IntoResponse};
use serde_json::json;

pub async fn gemini_metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let api_key = std::env::var("GEMINI_API_KEY").unwrap_or_default();

    if api_key.is_empty() {
        return api_error(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "GEMINI_API_KEY not configured",
        );
    }

    let budget_threshold: f64 = std::env::var("GEMINI_BALANCE_THRESHOLD")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10.0); // Default threshold for warning

    match fetch_gemini_all_metrics(&state, &api_key, budget_threshold).await {
        Ok(metrics) => api_success(json!({
            "provider": "gemini",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "gemini": metrics,
        })),
        Err(e) => api_error(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            format!("Gemini metrics unavailable: {}", e),
        ),
    }
}

async fn fetch_gemini_all_metrics(
    state: &AppState,
    api_key: &str,
    threshold: f64,
) -> Result<serde_json::Value, String> {
    let mut result = serde_json::Map::new();

    // 1. Fetch Models (includes quota/rate limit info)
    let models = fetch_gemini_models(state, api_key)
        .await
        .unwrap_or(json!([]));
    result.insert("models".to_string(), models.clone());

    // 2. Fetch Quota/Usage info if available
    let quota = fetch_gemini_quota(state, api_key).await;
    if let Ok(q) = quota {
        result.insert("quota".to_string(), q);
    }

    // 3. Process Models and Alerts
    if let Some(models_array) = models.as_array() {
        let model_summaries: Vec<serde_json::Value> = models_array
            .iter()
            .filter_map(extract_model_summary)
            .collect();

        result.insert("model_summaries".to_string(), json!(model_summaries));

        // Check if any model has quota issues
        let has_quota_warnings = model_summaries.iter().any(|m| {
            m.get("quota_exceeded").and_then(|v| v.as_bool()).unwrap_or(false)
                || m.get("rate_limit_remaining")
                    .and_then(|v| v.as_i64())
                    .map(|r| r < 100)
                    .unwrap_or(false)
        });

        result.insert(
            "summary".to_string(),
            json!({
                "total_models": models_array.len(),
                "has_quota_warnings": has_quota_warnings,
                "threshold": threshold,
            }),
        );
    }

    Ok(json!(result))
}

async fn fetch_gemini_models(
    state: &AppState,
    api_key: &str,
) -> Result<serde_json::Value, String> {
    // Google Generative Language API - list models
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={}", api_key);

    let response = state
        .http_client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Gemini API returned status {}: {}", status, text));
    }

    let data = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    // The API returns { "models": [...] }
    Ok(data.get("models").cloned().unwrap_or(json!([])))
}

async fn fetch_gemini_quota(
    _state: &AppState,
    _api_key: &str,
) -> Result<serde_json::Value, String> {
    // Note: Google AI Studio doesn't have a direct quota endpoint like DeepSeek's balance endpoint.
    // Quota info is typically available in the model responses or via Cloud Console.
    // We return a placeholder indicating this limitation.
    Ok(json!({
        "note": "Detailed quota information requires Google Cloud Console or billing API access. Model-level rate limits are included in model responses.",
        "api_key_configured": true,
    }))
}

fn extract_model_summary(model: &serde_json::Value) -> Option<serde_json::Value> {
    let name = model.get("name")?.as_str()?.to_string();
    let display_name = model.get("display_name")?.as_str().unwrap_or("").to_string();
    let description = model.get("description")?.as_str().unwrap_or("").to_string();
    let input_token_limit = model.get("input_token_limit")?.as_i64()?;
    let output_token_limit = model.get("output_token_limit")?.as_i64()?;
    let supported_generation_methods = model.get("supported_generation_methods")?.as_array()?;

    // Extract rate limit info if available (not always present in list models response)
    let rate_limit_remaining = model.get("rate_limit_remaining").and_then(|v| v.as_i64());
    let quota_exceeded = model.get("quota_exceeded").and_then(|v| v.as_bool());

    Some(json!({
        "name": name,
        "display_name": display_name,
        "description": description,
        "input_token_limit": input_token_limit,
        "output_token_limit": output_token_limit,
        "supported_generation_methods": supported_generation_methods.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>(),
        "rate_limit_remaining": rate_limit_remaining,
        "quota_exceeded": quota_exceeded.unwrap_or(false),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_model_summary() {
        let model = json!({
            "name": "models/gemini-1.5-pro",
            "display_name": "Gemini 1.5 Pro",
            "description": "Latest Gemini model",
            "input_token_limit": 2000000,
            "output_token_limit": 8192,
            "supported_generation_methods": ["generateContent", "countTokens"],
        });

        let summary = extract_model_summary(&model).unwrap();
        assert_eq!(summary["name"], "models/gemini-1.5-pro");
        assert_eq!(summary["display_name"], "Gemini 1.5 Pro");
        assert_eq!(summary["input_token_limit"], 2000000);
    }
}