use serde::{Deserialize, Serialize};
use std::env;

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
    result_type: String,
    result: Vec<PromResult>,
}

#[derive(Debug, Deserialize)]
struct PromResult {
    metric: serde_json::Value,
    value: (f64, String),
}

fn get_prometheus_url() -> String {
    env::var("PROMETHEUS_URL")
        .unwrap_or_else(|_| "http://prometheus-server.observability.svc:9090".to_string())
}

/// Execute a PromQL instant query
pub async fn query_instant(query: &str) -> Result<f64, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/query", get_prometheus_url());
    
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

/// Execute a raw PromQL query and return the full result
pub async fn query_raw(query: &str) -> Result<PrometheusQueryResult, String> {
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/query", get_prometheus_url());
    
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

/// Get comprehensive cluster metrics from Prometheus
pub async fn get_cluster_metrics() -> Result<PrometheusMetrics, String> {
    // CPU usage across all nodes (percentage)
    let cpu_query = r#"100 - (avg(rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)"#;
    let cpu_usage = query_instant(cpu_query).await.unwrap_or(0.0);
    
    // Memory usage percentage
    let mem_percent_query = r#"(1 - (sum(node_memory_MemAvailable_bytes) / sum(node_memory_MemTotal_bytes))) * 100"#;
    let memory_usage_percent = query_instant(mem_percent_query).await.unwrap_or(0.0);
    
    // Memory usage in bytes
    let mem_bytes_query = r#"sum(node_memory_MemTotal_bytes) - sum(node_memory_MemAvailable_bytes)"#;
    let memory_usage_bytes = query_instant(mem_bytes_query).await.unwrap_or(0.0) as i64;
    
    // Pod count
    let pod_query = r#"count(kube_pod_info)"#;
    let pod_count = query_instant(pod_query).await.unwrap_or(0.0) as i32;
    
    // Node count
    let node_query = r#"count(kube_node_info)"#;
    let node_count = query_instant(node_query).await.unwrap_or(0.0) as i32;
    
    // Container count
    let container_query = r#"count(kube_pod_container_info)"#;
    let container_count = query_instant(container_query).await.unwrap_or(0.0) as i32;
    
    // Firing alerts
    let alerts_firing_query = r#"count(ALERTS{alertstate="firing"}) or vector(0)"#;
    let alerts_firing = query_instant(alerts_firing_query).await.unwrap_or(0.0) as i32;
    
    // Pending alerts
    let alerts_pending_query = r#"count(ALERTS{alertstate="pending"}) or vector(0)"#;
    let alerts_pending = query_instant(alerts_pending_query).await.unwrap_or(0.0) as i32;
    
    Ok(PrometheusMetrics {
        cpu_usage_percent: cpu_usage,
        memory_usage_percent,
        memory_usage_bytes,
        pod_count,
        node_count,
        container_count,
        alerts_firing,
        alerts_pending,
    })
}

/// Get top resource-consuming pods
pub async fn get_top_pods(limit: usize) -> Result<Vec<serde_json::Value>, String> {
    let query = format!(
        r#"topk({}, sum by (pod, namespace) (rate(container_cpu_usage_seconds_total{{container!=""}}[5m])))"#,
        limit
    );
    
    let result = query_raw(&query).await?;
    
    if let Some(results) = result.data.get("result") {
        Ok(results.as_array().cloned().unwrap_or_default())
    } else {
        Ok(vec![])
    }
}

/// Get node resource utilization
pub async fn get_node_resources() -> Result<Vec<serde_json::Value>, String> {
    let cpu_query = r#"100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100)"#;
    let result = query_raw(cpu_query).await?;
    
    if let Some(results) = result.data.get("result") {
        Ok(results.as_array().cloned().unwrap_or_default())
    } else {
        Ok(vec![])
    }
}
