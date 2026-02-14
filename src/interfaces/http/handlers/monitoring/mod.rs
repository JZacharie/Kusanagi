//! Monitoring handlers

pub mod cilium;

use axum::response::IntoResponse;

pub use cilium::*;

use crate::interfaces::http::response::api_success;

/// Alerts endpoint
pub async fn alerts() -> impl IntoResponse {
    api_success(serde_json::json!({
        "alerts": [],
        "total": 0
    }))
}

/// Quotas endpoint
pub async fn quotas() -> impl IntoResponse {
    api_success(serde_json::json!({
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
    use crate::domain::services::trivy_service;

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

    // 4. Get Trivy vulnerabilities (don't block on error)
    let (trivy_critical, trivy_high, trivy_medium, trivy_low) =
        match trivy_service::get_vulnerabilities().await {
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

    // GPU metrics from dcgm_exporter (if available)
    // DCGM_FI_DEV_GPU_UTIL - GPU utilization
    let gpu_util_query = r#"avg(DCGM_FI_DEV_GPU_UTIL)"#;
    let gpu_util = query_prometheus_scalar(client, &url, gpu_util_query)
        .await
        .unwrap_or(0.0);

    // DCGM_FI_DEV_GPU_TEMP - GPU temperature
    let gpu_temp_query = r#"avg(DCGM_FI_DEV_GPU_TEMP)"#;
    let gpu_temp = query_prometheus_scalar(client, &url, gpu_temp_query)
        .await
        .unwrap_or(0.0);

    // DCGM_FI_DEV_POWER_USAGE - GPU power usage
    let gpu_power_query = r#"avg(DCGM_FI_DEV_POWER_USAGE)"#;
    let gpu_power = query_prometheus_scalar(client, &url, gpu_power_query)
        .await
        .unwrap_or(0.0);

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
