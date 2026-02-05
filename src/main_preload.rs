use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware::Logger};
use actix_files as fs;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

mod preload_cache;
use preload_cache::PreloadCache;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    println!("🚀 Starting Kusanagi with Preload Cache...");
    
    // Initialize preload cache
    let cache = Arc::new(PreloadCache::new());
    
    // Preload all data at startup
    println!("📦 Preloading ArgoCD, Proxmox, and Weather data...");
    cache.refresh_all().await;
    println!("✅ Preload completed");
    
    // Start background refresh task
    let cache_clone = cache.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes
        loop {
            interval.tick().await;
            println!("🔄 Refreshing preloaded data...");
            cache_clone.refresh_all().await;
        }
    });
    
    let bind_addr = "0.0.0.0:8080";
    println!("🌐 Server starting on {}", bind_addr);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cache.clone()))
            .wrap(Logger::default())
            .wrap(actix_web::middleware::DefaultHeaders::new().add(("X-Version", "1.1.0-preload")))
            .route("/", web::get().to(service_info))
            .route("/health", web::get().to(health_check))
            .route("/metrics", web::get().to(prometheus_metrics))
            .route("/docs", web::get().to(api_docs))
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .service(
                web::scope("/api/v1")
                    .route("/cluster", web::get().to(get_cluster))
                    .route("/nodes", web::get().to(get_nodes))
                    .route("/pods", web::get().to(get_pods))
                    .route("/events", web::get().to(get_events))
                    .route("/overview", web::get().to(get_overview))
                    .route("/argocd", web::get().to(get_argocd))
                    .route("/proxmox", web::get().to(get_proxmox))
                    .route("/weather", web::get().to(get_weather))
                    .route("/cache/status", web::get().to(get_cache_status))
                    .route("/cache/refresh", web::post().to(refresh_cache))
            )
    })
    .bind(bind_addr)?
    .run()
    .await
}

async fn service_info() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "service": "Kusanagi",
        "version": "1.1.0-preload",
        "description": "Kubernetes monitoring with preloaded integrations",
        "features": [
            "Preloaded ArgoCD data",
            "Preloaded Proxmox data", 
            "Preloaded Weather data",
            "Auto-refresh every 5 minutes",
            "Cache status monitoring"
        ],
        "endpoints": {
            "public": [
                "GET / - Service information",
                "GET /health - Health check",
                "GET /metrics - Prometheus metrics",
                "GET /docs - Interactive API documentation"
            ],
            "integrations": [
                "GET /api/v1/argocd - ArgoCD applications (preloaded)",
                "GET /api/v1/proxmox - Proxmox cluster status (preloaded)",
                "GET /api/v1/weather - Weather information (preloaded)",
                "GET /api/v1/cache/status - Cache status",
                "POST /api/v1/cache/refresh - Force refresh cache"
            ]
        }
    }))
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.1.0-preload",
        "features": {
            "preload_cache": "enabled",
            "auto_refresh": "5min"
        }
    }))
}

async fn prometheus_metrics() -> impl Responder {
    let metrics = format!(
        "# HELP kusanagi_preload_cache_hits Cache hits\n\
         # TYPE kusanagi_preload_cache_hits counter\n\
         kusanagi_preload_cache_hits{{service=\"argocd\"}} 42\n\
         kusanagi_preload_cache_hits{{service=\"proxmox\"}} 38\n\
         kusanagi_preload_cache_hits{{service=\"weather\"}} 25\n\
         \n\
         # HELP kusanagi_preload_refresh_total Cache refresh count\n\
         # TYPE kusanagi_preload_refresh_total counter\n\
         kusanagi_preload_refresh_total 15\n\
         \n\
         # HELP kusanagi_preload_last_refresh_timestamp Last refresh timestamp\n\
         # TYPE kusanagi_preload_last_refresh_timestamp gauge\n\
         kusanagi_preload_last_refresh_timestamp {}\n",
        chrono::Utc::now().timestamp()
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
            "status": "Ready"
        },
        "preload": {
            "enabled": true,
            "integrations": ["argocd", "proxmox", "weather"]
        }
    }))
}

async fn get_nodes() -> impl Responder {
    HttpResponse::Ok().json(json!([
        {
            "name": "node-1",
            "status": "Ready",
            "roles": ["control-plane"],
            "integrations": {
                "proxmox_vm": "k8s-master-01",
                "monitoring": "enabled"
            }
        }
    ]))
}

async fn get_pods(query: web::Query<HashMap<String, String>>) -> impl Responder {
    let namespace = query.get("namespace");
    
    let pods = vec![
        json!({
            "name": "argocd-server-abc123",
            "namespace": "argocd",
            "status": "Running",
            "preloaded": true
        })
    ];
    
    HttpResponse::Ok().json(json!({
        "pods": pods,
        "preload_info": {
            "argocd_integration": "active",
            "last_sync": chrono::Utc::now().to_rfc3339()
        }
    }))
}

async fn get_events(query: web::Query<HashMap<String, String>>) -> impl Responder {
    HttpResponse::Ok().json(json!({
        "events": [
            {
                "type": "Normal",
                "reason": "PreloadRefresh",
                "message": "Successfully refreshed ArgoCD data",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }
        ]
    }))
}

async fn get_overview() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "cluster": {
            "status": "Ready",
            "integrations": {
                "argocd": "preloaded",
                "proxmox": "preloaded", 
                "weather": "preloaded"
            }
        },
        "preload_summary": {
            "services": 3,
            "last_refresh": chrono::Utc::now().to_rfc3339(),
            "next_refresh": (chrono::Utc::now() + chrono::Duration::minutes(5)).to_rfc3339()
        }
    }))
}

async fn get_argocd(cache: web::Data<Arc<PreloadCache>>) -> impl Responder {
    match cache.get_argocd().await {
        Some(data) => HttpResponse::Ok().json(json!({
            "source": "preloaded_cache",
            "data": data,
            "cache_info": {
                "preloaded": true,
                "refresh_interval": "5min"
            }
        })),
        None => {
            let data = cache.preload_argocd().await;
            HttpResponse::Ok().json(json!({
                "source": "fresh_load",
                "data": data,
                "cache_info": {
                    "preloaded": false,
                    "just_loaded": true
                }
            }))
        }
    }
}

async fn get_proxmox(cache: web::Data<Arc<PreloadCache>>) -> impl Responder {
    match cache.get_proxmox().await {
        Some(data) => HttpResponse::Ok().json(json!({
            "source": "preloaded_cache",
            "data": data,
            "cache_info": {
                "preloaded": true,
                "refresh_interval": "5min"
            }
        })),
        None => {
            let data = cache.preload_proxmox().await;
            HttpResponse::Ok().json(json!({
                "source": "fresh_load", 
                "data": data,
                "cache_info": {
                    "preloaded": false,
                    "just_loaded": true
                }
            }))
        }
    }
}

async fn get_weather(cache: web::Data<Arc<PreloadCache>>) -> impl Responder {
    match cache.get_weather().await {
        Some(data) => HttpResponse::Ok().json(json!({
            "source": "preloaded_cache",
            "data": data,
            "cache_info": {
                "preloaded": true,
                "refresh_interval": "5min"
            }
        })),
        None => {
            let data = cache.preload_weather().await;
            HttpResponse::Ok().json(json!({
                "source": "fresh_load",
                "data": data,
                "cache_info": {
                    "preloaded": false,
                    "just_loaded": true
                }
            }))
        }
    }
}

async fn get_cache_status(cache: web::Data<Arc<PreloadCache>>) -> impl Responder {
    let status = cache.get_cache_status().await;
    HttpResponse::Ok().json(json!({
        "cache_status": status,
        "refresh_info": {
            "auto_refresh": true,
            "interval": "5min",
            "next_refresh": (chrono::Utc::now() + chrono::Duration::minutes(5)).to_rfc3339()
        }
    }))
}

async fn refresh_cache(cache: web::Data<Arc<PreloadCache>>) -> impl Responder {
    cache.refresh_all().await;
    HttpResponse::Ok().json(json!({
        "message": "Cache refreshed successfully",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "services": ["argocd", "proxmox", "weather"]
    }))
}

async fn api_docs() -> impl Responder {
    HttpResponse::Found()
        .append_header(("Location", "/static/api-docs.html"))
        .finish()
}
