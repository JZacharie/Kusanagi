use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
use serde_json::json;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("🚀 Starting Kusanagi Phase 3 Extended Server...");
    
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
    })
    .bind(bind_addr)?
    .run()
    .await
}

// Handlers
async fn service_info() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "service": "Kusanagi",
        "version": "0.2.0-phase3-extended",
        "description": "Kubernetes monitoring platform with Prometheus integration",
        "endpoints": [
            "GET / - Service information",
            "GET /health - Health check",
            "GET /api/cluster - Cluster overview",
            "GET /api/nodes - Node listing",
            "GET /api/pods?namespace=xxx - Pod listing (optional namespace filter)",
            "GET /api/events?namespace=xxx - Event listing (optional namespace filter)",
            "GET /api/metrics - Prometheus metrics",
            "GET /api/overview - Combined K8s + Prometheus overview"
        ]
    }))
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.2.0-phase3-extended"
    }))
}

async fn get_cluster_overview() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "cluster": {
            "name": "mock-cluster",
            "version": "v1.28.0",
            "status": "Ready"
        },
        "nodes": [
            {
                "name": "node-1",
                "status": "Ready",
                "roles": ["control-plane", "master"],
                "age": "30d"
            },
            {
                "name": "node-2", 
                "status": "Ready",
                "roles": ["worker"],
                "age": "30d"
            }
        ],
        "pods": [
            {
                "name": "kube-apiserver-node-1",
                "namespace": "kube-system",
                "status": "Running",
                "ready": "1/1"
            }
        ]
    }))
}

async fn get_nodes() -> impl Responder {
    HttpResponse::Ok().json(json!([
        {
            "name": "node-1",
            "status": "Ready",
            "roles": ["control-plane", "master"],
            "age": "30d",
            "version": "v1.28.0",
            "internal_ip": "10.0.0.1",
            "external_ip": "203.0.113.1"
        },
        {
            "name": "node-2",
            "status": "Ready", 
            "roles": ["worker"],
            "age": "30d",
            "version": "v1.28.0",
            "internal_ip": "10.0.0.2",
            "external_ip": "203.0.113.2"
        }
    ]))
}

async fn get_pods(query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    let namespace = query.get("namespace");
    
    let mut pods = vec![
        json!({
            "name": "kube-apiserver-node-1",
            "namespace": "kube-system",
            "status": "Running",
            "ready": "1/1",
            "restarts": 0,
            "age": "30d"
        }),
        json!({
            "name": "coredns-558bd4d5db-abc123",
            "namespace": "kube-system", 
            "status": "Running",
            "ready": "1/1",
            "restarts": 0,
            "age": "30d"
        }),
        json!({
            "name": "my-app-deployment-xyz789",
            "namespace": "default",
            "status": "Running", 
            "ready": "1/1",
            "restarts": 0,
            "age": "1d"
        })
    ];
    
    if let Some(ns) = namespace {
        pods.retain(|pod| pod["namespace"].as_str() == Some(ns));
    }
    
    HttpResponse::Ok().json(pods)
}

async fn get_events(query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    let namespace = query.get("namespace");
    
    let mut events = vec![
        json!({
            "type": "Normal",
            "reason": "Scheduled",
            "object": "pod/my-app-deployment-xyz789",
            "namespace": "default",
            "message": "Successfully assigned default/my-app-deployment-xyz789 to node-2",
            "first_timestamp": "2024-01-15T10:30:00Z",
            "last_timestamp": "2024-01-15T10:30:00Z",
            "count": 1
        }),
        json!({
            "type": "Normal",
            "reason": "Pulled",
            "object": "pod/my-app-deployment-xyz789", 
            "namespace": "default",
            "message": "Container image \"nginx:1.21\" already present on machine",
            "first_timestamp": "2024-01-15T10:30:05Z",
            "last_timestamp": "2024-01-15T10:30:05Z",
            "count": 1
        }),
        json!({
            "type": "Normal",
            "reason": "Started",
            "object": "pod/coredns-558bd4d5db-abc123",
            "namespace": "kube-system",
            "message": "Started container coredns",
            "first_timestamp": "2024-01-01T00:00:00Z",
            "last_timestamp": "2024-01-01T00:00:00Z", 
            "count": 1
        })
    ];
    
    if let Some(ns) = namespace {
        events.retain(|event| event["namespace"].as_str() == Some(ns));
    }
    
    HttpResponse::Ok().json(events)
}

async fn get_metrics() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "cluster_cpu_usage": 45.2,
        "cluster_memory_usage": 62.8,
        "node_metrics": [
            {
                "node": "node-1",
                "cpu_usage": 35.5,
                "memory_usage": 58.2,
                "disk_usage": 42.1
            },
            {
                "node": "node-2", 
                "cpu_usage": 55.0,
                "memory_usage": 67.4,
                "disk_usage": 38.9
            }
        ],
        "pod_metrics": [
            {
                "pod": "kube-apiserver-node-1",
                "namespace": "kube-system",
                "cpu_usage": 15.2,
                "memory_usage": 256.5
            },
            {
                "pod": "my-app-deployment-xyz789",
                "namespace": "default", 
                "cpu_usage": 5.1,
                "memory_usage": 128.0
            }
        ]
    }))
}

async fn get_combined_overview() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "kubernetes": {
            "cluster": {
                "name": "mock-cluster",
                "version": "v1.28.0", 
                "status": "Ready"
            },
            "nodes": [
                {
                    "name": "node-1",
                    "status": "Ready",
                    "roles": ["control-plane", "master"]
                },
                {
                    "name": "node-2",
                    "status": "Ready", 
                    "roles": ["worker"]
                }
            ],
            "pods": [
                {
                    "name": "kube-apiserver-node-1",
                    "namespace": "kube-system",
                    "status": "Running"
                },
                {
                    "name": "my-app-deployment-xyz789",
                    "namespace": "default",
                    "status": "Running"
                }
            ]
        },
        "prometheus": {
            "cluster_cpu_usage": 45.2,
            "cluster_memory_usage": 62.8,
            "node_metrics": [
                {
                    "node": "node-1",
                    "cpu_usage": 35.5,
                    "memory_usage": 58.2
                },
                {
                    "node": "node-2",
                    "cpu_usage": 55.0, 
                    "memory_usage": 67.4
                }
            ]
        },
        "summary": {
            "total_nodes": 2,
            "total_pods": 2,
            "metrics_available": true,
            "cluster_health": "healthy"
        }
    }))
}
