// HTTP Controllers - Interface Layer
// Migrated from Actix-web to Axum

pub mod alert_handlers;
pub mod backup_handlers;
pub mod chat;
pub mod cloudflare_handlers;
pub mod homeassistant_handlers;
pub mod proxmox_compose_handlers;
pub mod proxmox_handlers;
pub mod security_handlers;
pub mod weather_handlers;

// Re-export handler functions
pub use alert_handlers::get_alerts_handler;
pub use cloudflare_handlers::get_cloudflare_analytics_handler;

pub use backup_handlers::{get_backups_handler, trigger_backup_handler};
pub use chat::post_chat_handler;
pub use homeassistant_handlers::{
    get_automations_handler, get_devices_handler, get_sensors_handler,
};
pub use proxmox_compose_handlers::deploy_compose_handler;
pub use proxmox_handlers::{
    get_containers_handler, get_nodes_handler, get_vms_handler, get_zfs_handler,
};
pub use security_handlers::{
    get_security_handler, get_security_report_handler, get_security_reports_handler,
    get_vulnerabilities_handler, post_security_scan_handler,
};
pub use weather_handlers::get_weather_handler;
