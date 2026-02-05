use crate::error::Result;
use crate::domain::entities_simple::{PrometheusMetrics, NodeMetric, PodMetric};
use async_trait::async_trait;
use prometheus_http_query::{Client, Selector};
use std::collections::HashMap;

#[async_trait]
pub trait PrometheusRepository {
    async fn get_cluster_metrics(&self) -> Result<PrometheusMetrics>;
}

pub struct PrometheusRepo {
    client: Option<Client>,
    is_mock: bool,
}

impl PrometheusRepo {
    pub fn new() -> Self {
        let prometheus_url = std::env::var("PROMETHEUS_URL")
            .or_else(|_| std::env::var("KUSANAGI_PROMETHEUS_URL"))
            .unwrap_or_else(|_| "http://prometheus:9090".to_string());

        match Client::try_from(prometheus_url.as_str()) {
            Ok(client) => Self { 
                client: Some(client), 
                is_mock: false 
            },
            Err(_) => {
                println!("⚠️  Failed to connect to Prometheus at {}, using mock data", prometheus_url);
                Self { 
                    client: None, 
                    is_mock: true 
                }
            }
        }
    }
}

#[async_trait]
impl PrometheusRepository for PrometheusRepo {
    async fn get_cluster_metrics(&self) -> Result<PrometheusMetrics> {
        if self.is_mock || self.client.is_none() {
            return Ok(PrometheusMetrics {
                cluster_cpu_usage: 45.2,
                cluster_memory_usage: 67.8,
                node_metrics: vec![
                    NodeMetric {
                        node_name: "node-1".to_string(),
                        cpu_usage: 42.1,
                        memory_usage: 65.3,
                        disk_usage: 78.9,
                    },
                    NodeMetric {
                        node_name: "node-2".to_string(),
                        cpu_usage: 48.3,
                        memory_usage: 70.2,
                        disk_usage: 82.1,
                    },
                ],
                pod_metrics: vec![
                    PodMetric {
                        pod_name: "app-1".to_string(),
                        namespace: "default".to_string(),
                        cpu_usage: 15.2,
                        memory_usage: 234.5,
                    },
                    PodMetric {
                        pod_name: "app-2".to_string(),
                        namespace: "kube-system".to_string(),
                        cpu_usage: 8.7,
                        memory_usage: 156.3,
                    },
                ],
            });
        }

        let client = self.client.as_ref().unwrap();

        // Query cluster CPU usage
        let cluster_cpu = match client.query("100 - (avg(irate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100)").get().await {
            Ok(data) => data.as_vector().and_then(|v| v.first()).and_then(|s| s.sample().value().parse::<f64>().ok()).unwrap_or(0.0),
            Err(_) => 0.0,
        };

        // Query cluster memory usage
        let cluster_memory = match client.query("(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100").get().await {
            Ok(data) => data.as_vector().and_then(|v| v.first()).and_then(|s| s.sample().value().parse::<f64>().ok()).unwrap_or(0.0),
            Err(_) => 0.0,
        };

        // Query node metrics
        let mut node_metrics = Vec::new();
        if let Ok(data) = client.query("up{job=\"node-exporter\"}").get().await {
            if let Some(vector) = data.as_vector() {
                for sample in vector {
                    if let Some(instance) = sample.metric().get("instance") {
                        let node_name = instance.split(':').next().unwrap_or(instance).to_string();
                        node_metrics.push(NodeMetric {
                            node_name,
                            cpu_usage: cluster_cpu, // Simplified for now
                            memory_usage: cluster_memory,
                            disk_usage: 75.0, // Mock value
                        });
                    }
                }
            }
        }

        // Query pod metrics (simplified)
        let mut pod_metrics = Vec::new();
        if let Ok(data) = client.query("rate(container_cpu_usage_seconds_total{container!=\"POD\",container!=\"\"}[5m]) * 100").get().await {
            if let Some(vector) = data.as_vector() {
                for sample in vector.iter().take(10) { // Limit to 10 pods
                    if let (Some(pod), Some(namespace)) = (sample.metric().get("pod"), sample.metric().get("namespace")) {
                        let cpu_usage = sample.sample().value().parse::<f64>().unwrap_or(0.0);
                        pod_metrics.push(PodMetric {
                            pod_name: pod.to_string(),
                            namespace: namespace.to_string(),
                            cpu_usage,
                            memory_usage: 128.0, // Mock value for now
                        });
                    }
                }
            }
        }

        Ok(PrometheusMetrics {
            cluster_cpu_usage: cluster_cpu,
            cluster_memory_usage: cluster_memory,
            node_metrics,
            pod_metrics,
        })
    }
}
