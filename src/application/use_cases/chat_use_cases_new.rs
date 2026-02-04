use crate::domain::ports::{ChatRepository, AiService};
use crate::domain::ports::chat_repository::ChatMessage;
use crate::error::Result;
use std::sync::Arc;

pub struct ProcessChatUseCase {
    chat_repo: Arc<dyn ChatRepository>,
    ai_service: Arc<dyn AiService>,
}

impl ProcessChatUseCase {
    pub fn new(chat_repo: Arc<dyn ChatRepository>, ai_service: Arc<dyn AiService>) -> Self {
        Self { chat_repo, ai_service }
    }

    pub async fn execute(&self, message: &str) -> Result<String> {
        let response = self.ai_service.query(message, None).await?;
        self.chat_repo.store_message(message, &response).await?;
        Ok(response)
    }
}

pub struct GetChatHistoryUseCase {
    chat_repo: Arc<dyn ChatRepository>,
}

impl GetChatHistoryUseCase {
    pub fn new(chat_repo: Arc<dyn ChatRepository>) -> Self {
        Self { chat_repo }
    }

    pub async fn execute(&self, limit: Option<usize>) -> Result<Vec<ChatMessage>> {
        self.chat_repo.get_history(limit).await
    }
}
