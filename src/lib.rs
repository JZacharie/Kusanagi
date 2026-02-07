// Hexagonal Architecture - Core Modules + Legacy
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

// Hexagonal Layers
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;

// Re-exports
pub use advanced_cache::AdvancedCache;
pub use cache::{Cache, CacheStats, InMemoryCache};
pub use config::Config;
pub use error::{KusanagiError, Result};
