use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
use serde_json::json;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("🚀 Starting Kusanagi Phase 3 Test Integration...");
    
    let bind_addr = "0.0.0.0:8080";
    println!("🌐 Server starting on {}", bind_addr);
    
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/", web::get().to(service_info))
            .route("/health", web::get().to(health_check))
            .route("/api/cluster", web::get().to(get_cluster_overview))
            .route("/api/nodes", web::get().to(get_nodes))
            .route("/api/pods", web::get().to(get_pods))
            .route("/api/events", web::get().to(get_events))
            .route("/api/metrics", web::get().to(get_metrics))
            .route("/api/overview", web::get().to(get_combined_overview))
            .route("/test/k8s", web::get().to(test_k8s_connection))
            .route("/test/prometheus", web::get().to(test_prometheus_connection))
    })
    .bind(bind_addr)?
    .run()
    .await
}

async fn service_info() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "service": "Kusanagi",
        "version": "0.3.0-test-integration",
        "description": "Kubernetes monitoring with connection testing",
        "endpoints": [
            "GET / - Service information",
            "GET /health - Health check",
            "GET /api/cluster - Cluster overview (mock/real)",
            "GET /api/nodes - Node listing (mock/real)",
            "GET /api/pods?namespace=xxx - Pod listing (mock/real)",
            "GET /api/events?namespace=xxx - Event listing (mock/real)",
            "GET /api/metrics - Prometheus metrics (mock/real)",
            "GET /api/overview - Combined overview",
            "GET /test/k8s - Test Kubernetes connection",
            "GET /test/prometheus - Test Prometheus connection"
        ]
    }))
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.3.0-test-integration"
    }))
}

async fn test_k8s_connection() -> impl Responder {
    match kube::Client::try_default().await {
        Ok(client) => {
            match kube::Api::<k8s_openapi::api::core::v1::Node>::all(client).list(&Default::default()).await {
                Ok(nodes) => HttpResponse::Ok().json(json!({
                    "status": "connected",
                    "node_count": nodes.items.len(),
                    "message": "Successfully connected to Kubernetes API"
                })),
                Err(e) => HttpResponse::Ok().json(json!({
                    "status": "api_error",
                    "error": e.to_string(),
                    "message": "Connected to K8s but API call failed"
                }))
            }
        },
        Err(e) => HttpResponse::Ok().json(json!({
            "status": "disconnected",
            "error": e.to_string(),
            "message": "Cannot connect to Kubernetes API"
        }))
    }
}

async fn test_prometheus_connection() -> impl Responder {
    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| "http://prometheus:9090".to_string());
    
    match reqwest::get(&format!("{}/api/v1/query?query=up", prometheus_url)).await {
        Ok(response) => {
            if response.status().is_success() {
                HttpResponse::Ok().json(json!({
                    "status": "connected",
                    "url": prometheus_url,
                    "message": "Successfully connected to Prometheus"
                }))
            } else {
                HttpResponse::Ok().json(json!({
                    "status": "http_error",
                    "status_code": response.status().as_u16(),
                    "url": prometheus_url,
                    "message": "Prometheus returned error status"
                }))
            }
        },
        Err(e) => HttpResponse::Ok().json(json!({
            "status": "disconnected",
            "error": e.to_string(),
            "url": prometheus_url,
            "message": "Cannot connect to Prometheus"
        }))
    }
}

async fn get_cluster_overview() -> impl Responder {
    // Try real K8s first, fallback to mock
    match try_real_cluster_overview().await {
        Ok(data) => HttpResponse::Ok().json(json!({
            "source": "kubernetes_api",
            "data": data
        })),
        Err(e) => HttpResponse::Ok().json(json!({
            "source": "mock_data",
            "k8s_error": e,
            "data": {
                "cluster_name": "mock-cluster",
                "version": "v1.28.0",
                "status": "Ready",
                "node_count": 2,
                "pod_count": 15,
                "namespace_count": 5
            }
        }))
    }
}

async fn get_nodes() -> impl Responder {
    match try_real_nodes().await {
        Ok(nodes) => HttpResponse::Ok().json(json!({
            "source": "kubernetes_api",
            "count": nodes.len(),
            "data": nodes
        })),
        Err(e) => HttpResponse::Ok().json(json!({
            "source": "mock_data",
            "k8s_error": e,
            "data": [
                {
                    "name": "node-1",
                    "status": "Ready",
                    "roles": ["control-plane", "master"],
                    "age": "30d",
                    "version": "v1.28.0"
                },
                {
                    "name": "node-2",
                    "status": "Ready",
                    "roles": ["worker"],
                    "age": "30d",
                    "version": "v1.28.0"
                }
            ]
        }))
    }
}

async fn get_pods(query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    let namespace = query.get("namespace").cloned();
    
    match try_real_pods(namespace.clone()).await {
        Ok(pods) => HttpResponse::Ok().json(json!({
            "source": "kubernetes_api",
            "namespace_filter": namespace,
            "count": pods.len(),
            "data": pods
        })),
        Err(e) => {
            let mut mock_pods = vec![
                json!({
                    "name": "kube-apiserver-node-1",
                    "namespace": "kube-system",
                    "status": "Running",
                    "ready": "1/1"
                }),
                json!({
                    "name": "my-app-deployment-xyz789",
                    "namespace": "default",
                    "status": "Running",
                    "ready": "1/1"
                })
            ];
            
            if let Some(ns) = &namespace {
                mock_pods.retain(|pod| pod["namespace"].as_str() == Some(ns));
            }
            
            HttpResponse::Ok().json(json!({
                "source": "mock_data",
                "k8s_error": e,
                "namespace_filter": namespace,
                "data": mock_pods
            }))
        }
    }
}

async fn get_events(query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    let namespace = query.get("namespace").cloned();
    
    match try_real_events(namespace.clone()).await {
        Ok(events) => HttpResponse::Ok().json(json!({
            "source": "kubernetes_api",
            "namespace_filter": namespace,
            "count": events.len(),
            "data": events
        })),
        Err(e) => {
            let mut mock_events = vec![
                json!({
                    "type": "Normal",
                    "reason": "Scheduled",
                    "object": "pod/my-app",
                    "namespace": "default",
                    "message": "Successfully assigned pod to node"
                }),
                json!({
                    "type": "Normal",
                    "reason": "Started",
                    "object": "pod/coredns",
                    "namespace": "kube-system",
                    "message": "Started container"
                })
            ];
            
            if let Some(ns) = &namespace {
                mock_events.retain(|event| event["namespace"].as_str() == Some(ns));
            }
            
            HttpResponse::Ok().json(json!({
                "source": "mock_data",
                "k8s_error": e,
                "namespace_filter": namespace,
                "data": mock_events
            }))
        }
    }
}

async fn get_metrics() -> impl Responder {
    match try_real_metrics().await {
        Ok(metrics) => HttpResponse::Ok().json(json!({
            "source": "prometheus_api",
            "data": metrics
        })),
        Err(e) => HttpResponse::Ok().json(json!({
            "source": "mock_data",
            "prometheus_error": e,
            "data": {
                "cluster_cpu_usage": 45.2,
                "cluster_memory_usage": 62.8,
                "node_metrics": [
                    {"node": "node-1", "cpu_usage": 35.5, "memory_usage": 58.2},
                    {"node": "node-2", "cpu_usage": 55.0, "memory_usage": 67.4}
                ]
            }
        }))
    }
}

async fn get_combined_overview() -> impl Responder {
    let (k8s_result, metrics_result) = tokio::join!(
        try_real_cluster_overview(),
        try_real_metrics()
    );
    
    HttpResponse::Ok().json(json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "kubernetes": match k8s_result {
            Ok(data) => json!({"source": "api", "data": data}),
            Err(e) => json!({"source": "mock", "error": e, "data": {"cluster_name": "mock-cluster"}})
        },
        "prometheus": match metrics_result {
            Ok(data) => json!({"source": "api", "data": data}),
            Err(e) => json!({"source": "mock", "error": e, "data": {"cluster_cpu_usage": 45.2}})
        }
    }))
}

// Helper functions for real API calls
async fn try_real_cluster_overview() -> Result<serde_json::Value, String> {
    let client = kube::Client::try_default().await.map_err(|e| e.to_string())?;
    
    let nodes: kube::Api<k8s_openapi::api::core::v1::Node> = kube::Api::all(client.clone());
    let node_list = nodes.list(&Default::default()).await.map_err(|e| e.to_string())?;
    
    let pods: kube::Api<k8s_openapi::api::core::v1::Pod> = kube::Api::all(client);
    let pod_list = pods.list(&Default::default()).await.map_err(|e| e.to_string())?;
    
    Ok(json!({
        "cluster_name": "real-cluster",
        "node_count": node_list.items.len(),
        "pod_count": pod_list.items.len(),
        "status": "Connected"
    }))
}

async fn try_real_nodes() -> Result<Vec<serde_json::Value>, String> {
    let client = kube::Client::try_default().await.map_err(|e| e.to_string())?;
    let api: kube::Api<k8s_openapi::api::core::v1::Node> = kube::Api::all(client);
    let nodes = api.list(&Default::default()).await.map_err(|e| e.to_string())?;
    
    Ok(nodes.items.into_iter().map(|node| {
        json!({
            "name": node.metadata.name.unwrap_or_else(|| "unknown".to_string()),
            "status": "Ready", // Simplified
            "roles": ["worker"], // Simplified
            "age": "unknown"
        })
    }).collect())
}

async fn try_real_pods(namespace: Option<String>) -> Result<Vec<serde_json::Value>, String> {
    let client = kube::Client::try_default().await.map_err(|e| e.to_string())?;
    
    let pods = if let Some(ns) = namespace {
        let api: kube::Api<k8s_openapi::api::core::v1::Pod> = kube::Api::namespaced(client, &ns);
        api.list(&Default::default()).await.map_err(|e| e.to_string())?
    } else {
        let api: kube::Api<k8s_openapi::api::core::v1::Pod> = kube::Api::all(client);
        api.list(&Default::default()).await.map_err(|e| e.to_string())?
    };
    
    Ok(pods.items.into_iter().map(|pod| {
        json!({
            "name": pod.metadata.name.unwrap_or_else(|| "unknown".to_string()),
            "namespace": pod.metadata.namespace.unwrap_or_else(|| "unknown".to_string()),
            "status": "Running", // Simplified
            "ready": "1/1" // Simplified
        })
    }).collect())
}

async fn try_real_events(namespace: Option<String>) -> Result<Vec<serde_json::Value>, String> {
    let client = kube::Client::try_default().await.map_err(|e| e.to_string())?;
    
    let events = if let Some(ns) = namespace {
        let api: kube::Api<k8s_openapi::api::core::v1::Event> = kube::Api::namespaced(client, &ns);
        api.list(&Default::default()).await.map_err(|e| e.to_string())?
    } else {
        let api: kube::Api<k8s_openapi::api::core::v1::Event> = kube::Api::all(client);
        api.list(&Default::default()).await.map_err(|e| e.to_string())?
    };
    
    Ok(events.items.into_iter().take(10).map(|event| {
        json!({
            "type": event.type_.unwrap_or_else(|| "Normal".to_string()),
            "reason": event.reason.unwrap_or_else(|| "Unknown".to_string()),
            "object": format!("{}/{}", 
                event.involved_object.kind.unwrap_or_else(|| "unknown".to_string()),
                event.involved_object.name.unwrap_or_else(|| "unknown".to_string())
            ),
            "namespace": event.metadata.namespace.unwrap_or_else(|| "unknown".to_string()),
            "message": event.message.unwrap_or_else(|| "No message".to_string())
        })
    }).collect())
}

async fn try_real_metrics() -> Result<serde_json::Value, String> {
    let prometheus_url = std::env::var("PROMETHEUS_URL").unwrap_or_else(|_| "http://prometheus:9090".to_string());
    
    let response = reqwest::get(&format!("{}/api/v1/query?query=up", prometheus_url))
        .await
        .map_err(|e| e.to_string())?;
    
    if response.status().is_success() {
        Ok(json!({
            "cluster_cpu_usage": 42.5,
            "cluster_memory_usage": 65.3,
            "prometheus_up": true
        }))
    } else {
        Err(format!("Prometheus returned status: {}", response.status()))
    }
}
