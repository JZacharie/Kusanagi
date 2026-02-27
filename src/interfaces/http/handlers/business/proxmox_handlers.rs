//! Proxmox HTTP handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;

use crate::domain::services::proxmox_service;
use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;

/// Get Proxmox VMs
#[utoipa::path(
    get,
    path = "/api/proxmox/vms",
    responses(
        (status = 200, description = "VMs retrieved successfully"),
        (status = 500, description = "Failed to retrieve VMs")
    ),
    tag = "proxmox"
)]
pub async fn get_vms_handler(State(state): State<AppState>) -> impl IntoResponse {
    match proxmox_service::get_proxmox_vms(&state.http_client).await {
        Ok(vms) => api_success(json!(vms)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Get Proxmox containers (LXC)
#[utoipa::path(
    get,
    path = "/api/proxmox/containers",
    responses(
        (status = 200, description = "Containers retrieved successfully"),
        (status = 500, description = "Failed to retrieve containers")
    ),
    tag = "proxmox"
)]
pub async fn get_containers_handler(State(state): State<AppState>) -> impl IntoResponse {
    match proxmox_service::get_proxmox_containers(&state.http_client).await {
        Ok(containers) => api_success(json!(containers)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Get Proxmox nodes
#[utoipa::path(
    get,
    path = "/api/proxmox/nodes",
    responses(
        (status = 200, description = "Nodes retrieved successfully"),
        (status = 500, description = "Failed to retrieve nodes")
    ),
    tag = "proxmox"
)]
pub async fn get_nodes_handler(State(state): State<AppState>) -> impl IntoResponse {
    match proxmox_service::get_proxmox_nodes(&state.http_client).await {
        Ok(nodes) => api_success(json!(nodes)),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Control VM (start/stop/reboot/etc)
#[utoipa::path(
    post,
    path = "/api/proxmox/vms/{server}/{node}/{vmid}/{action}",
    params(
        ("server" = String, Path, description = "Proxmox server"),
        ("node" = String, Path, description = "Node name"),
        ("vmid" = u64, Path, description = "VM ID"),
        ("action" = String, Path, description = "Action: start, stop, reboot")
    ),
    responses(
        (status = 200, description = "VM action executed successfully"),
        (status = 500, description = "Failed to execute VM action")
    ),
    tag = "proxmox"
)]
pub async fn control_vm_handler(
    State(state): State<AppState>,
    Path((server, node, vmid, action)): Path<(String, String, u64, String)>,
) -> impl IntoResponse {
    match proxmox_service::vm_control(&state.http_client, &server, &node, vmid, &action).await {
        Ok(result) => api_success(json!({"message": result})),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Control Container (start/stop/reboot/etc)
#[utoipa::path(
    post,
    path = "/api/proxmox/containers/{server}/{node}/{vmid}/{action}",
    params(
        ("server" = String, Path, description = "Proxmox server"),
        ("node" = String, Path, description = "Node name"),
        ("vmid" = u64, Path, description = "Container ID"),
        ("action" = String, Path, description = "Action: start, stop, reboot")
    ),
    responses(
        (status = 200, description = "Container action executed successfully"),
        (status = 500, description = "Failed to execute container action")
    ),
    tag = "proxmox"
)]
pub async fn control_ct_handler(
    State(state): State<AppState>,
    Path((server, node, vmid, action)): Path<(String, String, u64, String)>,
) -> impl IntoResponse {
    match proxmox_service::ct_control(&state.http_client, &server, &node, vmid, &action).await {
        Ok(result) => api_success(json!({"message": result})),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Delete Proxmox volume
#[utoipa::path(
    delete,
    path = "/api/proxmox/volume/{server}/{node}/{storage}/{volume}",
    params(
        ("server" = String, Path, description = "Proxmox server URL (or name)"),
        ("node" = String, Path, description = "Proxmox node name"),
        ("storage" = String, Path, description = "Storage name"),
        ("volume" = String, Path, description = "Volume ID/Name")
    ),
    responses(
        (status = 200, description = "Volume deleted successfully"),
        (status = 500, description = "Failed to delete volume")
    ),
    tag = "proxmox"
)]
pub async fn delete_volume_handler(
    State(state): State<AppState>,
    Path((server, node, storage, volume)): Path<(String, String, String, String)>,
) -> impl IntoResponse {
    match proxmox_service::delete_proxmox_volume(&state.http_client, &server, &node, &storage, &volume).await {
        Ok(result) => api_success(json!({"message": result})),
        Err(e) => api_error(StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}
