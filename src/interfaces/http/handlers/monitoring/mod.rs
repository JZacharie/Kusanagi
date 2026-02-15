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

    let hot_metrics = fetch_gpu_from_hot_service(&state.http_client).await;

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
            "utilization": hot_metrics.utilization,
            "temperature": hot_metrics.temperature,
            "power": hot_metrics.power_draw,
            "memory_used": hot_metrics.memory_used,
            "memory_total": hot_metrics.memory_total,
            "memory_utilization": hot_metrics.memory_utilization,
            "fan_speed": hot_metrics.fan_speed,
            "clock_graphics": hot_metrics.clock_graphics,
            "clock_memory": hot_metrics.clock_memory,
        },
        "gpu_hot_raw_response": raw_json,
        "status": if hot_metrics.utilization > 0.0 || hot_metrics.temperature > 0.0 || hot_metrics.power_draw > 0.0 {
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
    let gpu_metrics = fetch_gpu_and_energy_metrics(&state.http_client).await;

    api_success(serde_json::json!({
        "cpu_usage_percent": cpu_percent,
        "memory_usage_percent": mem_percent,
        "pod_count": pod_count,
        "node_count": node_count,
        "alerts_firing": alerts_firing,
        "container_count": pod_count, // Approximation
        "gpu_utilization": gpu_metrics.utilization,
        "gpu_temperature": gpu_metrics.temperature,
        "gpu_power_usage": gpu_metrics.power_draw,
        "gpu_memory_used": gpu_metrics.memory_used,
        "gpu_memory_total": gpu_metrics.memory_total,
        "gpu_memory_utilization": gpu_metrics.memory_utilization,
        "gpu_fan_speed": gpu_metrics.fan_speed,
        "gpu_clock_graphics": gpu_metrics.clock_graphics,
        "gpu_clock_memory": gpu_metrics.clock_memory,
        "energy_solar_production": gpu_metrics.solar_production,
        "energy_house_consumption": gpu_metrics.house_consumption,
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

/// GPU Metrics structure
#[derive(Debug, Default)]
struct GpuMetrics {
    utilization: f64,
    temperature: f64,
    power_draw: f64,
    memory_used: f64,
    memory_total: f64,
    memory_utilization: f64,
    fan_speed: f64,
    clock_graphics: f64,
    clock_memory: f64,
    solar_production: f64,
    house_consumption: f64,
}

/// Fetch GPU and Energy metrics from Prometheus or Home Assistant
async fn fetch_gpu_and_energy_metrics(client: &reqwest::Client) -> GpuMetrics {
    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| {
        "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
    });

    let url = format!("{}/api/v1/query", prometheus_url);
    let mut metrics = GpuMetrics::default();

    // Try multiple GPU metric sources
    // 1. Try DCGM exporter (NVIDIA Data Center GPU Manager)
    if let Some(val) = query_prometheus_scalar(client, &url, r#"avg(DCGM_FI_DEV_GPU_UTIL)"#).await {
        metrics.utilization = val;
    }
    if let Some(val) = query_prometheus_scalar(client, &url, r#"avg(DCGM_FI_DEV_GPU_TEMP)"#).await {
        metrics.temperature = val;
    }
    if let Some(val) = query_prometheus_scalar(client, &url, r#"avg(DCGM_FI_DEV_POWER_USAGE)"#).await {
        metrics.power_draw = val;
    }

    // 2. Try nvidia_gpu_exporter (https://github.com/utkuozdemir/nvidia_gpu_exporter)
    if metrics.utilization == 0.0 {
        if let Some(val) = query_prometheus_scalar(client, &url, r#"nvidia_gpu_utilization_gpu_utilization"#).await {
            metrics.utilization = val;
        }
    }
    if metrics.temperature == 0.0 {
        if let Some(val) = query_prometheus_scalar(client, &url, r#"nvidia_gpu_temperature_gpu_temperature"#).await {
            metrics.temperature = val;
        }
    }
    if metrics.power_draw == 0.0 {
        if let Some(val) = query_prometheus_scalar(client, &url, r#"nvidia_gpu_power_usage_gpu_power_usage"#).await {
            metrics.power_draw = val;
        }
    }

    // 3. Try node_exporter with NVIDIA textfile collector
    if metrics.utilization == 0.0 {
        if let Some(val) = query_prometheus_scalar(client, &url, r#"nvidia_gpu_utilization_percentage"#).await {
            metrics.utilization = val;
        }
    }
    if metrics.temperature == 0.0 {
        if let Some(val) = query_prometheus_scalar(client, &url, r#"nvidia_gpu_temperature_celsius"#).await {
            metrics.temperature = val;
        }
    }
    if metrics.power_draw == 0.0 {
        if let Some(val) = query_prometheus_scalar(client, &url, r#"nvidia_gpu_power_draw_watts"#).await {
            metrics.power_draw = val;
        }
    }

    // 4. Try generic node_exporter GPU metrics (if available)
    if metrics.utilization == 0.0 {
        if let Some(val) = query_prometheus_scalar(client, &url, r#"gpu_utilization_percent"#).await {
            metrics.utilization = val;
        }
    }

    // 5. Try external GPU-HOT service as fallback
    if metrics.utilization == 0.0 && metrics.temperature == 0.0 && metrics.power_draw == 0.0 {
        tracing::info!("No GPU metrics from Prometheus, trying gpu-hot service");
        let hot_metrics = fetch_gpu_from_hot_service(client).await;
        metrics.utilization = hot_metrics.utilization;
        metrics.temperature = hot_metrics.temperature;
        metrics.power_draw = hot_metrics.power_draw;
        metrics.memory_used = hot_metrics.memory_used;
        metrics.memory_total = hot_metrics.memory_total;
        metrics.memory_utilization = hot_metrics.memory_utilization;
        metrics.fan_speed = hot_metrics.fan_speed;
        metrics.clock_graphics = hot_metrics.clock_graphics;
        metrics.clock_memory = hot_metrics.clock_memory;
    }

    // Try Home Assistant for Enphase data if Prometheus doesn't have it
    let (solar_prod, house_cons) = fetch_enphase_from_ha(client).await;
    metrics.solar_production = solar_prod;
    metrics.house_consumption = house_cons;

    metrics
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
async fn fetch_gpu_from_hot_service(client: &reqwest::Client) -> GpuMetrics {
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
            return GpuMetrics::default();
        }
        Err(e) => {
            tracing::warn!("Failed to connect to GPU-HOT API: {}", e);
            return GpuMetrics::default();
        }
    };

    let data: serde_json::Value = match response.json().await {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("Failed to parse GPU-HOT JSON: {}", e);
            return GpuMetrics::default();
        }
    };

    // The response format is: {"gpus": {"0": {...}}, "timestamp": "..."}
    // Get the first GPU's data
    let gpu_data = data["gpus"]
        .as_object()
        .and_then(|gpus| gpus.values().next())
        .cloned()
        .unwrap_or_default();

    tracing::debug!("GPU data extracted: {:?}", gpu_data);

    // Extract all metrics from the GPU data
    let mut metrics = GpuMetrics::default();
    
    metrics.utilization = gpu_data["utilization"].as_f64().unwrap_or(0.0);
    metrics.temperature = gpu_data["temperature"].as_f64().unwrap_or(0.0);
    metrics.power_draw = gpu_data["power_draw"].as_f64().unwrap_or(0.0);
    metrics.memory_used = gpu_data["memory_used"].as_f64().unwrap_or(0.0);
    metrics.memory_total = gpu_data["memory_total"].as_f64().unwrap_or(0.0);
    metrics.memory_utilization = gpu_data["memory_utilization"].as_f64().unwrap_or(0.0);
    metrics.fan_speed = gpu_data["fan_speed"].as_f64().unwrap_or(0.0);
    metrics.clock_graphics = gpu_data["clock_graphics"].as_f64().unwrap_or(0.0);
    metrics.clock_memory = gpu_data["clock_memory"].as_f64().unwrap_or(0.0);

    if metrics.utilization > 0.0 || metrics.temperature > 0.0 || metrics.power_draw > 0.0 {
        tracing::info!(
            "GPU-HOT API: util={:.1}%, temp={:.1}°C, power={:.1}W, mem={:.0}/{:.0}MB, fan={:.0}%",
            metrics.utilization, metrics.temperature, metrics.power_draw,
            metrics.memory_used, metrics.memory_total, metrics.fan_speed
        );
    } else {
        tracing::warn!("No GPU metrics found in GPU-HOT API response");
    }

    metrics
}
