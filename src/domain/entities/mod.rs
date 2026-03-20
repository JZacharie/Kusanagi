// Core Domain Entities
use serde::{Deserialize, Serialize};

// ==================== Modules ====================
pub mod a2ui;
pub mod business;
pub mod chat;
pub mod cilium;
pub mod diagnostic;
pub mod llm;
pub mod mcp;

pub use a2ui::*;
pub use business::*;

// Re-export Cilium entities
pub use cilium::{
    BandwidthMetrics, FlowMatrixEntry, HubbleFlowsResponse, NetworkAnomaly, NetworkFlow,
};

// Re-export Diagnostic entities
pub use diagnostic::{
    CheckResult, CheckStatus, DiagnosticReport, DiagnosticSummary, QuickHealthResponse,
};

// Re-export LLM entities
pub use llm::{LlmConfig, LlmConfigInfo, LlmError, LlmHealthResponse, LlmProvider, LlmResponse};

// Re-export MCP entities
pub use mcp::{
    CiliumPolicy, CiliumPolicySummary, K8sResourceSummary, McpConfig, McpRequest, McpResponse,
    PolicyReportOverview, PolicySummary, PolicyViolation, SteampipeResult, TrivyImageReport,
    TrivyVulnerabilitySummary,
};

// ==================== Cluster Entities ====================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub name: String,
    pub version: String,
    pub status: String,
    pub nodes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub status: String,
    pub role: String,
}

// ==================== Weather Entities ====================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastDay {
    pub date: String,
    pub temp: f32,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherInfo {
    pub city: String,
    pub temp: f32,
    pub description: String,
    pub icon: String,
    pub humidity: u8,
    pub wind_speed: f32,
    pub feels_like: f32,
    pub pressure: u32,
    pub visibility: u32,
    pub last_updated: String,
    pub forecast: Vec<ForecastDay>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherResponse {
    pub cities: Vec<WeatherInfo>,
    pub cached_at: String,
}

// ==================== Alert Entities ====================
use chrono::{DateTime, Utc};

/// Single alert from Alertmanager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub name: String,
    pub severity: String,
    pub state: String,
    pub summary: String,
    pub description: Option<String>,
    pub namespace: Option<String>,
    pub pod: Option<String>,
    pub started_at: DateTime<Utc>,
    pub fingerprint: String,
    pub generator_url: Option<String>,
}

/// Grouped alerts response
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlertsResponse {
    pub critical: Vec<Alert>,
    pub warning: Vec<Alert>,
    pub info: Vec<Alert>,
    pub total: i32,
    pub firing: i32,
    pub pending: i32,
}

// ==================== Backup Entities ====================

/// Backups response for the API
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BackupsResponse {
    pub total_cronjobs: usize,
    pub active_jobs: usize,
    pub succeeded_jobs: usize,
    pub failed_jobs: usize,
    pub cronjobs: Vec<CronJobInfo>,
}

/// CronJob information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CronJobInfo {
    pub name: String,
    pub namespace: String,
    pub schedule: String,
    pub last_schedule: Option<String>,
    pub last_schedule_age: Option<String>,
    pub active_jobs: i32,
    pub suspend: bool,
    pub recent_jobs: Vec<JobInfo>,
}

/// Job information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobInfo {
    pub name: String,
    pub status: String, // Running, Succeeded, Failed
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub duration: Option<String>,
}

// ==================== HomeAssistant Entities ====================

/// HomeAssistant sensor entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeAssistantSensor {
    pub entity_id: String,
    pub state: String,
    pub attributes: serde_json::Value,
    pub last_changed: String,
    pub last_updated: String,
}

/// HomeAssistant device entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeAssistantDevice {
    pub id: String,
    pub name: String,
    pub area_id: String,
    pub manufacturer: String,
    pub model: String,
    pub sw_version: String,
}

/// HomeAssistant automation entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeAssistantAutomation {
    pub entity_id: String,
    pub state: String,
    pub attributes: HomeAssistantAutomationAttributes,
    pub last_changed: String,
    pub last_triggered: Option<String>,
}

/// HomeAssistant automation attributes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HomeAssistantAutomationAttributes {
    pub friendly_name: String,
    pub last_triggered: String,
    pub mode: String,
    pub current: u32,
    pub max: u32,
}

/// HomeAssistant API state (internal representation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeAssistantState {
    pub entity_id: String,
    pub state: String,
    pub attributes: serde_json::Value,
    pub last_changed: String,
    pub last_updated: String,
}

/// Response wrapper for HomeAssistant sensors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeAssistantSensorsResponse {
    pub sensors: Vec<HomeAssistantSensor>,
    pub count: usize,
}

/// Response wrapper for HomeAssistant devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomeAssistantDevicesResponse {
    pub devices: Vec<HomeAssistantDevice>,
    pub count: usize,
}

// ==================== Security Entities ====================

/// Vulnerability information from Trivy scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub description: Option<String>,
    pub package_name: Option<String>,
    pub installed_version: Option<String>,
    pub fixed_version: Option<String>,
}

/// AI enrichment data for a security report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentData {
    pub summary: String,
    pub remediation_advice: String,
    pub criticality_score: f64,
}

/// Single security report from Trivy scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    pub name: String,
    pub report_type: String,
    pub original_data: serde_json::Value,
    pub enrichment: Option<EnrichmentData>,
    pub timestamp: String,
}

/// Security summary across all reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummary {
    pub total_reports: usize,
    pub total_vulnerabilities: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub reports: Vec<String>,
    pub last_updated: String,
}
