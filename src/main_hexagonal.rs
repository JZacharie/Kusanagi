use std::sync::Arc;
use tokio::signal;

// Modules
mod config_simple as config;
mod error;

// Domain layer
mod domain {
    pub mod entities_simple;
}

// Application layer  
mod application {
    pub mod use_cases_simple;
}

// Infrastructure layer
mod infrastructure {
    pub mod repositories {
        pub mod k8s_repository_simple;
    }
}

// Interface layer
mod interfaces {
    pub mod http_simple;
}

use crate::config::Config;
use crate::error::Result;
use crate::infrastructure::repositories::k8s_repository_simple::K8sRepository;
use crate::application::use_cases_simple::GetClusterOverviewUseCase;
use crate::interfaces::http_simple::configure_routes;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Kusanagi starting (Hexagonal Architecture)...");
    
    // Initialize configuration
    let config = Config::load()?;
    println!("✅ Configuration loaded");
    
    // Detect environment
    let is_k8s = std::env::var("KUBERNETES_SERVICE_HOST").is_ok();
    if is_k8s {
        println!("☸️  Running in Kubernetes mode");
    } else {
        println!("🏠 Running in local mode - services will be mocked");
    }
    
    // Initialize repositories
    let k8s_repo = Arc::new(K8sRepository::new().await?);
    
    // Initialize use cases
    let cluster_overview_use_case = GetClusterOverviewUseCase::new(k8s_repo.clone());
    
    // Setup HTTP server
    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(actix_web::web::Data::new(cluster_overview_use_case.clone()))
            .configure(configure_routes)
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?;
    
    println!("🌐 Server starting on {}:{}", config.server.host, config.server.port);
    println!("📋 Available endpoints:");
    println!("   - GET /              : Service info");
    println!("   - GET /health        : Health check");
    println!("   - GET /api/cluster/overview : Cluster overview");
    
    // Start server with graceful shutdown
    let server_handle = server.run();
    
    tokio::select! {
        _ = server_handle => {
            println!("Server stopped");
        }
        _ = signal::ctrl_c() => {
            println!("🛑 Received shutdown signal, stopping gracefully...");
        }
    }
    
    println!("👋 Kusanagi stopped");
    Ok(())
}
