use crate::interfaces::http::handlers::business::{
    alert_handlers, backup_handlers, homeassistant_handlers, proxmox_handlers, security_handlers,
    weather_handlers,
};
use crate::interfaces::http::handlers::core::{health, system};
use crate::interfaces::http::handlers::k8s;
use crate::interfaces::http::handlers::monitoring::{cilium, mqtt};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        // Core routes
        health::health_check,
        system::system_status,
        system::system_logs,
        // Chat
        crate::interfaces::http::handlers::business::chat::post_chat_handler,
        // Cilium
        cilium::get_flows_handler,
        cilium::get_matrix_handler,
        cilium::get_metrics_handler,
        cilium::get_anomalies_handler,
        cilium::get_namespaces_handler,
        // MQTT
        mqtt::get_mqtt_devices_handler,
        mqtt::get_mqtt_messages_handler,
        // Kubernetes
        k8s::cluster_overview,
        k8s::nodes_status,
        k8s::nodes_debug,
        k8s::pods_status,
        k8s::storage,
        k8s::ingress,
        k8s::services,
        k8s::argocd_status,
        k8s::pod_logs,
        k8s::delete_error_pods_handler,
        k8s::argocd_sync,
        k8s::force_delete_pod_handler,
        // Weather
        weather_handlers::get_weather_handler,
        // Alerts
        alert_handlers::get_alerts_handler,
        // Backups
        backup_handlers::get_backups_handler,
        backup_handlers::trigger_backup_handler,
        // Home Assistant
        homeassistant_handlers::get_sensors_handler,
        homeassistant_handlers::get_devices_handler,
        homeassistant_handlers::get_automations_handler,
        // Security
        security_handlers::get_security_handler,
        security_handlers::get_security_reports_handler,
        security_handlers::get_vulnerabilities_handler,
        security_handlers::get_security_report_handler,
        // Proxmox
        proxmox_handlers::get_vms_handler,
        proxmox_handlers::get_containers_handler,
        proxmox_handlers::get_nodes_handler,
        proxmox_handlers::control_vm_handler,
        proxmox_handlers::control_ct_handler,
        proxmox_compose_handlers::deploy_compose_handler,
    ),
    components(
        schemas(
            crate::domain::services::system_service::SystemStatus,
            crate::interfaces::http::handlers::business::chat::ChatRequest,
            crate::interfaces::http::handlers::business::chat::ChatResponse,
            k8s::DeletePodRequest,
            k8s::SyncAppRequest,
            crate::domain::services::mqtt_service::MqttDevice,
            crate::domain::services::mqtt_service::MqttMessage,
            security_handlers::ReportPath,
            crate::interfaces::http::handlers::business::proxmox_compose_handlers::DeployComposeInput,
            crate::interfaces::http::handlers::business::proxmox_compose_handlers::ServiceDeployResult,
        )
    ),
    tags(
        (name = "core", description = "Core API - Health & System"),
        (name = "chat", description = "AI Chat"),
        (name = "cilium", description = "Cilium Network Monitoring"),
        (name = "mqtt", description = "MQTT Monitoring"),
        (name = "kubernetes", description = "Kubernetes Operations"),
        (name = "weather", description = "Weather Information"),
        (name = "alerts", description = "Alert Management"),
        (name = "backups", description = "Backup Operations"),
        (name = "homeassistant", description = "Home Assistant Integration"),
        (name = "security", description = "Security Reports & Vulnerabilities"),
        (name = "proxmox", description = "Proxmox VE Management"),
    )
)]
pub struct ApiDoc;
