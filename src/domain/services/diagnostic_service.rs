//! Diagnostic Service - Domain Service for system diagnostics
//!
//! Performs comprehensive health checks on all system components

use crate::domain::entities::diagnostic::{
    CheckResult, CheckStatus, DiagnosticReport, DiagnosticSummary,
};
use crate::domain::services::llm_service::LlmService;
use k8s_openapi::api::core::v1::Pod;
use kube::{Api, Client};
use std::time::{Duration, Instant};

/// Diagnostic Service for system health checks
pub struct DiagnosticService {
    k8s_client: Client,
}

impl DiagnosticService {
    /// Create new diagnostic service
    pub fn new(k8s_client: Client) -> Self {
        Self { k8s_client }
    }

    /// Run all diagnostic checks
    pub async fn run_full_diagnostics(&self) -> DiagnosticReport {
        let mut checks = Vec::new();
        let mut summary = DiagnosticSummary::new();

        // Core checks
        checks.push(self.check_kubernetes_connection().await);
        checks.push(self.check_kubernetes_permissions().await);
        checks.push(self.check_prometheus_connection().await);
        checks.push(self.check_database_connection().await);
        checks.push(self.check_llm_connection().await);
        checks.push(self.check_memory_usage().await);

        // Update summary
        for check in &checks {
            summary.increment(&check.status);
        }

        // Generate recommendations
        let recommendations = self.generate_recommendations(&checks);

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

    /// Run quick diagnostics (essential checks only)
    pub async fn run_quick_diagnostics(&self) -> DiagnosticReport {
        let mut checks = Vec::new();
        let mut summary = DiagnosticSummary::new();

        // Only essential checks
        checks.push(self.check_kubernetes_connection().await);
        checks.push(self.check_kubernetes_permissions().await);

        // Update summary
        for check in &checks {
            summary.increment(&check.status);
        }

        let recommendations = self.generate_recommendations(&checks);

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

    /// Check Kubernetes connection
    async fn check_kubernetes_connection(&self) -> CheckResult {
        let start = Instant::now();

        match self.k8s_client.apiserver_version().await {
            Ok(version) => CheckResult {
                name: "Kubernetes Connection".to_string(),
                status: CheckStatus::Ok,
                message: format!("Connected to Kubernetes {}", version.git_version),
                details: Some(format!(
                    "Major: {}, Minor: {}",
                    version.major, version.minor
                )),
                recommendation: None,
                duration_ms: start.elapsed().as_millis() as u64,
            },
            Err(e) => CheckResult {
                name: "Kubernetes Connection".to_string(),
                status: CheckStatus::Error,
                message: "Failed to connect to Kubernetes API".to_string(),
                details: Some(e.to_string()),
                recommendation: Some(
                    "Check if running in a Kubernetes cluster or if kubeconfig is correct"
                        .to_string(),
                ),
                duration_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    /// Check Kubernetes permissions
    async fn check_kubernetes_permissions(&self) -> CheckResult {
        let start = Instant::now();

        let pods: Api<Pod> = Api::all(self.k8s_client.clone());

        match tokio::time::timeout(
            Duration::from_secs(10),
            pods.list(&kube::api::ListParams::default().limit(1)),
        )
        .await
        {
            Ok(Ok(_)) => CheckResult {
                name: "Kubernetes Permissions".to_string(),
                status: CheckStatus::Ok,
                message: "RBAC permissions are sufficient".to_string(),
                details: Some("Can list pods across all namespaces".to_string()),
                recommendation: None,
                duration_ms: start.elapsed().as_millis() as u64,
            },
            Ok(Err(e)) => {
                let is_rbac_error = e.to_string().contains("Forbidden");
                CheckResult {
                    name: "Kubernetes Permissions".to_string(),
                    status: CheckStatus::Error,
                    message: if is_rbac_error {
                        "RBAC permissions insufficient".to_string()
                    } else {
                        "Failed to list pods".to_string()
                    },
                    details: Some(e.to_string()),
                    recommendation: Some(if is_rbac_error {
                        "Apply deploy/rbac-fix.yaml to fix permissions".to_string()
                    } else {
                        "Check Kubernetes API connectivity".to_string()
                    }),
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
            Err(_) => CheckResult {
                name: "Kubernetes Permissions".to_string(),
                status: CheckStatus::Warning,
                message: "Pod listing timed out".to_string(),
                details: Some("Request took longer than 10 seconds".to_string()),
                recommendation: Some("Check if cluster is under heavy load".to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    /// Check Prometheus connection
    async fn check_prometheus_connection(&self) -> CheckResult {
        let start = Instant::now();

        // Try to query Prometheus
        let prometheus_url = std::env::var("PROMETHEUS_URL")
            .unwrap_or_else(|_| "http://prometheus.monitoring.svc.cluster.local:9090".to_string());

        let client = reqwest::Client::new();
        let query_url = format!("{}/api/v1/query?query=up", prometheus_url);

        match tokio::time::timeout(Duration::from_secs(5), client.get(&query_url).send()).await {
            Ok(Ok(response)) if response.status().is_success() => CheckResult {
                name: "Prometheus Connection".to_string(),
                status: CheckStatus::Ok,
                message: "Connected to Prometheus".to_string(),
                details: None,
                recommendation: None,
                duration_ms: start.elapsed().as_millis() as u64,
            },
            Ok(Ok(response)) => CheckResult {
                name: "Prometheus Connection".to_string(),
                status: CheckStatus::Warning,
                message: format!("Prometheus returned status {}", response.status()),
                details: None,
                recommendation: Some("Check PROMETHEUS_URL configuration".to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            Ok(Err(e)) => CheckResult {
                name: "Prometheus Connection".to_string(),
                status: CheckStatus::Warning,
                message: "Failed to connect to Prometheus".to_string(),
                details: Some(e.to_string()),
                recommendation: Some("Check PROMETHEUS_URL configuration".to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            },
            Err(_) => CheckResult {
                name: "Prometheus Connection".to_string(),
                status: CheckStatus::Warning,
                message: "Prometheus request timed out".to_string(),
                details: None,
                recommendation: Some("Check PROMETHEUS_URL configuration".to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    /// Check database connection
    async fn check_database_connection(&self) -> CheckResult {
        let start = Instant::now();

        // Simple check based on environment variables
        let db_host = std::env::var("POSTGRES_HOST").ok();

        if db_host.is_some() {
            CheckResult {
                name: "Database Connection".to_string(),
                status: CheckStatus::Ok,
                message: "Database configuration detected".to_string(),
                details: Some("PostgreSQL environment variables are set".to_string()),
                recommendation: None,
                duration_ms: start.elapsed().as_millis() as u64,
            }
        } else {
            CheckResult {
                name: "Database Connection".to_string(),
                status: CheckStatus::Warning,
                message: "Database not configured".to_string(),
                details: Some("POSTGRES_HOST environment variable not set".to_string()),
                recommendation: Some("Set POSTGRES_HOST to enable database features".to_string()),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
    }

    /// Check LLM connection
    async fn check_llm_connection(&self) -> CheckResult {
        let start = Instant::now();

        let service = LlmService::new();
        let config = service.config();

        if config.is_valid() {
            CheckResult {
                name: "LLM Configuration".to_string(),
                status: CheckStatus::Ok,
                message: format!("LLM configured (provider: {:?})", config.provider),
                details: Some(format!("Model: {}", config.model)),
                recommendation: None,
                duration_ms: start.elapsed().as_millis() as u64,
            }
        } else {
            CheckResult {
                name: "LLM Configuration".to_string(),
                status: CheckStatus::Warning,
                message: "LLM not properly configured".to_string(),
                details: Some(format!("Provider: {:?}", config.provider)),
                recommendation: Some(
                    "Check LLM_PROVIDER and LLM_BASE_URL configuration".to_string(),
                ),
                duration_ms: start.elapsed().as_millis() as u64,
            }
        }
    }

    /// Check memory usage
    async fn check_memory_usage(&self) -> CheckResult {
        let start = Instant::now();

        use sysinfo::System;
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

    /// Generate recommendations based on check results
    fn generate_recommendations(&self, checks: &[CheckResult]) -> Vec<String> {
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
}
