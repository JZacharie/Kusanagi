//! Infrastructure layer - External implementations
//!
//! This layer contains concrete implementations of the ports defined in the domain layer.
//! It depends on external frameworks and libraries.

pub mod repositories;
pub mod clients;

pub use repositories::*;

