use axum::response::{Html, IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    pub method: String,
    pub path: String,
    pub description: String,
    pub category: String,
}

pub fn get_routes() -> Vec<RouteInfo> {
    vec![
        // Core
        RouteInfo {
            method: "GET".to_string(),
            path: "/".to_string(),
            description: "Web interface".to_string(),
            category: "Core".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api".to_string(),
            description: "API information".to_string(),
            category: "Core".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/health".to_string(),
            description: "Health check".to_string(),
            category: "Core".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/config".to_string(),
            description: "Configuration".to_string(),
            category: "Core".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/cache/stats".to_string(),
            description: "Cache statistics".to_string(),
            category: "Core".to_string(),
        },
        RouteInfo {
            method: "POST".to_string(),
            path: "/api/slack/notify".to_string(),
            description: "Send Slack notification".to_string(),
            category: "Core".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/ws/notifications".to_string(),
            description: "WebSocket notifications".to_string(),
            category: "Core".to_string(),
        },
        // System
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/system/status".to_string(),
            description: "System status".to_string(),
            category: "System".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/system/logs".to_string(),
            description: "System logs".to_string(),
            category: "System".to_string(),
        },
        // Kubernetes
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/k8s/cluster".to_string(),
            description: "Cluster overview".to_string(),
            category: "Kubernetes".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/k8s/nodes".to_string(),
            description: "Nodes status".to_string(),
            category: "Kubernetes".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/k8s/pods".to_string(),
            description: "Pods status".to_string(),
            category: "Kubernetes".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/k8s/pods/:namespace/:name/logs".to_string(),
            description: "Pod logs".to_string(),
            category: "Kubernetes".to_string(),
        },
        RouteInfo {
            method: "POST".to_string(),
            path: "/api/pods/delete-error-pods".to_string(),
            description: "Delete error pods".to_string(),
            category: "Kubernetes".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/storage".to_string(),
            description: "Storage status".to_string(),
            category: "Kubernetes".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/ingress".to_string(),
            description: "Ingress resources".to_string(),
            category: "Kubernetes".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/services".to_string(),
            description: "Services".to_string(),
            category: "Kubernetes".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/argocd/status".to_string(),
            description: "ArgoCD status".to_string(),
            category: "Kubernetes".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/news".to_string(),
            description: "Cluster news/events".to_string(),
            category: "Kubernetes".to_string(),
        },
        // Monitoring
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/monitoring/alerts".to_string(),
            description: "Active alerts".to_string(),
            category: "Monitoring".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/monitoring/quotas".to_string(),
            description: "Resource quotas".to_string(),
            category: "Monitoring".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/metrics".to_string(),
            description: "System metrics".to_string(),
            category: "Monitoring".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/prometheus/range".to_string(),
            description: "Prometheus range query".to_string(),
            category: "Monitoring".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/database/health".to_string(),
            description: "Database health".to_string(),
            category: "Monitoring".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/fusion".to_string(),
            description: "Fusion stats".to_string(),
            category: "Monitoring".to_string(),
        },
        // Hexagonal (Integrations)
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/alerts".to_string(),
            description: "External alerts".to_string(),
            category: "Integrations".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/backups".to_string(),
            description: "Backups status".to_string(),
            category: "Integrations".to_string(),
        },
        RouteInfo {
            method: "POST".to_string(),
            path: "/api/backups/{namespace}/{name}/trigger".to_string(),
            description: "Trigger backup".to_string(),
            category: "Integrations".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/ha/devices".to_string(),
            description: "Home Assistant devices".to_string(),
            category: "Integrations".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/ha/sensors".to_string(),
            description: "Home Assistant sensors".to_string(),
            category: "Integrations".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/ha/automations".to_string(),
            description: "Home Assistant automations".to_string(),
            category: "Integrations".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/security/summary".to_string(),
            description: "Security summary".to_string(),
            category: "Integrations".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/security/reports".to_string(),
            description: "Security reports".to_string(),
            category: "Integrations".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/security/reports/{category}/{name}".to_string(),
            description: "Specific security report".to_string(),
            category: "Integrations".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/security/vulnerabilities".to_string(),
            description: "Vulnerabilities".to_string(),
            category: "Integrations".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/weather/current".to_string(),
            description: "Current weather".to_string(),
            category: "Integrations".to_string(),
        },
        // Proxmox
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/proxmox/vms".to_string(),
            description: "Proxmox VMs".to_string(),
            category: "Proxmox".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/proxmox/containers".to_string(),
            description: "Proxmox Containers".to_string(),
            category: "Proxmox".to_string(),
        },
        RouteInfo {
            method: "GET".to_string(),
            path: "/api/proxmox/nodes".to_string(),
            description: "Proxmox Nodes".to_string(),
            category: "Proxmox".to_string(),
        },
    ]
}

pub async fn docs_handler() -> impl IntoResponse {
    let routes = get_routes();

    let mut html = String::from(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Kusanagi API Documentation</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif; line-height: 1.6; color: #333; max-width: 1200px; margin: 0 auto; padding: 20px; background-color: #f4f7f6; }
        h1 { color: #2c3e50; border-bottom: 2px solid #3498db; padding-bottom: 10px; }
        .category { margin-bottom: 30px; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 5px rgba(0,0,0,0.05); }
        .category h2 { color: #34495e; margin-top: 0; border-bottom: 1px solid #eee; padding-bottom: 10px; }
        table { width: 100%; border-collapse: collapse; margin-top: 15px; }
        th, td { text-align: left; padding: 12px; border-bottom: 1px solid #eee; }
        th { background-color: #f8f9fa; color: #7f8c8d; font-weight: 600; }
        tr:hover { background-color: #f1f2f6; }
        .method { font-weight: bold; padding: 4px 8px; border-radius: 4px; font-size: 0.85em; display: inline-block; width: 60px; text-align: center; }
        .GET { background-color: #e1f5fe; color: #0288d1; }
        .POST { background-color: #e8f5e9; color: #388e3c; }
        .PUT { background-color: #fff3e0; color: #f57c00; }
        .DELETE { background-color: #ffebee; color: #d32f2f; }
        .path { font-family: 'Courier New', Courier, monospace; color: #d63384; font-weight: 600; }
        .summary { margin-bottom: 20px; font-size: 1.1em; color: #555; }
        .version { float: right; color: #95a5a6; font-size: 0.9em; }
    </style>
</head>
<body>
    <div class="version">Version: "#,
    );

    html.push_str(env!("CARGO_PKG_VERSION"));
    html.push_str(
        r#"</div>
    <h1>Kusanagi API Documentation</h1>
    <p class="summary">Documentation of all available API endpoints for the Kusanagi service.</p>
"#,
    );

    // Group by category
    let mut categories: std::collections::BTreeMap<String, Vec<&RouteInfo>> =
        std::collections::BTreeMap::new();
    for route in &routes {
        categories
            .entry(route.category.clone())
            .or_default()
            .push(route);
    }

    for (category, cat_routes) in categories {
        html.push_str(&format!(r#"<div class="category"><h2>{}</h2><table><thead><tr><th>Method</th><th>Path</th><th>Description</th></tr></thead><tbody>"#, category));
        for route in cat_routes {
            html.push_str(&format!(
                r#"<tr><td><span class="method {}">{}</span></td><td class="path">{}</td><td>{}</td></tr>"#,
                route.method, route.method, route.path, route.description
            ));
        }
        html.push_str("</tbody></table></div>");
    }

    html.push_str(r#"</body></html>"#);

    Html(html)
}
