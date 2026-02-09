// Legacy module - Extended working version
//
// NOTE: The following modules have been refactored to hexagonal architecture:
// - 'weather' -> Use crate::interfaces::http::weather_handlers
// - 'alertmanager' -> Use crate::interfaces::http::alert_handlers
// - 'backups' -> Use crate::interfaces::http::backup_handlers
// - 'security' -> Use crate::interfaces::http::security_handlers
// - 'homeassistant' -> Use crate::interfaces::http::homeassistant_handlers

pub mod argocd;
pub mod cluster;
pub mod events;
pub mod health;
pub mod ingress;
pub mod nodes;
pub mod pods;
pub mod prometheus;
pub mod services;
pub mod storage;

// Re-exports
pub use argocd::*;
pub use cluster::*;
pub use events::*;
pub use health::*;
pub use ingress::*;
pub use nodes::*;
pub use pods::*;
pub use prometheus::*;
pub use services::*;
pub use storage::*;
