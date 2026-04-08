// Re-exports des handlers core
pub mod core;
pub use core::*;

// Shared types for handlers
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct RefreshQuery {
    pub refresh: Option<bool>,
}

// Re-exports des handlers K8s
pub mod k8s;
pub use k8s::*;

// Re-exports des handlers monitoring
pub mod monitoring;
pub use monitoring::*;

// Re-exports des handlers business
pub mod business;
pub use business::*;
