// HTTP Controllers - Interface Layer
// Migrated from Actix-web to Axum

pub mod alert_handlers;
pub mod backup_handlers;
pub mod homeassistant_handlers;
pub mod security_handlers;
pub mod weather_handlers;

// Re-export handlers (temporary - will be restored when all handlers are migrated)
// pub use alert_handlers::{configure_alert_routes, create_alerts_use_case};
// pub use backup_handlers::{configure_backup_routes, create_backup_use_case};
// pub use homeassistant_handlers::{configure_ha_routes, create_homeassistant_use_case};
// pub use security_handlers::{configure_security_routes, create_security_use_case};
// pub use weather_handlers::{configure_weather_routes, create_weather_use_case};
