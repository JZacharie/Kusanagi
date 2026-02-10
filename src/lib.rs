// Kusanagi Library
// Re-exports for use in other crates

// Hexagonal Layers
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;
pub mod state;

// Legacy & Support modules
pub mod advanced_cache;
pub mod cache;
pub mod config;
pub mod error;
pub mod event_bus;
pub mod handlers;
pub mod init;
pub mod legacy;
pub mod perf_monitor;
pub mod routes;
pub mod utils;

// Re-exports
pub use advanced_cache::AdvancedCache;
pub use cache::{Cache, CacheStats};
pub use config::Config;
pub use error::{KusanagiError, Result};
