//! Proxmox HTTP handlers

use axum::{
    extract::{Path, State},
    response::Response,
};
use serde_json::json;

use crate::domain::services::proxmox_service;
use crate::interfaces::http::response::{api_error, api_success};
use crate::state::AppState;

/// Get Proxmox VMs
pub async fn get_vms_handler(State(state): State<AppState>) -> Response {
    match proxmox_service::get_proxmox_vms(&state.http_client).await {
        Ok(vms) => api_success(json!(vms)),
        Err(e) => api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Get Proxmox containers (LXC)
pub async fn get_containers_handler(State(state): State<AppState>) -> Response {
    match proxmox_service::get_proxmox_containers(&state.http_client).await {
        Ok(containers) => api_success(json!(containers)),
        Err(e) => api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Get Proxmox nodes
pub async fn get_nodes_handler(State(state): State<AppState>) -> Response {
    match proxmox_service::get_proxmox_nodes(&state.http_client).await {
        Ok(nodes) => api_success(json!(nodes)),
        Err(e) => api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Control VM (start/stop/reboot/etc)
pub async fn control_vm_handler(
    State(state): State<AppState>,
    Path((server, node, vmid, action)): Path<(String, String, u64, String)>,
) -> Response {
    match proxmox_service::vm_control(&state.http_client, &server, &node, vmid, &action).await {
        Ok(result) => api_success(json!({
            "message": "Action executed successfully",
            "result": result
        })),
        Err(e) => api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}

/// Control Container (start/stop/reboot/etc)
pub async fn control_ct_handler(
    State(state): State<AppState>,
    Path((server, node, vmid, action)): Path<(String, String, u64, String)>,
) -> Response {
    match proxmox_service::ct_control(&state.http_client, &server, &node, vmid, &action).await {
        Ok(result) => api_success(json!({
            "message": "Action executed successfully",
            "result": result
        })),
        Err(e) => api_error(axum::http::StatusCode::INTERNAL_SERVER_ERROR, e),
    }
}
