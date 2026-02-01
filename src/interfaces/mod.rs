//! Interface layer - Delivery mechanisms
//!
//! This layer contains:
//! - HTTP handlers (REST API)
//! - WebSocket handlers
//! - Middleware
//!
//! # Principles
//!
//! - Depends on application layer
//! - Handles HTTP concerns (headers, status codes, serialization)
//! - No business logic

pub mod http;
pub mod websocket;
pub mod middleware;

pub use http::*;
