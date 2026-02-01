//! Application layer - Use cases
//!
//! This layer contains:
//! - Use cases: Application-specific business rules
//! - DTOs: Data Transfer Objects for input/output
//! - Mappers: Conversions between domain and DTOs
//!
//! # Principles
//!
//! - Orchestrates domain services
//! - Handles transactions and coordination
//! - Maps between DTOs and domain entities
//! - No business logic (that's in domain)

pub mod dtos;
pub mod mappers;
pub mod use_cases;

// Re-export main use cases
pub use use_cases::*;
