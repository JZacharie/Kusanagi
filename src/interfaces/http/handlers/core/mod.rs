//! HTTP request handlers - Core system handlers

pub mod cache;
pub mod config;
pub mod database;
pub mod docs;
pub mod doctor;
pub mod health;
pub mod llm;
pub mod mcp;

pub mod metrics;
pub mod prometheus;
pub mod slack;
pub mod system;
pub mod websocket;

pub use cache::*;
pub use config::*;
pub use database::*;
pub use docs::*;
pub use doctor::*;
pub use health::*;
pub use llm::*;
pub use mcp::*;
pub use metrics::metrics_handler as core_metrics_handler;
pub use prometheus::*;
pub use slack::*;
pub use system::*;
pub use websocket::*;
