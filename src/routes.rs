use actix_web::web;

pub fn configure_api_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Health & Info
        .route(
            "/health",
            web::get().to(crate::handlers::health::health_check),
        )
        .route("/api", web::get().to(crate::handlers::health::service_info))
        // System
        .route(
            "/api/system/status",
            web::get().to(crate::handlers::system::system_status),
        )
        .route(
            "/api/system/logs",
            web::get().to(crate::handlers::system::system_logs),
        )
        // Cache
        .route(
            "/api/cache/stats",
            web::get().to(crate::handlers::cache::cache_stats),
        );
}

pub fn configure_k8s_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/api/k8s/cluster",
        web::get().to(crate::handlers::k8s::cluster_overview),
    )
    .route(
        "/api/k8s/nodes",
        web::get().to(crate::handlers::k8s::nodes_status),
    )
    .route(
        "/api/k8s/pods",
        web::get().to(crate::handlers::k8s::pods_status),
    );
}

pub fn configure_monitoring_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/api/alerts",
        web::get().to(crate::handlers::monitoring::alerts),
    )
    .route(
        "/api/monitoring/quotas",
        web::get().to(crate::handlers::monitoring::quotas),
    );
}
