//! Monitoring handlers

pub mod cilium;
pub mod mqtt;

use axum::response::IntoResponse;
use serde_json::json;

pub use cilium::*;
pub use mqtt::*;

use crate::interfaces::http::response::api_success;

/// Alerts endpoint
pub async fn alerts() -> impl IntoResponse {
    api_success(serde_json::json!({
        "alerts": [],
        "total": 0
    }))
}

/// Debug endpoint for GPU metrics
pub async fn gpu_debug_handler(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
) -> impl IntoResponse {
    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| {
        "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
    });
    
    let gpu_hot_url = std::env::var("GPU_HOT_URL")
        .unwrap_or_else(|_| "https://gpu-hot.p.zacharie.org".to_string());

    // Test Prometheus queries
    let url = format!("{}/api/v1/query", prometheus_url);
    let mut prometheus_results = serde_json::Map::new();

    let queries = [
        ("dcgm_gpu_util", r#"avg(DCGM_FI_DEV_GPU_UTIL)"#),
        ("nvidia_gpu_util", r#"nvidia_gpu_utilization_gpu_utilization"#),
        ("nvidia_gpu_temp", r#"nvidia_gpu_temperature_gpu_temperature"#),
        ("nvidia_gpu_power", r#"nvidia_gpu_power_usage_gpu_power_usage"#),
    ];

    for (name, query) in &queries {
        let result = query_prometheus_scalar(&state.http_client, &url, query).await;
        prometheus_results.insert(
            name.to_string(),
            json!(result.map(|v| format!("{:.2}", v)).unwrap_or_else(|| "N/A".to_string())),
        );
    }

    // Fetch raw GPU-HOT API response
    let api_url = format!("{}/api/gpu-data", gpu_hot_url);
    let hot_raw_response = state.http_client
        .get(&api_url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;

    let (hot_util, hot_temp, hot_power) = fetch_gpu_from_hot_service(&state.http_client).await;

    let raw_json = match hot_raw_response {
        Ok(r) if r.status().is_success() => r.json::<serde_json::Value>().await.unwrap_or_default(),
        _ => json!(null),
    };

    api_success(json!({
        "prometheus_url": prometheus_url,
        "gpu_hot_url": gpu_hot_url,
        "gpu_hot_api": api_url,
        "prometheus_queries": prometheus_results,
        "gpu_hot_metrics": {
            "utilization": hot_util,
            "temperature": hot_temp,
            "power": hot_power,
        },
        "gpu_hot_raw_response": raw_json,
        "status": if hot_util > 0.0 || hot_temp > 0.0 || hot_power > 0.0 {
            "gpu_hot_working"
        } else if prometheus_results.values().any(|v| v != "N/A") {
            "prometheus_working"
        } else {
            "no_gpu_metrics"
        }
    }))
}

/// Metrics endpoint for the dashboard
pub async fn metrics_handler(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
) -> impl IntoResponse {
    use crate::domain::services::kubernetes_service;
    use crate::domain::services::trivy_service;

    // 1. Get nodes status for resource usage
    let (cpu_percent, mem_percent, node_count) =
        match kubernetes_service::get_nodes_status(&state.http_client, &state.k8s_cache).await {
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

    // 4. Get Trivy vulnerabilities (don't block on error)
    let (trivy_critical, trivy_high, trivy_medium, trivy_low) =
        match trivy_service::get_vulnerabilities(&state.k8s_cache).await {
            Ok(data) => (
                data["critical"].as_i64().unwrap_or(0) as i32,
                data["high"].as_i64().unwrap_or(0) as i32,
                data["medium"].as_i64().unwrap_or(0) as i32,
                data["low"].as_i64().unwrap_or(0) as i32,
            ),
            Err(_) => (0, 0, 0, 0),
        };

    // 5. Get VPS metrics from Prometheus (node_exporter sur le VPS)
    let (vps_cpu, vps_disk, vps_net) = fetch_vps_metrics(&state.http_client).await;

    // 6. Try to get GPU and Enphase data from Home Assistant or Prometheus
    let (gpu_util, gpu_temp, gpu_power, solar_prod, house_cons) =
        fetch_gpu_and_energy_metrics(&state.http_client).await;

    api_success(serde_json::json!({
        "cpu_usage_percent": cpu_percent,
        "memory_usage_percent": mem_percent,
        "pod_count": pod_count,
        "node_count": node_count,
        "alerts_firing": alerts_firing,
        "container_count": pod_count, // Approximation
        "gpu_utilization": gpu_util,
        "gpu_temperature": gpu_temp,
        "gpu_power_usage": gpu_power,
        "energy_solar_production": solar_prod,
        "energy_house_consumption": house_cons,
        "vps_cpu_usage": vps_cpu,
        "vps_disk_usage": vps_disk,
        "vps_net_receive": vps_net,
        "trivy_critical_count": trivy_critical,
        "trivy_high_count": trivy_high,
        "trivy_medium_count": trivy_medium,
        "trivy_low_count": trivy_low,
    }))
}

/// Fetch VPS metrics from Prometheus (node_exporter on the VPS)
async fn fetch_vps_metrics(client: &reqwest::Client) -> (f64, f64, f64) {
    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| {
        "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
    });

    let url = format!("{}/api/v1/query", prometheus_url);

    // 1. VPS CPU Usage - avg over all CPUs
    // 100 - (avg(irate(node_cpu_seconds_total{mode="idle",job="vps"}[5m])) * 100)
    let cpu_query = r#"100 - (avg(irate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)"#;
    let cpu_value = query_prometheus_scalar(client, &url, cpu_query)
        .await
        .unwrap_or(0.0);

    // 2. VPS Disk Usage - use /sda1 or root filesystem
    // (1 - (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"})) * 100
    let disk_query = r#"(1 - (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"})) * 100"#;
    let disk_value = query_prometheus_scalar(client, &url, disk_query)
        .await
        .unwrap_or(0.0);

    // 3. VPS Network Receive Rate in Mbps
    // rate(node_network_receive_bytes_total{device="eth0"}[5m]) / 125000
    // (divide by 125000 to convert bytes/s to Mbps)
    let net_query = r#"rate(node_network_receive_bytes_total{device="eth0"}[5m]) / 125000"#;
    let net_value = query_prometheus_scalar(client, &url, net_query)
        .await
        .unwrap_or(0.0);

    (
        cpu_value.max(0.0),
        disk_value.clamp(0.0, 100.0),
        net_value.max(0.0),
    )
}

/// Fetch GPU and Energy metrics from Prometheus or Home Assistant
async fn fetch_gpu_and_energy_metrics(client: &reqwest::Client) -> (f64, f64, f64, f64, f64) {
    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| {
        "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
    });

    let url = format!("{}/api/v1/query", prometheus_url);

    // Try multiple GPU metric sources
    let mut gpu_util = 0.0;
    let mut gpu_temp = 0.0;
    let mut gpu_power = 0.0;

    // 1. Try DCGM exporter (NVIDIA Data Center GPU Manager)
    if let Some(val) = query_prometheus_scalar(client, &url, r#"avg(DCGM_FI_DEV_GPU_UTIL)"#).await {
        gpu_util = val;
    }
    if let Some(val) = query_prometheus_scalar(client, &url, r#"avg(DCGM_FI_DEV_GPU_TEMP)"#).await {
        gpu_temp = val;
    }
    if let Some(val) = query_prometheus_scalar(client, &url, r#"avg(DCGM_FI_DEV_POWER_USAGE)"#).await {
        gpu_power = val;
    }

    // 2. Try nvidia_gpu_exporter (https://github.com/utkuozdemir/nvidia_gpu_exporter)
    if gpu_util == 0.0 {
        if let Some(val) = query_prometheus_scalar(client, &url, r#"nvidia_gpu_utilization_gpu_utilization"#).await {
            gpu_util = val;
        }
    }
    if gpu_temp == 0.0 {
        if let Some(val) = query_prometheus_scalar(client, &url, r#"nvidia_gpu_temperature_gpu_temperature"#).await {
            gpu_temp = val;
        }
    }
    if gpu_power == 0.0 {
        if let Some(val) = query_prometheus_scalar(client, &url, r#"nvidia_gpu_power_usage_gpu_power_usage"#).await {
            gpu_power = val;
        }
    }

    // 3. Try node_exporter with NVIDIA textfile collector
    if gpu_util == 0.0 {
        // nvidia-smi output via textfile collector
        if let Some(val) = query_prometheus_scalar(client, &url, r#"nvidia_gpu_utilization_percentage"#).await {
            gpu_util = val;
        }
    }
    if gpu_temp == 0.0 {
        if let Some(val) = query_prometheus_scalar(client, &url, r#"nvidia_gpu_temperature_celsius"#).await {
            gpu_temp = val;
        }
    }
    if gpu_power == 0.0 {
        if let Some(val) = query_prometheus_scalar(client, &url, r#"nvidia_gpu_power_draw_watts"#).await {
            gpu_power = val;
        }
    }

    // 4. Try generic node_exporter GPU metrics (if available)
    if gpu_util == 0.0 {
        if let Some(val) = query_prometheus_scalar(client, &url, r#"gpu_utilization_percent"#).await {
            gpu_util = val;
        }
    }

    // 5. Try external GPU-HOT service as fallback
    if gpu_util == 0.0 && gpu_temp == 0.0 && gpu_power == 0.0 {
        tracing::info!("No GPU metrics from Prometheus, trying gpu-hot service");
        let (hot_util, hot_temp, hot_power) = fetch_gpu_from_hot_service(client).await;
        gpu_util = hot_util;
        gpu_temp = hot_temp;
        gpu_power = hot_power;
    }

    // Try Home Assistant for Enphase data if Prometheus doesn't have it
    let (solar_prod, house_cons) = fetch_enphase_from_ha(client).await;

    (
        gpu_util.max(0.0),
        gpu_temp.max(0.0),
        gpu_power.max(0.0),
        solar_prod,
        house_cons,
    )
}

/// Helper to query Prometheus and extract a scalar value
async fn query_prometheus_scalar(client: &reqwest::Client, url: &str, query: &str) -> Option<f64> {
    let response = client
        .get(url)
        .query(&[("query", query)])
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    let body: serde_json::Value = response.json().await.ok()?;

    // Extract value from Prometheus response format
    // {"data":{"result":[{"value":[timestamp, "value"]}]}}
    body.get("data")?
        .get("result")?
        .as_array()?
        .first()?
        .get("value")?
        .as_array()?
        .get(1)?
        .as_str()?
        .parse::<f64>()
        .ok()
}

/// Try to fetch Enphase data from Home Assistant
async fn fetch_enphase_from_ha(client: &reqwest::Client) -> (f64, f64) {
    use std::env;

    let ha_url = env::var("HOMEASSISTANT_URL")
        .unwrap_or_else(|_| "http://homeassistant.local:8123".to_string());
    let ha_token = env::var("HOMEASSISTANT_TOKEN").unwrap_or_default();

    if ha_token.is_empty() {
        return (0.0, 0.0);
    }

    let response = match client
        .get(format!("{}/api/states", ha_url))
        .header("Authorization", format!("Bearer {}", ha_token))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        _ => return (0.0, 0.0),
    };

    let states: Vec<serde_json::Value> = match response.json().await {
        Ok(s) => s,
        Err(_) => return (0.0, 0.0),
    };

    let mut solar_prod = 0.0;
    let mut house_cons = 0.0;

    for state in states {
        let entity_id = state["entity_id"].as_str().unwrap_or("");
        let state_value = state["state"].as_str().unwrap_or("0");
        let value = state_value.parse::<f64>().unwrap_or(0.0);

        match entity_id {
            "sensor.envoy_122304017410_current_power_production"
            | "sensor.enphase_solar_production" => solar_prod = value,
            "sensor.envoy_122304017410_current_power_consumption"
            | "sensor.enphase_house_consumption" => house_cons = value,
            _ => {}
        }
    }

    (solar_prod, house_cons)
}

/// Fetch GPU metrics from gpu-hot external service
/// https://gpu-hot.p.zacharie.org/api/gpu-data
async fn fetch_gpu_from_hot_service(client: &reqwest::Client) -> (f64, f64, f64) {
    let gpu_hot_url = std::env::var("GPU_HOT_URL")
        .unwrap_or_else(|_| "https://gpu-hot.p.zacharie.org".to_string());

    let api_url = format!("{}/api/gpu-data", gpu_hot_url);
    tracing::info!("Fetching GPU metrics from {}", api_url);

    let response = match client
        .get(&api_url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            tracing::warn!("GPU-HOT API returned status: {}", r.status());
            return (0.0, 0.0, 0.0);
        }
        Err(e) => {
            tracing::warn!("Failed to connect to GPU-HOT API: {}", e);
            return (0.0, 0.0, 0.0);
        }
    };

    let data: serde_json::Value = match response.json().await {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("Failed to parse GPU-HOT JSON: {}", e);
            return (0.0, 0.0, 0.0);
        }
    };

    // The response format is: {"gpus": {"0": {...}}, "timestamp": "..."}
    // Get the first GPU's data
    let gpu_data = data["gpus"]
        .as_object()
        .and_then(|gpus| gpus.values().next())
        .unwrap_or(&data);

    tracing::debug!("GPU data extracted: {:?}", gpu_data);

    // Extract metrics from the GPU data
    // Fields: utilization, temperature, power_draw
    let gpu_util = gpu_data["utilization"]
        .as_f64()
        .unwrap_or(0.0);

    let gpu_temp = gpu_data["temperature"]
        .as_f64()
        .unwrap_or(0.0);

    let gpu_power = gpu_data["power_draw"]
        .as_f64()
        .unwrap_or(0.0);

    if gpu_util > 0.0 || gpu_temp > 0.0 || gpu_power > 0.0 {
        tracing::info!(
            "GPU-HOT API metrics: util={:.1}%, temp={:.1}°C, power={:.1}W",
            gpu_util, gpu_temp, gpu_power
        );
    } else {
        tracing::warn!("No GPU metrics found in GPU-HOT API response: {:?}", data);
    }

    (gpu_util, gpu_temp, gpu_power)
}
