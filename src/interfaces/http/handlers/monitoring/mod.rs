//! Monitoring handlers

pub mod cilium;
pub mod mqtt;

use axum::response::IntoResponse;
use serde_json::json;

pub use cilium::*;
pub use mqtt::*;

use crate::interfaces::http::response::{api_success, api_error};

/// Alerts endpoint
pub async fn alerts() -> impl IntoResponse {
    api_success(serde_json::json!({
        "alerts": [],
        "total": 0
    }))
}

/// Debug endpoint for Trivy vulnerabilities
pub async fn trivy_debug_handler(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
) -> impl IntoResponse {
    use crate::domain::services::trivy_service;

    let trivy_url = std::env::var("TRIVY_SERVER_URL")
        .unwrap_or_else(|_| "http://trivy-json-server.trivy-system.svc:8080".to_string());

    // Try to get vulnerabilities
    let vuln_result = trivy_service::get_vulnerabilities(&state.k8s_cache).await;
    let vuln_status = vuln_result.is_ok();
    let vuln_data = vuln_result.unwrap_or_else(|e| json!({"error": e}));

    // Try to list reports from S3
    let reports_result = trivy_service::list_reports().await;
    let reports_data = reports_result.unwrap_or_else(|e| json!({"error": e}));

    api_success(json!({
        "trivy_server_url": trivy_url,
        "vulnerabilities": vuln_data,
        "available_reports": reports_data,
        "status": if vuln_status { "ok" } else { "error" }
    }))
}

/// Endpoint for Enphase historical data (24h)
pub async fn enphase_history_handler(
    axum::extract::State(state): axum::extract::State<crate::state::AppState>,
) -> impl IntoResponse {
    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| {
        "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
    });
    
    // Query for the specific Enphase entity over 24h
    let entity_id = "envoy_122304017410_current_power_production";
    let query = format!(
        r#"homeassistant_sensor_unit_w{{entity="sensor.{}"}}"#,
        entity_id
    );
    
    let end = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let start = end - (24 * 3600); // 24 hours ago
    
    let url = format!("{}/api/v1/query_range", prometheus_url);
    
    let result = state.http_client
        .get(&url)
        .query(&[
            ("query", query.as_str()),
            ("start", &start.to_string()),
            ("end", &end.to_string()),
            ("step", "300"), // 5 minute intervals
        ])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;
    
    match result {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                api_success(json!({
                    "source": "prometheus",
                    "entity": entity_id,
                    "range": "24h",
                    "data": data
                }))
            } else {
                api_error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to parse Prometheus response"
                )
            }
        }
        Ok(resp) => {
            api_error(
                axum::http::StatusCode::BAD_GATEWAY,
                format!("Prometheus returned status: {}", resp.status())
            )
        }
        Err(e) => {
            api_error(
                axum::http::StatusCode::BAD_GATEWAY,
                format!("Failed to fetch from Prometheus: {}", e)
            )
        }
    }
}

/// Debug endpoint for Enphase data
pub async fn enphase_debug_handler(
    axum::extract::State(_state): axum::extract::State<crate::state::AppState>,
) -> impl IntoResponse {
    use std::env;
    
    let ha_url = env::var("HOME_ASSISTANT_URL")
        .unwrap_or_else(|_| "http://homeassistant.local:8123".to_string());
    let ha_token = env::var("HOME_ASSISTANT_TOKEN").unwrap_or_default();
    
    if ha_token.is_empty() {
        return api_error(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "HOME_ASSISTANT_TOKEN not configured"
        );
    }
    
    // Fetch specific entity
    let entity_id = "sensor.envoy_122304017410_current_power_production";
    let entity_url = format!("{}/api/states/{}", ha_url, entity_id);
    
    let client = reqwest::Client::new();
    let result = client
        .get(&entity_url)
        .header("Authorization", format!("Bearer {}", ha_token))
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await;
    
    match result {
        Ok(resp) if resp.status().is_success() => {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                api_success(json!({
                    "home_assistant_url": ha_url,
                    "entity_id": entity_id,
                    "entity_data": data,
                    "note": "If state is very large (>1M), it's likely cumulative energy (Wh), not power (W)"
                }))
            } else {
                api_error(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to parse entity data"
                )
            }
        }
        Ok(resp) => {
            api_error(
                axum::http::StatusCode::BAD_GATEWAY,
                format!("Home Assistant returned status: {}", resp.status())
            )
        }
        Err(e) => {
            api_error(
                axum::http::StatusCode::BAD_GATEWAY,
                format!("Failed to connect to Home Assistant: {}", e)
            )
        }
    }
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
        (
            "nvidia_gpu_util",
            r#"nvidia_gpu_utilization_gpu_utilization"#,
        ),
        (
            "nvidia_gpu_temp",
            r#"nvidia_gpu_temperature_gpu_temperature"#,
        ),
        (
            "nvidia_gpu_power",
            r#"nvidia_gpu_power_usage_gpu_power_usage"#,
        ),
    ];

    for (name, query) in &queries {
        let result = query_prometheus_scalar(&state.http_client, &url, query).await;
        prometheus_results.insert(
            name.to_string(),
            json!(result
                .map(|v| format!("{:.2}", v))
                .unwrap_or_else(|| "N/A".to_string())),
        );
    }

    // Fetch raw GPU-HOT API response
    let api_url = format!("{}/api/gpu-data", gpu_hot_url);
    let hot_raw_response = state
        .http_client
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
    if let Some(val) =
        query_prometheus_scalar(client, &url, r#"avg(DCGM_FI_DEV_POWER_USAGE)"#).await
    {
        metrics.power_draw = val;
    }

    // 2. Try nvidia_gpu_exporter (https://github.com/utkuozdemir/nvidia_gpu_exporter)
    if metrics.utilization == 0.0 {
        if let Some(val) =
            query_prometheus_scalar(client, &url, r#"nvidia_gpu_utilization_gpu_utilization"#).await
        {
            metrics.utilization = val;
        }
    }
    if metrics.temperature == 0.0 {
        if let Some(val) =
            query_prometheus_scalar(client, &url, r#"nvidia_gpu_temperature_gpu_temperature"#).await
        {
            metrics.temperature = val;
        }
    }
    if metrics.power_draw == 0.0 {
        if let Some(val) =
            query_prometheus_scalar(client, &url, r#"nvidia_gpu_power_usage_gpu_power_usage"#).await
        {
            metrics.power_draw = val;
        }
    }

    // 3. Try node_exporter with NVIDIA textfile collector
    if metrics.utilization == 0.0 {
        if let Some(val) =
            query_prometheus_scalar(client, &url, r#"nvidia_gpu_utilization_percentage"#).await
        {
            metrics.utilization = val;
        }
    }
    if metrics.temperature == 0.0 {
        if let Some(val) =
            query_prometheus_scalar(client, &url, r#"nvidia_gpu_temperature_celsius"#).await
        {
            metrics.temperature = val;
        }
    }
    if metrics.power_draw == 0.0 {
        if let Some(val) =
            query_prometheus_scalar(client, &url, r#"nvidia_gpu_power_draw_watts"#).await
        {
            metrics.power_draw = val;
        }
    }

    // 4. Try generic node_exporter GPU metrics (if available)
    if metrics.utilization == 0.0 {
        if let Some(val) = query_prometheus_scalar(client, &url, r#"gpu_utilization_percent"#).await
        {
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
/// Specifically looks for envoy_122304017410_current_power_production
async fn fetch_enphase_from_ha(client: &reqwest::Client) -> (f64, f64) {
    use std::env;

    let ha_url = env::var("HOME_ASSISTANT_URL")
        .unwrap_or_else(|_| "http://homeassistant.local:8123".to_string());
    let ha_token = env::var("HOME_ASSISTANT_TOKEN").unwrap_or_default();

    if ha_token.is_empty() {
        tracing::warn!("HOME_ASSISTANT_TOKEN not set, skipping Enphase data");
        return (0.0, 0.0);
    }

    tracing::info!("Fetching Enphase data from Home Assistant at {}", ha_url);

    // Try specific entity endpoints first
    let prod_entity = "sensor.envoy_122304017410_current_power_production";
    let cons_entity = "sensor.envoy_122304017410_current_power_consumption";
    
    let mut solar_prod = 0.0;
    let mut house_cons = 0.0;

    // Helper function to fetch a specific entity
    async fn fetch_entity_value(
        client: &reqwest::Client,
        ha_url: &str,
        ha_token: &str,
        entity: &str,
    ) -> f64 {
        let url = format!("{}/api/states/{}", ha_url, entity);
        match client
            .get(&url)
            .header("Authorization", format!("Bearer {}", ha_token))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(state) = resp.json::<serde_json::Value>().await {
                    let entity_id = state["entity_id"].as_str().unwrap_or("");
                    let state_value = state["state"].as_str().unwrap_or("0");
                    let attrs = state["attributes"].as_object().cloned().unwrap_or_default();
                    let unit = attrs.get("unit_of_measurement").and_then(|u| u.as_str()).unwrap_or("W");
                    
                    tracing::info!("Found Enphase entity: {} = {} {}", entity_id, state_value, unit);
                    
                    // Parse value, handling different units
                    if let Ok(val) = state_value.parse::<f64>() {
                        // Convert to Watts if needed
                        let watts = match unit {
                            "kW" => val * 1000.0,
                            "MW" => val * 1_000_000.0,
                            "Wh" => val, // Energy, not power
                            "kWh" => val * 1000.0,
                            _ => val, // Assume Watts
                        };
                        
                        // If value is extremely large (like 28 million), it might be lifetime energy
                        if watts > 1_000_000.0 && unit.contains("Wh") {
                            tracing::warn!("Large energy value detected ({} {}), this is likely cumulative energy, not power", val, unit);
                            return 0.0;
                        }
                        return watts;
                    }
                }
            }
            Ok(resp) => tracing::warn!("Failed to fetch {}: status {}", entity, resp.status()),
            Err(e) => tracing::warn!("Error fetching {}: {}", entity, e),
        }
        0.0
    }

    // Fetch both production and consumption
    solar_prod = fetch_entity_value(client, &ha_url, &ha_token, prod_entity).await;
    house_cons = fetch_entity_value(client, &ha_url, &ha_token, cons_entity).await;

    // If we didn't get consumption from specific endpoint, try all states
    let response = match client
        .get(format!("{}/api/states", ha_url))
        .header("Authorization", format!("Bearer {}", ha_token))
        .header("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            tracing::warn!("Home Assistant returned status: {}", r.status());
            return (0.0, 0.0);
        }
        Err(e) => {
            tracing::warn!("Failed to connect to Home Assistant: {}", e);
            return (0.0, 0.0);
        }
    };

    let states: Vec<serde_json::Value> = match response.json().await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("Failed to parse Home Assistant response: {}", e);
            return (solar_prod, house_cons); // Return what we got from specific endpoints
        }
    };

    let mut found_solar = solar_prod > 0.0;
    let mut found_consumption = house_cons > 0.0;

    for state in states {
        let entity_id = state["entity_id"].as_str().unwrap_or("");
        let state_value = state["state"].as_str().unwrap_or("0");
        let attrs = state["attributes"].as_object().cloned().unwrap_or_default();
        let unit = attrs.get("unit_of_measurement").and_then(|u| u.as_str()).unwrap_or("W");

        // Skip unavailable or unknown states
        if state_value == "unavailable" || state_value == "unknown" || state_value == "null" {
            continue;
        }

        let raw_value = state_value.parse::<f64>().unwrap_or(0.0);
        
        // Convert to Watts if needed
        let value = match unit {
            "kW" => raw_value * 1000.0,
            "MW" => raw_value * 1_000_000.0,
            _ => raw_value,
        };

        // Match various Enphase entity ID patterns
        if entity_id.contains("envoy") || entity_id.contains("enphase") {
            tracing::debug!("Found Enphase entity: {} = {} {}", entity_id, raw_value, unit);

            if entity_id.contains("production") || entity_id.contains("power_production") {
                // Skip if we already have a value from the specific endpoint
                if !found_solar && value > 0.0 && value < 1_000_000.0 {
                    solar_prod = value;
                    found_solar = true;
                    tracing::info!("Solar production from list: {}W from {}", value, entity_id);
                }
            }

            if entity_id.contains("consumption") || entity_id.contains("power_consumption") {
                if !found_consumption && value > 0.0 && value < 1_000_000.0 {
                    house_cons = value;
                    found_consumption = true;
                    tracing::info!("House consumption: {}W from {}", value, entity_id);
                }
            }
        }
    }

    if !found_solar && !found_consumption {
        tracing::warn!("No Enphase data found in Home Assistant states");
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
            metrics.utilization,
            metrics.temperature,
            metrics.power_draw,
            metrics.memory_used,
            metrics.memory_total,
            metrics.fan_speed
        );
    } else {
        tracing::warn!("No GPU metrics found in GPU-HOT API response");
    }

    metrics
}
