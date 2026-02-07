use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use std::sync::Arc;

pub async fn cache_stats(
    k8s_cache: web::Data<Arc<crate::AdvancedCache<String>>>,
    argocd_cache: web::Data<Arc<crate::AdvancedCache<String>>>,
    general_cache: web::Data<Arc<crate::AdvancedCache<String>>>,
) -> impl Responder {
    let k8s = k8s_cache.stats().await;
    let argocd = argocd_cache.stats().await;
    let general = general_cache.stats().await;

    HttpResponse::Ok().json(json!({
        "k8s": {
            "entries": k8s.entries,
            "expired": k8s.expired,
            "memory_bytes": k8s.memory_bytes,
            "ttl_seconds": 60
        },
        "argocd": {
            "entries": argocd.entries,
            "expired": argocd.expired,
            "memory_bytes": argocd.memory_bytes,
            "ttl_seconds": 600
        },
        "general": {
            "entries": general.entries,
            "expired": general.expired,
            "memory_bytes": general.memory_bytes,
            "ttl_seconds": 120
        },
        "total": {
            "entries": k8s.entries + argocd.entries + general.entries,
            "expired": k8s.expired + argocd.expired + general.expired,
            "memory_bytes": k8s.memory_bytes + argocd.memory_bytes + general.memory_bytes
        }
    }))
}
