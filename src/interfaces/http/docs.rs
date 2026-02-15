use crate::interfaces::http::handlers::business::chat;
use crate::interfaces::http::handlers::core::{health, system};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        system::system_status,
        system::system_logs,
        chat::post_chat_handler,
        crate::interfaces::http::handlers::k8s::force_delete_pod_handler,
        crate::interfaces::http::handlers::monitoring::cilium::get_flows_handler,
        crate::interfaces::http::handlers::monitoring::cilium::get_matrix_handler,
        crate::interfaces::http::handlers::monitoring::cilium::get_metrics_handler,
        crate::interfaces::http::handlers::monitoring::cilium::get_anomalies_handler,
        crate::interfaces::http::handlers::monitoring::cilium::get_namespaces_handler,
    ),
    components(
        schemas(
            crate::domain::services::system_service::SystemStatus,
            chat::ChatRequest,
            chat::ChatResponse,
            crate::interfaces::http::handlers::k8s::DeletePodRequest,
        )
    ),
    tags(
        (name = "kusanagi", description = "Kusanagi API"),
        (name = "kubernetes", description = "Kubernetes Operations"),
        (name = "cilium", description = "Cilium Network Monitoring")
    )
)]
pub struct ApiDoc;
