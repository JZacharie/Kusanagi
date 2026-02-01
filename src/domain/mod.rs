//! Domain layer - Core business logic
//!
//! This layer contains:
//! - Entities: Domain objects with business rules
//! - Ports: Interfaces for external dependencies (driven and driving)
//! - Services: Domain services that orchestrate entities
//!
//! # Principles
//!
//! - No dependencies on external frameworks
//! - No dependencies on infrastructure
//! - Pure business logic

pub mod entities;
pub mod ports;
pub mod services;

// Re-export common types
pub use entities::*;
pub use ports::*;
pub use services::*;
