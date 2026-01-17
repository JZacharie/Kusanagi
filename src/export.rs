use chrono::Utc;
use serde::Serialize;

use crate::alertmanager::{self, AlertsResponse};
use crate::argocd::{self, ArgoStatusResponse};
use crate::events::{self, EventsResponse};
use crate::nodes::{self, NodesStatusResponse};
use crate::prometheus::{self, PrometheusMetrics};
use crate::storage::{self, StorageStatusResponse};

/// Complete cluster report structure
#[derive(Debug, Serialize)]
pub struct ClusterReport {
    pub generated_at: String,
    pub cluster_name: String,
    pub summary: ReportSummary,
    pub nodes: NodesStatusResponse,
    pub argocd_apps: ArgoStatusResponse,
    pub alerts: Option<AlertsResponse>,
    pub events: EventsResponse,
    pub storage: StorageStatusResponse,
    pub metrics: Option<PrometheusMetrics>,
}

#[derive(Debug, Serialize)]
pub struct ReportSummary {
    pub total_nodes: usize,
    pub ready_nodes: usize,
    pub total_apps: usize,
    pub healthy_apps: usize,
    pub unhealthy_apps: usize,
    pub total_alerts: i32,
    pub critical_alerts: i32,
    pub warning_alerts: i32,
    pub total_events: usize,
    pub warning_events: usize,
    pub total_pvcs: usize,
}

/// Generate a complete cluster report
pub async fn generate_report() -> Result<ClusterReport, String> {
    // Gather all data concurrently
    let (nodes_result, argocd_result, alerts_result, events_result, storage_result, metrics_result) = tokio::join!(
        nodes::get_nodes_status(),
        argocd::get_argocd_status(),
        alertmanager::get_active_alerts(),
        events::get_events(None),
        storage::get_storage_status(),
        prometheus::get_cluster_metrics()
    );
    
    // Process nodes - required
    let nodes_data = nodes_result.map_err(|e| format!("Failed to get nodes: {}", e))?;
    
    // Process ArgoCD - required
    let argocd_data = argocd_result.map_err(|e| format!("Failed to get ArgoCD status: {}", e))?;
    
    // Process events - required
    let events_data = events_result.map_err(|e| format!("Failed to get events: {}", e))?;
    
    // Process storage - required
    let storage_data = storage_result.map_err(|e| format!("Failed to get storage: {}", e))?;
    
    // Alerts and metrics are optional (may fail if Prometheus/Alertmanager not available)
    let alerts_data = alerts_result.ok();
    let metrics_data = metrics_result.ok();
    
    // Build summary
    let summary = ReportSummary {
        total_nodes: nodes_data.total_nodes,
        ready_nodes: nodes_data.ready_nodes,
        total_apps: argocd_data.total,
        healthy_apps: argocd_data.healthy,
        unhealthy_apps: argocd_data.unhealthy,
        total_alerts: alerts_data.as_ref().map(|a| a.total).unwrap_or(0),
        critical_alerts: alerts_data.as_ref().map(|a| a.critical.len() as i32).unwrap_or(0),
        warning_alerts: alerts_data.as_ref().map(|a| a.warning.len() as i32).unwrap_or(0),
        total_events: events_data.total_events,
        warning_events: events_data.warning_count,
        total_pvcs: storage_data.pvc_count,
    };
    
    Ok(ClusterReport {
        generated_at: Utc::now().to_rfc3339(),
        cluster_name: "k3s-cluster".to_string(),
        summary,
        nodes: nodes_data,
        argocd_apps: argocd_data,
        alerts: alerts_data,
        events: events_data,
        storage: storage_data,
        metrics: metrics_data,
    })
}

/// Export report as JSON
pub fn export_json(report: &ClusterReport) -> Result<String, String> {
    serde_json::to_string_pretty(report)
        .map_err(|e| format!("Failed to serialize report: {}", e))
}

/// Export report as CSV (summary only)
pub fn export_csv(report: &ClusterReport) -> Result<String, String> {
    let mut csv = String::new();
    
    // Header
    csv.push_str("Metric,Value\n");
    
    // Summary data
    csv.push_str(&format!("Generated At,{}\n", report.generated_at));
    csv.push_str(&format!("Cluster Name,{}\n", report.cluster_name));
    csv.push_str(&format!("Total Nodes,{}\n", report.summary.total_nodes));
    csv.push_str(&format!("Ready Nodes,{}\n", report.summary.ready_nodes));
    csv.push_str(&format!("Total Apps,{}\n", report.summary.total_apps));
    csv.push_str(&format!("Healthy Apps,{}\n", report.summary.healthy_apps));
    csv.push_str(&format!("Unhealthy Apps,{}\n", report.summary.unhealthy_apps));
    csv.push_str(&format!("Total Alerts,{}\n", report.summary.total_alerts));
    csv.push_str(&format!("Critical Alerts,{}\n", report.summary.critical_alerts));
    csv.push_str(&format!("Warning Alerts,{}\n", report.summary.warning_alerts));
    csv.push_str(&format!("Total Events,{}\n", report.summary.total_events));
    csv.push_str(&format!("Warning Events,{}\n", report.summary.warning_events));
    csv.push_str(&format!("Total PVCs,{}\n", report.summary.total_pvcs));
    
    Ok(csv)
}

/// Export report as Markdown
pub fn export_markdown(report: &ClusterReport) -> Result<String, String> {
    let mut md = String::new();
    
    md.push_str("# Kusanagi Cluster Report\n\n");
    md.push_str(&format!("**Generated:** {}\n\n", report.generated_at));
    md.push_str(&format!("**Cluster:** {}\n\n", report.cluster_name));
    
    md.push_str("---\n\n");
    md.push_str("## Summary\n\n");
    md.push_str("| Metric | Value |\n");
    md.push_str("|--------|-------|\n");
    md.push_str(&format!("| Total Nodes | {} |\n", report.summary.total_nodes));
    md.push_str(&format!("| Ready Nodes | {} |\n", report.summary.ready_nodes));
    md.push_str(&format!("| Total Apps | {} |\n", report.summary.total_apps));
    md.push_str(&format!("| Healthy Apps | {} |\n", report.summary.healthy_apps));
    md.push_str(&format!("| Unhealthy Apps | {} |\n", report.summary.unhealthy_apps));
    md.push_str(&format!("| Total Alerts | {} |\n", report.summary.total_alerts));
    md.push_str(&format!("| Critical Alerts | {} |\n", report.summary.critical_alerts));
    md.push_str(&format!("| Warning Alerts | {} |\n", report.summary.warning_alerts));
    md.push_str(&format!("| Total Events | {} |\n", report.summary.total_events));
    md.push_str(&format!("| Warning Events | {} |\n", report.summary.warning_events));
    md.push_str(&format!("| PVCs | {} |\n", report.summary.total_pvcs));
    
    md.push_str("\n---\n\n");
    
    // Alerts section
    if report.summary.total_alerts > 0 {
        md.push_str("## Active Alerts\n\n");
        if report.summary.critical_alerts > 0 {
            md.push_str(&format!("🔴 **Critical:** {}\n\n", report.summary.critical_alerts));
        }
        if report.summary.warning_alerts > 0 {
            md.push_str(&format!("🟠 **Warning:** {}\n\n", report.summary.warning_alerts));
        }
    } else {
        md.push_str("## Alerts\n\n✅ No active alerts\n\n");
    }
    
    // Nodes section
    md.push_str("---\n\n");
    md.push_str("## Nodes\n\n");
    md.push_str("| Name | Status | Architecture | CPU | Memory |\n");
    md.push_str("|------|--------|--------------|-----|--------|\n");
    for node in &report.nodes.nodes {
        md.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            node.name, node.status, node.architecture, node.cpu_capacity, node.memory_capacity
        ));
    }
    
    // ArgoCD section
    md.push_str("\n---\n\n");
    md.push_str("## ArgoCD Applications\n\n");
    if !report.argocd_apps.apps_with_issues.is_empty() {
        md.push_str("### Issues\n\n");
        for app in &report.argocd_apps.apps_with_issues {
            md.push_str(&format!("- **{}** ({}) - Health: {}, Sync: {}\n", 
                app.name, app.namespace, app.health_status, app.sync_status));
        }
        md.push_str("\n");
    }
    
    md.push_str("---\n\n");
    md.push_str("*Report generated by Kusanagi Agent Controller*\n");
    
    Ok(md)
}
