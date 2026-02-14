// HTTP Controllers - Interface Layer
// Migrated from Actix-web to Axum

pub mod handlers;
pub mod helpers;
pub mod routes;

// Legacy individual handler files (will be removed once fully migrated to handlers/)
pub mod alert_handlers;
pub mod backup_handlers;

pub mod homeassistant_handlers;
pub mod proxmox_handlers;
pub mod security_handlers;
pub mod weather_handlers;

// Re-export handler functions for backward compatibility
pub use alert_handlers::get_alerts_handler;
pub use backup_handlers::{get_backups_handler, trigger_backup_handler};
pub use homeassistant_handlers::{
    get_automations_handler, get_devices_handler, get_sensors_handler,
};
pub use proxmox_handlers::{get_containers_handler, get_nodes_handler, get_vms_handler};
pub use security_handlers::{
    get_security_handler, get_security_report_handler, get_security_reports_handler,
    get_vulnerabilities_handler,
};
pub use weather_handlers::get_weather_handler;
