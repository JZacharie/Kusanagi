// Legacy module - Extended working version
pub mod cluster;
pub mod nodes;
pub mod pods;
pub mod argocd;
pub mod prometheus;
pub mod events;
pub mod services;
pub mod storage;
pub mod ingress;
pub mod health;
pub mod alertmanager;

// Re-exports
pub use cluster::*;
pub use nodes::*;
pub use pods::*;
pub use argocd::*;
pub use prometheus::*;
pub use events::*;
pub use services::*;
pub use storage::*;
pub use ingress::*;
pub use health::*;
pub use alertmanager::*;
