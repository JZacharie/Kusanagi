use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::{argocd, cluster, events, nodes, backups};

/// Chat message request
#[derive(Clone, Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

/// Chat response
#[derive(Clone, Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub response_type: String,
    pub data: Option<serde_json::Value>,
}

/// Ollama configuration
const OLLAMA_URL: &str = "http://192.168.0.52:11434/api/generate";
const OLLAMA_MODEL: &str = "ministral-3:14b";

/// Ollama request structure
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

/// Ollama response structure
#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

/// Available commands
const HELP_TEXT: &str = r#"**Kusanagi Chat Commands** 🤖

Available commands:
- `/help` - Show this help message
- `/status` - Show cluster overview
- `/nodes` - Show nodes status
- `/pods` - Show pods in error
- `/events` - Show recent warning events
- `/argocd` - Show ArgoCD issues
- `/backups` - Show backup jobs status
- `/namespaces` - Show namespace count
- `/pvcs` - Show PVC summary

Or just ask me anything in natural language! I'm powered by Ollama AI."#;

/// Process chat message and return response
pub async fn process_message(request: ChatRequest) -> ChatResponse {
    let message = request.message.trim();
    let message_lower = message.to_lowercase();
    
    info!("Chat message received: {}", message);

    // Handle commands
    if message_lower.starts_with('/') {
        return handle_command(&message_lower).await;
    }

    // Handle natural language queries with Ollama
    handle_query_with_ollama(message).await
}

async fn handle_command(command: &str) -> ChatResponse {
    match command {
        "/help" => ChatResponse {
            response: HELP_TEXT.to_string(),
            response_type: "help".to_string(),
            data: None,
        },
        
        "/status" => get_cluster_status().await,
        "/nodes" => get_nodes_summary().await,
        "/pods" => get_error_pods().await,
        "/events" => get_warning_events().await,
        "/argocd" => get_argocd_summary().await,
        "/backups" => get_backups_summary().await,
        "/namespaces" => get_namespaces_summary().await,
        "/pvcs" => get_pvcs_summary().await,
        
        _ => ChatResponse {
            response: format!("Unknown command: `{}`. Type `/help` for available commands.", command),
            response_type: "error".to_string(),
            data: None,
        },
    }
}

/// Query Ollama with context about the Kubernetes cluster
async fn handle_query_with_ollama(query: &str) -> ChatResponse {
    // Build context from cluster state
    let context = build_cluster_context().await;
    
    let system_prompt = format!(
        r#"Tu es Kusanagi, un assistant IA pour la gestion d'un cluster Kubernetes K3s. 
Tu es inspiré par Ghost in the Shell et tu as un style cyberpunk.
Voici l'état actuel du cluster:

{}

L'utilisateur te pose une question. Réponds de manière concise et utile.
Si la question concerne l'état du cluster, utilise les données ci-dessus.
Question: {}"#,
        context, query
    );

    match query_ollama(&system_prompt).await {
        Ok(response) => ChatResponse {
            response,
            response_type: "ai".to_string(),
            data: None,
        },
        Err(e) => {
            warn!("Ollama query failed: {}", e);
            // Fallback to simple response
            ChatResponse {
                response: format!(
                    "⚠️ AI response unavailable ({})\n\nTry using commands like `/status`, `/nodes`, `/events` or `/help`.",
                    e
                ),
                response_type: "error".to_string(),
                data: None,
            }
        }
    }
}

/// Build context string from cluster state
async fn build_cluster_context() -> String {
    let mut context_parts = vec![];

    if let Ok(nodes) = nodes::get_nodes_status().await {
        context_parts.push(format!(
            "Nodes: {} total, {} ready, {} not ready",
            nodes.total_nodes, nodes.ready_nodes, nodes.not_ready_nodes
        ));
    }

    if let Ok(overview) = cluster::get_cluster_overview().await {
        context_parts.push(format!(
            "Namespaces: {}, PVCs: {} ({})",
            overview.namespace_count, overview.pvc_count, overview.pvc_total_capacity
        ));
    }

    if let Ok(events) = events::get_events().await {
        context_parts.push(format!(
            "Events (1h): {} total, {} warnings",
            events.total_events, events.warning_count
        ));
    }

    if let Ok(argocd) = argocd::get_argocd_status().await {
        context_parts.push(format!(
            "ArgoCD: {}/{} healthy, {} issues",
            argocd.healthy, argocd.total, argocd.apps_with_issues.len()
        ));
    }

    if let Ok(backups) = backups::get_backups_status().await {
        context_parts.push(format!(
            "Backups: {} CronJobs, {} active, {} succeeded, {} failed",
            backups.total_cronjobs, backups.active_jobs, backups.succeeded_jobs, backups.failed_jobs
        ));
    }

    context_parts.join("\n")
}

/// Query Ollama API
async fn query_ollama(prompt: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let request = OllamaRequest {
        model: OLLAMA_MODEL.to_string(),
        prompt: prompt.to_string(),
        stream: false,
    };

    let response = client
        .post(OLLAMA_URL)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Ollama returned status: {}", response.status()));
    }

    let ollama_response: OllamaResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    Ok(ollama_response.response)
}

async fn get_cluster_status() -> ChatResponse {
    let mut status_lines = vec!["## 📊 Cluster Status\n".to_string()];

    // Get nodes
    if let Ok(nodes) = nodes::get_nodes_status().await {
        status_lines.push(format!(
            "**Nodes:** {} total ({} ready, {} not ready)",
            nodes.total_nodes, nodes.ready_nodes, nodes.not_ready_nodes
        ));
    }

    // Get cluster overview
    if let Ok(overview) = cluster::get_cluster_overview().await {
        status_lines.push(format!("**Namespaces:** {}", overview.namespace_count));
        status_lines.push(format!(
            "**PVCs:** {} ({})",
            overview.pvc_count, overview.pvc_total_capacity
        ));
    }

    // Get events
    if let Ok(events) = events::get_events().await {
        status_lines.push(format!(
            "**Events (1h):** {} ({} warnings)",
            events.total_events, events.warning_count
        ));
    }

    // Get ArgoCD
    if let Ok(argocd) = argocd::get_argocd_status().await {
        status_lines.push(format!(
            "**ArgoCD:** {}/{} healthy ({} issues)",
            argocd.healthy, argocd.total, argocd.apps_with_issues.len()
        ));
    }

    ChatResponse {
        response: status_lines.join("\n"),
        response_type: "status".to_string(),
        data: None,
    }
}

async fn get_nodes_summary() -> ChatResponse {
    match nodes::get_nodes_status().await {
        Ok(nodes) => {
            let mut lines = vec![format!(
                "## 🖥️ Nodes Status\n\n**Total:** {} ({} ready)\n",
                nodes.total_nodes, nodes.ready_nodes
            )];

            for node in nodes.nodes.iter().take(10) {
                let status_emoji = if node.status == "Ready" { "✅" } else { "❌" };
                let error_info = if node.pods_in_error > 0 {
                    format!(" ⚠️ {} pods in error", node.pods_in_error)
                } else {
                    String::new()
                };
                
                lines.push(format!(
                    "{} **{}** | {} | {} CPU | {} RAM | {} pods{}",
                    status_emoji,
                    node.name,
                    node.architecture,
                    node.cpu_capacity,
                    node.memory_allocatable,
                    node.pod_count,
                    error_info
                ));
            }

            ChatResponse {
                response: lines.join("\n"),
                response_type: "nodes".to_string(),
                data: Some(serde_json::json!({
                    "total": nodes.total_nodes,
                    "ready": nodes.ready_nodes
                })),
            }
        }
        Err(e) => ChatResponse {
            response: format!("Failed to get nodes: {}", e),
            response_type: "error".to_string(),
            data: None,
        },
    }
}

async fn get_error_pods() -> ChatResponse {
    match nodes::get_nodes_status().await {
        Ok(nodes) => {
            let mut error_pods: Vec<(String, String)> = vec![];
            
            for node in &nodes.nodes {
                for pod in &node.error_pod_names {
                    error_pods.push((pod.clone(), node.name.clone()));
                }
            }

            if error_pods.is_empty() {
                return ChatResponse {
                    response: "## ✅ No Pods in Error\n\nAll pods are running healthy!".to_string(),
                    response_type: "pods".to_string(),
                    data: None,
                };
            }

            let mut lines = vec![format!(
                "## ⚠️ Pods in Error\n\n**Total:** {} pods\n",
                error_pods.len()
            )];

            for (pod, node) in error_pods.iter().take(15) {
                lines.push(format!("- `{}` on **{}**", pod, node));
            }

            if error_pods.len() > 15 {
                lines.push(format!("\n... and {} more", error_pods.len() - 15));
            }

            ChatResponse {
                response: lines.join("\n"),
                response_type: "pods".to_string(),
                data: Some(serde_json::json!({ "count": error_pods.len() })),
            }
        }
        Err(e) => ChatResponse {
            response: format!("Failed to get pods: {}", e),
            response_type: "error".to_string(),
            data: None,
        },
    }
}

async fn get_warning_events() -> ChatResponse {
    match events::get_events().await {
        Ok(events) => {
            let warnings: Vec<_> = events.events.iter()
                .filter(|e| e.event_type == "Warning")
                .take(10)
                .collect();

            if warnings.is_empty() {
                return ChatResponse {
                    response: "## ✅ No Warning Events\n\nNo warnings in the last hour!".to_string(),
                    response_type: "events".to_string(),
                    data: None,
                };
            }

            let mut lines = vec![format!(
                "## ⚠️ Warning Events (Last Hour)\n\n**Total:** {} warnings\n",
                events.warning_count
            )];

            for evt in warnings {
                lines.push(format!(
                    "- **{}** | `{}/{}` | {}",
                    evt.reason,
                    evt.involved_object_kind,
                    evt.involved_object_name.chars().take(25).collect::<String>(),
                    evt.age.as_deref().unwrap_or("-")
                ));
            }

            ChatResponse {
                response: lines.join("\n"),
                response_type: "events".to_string(),
                data: Some(serde_json::json!({ "warnings": events.warning_count })),
            }
        }
        Err(e) => ChatResponse {
            response: format!("Failed to get events: {}", e),
            response_type: "error".to_string(),
            data: None,
        },
    }
}

async fn get_argocd_summary() -> ChatResponse {
    match argocd::get_argocd_status().await {
        Ok(status) => {
            let mut lines = vec![format!(
                "## 🚀 ArgoCD Status\n\n**Total Apps:** {} | **Healthy:** {} | **Issues:** {}\n",
                status.total, status.healthy, status.apps_with_issues.len()
            )];

            if status.apps_with_issues.is_empty() {
                lines.push("✅ All applications are healthy!".to_string());
            } else {
                lines.push("**Applications with issues:**\n".to_string());
                for issue in status.apps_with_issues.iter().take(10) {
                    lines.push(format!(
                        "- `{}` | {} | {} | {}",
                        issue.name,
                        issue.health_status,
                        issue.sync_status,
                        issue.error_duration.as_deref().unwrap_or("-")
                    ));
                }
            }

            ChatResponse {
                response: lines.join("\n"),
                response_type: "argocd".to_string(),
                data: Some(serde_json::json!({
                    "total": status.total,
                    "healthy": status.healthy,
                    "issues": status.apps_with_issues.len()
                })),
            }
        }
        Err(e) => ChatResponse {
            response: format!("Failed to get ArgoCD status: {}", e),
            response_type: "error".to_string(),
            data: None,
        },
    }
}

async fn get_backups_summary() -> ChatResponse {
    match backups::get_backups_status().await {
        Ok(status) => {
            let mut lines = vec![format!(
                "## 📦 Backup Jobs Status\n\n**CronJobs:** {} | **Active:** {} | **Succeeded:** {} | **Failed:** {}\n",
                status.total_cronjobs, status.active_jobs, status.succeeded_jobs, status.failed_jobs
            )];

            if status.cronjobs.is_empty() {
                lines.push("No CronJobs found.".to_string());
            } else {
                lines.push("**CronJobs:**\n".to_string());
                for cj in status.cronjobs.iter().take(10) {
                    let status_emoji = if cj.suspend {
                        "⏸️"
                    } else if cj.active_jobs > 0 {
                        "🔄"
                    } else {
                        "✅"
                    };
                    lines.push(format!(
                        "{} `{}` ({}) | `{}` | Last: {}",
                        status_emoji,
                        cj.name,
                        cj.namespace,
                        cj.schedule,
                        cj.last_schedule_age.as_deref().unwrap_or("-")
                    ));
                }
            }

            ChatResponse {
                response: lines.join("\n"),
                response_type: "backups".to_string(),
                data: Some(serde_json::json!({
                    "cronjobs": status.total_cronjobs,
                    "active": status.active_jobs,
                    "succeeded": status.succeeded_jobs,
                    "failed": status.failed_jobs
                })),
            }
        }
        Err(e) => ChatResponse {
            response: format!("Failed to get backup status: {}", e),
            response_type: "error".to_string(),
            data: None,
        },
    }
}

async fn get_namespaces_summary() -> ChatResponse {
    match cluster::get_cluster_overview().await {
        Ok(overview) => {
            let mut lines = vec![format!(
                "## 📁 Namespaces\n\n**Total:** {}\n",
                overview.namespace_count
            )];

            for ns in overview.namespaces.iter().take(20) {
                lines.push(format!("- `{}`", ns.name));
            }

            if overview.namespaces.len() > 20 {
                lines.push(format!("\n... and {} more", overview.namespaces.len() - 20));
            }

            ChatResponse {
                response: lines.join("\n"),
                response_type: "namespaces".to_string(),
                data: Some(serde_json::json!({ "count": overview.namespace_count })),
            }
        }
        Err(e) => ChatResponse {
            response: format!("Failed to get namespaces: {}", e),
            response_type: "error".to_string(),
            data: None,
        },
    }
}

async fn get_pvcs_summary() -> ChatResponse {
    match cluster::get_cluster_overview().await {
        Ok(overview) => {
            let mut lines = vec![format!(
                "## 💾 PVC Summary\n\n**Total:** {} | **Capacity:** {}\n",
                overview.pvc_count, overview.pvc_total_capacity
            )];

            // Show top 10 by capacity
            let mut pvcs = overview.pvcs.clone();
            pvcs.sort_by(|a, b| b.capacity_bytes.cmp(&a.capacity_bytes));

            lines.push("**Largest PVCs:**\n".to_string());
            for pvc in pvcs.iter().take(10) {
                lines.push(format!(
                    "- `{}` ({}) | {} | {}",
                    pvc.name, pvc.namespace, pvc.capacity, pvc.status
                ));
            }

            ChatResponse {
                response: lines.join("\n"),
                response_type: "pvcs".to_string(),
                data: Some(serde_json::json!({
                    "count": overview.pvc_count,
                    "capacity": overview.pvc_total_capacity
                })),
            }
        }
        Err(e) => ChatResponse {
            response: format!("Failed to get PVCs: {}", e),
            response_type: "error".to_string(),
            data: None,
        },
    }
}
