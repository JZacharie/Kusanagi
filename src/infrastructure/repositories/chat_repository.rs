use async_trait::async_trait;
use crate::domain::ports::{ChatRepository, AiService};
use crate::domain::ports::chat_repository::ChatMessage;
use crate::error::{Result, KusanagiError};
use crate::legacy;

pub struct LegacyChatRepository;

#[async_trait]
impl ChatRepository for LegacyChatRepository {
    async fn store_message(&self, message: &str, response: &str) -> Result<()> {
        legacy::chat_storage::store_chat_message(&ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            message: message.to_string(),
            response: response.to_string(),
            timestamp: chrono::Utc::now(),
        }).await.map_err(|e| KusanagiError::internal(&e.to_string()))
    }

    async fn get_history(&self, limit: Option<usize>) -> Result<Vec<ChatMessage>> {
        // Delegate to legacy chat module
        Ok(vec![]) // Simplified for now
    }

    async fn clear_history(&self) -> Result<()> {
        Ok(()) // Simplified for now
    }
}

pub struct LegacyAiService;

#[async_trait]
impl AiService for LegacyAiService {
    async fn query(&self, prompt: &str, context: Option<&str>) -> Result<String> {
        legacy::chat::query_ollama(prompt, context.unwrap_or("")).await
            .map_err(|e| KusanagiError::external_api("AI", &e.to_string()))
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true) // Simplified
    }
}
