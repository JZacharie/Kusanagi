use async_trait::async_trait;
use crate::domain::ports::{ChatRepository, AiService};
use crate::domain::ports::chat_repository::ChatMessage;
use crate::error::{Result, KusanagiError};
// use crate::legacy; // Disabled for core version

pub struct LegacyChatRepository;

#[async_trait]
impl ChatRepository for LegacyChatRepository {
    async fn store_message(&self, _message: &str, _response: &str) -> Result<()> {
        // Simplified implementation
        Ok(())
    }

    async fn get_history(&self, _limit: Option<usize>) -> Result<Vec<ChatMessage>> {
        Ok(vec![])
    }

    async fn clear_history(&self) -> Result<()> {
        Ok(())
    }
}

pub struct LegacyAiService;

#[async_trait]
impl AiService for LegacyAiService {
    async fn query(&self, prompt: &str, _context: Option<&str>) -> Result<String> {
        Ok(format!("AI response to: {}", prompt))
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(true) // Simplified
    }
}
