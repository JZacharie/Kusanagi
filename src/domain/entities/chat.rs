//! Chat Entities
//!
//! Domain entities for chat operations.

use serde::{Deserialize, Serialize};

/// Chat message request
#[derive(Clone, Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub language: Option<String>,
}

/// Chat response
#[derive(Clone, Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub response_type: ResponseType,
    pub data: Option<serde_json::Value>,
}

/// Response type
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ResponseType {
    Text,
    Command,
    Ai,
    Error,
    Help,
}

impl std::fmt::Display for ResponseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResponseType::Text => write!(f, "text"),
            ResponseType::Command => write!(f, "command"),
            ResponseType::Ai => write!(f, "ai"),
            ResponseType::Error => write!(f, "error"),
            ResponseType::Help => write!(f, "help"),
        }
    }
}

impl Default for ResponseType {
    fn default() -> Self {
        ResponseType::Text
    }
}

/// Chat message stored in history
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub user_message: String,
    pub ai_response: String,
    pub response_type: ResponseType,
    pub timestamp: String,
}

/// Available commands
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChatCommand {
    Help,
    Status,
    Nodes,
    Pods,
    Events,
    ArgoCD,
    Backups,
    Namespaces,
    PVCs,
    K8s,
    Cilium,
    Trivy,
    Query(String),
    Unknown(String),
}

impl From<&str> for ChatCommand {
    fn from(cmd: &str) -> Self {
        let cmd_lower = cmd.to_lowercase();
        match cmd_lower.as_str() {
            "/help" => ChatCommand::Help,
            "/status" => ChatCommand::Status,
            "/nodes" => ChatCommand::Nodes,
            "/pods" => ChatCommand::Pods,
            "/events" => ChatCommand::Events,
            "/argocd" => ChatCommand::ArgoCD,
            "/backups" => ChatCommand::Backups,
            "/namespaces" => ChatCommand::Namespaces,
            "/pvcs" => ChatCommand::PVCs,
            "/k8s" => ChatCommand::K8s,
            "/cilium" => ChatCommand::Cilium,
            "/trivy" => ChatCommand::Trivy,
            cmd if cmd.starts_with("/query ") => {
                ChatCommand::Query(cmd.strip_prefix("/query ").unwrap_or("").to_string())
            }
            _ => ChatCommand::Unknown(cmd.to_string()),
        }
    }
}
