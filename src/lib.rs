// Hexagonal Architecture - Core Modules + Legacy
pub mod cache;
pub mod advanced_cache;
pub mod config;
pub mod error;
pub mod event_bus;
pub mod legacy;

// Hexagonal Layers
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;

// Re-exports
pub use cache::{Cache, InMemoryCache, CacheStats};
pub use advanced_cache::AdvancedCache;
pub use config::Config;
pub use error::{KusanagiError, Result};
