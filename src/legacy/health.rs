//! Global Health Check Module
//! 
//! Provides comprehensive health checks for all Kusanagi dependencies:
//! - Kubernetes API
//! - MQTT Broker
//! - PostgreSQL Database
//! - Prometheus
//! - AlertManager
//! - External APIs (ArgoCD, Cilium, etc.)

use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use kube::Client;

/// Health status variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub response_time_ms: u64,
    pub message: Option<String>,
    pub last_check: String,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Overall health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub version: String,
    pub timestamp: String,
    pub uptime_seconds: u64,
    pub components: Vec<ComponentHealth>,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthConfig {
    pub timeout: Duration,
    pub check_kubernetes: bool,
    pub check_mqtt: bool,
    pub check_database: bool,
    pub check_prometheus: bool,
    pub check_alertmanager: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            check_kubernetes: true,
            check_mqtt: true,
            check_database: true,
            check_prometheus: true,
            check_alertmanager: true,
        }
    }
}

lazy_static::lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

/// Get application uptime in seconds
pub fn get_uptime() -> u64 {
    START_TIME.elapsed().as_secs()
}

/// Check Kubernetes API health
async fn check_kubernetes(client: &Client) -> ComponentHealth {
    let start = Instant::now();
    
    match client.apiserver_version().await {
        Ok(version) => {
            let elapsed = start.elapsed().as_millis() as u64;
            let mut metadata = HashMap::new();
            metadata.insert("git_version".to_string(), 
                serde_json::Value::String(version.git_version.clone()));
            metadata.insert("major".to_string(), 
                serde_json::Value::String(version.major.clone()));
            metadata.insert("minor".to_string(), 
                serde_json::Value::String(version.minor.clone()));
            
            ComponentHealth {
                name: "kubernetes".to_string(),
                status: HealthStatus::Healthy,
                response_time_ms: elapsed,
                message: Some(format!("K8s {}", version.git_version)),
                last_check: chrono::Utc::now().to_rfc3339(),
                metadata: Some(metadata),
            }
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ComponentHealth {
                name: "kubernetes".to_string(),
                status: HealthStatus::Unhealthy,
                response_time_ms: elapsed,
                message: Some(format!("Failed to connect: {}", e)),
                last_check: chrono::Utc::now().to_rfc3339(),
                metadata: None,
            }
        }
    }
}

/// Check MQTT broker health
async fn check_mqtt() -> ComponentHealth {
    let start = Instant::now();
    
    let state = crate::legacy::mqtt::MQTT_STATE.lock().unwrap();
    let elapsed = start.elapsed().as_millis() as u64;
    
    if state.connected {
        let mut metadata = HashMap::new();
        metadata.insert("broker_host".to_string(), 
            serde_json::Value::String(state.broker_host.clone()));
        metadata.insert("broker_port".to_string(), 
            serde_json::Value::Number(state.broker_port.into()));
        metadata.insert("message_count".to_string(), 
            serde_json::Value::Number(state.message_count.into()));
        metadata.insert("device_count".to_string(), 
            serde_json::Value::Number(state.devices.len().into()));
        
        ComponentHealth {
            name: "mqtt".to_string(),
            status: HealthStatus::Healthy,
            response_time_ms: elapsed,
            message: Some(format!("Connected to {}:{}", state.broker_host, state.broker_port)),
            last_check: chrono::Utc::now().to_rfc3339(),
            metadata: Some(metadata),
        }
    } else {
        ComponentHealth {
            name: "mqtt".to_string(),
            status: HealthStatus::Unhealthy,
            response_time_ms: elapsed,
            message: state.last_error.clone().or_else(|| Some("Not connected".to_string())),
            last_check: chrono::Utc::now().to_rfc3339(),
            metadata: None,
        }
    }
}

/// Check PostgreSQL database health
async fn check_database() -> ComponentHealth {
    let start = Instant::now();
    
    // Try to get database pool from database module
    match crate::legacy::database::get_pool().await {
        Ok(pool) => {
            match sqlx::query("SELECT 1").fetch_one(&*pool).await {
                Ok(_) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    ComponentHealth {
                        name: "database".to_string(),
                        status: HealthStatus::Healthy,
                        response_time_ms: elapsed,
                        message: Some("Connected".to_string()),
                        last_check: chrono::Utc::now().to_rfc3339(),
                        metadata: None,
                    }
                }
                Err(e) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    ComponentHealth {
                        name: "database".to_string(),
                        status: HealthStatus::Unhealthy,
                        response_time_ms: elapsed,
                        message: Some(format!("Query failed: {}", e)),
                        last_check: chrono::Utc::now().to_rfc3339(),
                        metadata: None,
                    }
                }
            }
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ComponentHealth {
                name: "database".to_string(),
                status: HealthStatus::Unhealthy,
                response_time_ms: elapsed,
                message: Some(format!("Pool error: {}", e)),
                last_check: chrono::Utc::now().to_rfc3339(),
                metadata: None,
            }
        }
    }
}

/// Check Prometheus health
async fn check_prometheus() -> ComponentHealth {
    let start = Instant::now();
    
    match crate::legacy::prometheus::query_raw("up").await {
        Ok(_) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ComponentHealth {
                name: "prometheus".to_string(),
                status: HealthStatus::Healthy,
                response_time_ms: elapsed,
                message: Some("Connected".to_string()),
                last_check: chrono::Utc::now().to_rfc3339(),
                metadata: None,
            }
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ComponentHealth {
                name: "prometheus".to_string(),
                status: HealthStatus::Unhealthy,
                response_time_ms: elapsed,
                message: Some(format!("Failed: {}", e)),
                last_check: chrono::Utc::now().to_rfc3339(),
                metadata: None,
            }
        }
    }
}

/// Check AlertManager health
async fn check_alertmanager() -> ComponentHealth {
    let start = Instant::now();
    
    match crate::legacy::alertmanager::get_active_alerts().await {
        Ok(_) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ComponentHealth {
                name: "alertmanager".to_string(),
                status: HealthStatus::Healthy,
                response_time_ms: elapsed,
                message: Some("Connected".to_string()),
                last_check: chrono::Utc::now().to_rfc3339(),
                metadata: None,
            }
        }
        Err(e) => {
            let elapsed = start.elapsed().as_millis() as u64;
            ComponentHealth {
                name: "alertmanager".to_string(),
                status: HealthStatus::Unhealthy,
                response_time_ms: elapsed,
                message: Some(format!("Failed: {}", e)),
                last_check: chrono::Utc::now().to_rfc3339(),
                metadata: None,
            }
        }
    }
}

/// Perform comprehensive health check
pub async fn check_health(client: &Client, config: &HealthConfig) -> HealthReport {
    let mut components = Vec::new();
    let mut has_unhealthy = false;
    let mut has_degraded = false;

    // Check Kubernetes
    if config.check_kubernetes {
        let health = check_kubernetes(client).await;
        if health.status == HealthStatus::Unhealthy {
            has_unhealthy = true;
        }
        components.push(health);
    }

    // Check MQTT
    if config.check_mqtt {
        let health = check_mqtt().await;
        if health.status == HealthStatus::Unhealthy {
            has_unhealthy = true;
        }
        components.push(health);
    }

    // Check Database
    if config.check_database {
        let health = check_database().await;
        if health.status == HealthStatus::Unhealthy {
            has_unhealthy = true;
        } else if health.status == HealthStatus::Degraded {
            has_degraded = true;
        }
        components.push(health);
    }

    // Check Prometheus
    if config.check_prometheus {
        let health = check_prometheus().await;
        if health.status == HealthStatus::Unhealthy {
            has_unhealthy = true;
        }
        components.push(health);
    }

    // Check AlertManager
    if config.check_alertmanager {
        let health = check_alertmanager().await;
        if health.status == HealthStatus::Unhealthy {
            has_unhealthy = true;
        }
        components.push(health);
    }

    // Determine overall status
    let status = if has_unhealthy {
        HealthStatus::Unhealthy
    } else if has_degraded {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    };

    HealthReport {
        status,
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        uptime_seconds: get_uptime(),
        components,
    }
}

/// Simple liveness check - is the application running?
#[get("/health/live")]
pub async fn liveness_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

/// Readiness check - is the application ready to serve traffic?
#[get("/health/ready")]
pub async fn readiness_check(data: web::Data<crate::AppState>) -> HttpResponse {
    let config = HealthConfig {
        check_kubernetes: true,
        check_mqtt: false,
        check_database: false,
        check_prometheus: false,
        check_alertmanager: false,
        ..Default::default()
    };

    let report = check_health(&data.client, &config).await;

    match report.status {
        HealthStatus::Healthy => HttpResponse::Ok().json(report),
        _ => HttpResponse::ServiceUnavailable().json(report),
    }
}

/// Full health check with all components
#[get("/health/full")]
pub async fn full_health_check(data: web::Data<crate::AppState>) -> HttpResponse {
    let config = HealthConfig::default();
    let report = check_health(&data.client, &config).await;

    match report.status {
        HealthStatus::Healthy => HttpResponse::Ok().json(report),
        HealthStatus::Degraded => HttpResponse::Ok().json(report),
        HealthStatus::Unhealthy => HttpResponse::ServiceUnavailable().json(report),
    }
}

pub fn configure_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(liveness_check)
       .service(readiness_check)
       .service(full_health_check);
}
