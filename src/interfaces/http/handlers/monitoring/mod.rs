//! Monitoring handlers

pub mod cilium;

use axum::response::IntoResponse;
use axum::Json;
pub use cilium::*;

/// Alerts endpoint
pub async fn alerts() -> impl IntoResponse {
    Json(serde_json::json!({
        "alerts": [],
        "total": 0
    }))
}

/// Quotas endpoint
pub async fn quotas() -> impl IntoResponse {
    Json(serde_json::json!({
        "antigravity_percentage": 15,
        "notebooklm_percentage": 30,
        "storage_used_gb": 45.5,
        "storage_total_gb": 100.0,
        "last_updated": chrono::Utc::now().to_rfc3339()
    }))
}

/// Metrics endpoint for the dashboard
pub async fn metrics_handler(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
) -> impl IntoResponse {
    use crate::domain::services::kubernetes_service;

    // 1. Get nodes status for resource usage
    let (cpu_percent, mem_percent, node_count) =
        match kubernetes_service::get_nodes_status(&state.http_client).await {
            Ok(data) => {
                if let Some(nodes) = data["nodes"].as_array() {
                    let mut cpu_sum = 0.0;
                    let mut mem_sum = 0.0;
                    let count = nodes.len() as f64;

                    if count > 0.0 {
                        for node in nodes {
                            cpu_sum += node["cpu_usage_percent"].as_f64().unwrap_or(0.0);
                            mem_sum += node["memory_usage_percent"].as_f64().unwrap_or(0.0);
                        }
                        (cpu_sum / count, mem_sum / count, nodes.len())
                    } else {
                        (0.0, 0.0, 0)
                    }
                } else {
                    (0.0, 0.0, 0)
                }
            }
            Err(_) => (0.0, 0.0, 0),
        };

    // 2. Get cluster overview for pod count
    let pod_count = match kubernetes_service::get_cluster_overview(
        &state.http_client,
        &state.kube_client,
        &state.k8s_cache,
    )
    .await
    {
        Ok(data) => data["pods"].as_i64().unwrap_or(0),
        Err(_) => 0,
    };

    // 3. Get alerts
    let alerts_firing = match state.alerts_use_case.get_active_alerts().await {
        Ok(response) => response.total,
        Err(_) => 0,
    };

    Json(serde_json::json!({
        "cpu_usage_percent": cpu_percent,
        "memory_usage_percent": mem_percent,
        "pod_count": pod_count,
        "node_count": node_count,
        "alerts_firing": alerts_firing,
        "container_count": pod_count, // Approximation as we don't have container count readily available
        "gpu_utilization": 0,
        "gpu_temperature": 0,
        "gpu_power_usage": 0,
        "energy_solar_production": 0
    }))
}
