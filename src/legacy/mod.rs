// Legacy module - Extended working version
//
// NOTE: The following modules have been REMOVED (refactored to hexagonal architecture):
// - 'weather' -> Use crate::interfaces::http::weather_handlers
// - 'alertmanager' -> Use crate::interfaces::http::alert_handlers
// - 'backups' -> Use crate::interfaces::http::backup_handlers
// - 'security' -> Use crate::interfaces::http::security_handlers
// - 'homeassistant' -> Use crate::interfaces::http::homeassistant_handlers
// - 'proxmox' -> Use crate::interfaces::http::proxmox_handlers

pub mod health;
pub mod prometheus;

// Re-exports
pub use health::*;
pub use prometheus::*;
