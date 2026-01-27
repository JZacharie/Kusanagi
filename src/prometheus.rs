use serde::{Deserialize, Serialize};

/// Prometheus metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct PrometheusMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_percent: f64,
    pub memory_usage_bytes: i64,
    pub pod_count: i32,
    pub node_count: i32,
    pub container_count: i32,
    pub alerts_firing: i32,
    pub alerts_pending: i32,
    pub gpu_utilization: f64,
    pub gpu_temperature: f64,
    pub gpu_power_usage: f64,
    pub energy_solar_production: f64,
    pub energy_house_consumption: f64,
    pub vps_cpu_usage: f64,
    pub vps_disk_usage: f64,
    pub vps_net_receive: f64,
}

lazy_static::lazy_static! {
    static ref PROMETHEUS_URL: String = {
        std::env::var("PROMETHEUS_URL")
            .unwrap_or_else(|_| {
                tracing::warn!("PROMETHEUS_URL not set, using default local K8s service URL");
                "http://kube-prometheus-stack-prometheus.kube-prometheus-stack.svc:9090".to_string()
            })
    };

    static ref PROMETHEUS_URL_HA: String = {
        std::env::var("PROMETHEUS_URL_HA")
            .unwrap_or_else(|_| PROMETHEUS_URL.clone())
    };
}

/// Prometheus query result
#[derive(Debug, Serialize, Deserialize)]
pub struct PrometheusQueryResult {
    pub status: String,
    pub data: serde_json::Value,
}

/// Prometheus instant query response
#[derive(Debug, Deserialize)]
struct PromResponse {
    status: String,
    data: PromData,
}

#[derive(Debug, Deserialize)]
struct PromData {
    #[serde(rename = "resultType")]
    _result_type: String,
    result: Vec<PromResult>,
}


#[derive(Debug, Deserialize)]
struct PromResult {
    #[serde(rename = "metric")]
    _metric: serde_json::Value,
    value: (f64, String),
}

fn get_prometheus_url() -> String {
    PROMETHEUS_URL.clone()
}

fn get_prometheus_url_ha() -> String {
    PROMETHEUS_URL_HA.clone()
}

/// Execute a PromQL instant query at a specific Prometheus URL
pub async fn query_instant_at(query: &str, url: &str) -> Result<f64, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/query", url);
    
    let response = client
        .get(&url)
        .query(&[("query", query)])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Prometheus request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Prometheus returned status: {}", response.status()));
    }
    
    let prom_response: PromResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Prometheus response: {}", e))?;
    
    if prom_response.status != "success" {
        return Err("Prometheus query failed".to_string());
    }
    
    // Get first result value
    if let Some(result) = prom_response.data.result.first() {
        result.value.1.parse::<f64>()
            .map_err(|e| format!("Failed to parse metric value: {}", e))
    } else {
        Ok(0.0)
    }
}

/// Execute a PromQL instant query (default URL)
pub async fn query_instant(query: &str) -> Result<f64, String> {
    query_instant_at(query, &get_prometheus_url()).await
}

/// Execute a raw PromQL query at a specific URL
pub async fn query_raw_at(query: &str, url: &str) -> Result<PrometheusQueryResult, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/query", url);
    
    let response = client
        .get(&url)
        .query(&[("query", query)])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Prometheus request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Prometheus returned status: {}", response.status()));
    }
    
    let result: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Prometheus response: {}", e))?;
    
    Ok(PrometheusQueryResult {
        status: result["status"].as_str().unwrap_or("unknown").to_string(),
        data: result["data"].clone(),
    })
}

/// Execute a raw PromQL query (default URL)
pub async fn query_raw(query: &str) -> Result<PrometheusQueryResult, String> {
    query_raw_at(query, &get_prometheus_url()).await
}

/// Execute a PromQL range query at a specific URL
pub async fn query_range_at(query: &str, start: i64, end: i64, step: &str, url: &str) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/query_range", url);
    
    let response = client
        .get(&url)
        .query(&[
            ("query", query),
            ("start", &start.to_string()),
            ("end", &end.to_string()),
            ("step", step),
        ])
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Prometheus request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Prometheus returned status: {}", response.status()));
    }
    
    let result: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Prometheus response: {}", e))?;
    
    Ok(result)
}

/// Execute a PromQL range query (default URL)
pub async fn query_range(query: &str, start: i64, end: i64, step: &str) -> Result<serde_json::Value, String> {
    query_range_at(query, start, end, step, &get_prometheus_url()).await
}

/// Get comprehensive cluster metrics from Prometheus
pub async fn get_cluster_metrics() -> Result<PrometheusMetrics, String> {
    let mut errors = Vec::new();

    // CPU usage across all nodes (percentage)
    let cpu_query = r#"100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)"#;
    let cpu_usage = match query_instant(cpu_query).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to query CPU usage: {}", e);
            errors.push(format!("CPU: {}", e));
            0.0
        }
    };
    
    // Memory usage percentage
    let mem_percent_query = r#"(1 - (sum(node_memory_MemAvailable_bytes) / sum(node_memory_MemTotal_bytes))) * 100"#;
    let memory_usage_percent = match query_instant(mem_percent_query).await {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to query memory percentage: {}", e);
            errors.push(format!("Memory %: {}", e));
            0.0
        }
    };
    
    // Memory usage in bytes
    let mem_bytes_query = r#"sum(node_memory_MemTotal_bytes) - sum(node_memory_MemAvailable_bytes)"#;
    let memory_usage_bytes = match query_instant(mem_bytes_query).await {
        Ok(v) => v as i64,
        Err(e) => {
            tracing::error!("Failed to query memory bytes: {}", e);
            0.0 as i64
        }
    };
    
    // Pod count
    let pod_query = r#"count(kube_pod_info) or vector(0)"#;
    let pod_count = match query_instant(pod_query).await {
        Ok(v) => v as i32,
        Err(e) => {
            tracing::error!("Failed to query pod count: {}", e);
            0
        }
    };
    
    // Node count
    let node_query = r#"count(kube_node_info) or vector(0)"#;
    let node_count = match query_instant(node_query).await {
        Ok(v) => v as i32,
        Err(e) => {
            tracing::error!("Failed to query node count: {}", e);
            0
        }
    };
    
    // Container count
    let container_query = r#"count(kube_pod_container_info) or vector(0)"#;
    let container_count = match query_instant(container_query).await {
        Ok(v) => v as i32,
        Err(e) => {
            tracing::error!("Failed to query container count: {}", e);
            0
        }
    };
    
    // Firing alerts
    let alerts_firing_query = r#"count(ALERTS{alertstate="firing"}) or vector(0)"#;
    let alerts_firing = query_instant(alerts_firing_query).await.unwrap_or(0.0) as i32;
    
    // Pending alerts
    let alerts_pending_query = r#"count(ALERTS{alertstate="pending"}) or vector(0)"#;
    let alerts_pending = query_instant(alerts_pending_query).await.unwrap_or(0.0) as i32;
    
    // Custom Job Status: NVIDIA GPU
    let gpu_query = r#"avg(nvidia_gpu_utilization) or avg(dcgm_gpu_utilization) or avg(DCGM_FI_DEV_GPU_UTIL) or vector(0)"#;
    let gpu_utilization = query_instant(gpu_query).await.unwrap_or(0.0);

    let gpu_temp_query = r#"avg(nvidia_gpu_temperature_celsius) or avg(dcgm_gpu_temp) or avg(DCGM_FI_DEV_GPU_TEMP) or vector(0)"#;
    let gpu_temperature = query_instant(gpu_temp_query).await.unwrap_or(0.0);

    let gpu_power_query = r#"avg(nvidia_gpu_power_usage_watts) or avg(dcgm_gpu_power_usage) or avg(DCGM_FI_DEV_POWER_USAGE) or vector(0)"#;
    let gpu_power_usage = query_instant(gpu_power_query).await.unwrap_or(0.0);

    // Energy Metrics from Home Assistant via Prometheus
    let solar_query = r#"avg(homeassistant_sensor_unit_w{entity="sensor.envoy_122304017410_current_power_production"}) or avg(homeassistant_sensor_unit_w{entity="sensor.solar_production"}) or avg(homeassistant_sensor_unit_w{entity="sensor.pv_production"}) or vector(0)"#;
    let energy_solar_production = query_instant_at(solar_query, &get_prometheus_url_ha()).await.unwrap_or(0.0);

    let consumption_query = r#"avg(homeassistant_sensor_unit_w{entity="sensor.envoy_122304017410_current_power_consumption"}) or avg(homeassistant_sensor_unit_w{entity="sensor.house_consumption"}) or avg(homeassistant_sensor_unit_w{entity="sensor.household_consumption"}) or vector(0)"#;
    let energy_house_consumption = query_instant_at(consumption_query, &get_prometheus_url_ha()).await.unwrap_or(0.0);

    // VPS Metrics from VPS.json - using sum/avg to ensure a single scalar result
    let vps_cpu_query = r#"avg(system_cpu_utilization{state!="idle"}) * 100 or vector(0)"#;
    let vps_cpu_usage = query_instant(vps_cpu_query).await.unwrap_or(0.0);

    let vps_disk_query = r#"sum(system_filesystem_usage_bytes{device="/dev/sda1",state="used"}) / sum(system_filesystem_usage_bytes{device="/dev/sda1"}) * 100 or vector(0)"#;
    let vps_disk_usage = query_instant(vps_disk_query).await.unwrap_or(0.0);

    let vps_net_query = r#"sum(rate(system_network_io_bytes_total{direction="receive", device="eth0"}[5m])) / 125000000 * 100 or vector(0)"#;
    let vps_net_receive = query_instant(vps_net_query).await.unwrap_or(0.0);
    
    if !errors.is_empty() {
        tracing::warn!("Some Prometheus metrics failed to load: {:?}", errors);
    }

    Ok(PrometheusMetrics {
        cpu_usage_percent: cpu_usage,
        memory_usage_percent,
        memory_usage_bytes,
        pod_count,
        node_count,
        container_count,
        alerts_firing,
        alerts_pending,
        gpu_utilization,
        gpu_temperature,
        gpu_power_usage,
        energy_solar_production,
        energy_house_consumption,
        vps_cpu_usage,
        vps_disk_usage,
        vps_net_receive,
    })
}

