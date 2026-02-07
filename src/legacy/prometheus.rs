// Legacy Prometheus module - minimal
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PrometheusMetric {
    pub name: String,
    pub value: f64,
    pub timestamp: u64,
}

pub async fn get_metrics() -> Result<Vec<PrometheusMetric>, Box<dyn std::error::Error>> {
    Ok(vec![
        PrometheusMetric {
            name: "legacy_cpu_usage".to_string(),
            value: 25.5,
            timestamp: chrono::Utc::now().timestamp() as u64,
        },
        PrometheusMetric {
            name: "legacy_memory_usage".to_string(),
            value: 55.2,
            timestamp: chrono::Utc::now().timestamp() as u64,
        },
    ])
}
