// Complete lib.rs - Core modules only
pub mod application;
pub mod cache;
pub mod config;
pub mod domain;
pub mod error;
pub mod event_bus;
pub mod features;
pub mod infrastructure;
pub mod interfaces;
pub mod jobs;
pub mod metrics;
pub mod middleware;
pub mod resilience;
pub mod response;
pub mod slack;
pub mod validation;

// Re-exports for public API
pub use cache::{Cache, InMemoryCache, CacheStats};
pub use config::Config;
pub use error::{KusanagiError, Result};
pub use features::*;
pub use response::ApiResponse;
pub use validation::{ValidationErrorResponse, FieldError};
