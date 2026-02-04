use async_trait::async_trait;
use crate::error::Result;

#[async_trait]
pub trait ChatRepository: Send + Sync {
    async fn store_message(&self, message: &str, response: &str) -> Result<()>;
    async fn get_history(&self, limit: Option<usize>) -> Result<Vec<ChatMessage>>;
    async fn clear_history(&self) -> Result<()>;
}

#[async_trait]
pub trait AiService: Send + Sync {
    async fn query(&self, prompt: &str, context: Option<&str>) -> Result<String>;
    async fn health_check(&self) -> Result<bool>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub message: String,
    pub response: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
