use actix_web::{get, post, web, HttpResponse, Responder};
use crate::application::use_cases::{nodes_use_cases::*, pods_use_cases::*};
use crate::infrastructure::repositories::K8sRepository;
use crate::error::KusanagiError;
use std::sync::Arc;

#[get("/api/nodes")]
async fn list_nodes() -> impl Responder {
    let k8s_repo = Arc::new(K8sRepository::new(
        kube::Client::try_default().await.unwrap()
    ));
    let use_case = GetNodesUseCase::new(k8s_repo);
    
    match use_case.execute().await {
        Ok(nodes) => HttpResponse::Ok().json(nodes),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[get("/api/nodes/{name}")]
async fn get_node_details(path: web::Path<String>) -> impl Responder {
    let name = path.into_inner();
    let k8s_repo = Arc::new(K8sRepository::new(
        kube::Client::try_default().await.unwrap()
    ));
    let use_case = GetNodeDetailsUseCase::new(k8s_repo);
    
    match use_case.execute(&name).await {
        Ok(node) => HttpResponse::Ok().json(node),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[get("/api/pods")]
async fn list_pods(query: web::Query<serde_json::Value>) -> impl Responder {
    let namespace = query.get("namespace").and_then(|v| v.as_str());
    let k8s_repo = Arc::new(K8sRepository::new(
        kube::Client::try_default().await.unwrap()
    ));
    let use_case = GetPodsUseCase::new(k8s_repo);
    
    match use_case.execute(namespace).await {
        Ok(pods) => HttpResponse::Ok().json(pods),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[post("/api/pods/{namespace}/{name}/scale")]
async fn scale_pod(
    path: web::Path<(String, String)>,
    body: web::Json<serde_json::Value>
) -> impl Responder {
    let (namespace, name) = path.into_inner();
    let replicas = body.get("replicas").and_then(|r| r.as_i64()).unwrap_or(1) as i32;
    let resource_type = body.get("type").and_then(|t| t.as_str()).unwrap_or("deployment");
    
    let k8s_repo = Arc::new(K8sRepository::new(
        kube::Client::try_default().await.unwrap()
    ));
    let use_case = ScalePodUseCase::new(k8s_repo);
    
    let result = match resource_type {
        "statefulset" => use_case.scale_statefulset(&namespace, &name, replicas).await,
        _ => use_case.scale_deployment(&namespace, &name, replicas).await,
    };
    
    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "scaled"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[post("/api/pods/{namespace}/{name}/delete")]
async fn delete_pod(path: web::Path<(String, String)>) -> impl Responder {
    let (namespace, name) = path.into_inner();
    let k8s_repo = Arc::new(K8sRepository::new(
        kube::Client::try_default().await.unwrap()
    ));
    let use_case = DeletePodUseCase::new(k8s_repo);
    
    match use_case.delete(&namespace, &name).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "deleted"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

#[post("/api/pods/{namespace}/{name}/force-delete")]
async fn force_delete_pod(path: web::Path<(String, String)>) -> impl Responder {
    let (namespace, name) = path.into_inner();
    let k8s_repo = Arc::new(K8sRepository::new(
        kube::Client::try_default().await.unwrap()
    ));
    let use_case = DeletePodUseCase::new(k8s_repo);
    
    match use_case.force_delete(&namespace, &name).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "force_deleted"})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))
    }
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(list_nodes)
        .service(get_node_details)
        .service(list_pods)
        .service(scale_pod)
        .service(delete_pod)
        .service(force_delete_pod);
}
