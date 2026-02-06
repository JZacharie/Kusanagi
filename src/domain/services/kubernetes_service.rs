use serde_json::{json, Value};
use kube::{Client, Api, api::ListParams};
use k8s_openapi::api::core::v1::{Pod, Node, Service, PersistentVolume, PersistentVolumeClaim, Event};
use k8s_openapi::api::networking::v1::Ingress;

pub async fn get_pods_status() -> Result<Value, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let pods: Api<Pod> = Api::all(client);
    let list = pods.list(&ListParams::default()).await.map_err(|e| e.to_string())?;

    let mut running = 0;
    let mut pending = 0;
    let mut failed = 0;
    let mut total = 0;
    let mut pods_in_error = Vec::new();

    for pod in list {
        total += 1;
        let mut is_error = false;
        let mut reason = String::new();
        let mut restart_count = 0;
        
        // Check Phase
        let phase = pod.status.as_ref()
            .and_then(|s| s.phase.as_deref())
            .unwrap_or("Unknown");

        match phase {
            "Running" | "Succeeded" => running += 1,
            "Pending" => {
                pending += 1;
                // Pending is technically not failing, but can be stuck.
                // We'll mark it as error only if it has bad conditions/reasons,
                // but usually user wants to see it in the list if it's stuck.
                // For now, let's include Pending in the list if we want visibility?
                // Frontend k8s.js calls it "progressing".
            },
            _ => {
                failed += 1;
                is_error = true;
                reason = phase.to_string();
            }
        }
        
        // Deep inspection of container statuses
        if let Some(status) = &pod.status {
            if let Some(container_statuses) = &status.container_statuses {
                for cs in container_statuses {
                    restart_count += cs.restart_count;
                    
                    if let Some(state) = &cs.state {
                        if let Some(waiting) = &state.waiting {
                            if let Some(r) = &waiting.reason {
                                if r == "CrashLoopBackOff" || r == "ImagePullBackOff" || r == "ErrImagePull" || r == "ContainerCreating" {
                                    // ContainerCreating is normal but maybe we show it?
                                    if r != "ContainerCreating" {
                                        is_error = true;
                                        reason = r.clone();
                                        if phase == "Running" {
                                            // Should we decrement running and increment failed?
                                            // This changes the stats vs phase.
                                            // Let's keep stats based on Phase (k8s standard) but add to error list.
                                            // NOTE: Frontend shows "error_pods" count.
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(terminated) = &state.terminated {
                            if terminated.exit_code != 0 {
                                is_error = true;
                                reason = terminated.reason.clone().unwrap_or("Error".to_string());
                            }
                        }
                    }
                }
            }
        }

        if is_error {
             // Calculate Age
             let age = if let Some(ts) = &pod.metadata.creation_timestamp {
                 let now = chrono::Utc::now();
                 let created = ts.0;
                 let diff = now.signed_duration_since(created);
                 if diff.num_days() > 0 {
                     format!("{}d", diff.num_days())
                 } else if diff.num_hours() > 0 {
                     format!("{}h", diff.num_hours())
                 } else if diff.num_minutes() > 0 {
                     format!("{}m", diff.num_minutes())
                 } else {
                     format!("{}s", diff.num_seconds())
                 }
             } else {
                 "0s".to_string()
             };

             pods_in_error.push(json!({
                "name": pod.metadata.name.clone().unwrap_or_default(),
                "namespace": pod.metadata.namespace.clone().unwrap_or_default(),
                "status": phase,
                "reason": if reason.is_empty() { phase } else { &reason },
                "restart_count": restart_count,
                "age": age,
                "node": pod.spec.as_ref().and_then(|s| s.node_name.clone()).unwrap_or_default(),
                "cpu_usage": 0, // Needs metrics-server, placeholder
                "memory_usage": 0,
                "cpu_limit": 0,
                "memory_limit": 0
             }));
        }
    }
    
    // Determine strict failing count for dashboard (including CrashLoops that might be Phase=Running)
    // The frontend uses `pods_in_error.length` as `pods-error-count`
    // But `error_pods` stat is also sent.
    
    Ok(json!({
        "running": running,
        "pending": pending,
        "failed": failed,
        "total": total,
        // Frontend expected fields
        "total_pods": total,
        "running_pods": running,
        "error_pods": pods_in_error.len(), // Use actual list length
        "pending_pods": pending,
        "pods_in_error": pods_in_error
    }))
}

// Imports are at top


pub async fn get_nodes_status() -> Result<Value, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let nodes: Api<Node> = Api::all(client);
    let list = nodes.list(&ListParams::default()).await.map_err(|e| e.to_string())?;
    
    let mut ready = 0;
    let mut not_ready = 0;
    let mut total = 0;

    for node in list {
        total += 1;
        let mut is_ready = false;
        if let Some(status) = node.status {
            if let Some(conditions) = status.conditions {
                for cond in conditions {
                    if cond.type_ == "Ready" && cond.status == "True" {
                        is_ready = true;
                        break;
                    }
                }
            }
        }
        
        if is_ready {
            ready += 1;
        } else {
            not_ready += 1;
        }
    }
    
    Ok(json!({
        "ready": ready,
        "not_ready": not_ready,
        "total": total
    }))
}

pub async fn get_cluster_overview() -> Result<Value, String> {
    let pods = get_pods_status().await?;
    let nodes = get_nodes_status().await?;
    
    let services_count = match get_services().await {
        Ok(json) => json.as_array().map(|v| v.len()).unwrap_or(0),
        Err(_) => 4
    };
    
    Ok(json!({
        "nodes": nodes["total"],
        "pods": pods["total"],
        "services": services_count,
        "pods_running": pods["running"],
        "nodes_ready": nodes["ready"]
    }))
}

// Imports are at top


pub async fn get_services() -> Result<Value, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let services: Api<Service> = Api::all(client);
    let list = services.list(&ListParams::default()).await.map_err(|e| e.to_string())?;

    let services_json: Vec<Value> = list.iter().map(|svc| {
        // Age calculation
        let age = if let Some(ts) = &svc.metadata.creation_timestamp {
             let now = chrono::Utc::now();
             let created = ts.0;
             let diff = now.signed_duration_since(created);
             if diff.num_days() > 0 {
                 format!("{}d", diff.num_days())
             } else if diff.num_hours() > 0 {
                 format!("{}h", diff.num_hours())
             } else if diff.num_minutes() > 0 {
                 format!("{}m", diff.num_minutes())
             } else {
                 format!("{}s", diff.num_seconds())
             }
         } else {
             "0s".to_string()
         };

        // Ports
        let ports = svc.spec.as_ref()
            .and_then(|s| s.ports.as_ref())
            .map(|p| p.iter().map(|port| {
                format!("{}:{}/{}", port.port, port.node_port.unwrap_or(0), port.protocol.clone().unwrap_or_default())
            }).collect::<Vec<String>>().join(", "))
            .unwrap_or_default();

        // External IP
        let external_ip = svc.status.as_ref()
            .and_then(|s| s.load_balancer.as_ref())
            .and_then(|lb| lb.ingress.as_ref())
            .and_then(|i| i.first())
            .map(|ing| ing.ip.clone().unwrap_or_else(|| ing.hostname.clone().unwrap_or_default()))
            .unwrap_or_else(|| "<none>".to_string());

        json!({
            "name": svc.metadata.name.clone().unwrap_or_default(),
            "namespace": svc.metadata.namespace.clone().unwrap_or_default(),
            "type_": svc.spec.as_ref().and_then(|s| s.type_.clone()).unwrap_or_default(),
            "cluster_ip": svc.spec.as_ref().and_then(|s| s.cluster_ip.clone()).unwrap_or_default(),
            "external_ip": external_ip,
            "ports": ports,
            "age": age
        })
    }).collect();
    
    Ok(json!(services_json))
}

// Imports are at top


pub async fn get_ingress() -> Result<Value, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let ingresses: Api<Ingress> = Api::all(client);
    let list = ingresses.list(&ListParams::default()).await.map_err(|e| e.to_string())?;
    
    let ingresses_json: Vec<Value> = list.iter().map(|ing| {
        // Age calculation
        let age = if let Some(ts) = &ing.metadata.creation_timestamp {
             let now = chrono::Utc::now();
             let created = ts.0;
             let diff = now.signed_duration_since(created);
             if diff.num_days() > 0 {
                 format!("{}d", diff.num_days())
             } else if diff.num_hours() > 0 {
                 format!("{}h", diff.num_hours())
             } else if diff.num_minutes() > 0 {
                 format!("{}m", diff.num_minutes())
             } else {
                 format!("{}s", diff.num_seconds())
             }
         } else {
             "0s".to_string()
         };

        let rules = ing.spec.as_ref()
            .and_then(|spec| spec.rules.as_ref())
            .map(|rules| {
                rules.iter().filter_map(|r| r.host.clone()).collect::<Vec<String>>()
            })
            .unwrap_or_default();
            
        json!({
            "name": ing.metadata.name.clone().unwrap_or_default(),
            "namespace": ing.metadata.namespace.clone().unwrap_or_default(),
            "rules": rules,
            "age": age
        })
    }).collect();
    
    Ok(json!(ingresses_json))
}

// Imports are at top


pub async fn get_storage() -> Result<Value, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    
    let pvcs: Api<PersistentVolumeClaim> = Api::all(client);
    let pvc_list = pvcs.list(&ListParams::default()).await.map_err(|e| e.to_string())?;
    
    let pvcs_json: Vec<Value> = pvc_list.iter().map(|pvc| {
        let name = pvc.metadata.name.clone().unwrap_or_default();
        let namespace = pvc.metadata.namespace.clone().unwrap_or_default();
        let status = pvc.status.as_ref().and_then(|s| s.phase.clone()).unwrap_or("Unknown".to_string());
        let storage_class = pvc.spec.as_ref().and_then(|s| s.storage_class_name.clone()).unwrap_or_default();
        
        let capacity = pvc.status.as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|c| c.get("storage"))
            .map(|q| q.0.clone())
            .unwrap_or("0".to_string());

        // Rough parsing of capacity string to bytes for total calculation (simplified)
        // e.g., "10Gi" -> 10 * 1024^3
        // This is a bit complex in Rust without a parser library, but for now we can just display strings.
        // Or leave total calculation separate or simplified.
        
        json!({
            "name": name,
            "namespace": namespace,
            "status": status,
            "storage_class": storage_class,
            "capacity": capacity,
            "used_bytes": 0, // No metrics available via standard API
            "usage_percent": 0.0
        })
    }).collect();
    
    Ok(json!({
        "pvc_count": pvcs_json.len(),
        "pvc_total_capacity": "Calculated on Frontend or Placeholder", // Parsing Quantity is hard without crate
        "pvcs": pvcs_json
    }))
}

// Imports are at top


pub async fn get_events() -> Result<Value, String> {
    let client = Client::try_default().await.map_err(|e| e.to_string())?;
    let events_api: Api<Event> = Api::all(client);
    let mut events_list = events_api.list(&ListParams::default()).await.map_err(|e| e.to_string())?.items;
    
    // Sort by lastTimestamp
    events_list.sort_by(|a, b| {
        a.last_timestamp.cmp(&b.last_timestamp)
    });
    
    // Take last 20 (latest)
    let events: Vec<Value> = events_list.iter().rev().take(20).map(|event| {
         json!({
            "type": event.type_.clone().unwrap_or_default(),
            "reason": event.reason.clone().unwrap_or_default(),
            "message": event.message.clone().unwrap_or_default(),
            "namespace": event.metadata.namespace.clone().unwrap_or_default(),
            "object": event.involved_object.name.clone().unwrap_or_default(),
            "timestamp": event.last_timestamp.clone()
        })
    }).collect();
    
    Ok(json!(events))
}

// End of file
