//! Chat Port
//!
//! Port defining the interface for chat operations.

use async_trait::async_trait;
use crate::domain::entities::{ChatRequest, ChatResponse, ChatMessage, ChatCommand};

/// Port for chat operations
#[async_trait]
pub trait ChatService: Send + Sync {
    /// Process a chat message
    async fn process_message(&self, request: ChatRequest) -> Result<ChatResponse, String>;
    
    /// Handle a command
    async fn handle_command(&self, command: ChatCommand) -> Result<ChatResponse, String>;
    
    /// Query AI with natural language
    async fn query_ai(&self, query: &str, context: &str, language: &str) -> Result<String, String>;
}

/// Port for chat history storage
#[async_trait]
pub trait ChatHistoryRepository: Send + Sync {
    /// Store a chat message
    async fn store_message(&self, user_msg: &str, ai_response: &str, response_type: &str) -> Result<(), String>;
    
    /// Get chat history
    async fn get_history(&self, limit: usize) -> Result<Vec<ChatMessage>, String>;
    
    /// Clear chat history
    async fn clear_history(&self) -> Result<(), String>;
}

/// Port for AI provider (Ollama, OpenAI, etc.)
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Generate response from prompt
    async fn generate(&self, prompt: &str, model: Option<&str>) -> Result<String, String>;
    
    /// Check if AI is available
    async fn health_check(&self) -> Result<bool, String>;
}
