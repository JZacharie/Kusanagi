// Minimal lib.rs for compilation
pub mod cache;
pub mod config;
pub mod error;
pub mod features;
pub mod response;
pub mod validation;

// Selective re-exports to avoid conflicts
pub use cache::{Cache, InMemoryCache};
pub use config::Config;
pub use error::KusanagiError;
pub use features::*;
pub use response::ApiResponse;
pub use validation::{ValidationErrorResponse, FieldError};
