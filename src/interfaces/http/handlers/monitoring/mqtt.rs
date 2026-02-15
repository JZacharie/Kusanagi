//! MQTT HTTP Handlers
//! Axum handlers for MQTT device and message monitoring

use crate::domain::services::mqtt_service::{MqttDevice, MqttMessage};
use crate::interfaces::http::response::api_success;
use crate::state::AppState;
use axum::{extract::State, response::Response};

/// Get detected MQTT devices
#[utoipa::path(
    get,
    path = "/api/mqtt/devices",
    responses(
        (status = 200, description = "List of MQTT devices", body = Vec<MqttDevice>),
    ),
    tag = "monitoring"
)]
pub async fn get_mqtt_devices_handler(State(state): State<AppState>) -> Response {
    api_success(state.mqtt_state.get_devices())
}

/// Get recent MQTT messages
#[utoipa::path(
    get,
    path = "/api/mqtt/messages",
    responses(
        (status = 200, description = "List of recent MQTT messages", body = Vec<MqttMessage>),
    ),
    tag = "monitoring"
)]
pub async fn get_mqtt_messages_handler(State(state): State<AppState>) -> Response {
    api_success(state.mqtt_state.get_messages())
}
