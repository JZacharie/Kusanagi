// Legacy module - Minimal working version
pub mod cluster;
pub mod nodes;
pub mod pods;
pub mod argocd;
pub mod prometheus;

// Re-exports
pub use cluster::*;
pub use nodes::*;
pub use pods::*;
pub use argocd::*;
pub use prometheus::*;
