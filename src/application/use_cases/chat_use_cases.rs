//! Chat Use Cases
//!
//! Application layer use cases for chat operations.

use crate::domain::entities::{ChatRequest, ChatResponse, ChatMessage, ChatCommand};
use crate::domain::ports::{ChatService, ChatHistoryRepository};
use crate::error::{KusanagiError, Result};
use std::sync::Arc;

/// Process chat message use case
pub struct ProcessChatMessageUseCase {
    chat_service: Arc<dyn ChatService>,
    history_repo: Arc<dyn ChatHistoryRepository>,
}

impl ProcessChatMessageUseCase {
    pub fn new(chat_service: Arc<dyn ChatService>, history_repo: Arc<dyn ChatHistoryRepository>) -> Self {
        Self { chat_service, history_repo }
    }

    pub async fn execute(&self, request: ChatRequest) -> Result<ChatResponse> {
        // Process the message
        let response = self.chat_service.process_message(request).await
            .map_err(|e| KusanagiError::internal(format!("Failed to process message: {}", e)))?;
        
        // Store in history
        let _ = self.history_repo.store_message("user", &response.response, &response.response_type.to_string()).await;
        
        Ok(response)
    }
}

/// Handle chat command use case
pub struct HandleChatCommandUseCase {
    chat_service: Arc<dyn ChatService>,
}

impl HandleChatCommandUseCase {
    pub fn new(chat_service: Arc<dyn ChatService>) -> Self {
        Self { chat_service }
    }

    pub async fn execute(&self, command: ChatCommand) -> Result<ChatResponse> {
        self.chat_service.handle_command(command).await
            .map_err(|e| KusanagiError::internal(format!("Failed to handle command: {}", e)))
    }
}

/// Query AI use case
pub struct QueryAiUseCase {
    chat_service: Arc<dyn ChatService>,
}

impl QueryAiUseCase {
    pub fn new(chat_service: Arc<dyn ChatService>) -> Self {
        Self { chat_service }
    }

    pub async fn execute(&self, query: &str, context: &str, language: &str) -> Result<String> {
        self.chat_service.query_ai(query, context, language).await
            .map_err(|e| KusanagiError::internal(format!("AI query failed: {}", e)))
    }
}

/// Get chat history use case
pub struct GetChatHistoryUseCase {
    history_repo: Arc<dyn ChatHistoryRepository>,
}

impl GetChatHistoryUseCase {
    pub fn new(history_repo: Arc<dyn ChatHistoryRepository>) -> Self {
        Self { history_repo }
    }

    pub async fn execute(&self, limit: usize) -> Result<Vec<ChatMessage>> {
        self.history_repo.get_history(limit).await
            .map_err(|e| KusanagiError::internal(format!("Failed to get chat history: {}", e)))
    }
}

/// Clear chat history use case
pub struct ClearChatHistoryUseCase {
    history_repo: Arc<dyn ChatHistoryRepository>,
}

impl ClearChatHistoryUseCase {
    pub fn new(history_repo: Arc<dyn ChatHistoryRepository>) -> Self {
        Self { history_repo }
    }

    pub async fn execute(&self) -> Result<()> {
        self.history_repo.clear_history().await
            .map_err(|e| KusanagiError::internal(format!("Failed to clear chat history: {}", e)))
    }
}

/// Chat service - aggregates all chat use cases
pub struct ChatUseCaseService {
    pub process_message: ProcessChatMessageUseCase,
    pub handle_command: HandleChatCommandUseCase,
    pub query_ai: QueryAiUseCase,
    pub get_history: GetChatHistoryUseCase,
    pub clear_history: ClearChatHistoryUseCase,
}

impl ChatUseCaseService {
    pub fn new(
        chat_service: Arc<dyn ChatService>,
        history_repo: Arc<dyn ChatHistoryRepository>,
    ) -> Self {
        Self {
            process_message: ProcessChatMessageUseCase::new(chat_service.clone(), history_repo.clone()),
            handle_command: HandleChatCommandUseCase::new(chat_service.clone()),
            query_ai: QueryAiUseCase::new(chat_service.clone()),
            get_history: GetChatHistoryUseCase::new(history_repo.clone()),
            clear_history: ClearChatHistoryUseCase::new(history_repo),
        }
    }
}
