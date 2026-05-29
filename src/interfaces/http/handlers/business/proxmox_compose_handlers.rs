use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;

use crate::domain::services::proxmox_service;
use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct DeployComposeInput {
    pub yaml: String,
    pub target_node: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct DeployComposeResponse {
    pub results: Vec<ServiceDeployResult>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ServiceDeployResult {
    pub service_name: String,
    pub status: String,
    pub message: String,
}

/// Deploy a Docker Compose stack to Proxmox LXC
#[utoipa::path(
    post,
    path = "/api/proxmox/deploy-compose",
    request_body = DeployComposeInput,
    responses(
        (status = 200, description = "Compose stack deployment initiated"),
        (status = 400, description = "Invalid YAML or missing data"),
        (status = 500, description = "Deployment failed")
    ),
    tag = "proxmox"
)]
pub async fn deploy_compose_handler(
    State(state): State<AppState>,
    Json(input): Json<DeployComposeInput>,
) -> impl IntoResponse {
    info!("🚀 Deploying compose stack to Proxmox...");

    match proxmox_service::deploy_docker_compose_to_proxmox(
        &state.http_client,
        &input.yaml,
        input.target_node.as_deref(),
    )
    .await
    {
        Ok(results) => api_success(json!({ "results": results })),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}
