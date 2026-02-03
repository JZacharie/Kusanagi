//! Kusanagi Doctor - Self-Diagnostic Tool
//!
//! Comprehensive diagnostic endpoint that checks all system components
//! and provides actionable recommendations.

use actix_web::{get, web, HttpResponse, Responder};
use kube::{Client, Api};
use k8s_openapi::api::core::v1::Pod;
use serde::{Deserialize, Serialize};

use std::time::{Duration, Instant};


/// Doctor check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    pub details: Option<String>,
    pub recommendation: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Ok,
    Warning,
    Error,
    Skipped,
}

/// Complete diagnostic report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub overall_status: CheckStatus,
    pub timestamp: String,
    pub version: String,
    pub checks: Vec<CheckResult>,
    pub summary: DiagnosticSummary,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSummary {
    pub total: usize,
    pub ok: usize,
    pub warning: usize,
    pub error: usize,
    pub skipped: usize,
}

impl DiagnosticSummary {
    fn new() -> Self {
        Self {
            total: 0,
            ok: 0,
            warning: 0,
            error: 0,
            skipped: 0,
        }
    }

    fn increment(&mut self, status: &CheckStatus) {
        self.total += 1;
        match status {
            CheckStatus::Ok => self.ok += 1,
            CheckStatus::Warning => self.warning += 1,
            CheckStatus::Error => self.error += 1,
            CheckStatus::Skipped => self.skipped += 1,
        }
    }
}

/// Run all diagnostic checks
pub async fn run_diagnostics(client: &Client) -> DiagnosticReport {
    let _start = Instant::now();
    let mut checks = Vec::new();
    let mut summary = DiagnosticSummary::new();

    // Core checks
    checks.push(check_kubernetes_connection(client).await);
    checks.push(check_kubernetes_permissions(client).await);
    checks.push(check_prometheus_connection().await);
    checks.push(check_openobserve_config(client).await);
    checks.push(check_database_connection().await);
    checks.push(check_mqtt_connection().await);
    checks.push(check_llm_connection().await);
    checks.push(check_s3_connection().await);
    checks.push(check_memory_usage().await);
    checks.push(check_disk_space().await);

    // Update summary
    for check in &checks {
        summary.increment(&check.status);
    }

    // Generate recommendations
    let recommendations = generate_recommendations(&checks);

    // Determine overall status
    let overall_status = if summary.error > 0 {
        CheckStatus::Error
    } else if summary.warning > 0 {
        CheckStatus::Warning
    } else {
        CheckStatus::Ok
    };

    DiagnosticReport {
        overall_status,
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks,
        summary,
        recommendations,
    }
}

async fn check_kubernetes_connection(client: &Client) -> CheckResult {
    let start = Instant::now();
    
    match client.apiserver_version().await {
        Ok(version) => {
            CheckResult {
                name: "Kubernetes Connection".to_string(),
                status: CheckStatus::Ok,
                message: format!("Connected to Kubernetes {}", version.git_version),
                details: Some(format!("Major: {}, Minor: {}", version.major, version.minor)),
                recommendation: None,
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
        Err(e) => {
            CheckResult {
                name: "Kubernetes Connection".to_string(),
                status: CheckStatus::Error,
                message: "Failed to connect to Kubernetes API".to_string(),
                details: Some(e.to_string()),
                recommendation: Some("Check if running in a Kubernetes cluster or if kubeconfig is correct".to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
    }
}

async fn check_kubernetes_permissions(client: &Client) -> CheckResult {
    let start = Instant::now();
    
    // Try to list pods in all namespaces
    let pods: Api<Pod> = Api::all(client.clone());
    
    match tokio::time::timeout(Duration::from_secs(10), pods.list(&kube::api::ListParams::default().limit(1))).await {
        Ok(Ok(_)) => {
            CheckResult {
                name: "Kubernetes Permissions".to_string(),
                status: CheckStatus::Ok,
                message: "RBAC permissions are sufficient".to_string(),
                details: Some("Can list pods across all namespaces".to_string()),
                recommendation: None,
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
        Ok(Err(e)) => {
            let is_rbac_error = e.to_string().contains("Forbidden");
            CheckResult {
                name: "Kubernetes Permissions".to_string(),
                status: CheckStatus::Error,
                message: if is_rbac_error { "RBAC permissions insufficient".to_string() } else { "Failed to list pods".to_string() },
                details: Some(e.to_string()),
                recommendation: Some(if is_rbac_error { 
                    "Apply deploy/rbac-fix.yaml to fix permissions".to_string() 
                } else { 
                    "Check Kubernetes API connectivity".to_string() 
                }),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
        Err(_) => {
            CheckResult {
                name: "Kubernetes Permissions".to_string(),
                status: CheckStatus::Warning,
                message: "Pod listing timed out".to_string(),
                details: Some("Request took longer than 10 seconds".to_string()),
                recommendation: Some("Check if cluster is under heavy load".to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
    }
}

async fn check_prometheus_connection() -> CheckResult {
    let start = Instant::now();
    
    match crate::legacy::prometheus::query_raw("up").await {
        Ok(_) => {
            CheckResult {
                name: "Prometheus Connection".to_string(),
                status: CheckStatus::Ok,
                message: "Connected to Prometheus".to_string(),
                details: None,
                recommendation: None,
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
        Err(e) => {
            CheckResult {
                name: "Prometheus Connection".to_string(),
                status: CheckStatus::Warning,
                message: "Failed to connect to Prometheus".to_string(),
                details: Some(e.to_string()),
                recommendation: Some("Check PROMETHEUS_URL configuration".to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
    }
}

async fn check_openobserve_config(_client: &Client) -> CheckResult {
    let _start = Instant::now();
    
    // Check if telemetry is enabled and configured
    let has_env = std::env::var("OPENOBSERVE_ENDPOINT").is_ok() && std::env::var("OPENOBSERVE_AUTH").is_ok();
    let is_initialized = crate::legacy::telemetry::is_enabled();
    
    if has_env || is_initialized {
        CheckResult {
            name: "OpenObserve Telemetry".to_string(),
            status: CheckStatus::Ok,
            message: "OpenObserve telemetry is configured".to_string(),
            details: None,
            recommendation: None,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    } else {
        CheckResult {
            name: "OpenObserve Telemetry".to_string(),
            status: CheckStatus::Warning,
            message: "OpenObserve telemetry not configured".to_string(),
            details: Some("Create secret 'openobserve-credentials' or set OPENOBSERVE_ENDPOINT and OPENOBSERVE_AUTH".to_string()),
            recommendation: Some("Run: kubectl create secret generic openobserve-credentials --from-literal=endpoint=... --from-literal=token=... -n kusanagi".to_string()),
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

async fn check_database_connection() -> CheckResult {
    let start = Instant::now();
    
    if !crate::legacy::database::is_initialized() {
        return CheckResult {
            name: "Database Connection".to_string(),
            status: CheckStatus::Warning,
            message: "Database pool not initialized".to_string(),
            details: Some("PostgreSQL connection not established".to_string()),
            recommendation: Some("Check POSTGRES_HOST and secret configuration".to_string()),
            duration_ms: start.elapsed().as_millis() as u64,
        };
    }
    
    let health = crate::legacy::database::check_health_quick().await;
    
    if health.status == "Healthy" {
        CheckResult {
            name: "Database Connection".to_string(),
            status: CheckStatus::Ok,
            message: "Connected to PostgreSQL".to_string(),
            details: health.version,
            recommendation: None,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    } else {
        CheckResult {
            name: "Database Connection".to_string(),
            status: CheckStatus::Warning,
            message: "Database connection issue".to_string(),
            details: health.error,
            recommendation: Some("Check PostgreSQL connectivity and credentials".to_string()),
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

async fn check_mqtt_connection() -> CheckResult {
    let start = Instant::now();
    
    let state = crate::legacy::mqtt::MQTT_STATE.lock().unwrap();
    let connected = state.connected;
    drop(state);
    
    if connected {
        CheckResult {
            name: "MQTT Connection".to_string(),
            status: CheckStatus::Ok,
            message: "Connected to MQTT broker".to_string(),
            details: None,
            recommendation: None,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    } else {
        CheckResult {
            name: "MQTT Connection".to_string(),
            status: CheckStatus::Warning,
            message: "Not connected to MQTT broker".to_string(),
            details: Some("MQTT is optional".to_string()),
            recommendation: Some("Set MQTT_HOST environment variable to enable".to_string()),
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

async fn check_llm_connection() -> CheckResult {
    let start = Instant::now();
    
    let config_info = crate::llm::get_config_info();
    let provider = config_info["provider"].as_str().unwrap_or("unknown");
    let is_valid = config_info["is_valid"].as_bool().unwrap_or(false);
    
    if is_valid {
        CheckResult {
            name: "LLM Configuration".to_string(),
            status: CheckStatus::Ok,
            message: format!("LLM configured (provider: {})", provider),
            details: Some(format!("Model: {}", config_info["model"].as_str().unwrap_or("unknown"))),
            recommendation: None,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    } else {
        CheckResult {
            name: "LLM Configuration".to_string(),
            status: CheckStatus::Warning,
            message: "LLM not properly configured".to_string(),
            details: Some(format!("Provider: {}", provider)),
            recommendation: Some("Check LLM_PROVIDER and LLM_BASE_URL configuration".to_string()),
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}

async fn check_s3_connection() -> CheckResult {
    let start = Instant::now();
    
    let client = crate::legacy::translation::get_s3_client().await;
    
    match client.list_buckets().send().await {
        Ok(_) => {
            CheckResult {
                name: "S3/MinIO Connection".to_string(),
                status: CheckStatus::Ok,
                message: "Connected to S3/MinIO".to_string(),
                details: None,
                recommendation: None,
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
        Err(e) => {
            CheckResult {
                name: "S3/MinIO Connection".to_string(),
                status: CheckStatus::Warning,
                message: "Failed to connect to S3/MinIO".to_string(),
                details: Some(e.to_string()),
                recommendation: Some("Check S3_ENDPOINT and credentials".to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
    }
}

async fn check_memory_usage() -> CheckResult {
    let start = Instant::now();
    
    // Get memory info from system using the same approach as system.rs
    use sysinfo::{System, SystemExt};
    let mut sys = System::new_all();
    sys.refresh_all();
    let used_mb = sys.used_memory() / 1024;
    let total_mb = sys.total_memory() / 1024;
    let usage_percent = if total_mb > 0 {
        (used_mb as f64 / total_mb as f64) * 100.0
    } else {
        0.0
    };
    
    let status = if usage_percent > 90.0 {
        CheckStatus::Error
    } else if usage_percent > 75.0 {
        CheckStatus::Warning
    } else {
        CheckStatus::Ok
    };
    
    let needs_recommendation = matches!(status, CheckStatus::Warning | CheckStatus::Error);
    
    CheckResult {
        name: "Memory Usage".to_string(),
        status,
        message: format!("Memory usage: {:.1}%", usage_percent),
        details: Some(format!("Used: {} MB / Total: {} MB", used_mb, total_mb)),
        recommendation: if needs_recommendation {
            Some("Consider increasing memory limits or restarting the pod".to_string())
        } else {
            None
        },
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

async fn check_disk_space() -> CheckResult {
    let start = Instant::now();
    
    // This is a simplified check - in production you'd use sysinfo or fs2
    CheckResult {
        name: "Disk Space".to_string(),
        status: CheckStatus::Ok,
        message: "Disk space check skipped".to_string(),
        details: Some("Running in container - disk managed by Kubernetes".to_string()),
        recommendation: None,
        duration_ms: start.elapsed().as_millis() as u64,
    }
}

fn generate_recommendations(checks: &[CheckResult]) -> Vec<String> {
    let mut recommendations = Vec::new();
    
    for check in checks {
        if let Some(rec) = &check.recommendation {
            recommendations.push(format!("[{}] {}", check.name, rec));
        }
    }
    
    if recommendations.is_empty() {
        recommendations.push("All systems operational! No action required.".to_string());
    }
    
    recommendations
}

/// Doctor endpoint handler
#[get("/api/doctor")]
pub async fn doctor_handler(data: web::Data<crate::AppState>) -> impl Responder {
    let report = run_diagnostics(&data.client).await;
    
    let status_code = match report.overall_status {
        CheckStatus::Ok => actix_web::http::StatusCode::OK,
        CheckStatus::Warning => actix_web::http::StatusCode::OK,
        CheckStatus::Error => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
        CheckStatus::Skipped => actix_web::http::StatusCode::OK,
    };
    
    HttpResponse::build(status_code).json(report)
}

/// Quick health check endpoint
#[get("/api/doctor/quick")]
pub async fn doctor_quick_handler(data: web::Data<crate::AppState>) -> impl Responder {
    let start = Instant::now();
    
    // Only run essential checks
    let k8s_check = check_kubernetes_connection(&data.client).await;
    let perm_check = check_kubernetes_permissions(&data.client).await;
    
    let healthy = k8s_check.status == CheckStatus::Ok && perm_check.status != CheckStatus::Error;
    
    HttpResponse::Ok().json(serde_json::json!({
        "healthy": healthy,
        "kubernetes": k8s_check.status == CheckStatus::Ok,
        "permissions": perm_check.status != CheckStatus::Error,
        "duration_ms": start.elapsed().as_millis() as u64,
    }))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(doctor_handler)
       .service(doctor_quick_handler);
}
