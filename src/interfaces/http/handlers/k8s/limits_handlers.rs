use axum::extract::State;
use axum::response::IntoResponse;
use serde_json::json;

use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;

pub async fn get_limits_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    const CACHE_KEY: &str = "kusanagi_limits_data";

    if let Some(cached) = state.general_cache.get(CACHE_KEY).await {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&cached) {
            return api_success(value);
        }
    }

    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| {
        "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
    });

    let namespaces = match fetch_namespace_list(&state.http_client, &prometheus_url).await {
        Ok(ns) => ns,
        Err(_) => {
            return api_error(
                axum::http::StatusCode::BAD_GATEWAY,
                "Failed to fetch namespace list from Prometheus".to_string(),
            )
        }
    };

    let mut applications = Vec::new();

    for ns in &namespaces {
        let cpu_usage = query_prometheus_sum(
            &state.http_client,
            &prometheus_url,
            &format!(
                r#"sum(rate(container_cpu_usage_seconds_total{{namespace="{}"}}[5m]))"#,
                ns
            ),
        )
        .await
        .unwrap_or(0.0);

        let cpu_requests = query_prometheus_sum(
            &state.http_client,
            &prometheus_url,
            &format!(
                r#"sum(kube_pod_container_resource_requests{{namespace="{}",resource="cpu"}})"#,
                ns
            ),
        )
        .await
        .unwrap_or(0.0);

        let cpu_limits = query_prometheus_sum(
            &state.http_client,
            &prometheus_url,
            &format!(
                r#"sum(kube_pod_container_resource_limits{{namespace="{}",resource="cpu"}})"#,
                ns
            ),
        )
        .await
        .unwrap_or(0.0);

        let memory_usage = query_prometheus_sum(
            &state.http_client,
            &prometheus_url,
            &format!(
                r#"sum(container_memory_usage_bytes{{namespace="{}"}})"#,
                ns
            ),
        )
        .await
        .unwrap_or(0.0);

        let memory_requests = query_prometheus_sum(
            &state.http_client,
            &prometheus_url,
            &format!(
                r#"sum(kube_pod_container_resource_requests{{namespace="{}",resource="memory"}})"#,
                ns
            ),
        )
        .await
        .unwrap_or(0.0);

        let memory_limits = query_prometheus_sum(
            &state.http_client,
            &prometheus_url,
            &format!(
                r#"sum(kube_pod_container_resource_limits{{namespace="{}",resource="memory"}})"#,
                ns
            ),
        )
        .await
        .unwrap_or(0.0);

        let gpu_usage = query_prometheus_sum(
            &state.http_client,
            &prometheus_url,
            &format!(
                r#"sum(DCGM_FI_DEV_GPU_UTIL{{pod=~".*",namespace="{}"}} or on() 0)"#,
                ns
            ),
        )
        .await
        .unwrap_or(0.0);

        let vpc_rx = query_prometheus_sum(
            &state.http_client,
            &prometheus_url,
            &format!(
                r#"sum(rate(container_network_receive_bytes_total{{namespace="{}"}}[5m])))"#,
                ns
            ),
        )
        .await
        .unwrap_or(0.0);

        let vpc_tx = query_prometheus_sum(
            &state.http_client,
            &prometheus_url,
            &format!(
                r#"sum(rate(container_network_transmit_bytes_total{{namespace="{}"}}[5m])))"#,
                ns
            ),
        )
        .await
        .unwrap_or(0.0);

        let pod_count = query_prometheus_count(
            &state.http_client,
            &prometheus_url,
            &format!(r#"count(kube_pod_info{{namespace="{}"}})"#, ns),
        )
        .await
        .unwrap_or(0.0) as i64;

        applications.push(json!({
            "namespace": ns,
            "cpu": {
                "usage": (cpu_usage * 1000.0).round() / 1000.0,
                "requests": (cpu_requests * 1000.0).round() / 1000.0,
                "limits": (cpu_limits * 1000.0).round() / 1000.0,
                "usage_percent": if cpu_limits > 0.0 { ((cpu_usage / cpu_limits) * 100.0).round() } else { 0.0 },
                "request_ratio": if cpu_requests > 0.0 { ((cpu_usage / cpu_requests) * 100.0).round() } else { 0.0 }
            },
            "memory": {
                "usage_bytes": memory_usage.round() as i64,
                "usage_mb": (memory_usage / 1048576.0).round(),
                "requests_bytes": memory_requests.round() as i64,
                "requests_mb": (memory_requests / 1048576.0).round(),
                "limits_bytes": memory_limits.round() as i64,
                "limits_mb": (memory_limits / 1048576.0).round(),
                "usage_percent": if memory_limits > 0.0 { ((memory_usage / memory_limits) * 100.0).round() } else { 0.0 }
            },
            "gpu": {
                "usage_percent": gpu_usage.round(),
                "available": gpu_usage > 0.0
            },
            "network": {
                "rx_bytes_per_sec": vpc_rx.round() as i64,
                "tx_bytes_per_sec": vpc_tx.round() as i64,
                "rx_mbps": (vpc_rx * 8.0 / 1_000_000.0 * 100.0).round() / 100.0,
                "tx_mbps": (vpc_tx * 8.0 / 1_000_000.0 * 100.0).round() / 100.0
            },
            "pod_count": pod_count
        }));
    }

    let total_cpu_limits: f64 = applications.iter().filter_map(|a| a["cpu"]["limits"].as_f64()).sum();
    let total_cpu_usage: f64 = applications.iter().filter_map(|a| a["cpu"]["usage"].as_f64()).sum();
    let total_mem_limits: f64 = applications.iter().filter_map(|a| a["memory"]["limits_mb"].as_f64()).sum();
    let total_mem_usage: f64 = applications.iter().filter_map(|a| a["memory"]["usage_mb"].as_f64()).sum();

    let result = json!({
        "applications": applications,
        "total": {
            "namespaces": applications.len(),
            "cpu_cores_limit": (total_cpu_limits * 100.0).round() / 100.0,
            "cpu_cores_usage": (total_cpu_usage * 100.0).round() / 100.0,
            "cpu_utilization_percent": if total_cpu_limits > 0.0 { ((total_cpu_usage / total_cpu_limits) * 100.0).round() } else { 0.0 },
            "memory_gb_limit": (total_mem_limits / 1024.0 * 100.0).round() / 100.0,
            "memory_gb_usage": (total_mem_usage / 1024.0 * 100.0).round() / 100.0,
            "memory_utilization_percent": if total_mem_limits > 0.0 { ((total_mem_usage / total_mem_limits) * 100.0).round() } else { 0.0 }
        }
    });

    if let Ok(json_str) = serde_json::to_string(&result) {
        state
            .general_cache
            .set(
                CACHE_KEY.to_string(),
                json_str,
                Some(std::time::Duration::from_secs(30)),
            )
            .await;
    }

    api_success(result)
}

async fn fetch_namespace_list(
    client: &reqwest::Client,
    prometheus_url: &str,
) -> Result<Vec<String>, String> {
    let url = format!("{}/api/v1/query", prometheus_url);
    let query = r#"count by (namespace) (kube_pod_info)"#;

    let response = client
        .get(&url)
        .query(&[("query", query)])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Prometheus query failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Prometheus returned {}", response.status()));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Prometheus response: {}", e))?;

    let mut namespaces = Vec::new();
    if let Some(results) = body["data"]["result"].as_array() {
        for result in results {
            if let Some(ns) = result["metric"]["namespace"].as_str() {
                namespaces.push(ns.to_string());
            }
        }
    }

    namespaces.sort();
    Ok(namespaces)
}

async fn query_prometheus_sum(
    client: &reqwest::Client,
    prometheus_url: &str,
    query: &str,
) -> Option<f64> {
    let url = format!("{}/api/v1/query", prometheus_url);

    let response = client
        .get(&url)
        .query(&[("query", query)])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let body: serde_json::Value = response.json().await.ok()?;

    let mut total = 0.0_f64;
    if let Some(results) = body["data"]["result"].as_array() {
        for result in results {
            if let Some(value) = result["value"].as_array() {
                if let Some(val_str) = value.get(1).and_then(|v| v.as_str()) {
                    if let Ok(val) = val_str.parse::<f64>() {
                        total += val;
                    }
                }
            }
        }
    }

    Some(total)
}

async fn query_prometheus_count(
    client: &reqwest::Client,
    prometheus_url: &str,
    query: &str,
) -> Option<f64> {
    let url = format!("{}/api/v1/query", prometheus_url);

    let response = client
        .get(&url)
        .query(&[("query", query)])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let body: serde_json::Value = response.json().await.ok()?;

    if let Some(results) = body["data"]["result"].as_array() {
        if let Some(result) = results.first() {
            if let Some(value) = result["value"].as_array() {
                if let Some(val_str) = value.get(1).and_then(|v| v.as_str()) {
                    if let Ok(val) = val_str.parse::<f64>() {
                        return Some(val);
                    }
                }
            }
        }
    }

    None
}
