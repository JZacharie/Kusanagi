// Legacy module - Extended working version
pub mod alertmanager;
pub mod argocd;
pub mod cluster;
pub mod events;
pub mod health;
pub mod ingress;
pub mod nodes;
pub mod pods;
pub mod prometheus;
pub mod services;
pub mod storage;
pub mod weather;

// Re-exports
pub use alertmanager::*;
pub use argocd::*;
pub use cluster::*;
pub use events::*;
pub use health::*;
pub use ingress::*;
pub use nodes::*;
pub use pods::*;
pub use prometheus::*;
pub use services::*;
pub use storage::*;
pub use weather::*;
