//! Proxmox HTTP handlers

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::domain::services::proxmox_service;
use crate::state::AppState;

/// Get Proxmox VMs
pub async fn get_vms_handler(State(state): State<AppState>) -> impl IntoResponse {
    match proxmox_service::get_proxmox_vms(&state.http_client).await {
        Ok(vms) => Json(json!({
            "status": "success",
            "data": vms
        }))
        .into_response(),
        Err(e) => Json(json!({
            "status": "error",
            "message": e
        }))
        .into_response(),
    }
}

/// Get Proxmox containers (LXC)
pub async fn get_containers_handler(State(state): State<AppState>) -> impl IntoResponse {
    match proxmox_service::get_proxmox_containers(&state.http_client).await {
        Ok(containers) => Json(json!({
            "status": "success",
            "data": containers
        }))
        .into_response(),
        Err(e) => Json(json!({
            "status": "error",
            "message": e
        }))
        .into_response(),
    }
}

/// Get Proxmox nodes
pub async fn get_nodes_handler(State(state): State<AppState>) -> impl IntoResponse {
    match proxmox_service::get_proxmox_nodes(&state.http_client).await {
        Ok(nodes) => Json(json!({
            "status": "success",
            "data": nodes
        }))
        .into_response(),
        Err(e) => Json(json!({
            "status": "error",
            "message": e
        }))
        .into_response(),
    }
}

/// Control VM (start/stop/reboot/etc)
pub async fn control_vm_handler(
    State(state): State<AppState>,
    Path((server, node, vmid, action)): Path<(String, String, u64, String)>,
) -> impl IntoResponse {
    match proxmox_service::vm_control(&state.http_client, &server, &node, vmid, &action).await {
        Ok(result) => Json(json!({
            "status": "success",
            "data": result
        }))
        .into_response(),
        Err(e) => Json(json!({
            "status": "error",
            "message": e
        }))
        .into_response(),
    }
}

/// Control Container (start/stop/reboot/etc)
pub async fn control_ct_handler(
    State(state): State<AppState>,
    Path((server, node, vmid, action)): Path<(String, String, u64, String)>,
) -> impl IntoResponse {
    match proxmox_service::ct_control(&state.http_client, &server, &node, vmid, &action).await {
        Ok(result) => Json(json!({
            "status": "success",
            "data": result
        }))
        .into_response(),
        Err(e) => Json(json!({
            "status": "error",
            "message": e
        }))
        .into_response(),
    }
}
