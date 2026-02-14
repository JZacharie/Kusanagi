use crate::domain::services::chat_service::ChatService;
use std::sync::Arc;

pub struct ChatUseCase {
    chat_service: Arc<ChatService>,
}

impl ChatUseCase {
    pub fn new(
        // we keep repositories in signature for now to avoid breaking state.rs if I missed something,
        // but actually state.rs passes them. I should match state.rs signature I just wrote.
        // Wait, I wrote state.rs to pass cluster_repo, alert_repo, chat_service.
        // So I must accept them, but I can ignore them.
        _cluster_repository: Arc<dyn crate::domain::ports::ClusterRepository>,
        _alert_repository: Arc<dyn crate::domain::ports::AlertRepository>,
        chat_service: Arc<ChatService>,
    ) -> Self {
        Self { chat_service }
    }

    pub async fn execute(&self, message: &str, language: &str) -> String {
        // ChatService returns ChatResponse, but this method returns String.
        // We need to adapt it. The previous implementation returned String (response text).
        // The handler `post_chat_handler` calls this and wraps it in `ChatResponse` struct from `chat_handlers.rs`.
        // Wait, `chat_handlers.rs` defines its own `ChatResponse` which has only `response: String`.
        // My `ChatService` returns `domain::entities::chat::ChatResponse` which has more fields.
        // I should probably update `chat_handlers.rs` to use `domain::entities::ChatResponse` too,
        // to expose the full richness (data, response_type).
        // But for now, to keep changes scoped, I will just return the text response from `ChatService`.

        let response = self.chat_service.process_message(message, language).await;

        response.response
    }
}
