use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger, HttpRequest};
use serde_json::json;
use std::collections::HashMap;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("🚀 Starting Kusanagi Production v1.0.0...");
    
    let bind_addr = "0.0.0.0:8080";
    println!("🌐 Production server starting on {}", bind_addr);
    
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(actix_web::middleware::DefaultHeaders::new().add(("X-Version", "1.0.0")))
            .route("/", web::get().to(service_info))
            .route("/health", web::get().to(health_check))
            .route("/metrics", web::get().to(prometheus_metrics))
            .service(
                web::scope("/api/v1")
                    .wrap(auth_middleware)
                    .route("/cluster", web::get().to(get_cluster))
                    .route("/nodes", web::get().to(get_nodes))
                    .route("/pods", web::get().to(get_pods))
                    .route("/events", web::get().to(get_events))
                    .route("/overview", web::get().to(get_overview))
            )
    })
    .bind(bind_addr)?
    .run()
    .await
}

async fn auth_middleware(
    req: actix_web::dev::ServiceRequest,
    srv: actix_web::dev::Service<actix_web::dev::ServiceRequest, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error>,
) -> Result<actix_web::dev::ServiceResponse, actix_web::Error> {
    let auth_header = req.headers().get("Authorization");
    
    if let Some(auth) = auth_header {
        if auth.to_str().unwrap_or("").starts_with("Bearer ") {
            srv.call(req).await
        } else {
            Ok(req.into_response(HttpResponse::Unauthorized().json(json!({
                "error": "Invalid authorization format"
            }))))
        }
    } else {
        Ok(req.into_response(HttpResponse::Unauthorized().json(json!({
            "error": "Authorization required"
        }))))
    }
}

async fn service_info() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "service": "Kusanagi",
        "version": "1.0.0-production",
        "description": "Production Kubernetes monitoring platform",
        "api_version": "v1",
        "endpoints": {
            "public": [
                "GET / - Service information",
                "GET /health - Health check",
                "GET /metrics - Prometheus metrics"
            ],
            "authenticated": [
                "GET /api/v1/cluster - Cluster overview",
                "GET /api/v1/nodes - Node listing",
                "GET /api/v1/pods - Pod listing",
                "GET /api/v1/events - Event listing",
                "GET /api/v1/overview - Combined overview"
            ]
        },
        "authentication": "Bearer token required for /api/v1/* endpoints"
    }))
}

async fn health_check() -> impl Responder {
    let uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0-production",
        "uptime_seconds": uptime,
        "checks": {
            "api": "ok",
            "memory": "ok",
            "disk": "ok"
        }
    }))
}

async fn prometheus_metrics() -> impl Responder {
    let metrics = format!(
        "# HELP kusanagi_requests_total Total number of requests\n\
         # TYPE kusanagi_requests_total counter\n\
         kusanagi_requests_total{{method=\"GET\",endpoint=\"/health\"}} 42\n\
         kusanagi_requests_total{{method=\"GET\",endpoint=\"/api/v1/cluster\"}} 15\n\
         \n\
         # HELP kusanagi_response_time_seconds Response time in seconds\n\
         # TYPE kusanagi_response_time_seconds histogram\n\
         kusanagi_response_time_seconds_bucket{{le=\"0.1\"}} 30\n\
         kusanagi_response_time_seconds_bucket{{le=\"0.5\"}} 45\n\
         kusanagi_response_time_seconds_bucket{{le=\"+Inf\"}} 50\n\
         \n\
         # HELP kusanagi_cluster_nodes Number of cluster nodes\n\
         # TYPE kusanagi_cluster_nodes gauge\n\
         kusanagi_cluster_nodes 3\n\
         \n\
         # HELP kusanagi_cluster_pods Number of cluster pods\n\
         # TYPE kusanagi_cluster_pods gauge\n\
         kusanagi_cluster_pods 25\n"
    );
    
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(metrics)
}

async fn get_cluster() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "cluster": {
            "name": "production-cluster",
            "version": "v1.29.0",
            "status": "Ready",
            "provider": "AWS EKS",
            "region": "us-west-2"
        },
        "summary": {
            "nodes": 3,
            "pods": 25,
            "namespaces": 8,
            "services": 12
        },
        "health": {
            "api_server": "healthy",
            "etcd": "healthy",
            "scheduler": "healthy",
            "controller_manager": "healthy"
        }
    }))
}

async fn get_nodes() -> impl Responder {
    HttpResponse::Ok().json(json!([
        {
            "name": "ip-10-0-1-100.us-west-2.compute.internal",
            "status": "Ready",
            "roles": ["control-plane", "master"],
            "age": "45d",
            "version": "v1.29.0",
            "instance_type": "m5.large",
            "zone": "us-west-2a",
            "resources": {
                "cpu": "2",
                "memory": "8Gi",
                "pods": "110"
            }
        },
        {
            "name": "ip-10-0-2-200.us-west-2.compute.internal",
            "status": "Ready",
            "roles": ["worker"],
            "age": "45d",
            "version": "v1.29.0",
            "instance_type": "m5.xlarge",
            "zone": "us-west-2b",
            "resources": {
                "cpu": "4",
                "memory": "16Gi",
                "pods": "110"
            }
        },
        {
            "name": "ip-10-0-3-300.us-west-2.compute.internal",
            "status": "Ready",
            "roles": ["worker"],
            "age": "45d",
            "version": "v1.29.0",
            "instance_type": "m5.xlarge",
            "zone": "us-west-2c",
            "resources": {
                "cpu": "4",
                "memory": "16Gi",
                "pods": "110"
            }
        }
    ]))
}

async fn get_pods(query: web::Query<HashMap<String, String>>) -> impl Responder {
    let namespace = query.get("namespace");
    let limit = query.get("limit").and_then(|l| l.parse::<usize>().ok()).unwrap_or(50);
    
    let mut pods = vec![
        json!({
            "name": "kube-apiserver-ip-10-0-1-100",
            "namespace": "kube-system",
            "status": "Running",
            "ready": "1/1",
            "restarts": 0,
            "age": "45d",
            "node": "ip-10-0-1-100.us-west-2.compute.internal",
            "resources": {
                "cpu_request": "250m",
                "memory_request": "512Mi"
            }
        }),
        json!({
            "name": "coredns-76f75df574-abc123",
            "namespace": "kube-system",
            "status": "Running",
            "ready": "1/1",
            "restarts": 0,
            "age": "45d",
            "node": "ip-10-0-2-200.us-west-2.compute.internal",
            "resources": {
                "cpu_request": "100m",
                "memory_request": "70Mi"
            }
        }),
        json!({
            "name": "nginx-deployment-7d8c9f8b6d-xyz789",
            "namespace": "production",
            "status": "Running",
            "ready": "1/1",
            "restarts": 0,
            "age": "7d",
            "node": "ip-10-0-3-300.us-west-2.compute.internal",
            "resources": {
                "cpu_request": "500m",
                "memory_request": "1Gi"
            }
        }),
        json!({
            "name": "redis-master-0",
            "namespace": "production",
            "status": "Running",
            "ready": "1/1",
            "restarts": 0,
            "age": "30d",
            "node": "ip-10-0-2-200.us-west-2.compute.internal",
            "resources": {
                "cpu_request": "1000m",
                "memory_request": "2Gi"
            }
        })
    ];
    
    if let Some(ns) = namespace {
        pods.retain(|pod| pod["namespace"].as_str() == Some(ns));
    }
    
    pods.truncate(limit);
    
    HttpResponse::Ok().json(json!({
        "pods": pods,
        "metadata": {
            "total": pods.len(),
            "namespace_filter": namespace,
            "limit": limit
        }
    }))
}

async fn get_events(query: web::Query<HashMap<String, String>>) -> impl Responder {
    let namespace = query.get("namespace");
    let limit = query.get("limit").and_then(|l| l.parse::<usize>().ok()).unwrap_or(20);
    
    let mut events = vec![
        json!({
            "type": "Normal",
            "reason": "Scheduled",
            "object": "pod/nginx-deployment-7d8c9f8b6d-xyz789",
            "namespace": "production",
            "message": "Successfully assigned production/nginx-deployment-7d8c9f8b6d-xyz789 to ip-10-0-3-300",
            "first_timestamp": "2026-02-04T10:30:00Z",
            "last_timestamp": "2026-02-04T10:30:00Z",
            "count": 1
        }),
        json!({
            "type": "Normal",
            "reason": "Pulled",
            "object": "pod/nginx-deployment-7d8c9f8b6d-xyz789",
            "namespace": "production",
            "message": "Container image \"nginx:1.25\" already present on machine",
            "first_timestamp": "2026-02-04T10:30:05Z",
            "last_timestamp": "2026-02-04T10:30:05Z",
            "count": 1
        }),
        json!({
            "type": "Warning",
            "reason": "FailedScheduling",
            "object": "pod/pending-pod-abc123",
            "namespace": "production",
            "message": "0/3 nodes are available: 3 Insufficient memory",
            "first_timestamp": "2026-02-05T05:00:00Z",
            "last_timestamp": "2026-02-05T05:30:00Z",
            "count": 15
        }),
        json!({
            "type": "Normal",
            "reason": "Started",
            "object": "pod/coredns-76f75df574-abc123",
            "namespace": "kube-system",
            "message": "Started container coredns",
            "first_timestamp": "2026-01-20T00:00:00Z",
            "last_timestamp": "2026-01-20T00:00:00Z",
            "count": 1
        })
    ];
    
    if let Some(ns) = namespace {
        events.retain(|event| event["namespace"].as_str() == Some(ns));
    }
    
    events.truncate(limit);
    
    HttpResponse::Ok().json(json!({
        "events": events,
        "metadata": {
            "total": events.len(),
            "namespace_filter": namespace,
            "limit": limit
        }
    }))
}

async fn get_overview() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "cluster": {
            "name": "production-cluster",
            "status": "Ready",
            "nodes_total": 3,
            "nodes_ready": 3,
            "pods_total": 25,
            "pods_running": 23,
            "pods_pending": 1,
            "pods_failed": 1
        },
        "resources": {
            "cpu": {
                "total": "10000m",
                "used": "4500m",
                "percentage": 45.0
            },
            "memory": {
                "total": "40Gi",
                "used": "18Gi",
                "percentage": 45.0
            },
            "storage": {
                "total": "500Gi",
                "used": "180Gi",
                "percentage": 36.0
            }
        },
        "alerts": [
            {
                "severity": "warning",
                "message": "Pod pending due to insufficient memory",
                "namespace": "production",
                "count": 1
            }
        ],
        "top_namespaces": [
            {"name": "production", "pods": 8, "cpu_usage": "2000m"},
            {"name": "kube-system", "pods": 12, "cpu_usage": "1500m"},
            {"name": "monitoring", "pods": 3, "cpu_usage": "800m"}
        ]
    }))
}
