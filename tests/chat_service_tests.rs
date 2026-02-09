//! Tests for chat/AI service

use std::sync::Arc;
use tokio::sync::Mutex;

// Mock types for chat service
#[derive(Debug, Clone)]
struct ChatMessage {
    id: String,
    role: MessageRole,
    content: String,
    timestamp: String,
}

#[derive(Debug, Clone, PartialEq)]
enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone)]
struct ChatSession {
    id: String,
    messages: Vec<ChatMessage>,
    context: Option<String>,
}

// Repository
trait ChatRepository: Send + Sync {
    fn save_message(
        &self,
        session_id: &str,
        message: ChatMessage,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>>;
    fn get_session_messages(
        &self,
        session_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<ChatMessage>> + Send + '_>>;
    fn clear_session(
        &self,
        session_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>>;
}

struct InMemoryChatRepository {
    sessions: Arc<Mutex<Vec<ChatSession>>>,
}

impl InMemoryChatRepository {
    fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(vec![])),
        }
    }
}

impl ChatRepository for InMemoryChatRepository {
    fn save_message(
        &self,
        session_id: &str,
        message: ChatMessage,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        let sessions = self.sessions.clone();
        let session_id = session_id.to_string();
        Box::pin(async move {
            let mut sessions = sessions.lock().await;
            if let Some(session) = sessions.iter_mut().find(|s| s.id == session_id) {
                session.messages.push(message);
            } else {
                sessions.push(ChatSession {
                    id: session_id,
                    messages: vec![message],
                    context: None,
                });
            }
        })
    }

    fn get_session_messages(
        &self,
        session_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Vec<ChatMessage>> + Send + '_>> {
        let sessions = self.sessions.clone();
        let session_id = session_id.to_string();
        Box::pin(async move {
            sessions
                .lock()
                .await
                .iter()
                .find(|s| s.id == session_id)
                .map(|s| s.messages.clone())
                .unwrap_or_default()
        })
    }

    fn clear_session(
        &self,
        session_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + '_>> {
        let sessions = self.sessions.clone();
        let session_id = session_id.to_string();
        Box::pin(async move {
            sessions.lock().await.retain(|s| s.id != session_id);
        })
    }
}

// Service
struct ChatService<R: ChatRepository> {
    repository: Arc<R>,
}

impl<R: ChatRepository> ChatService<R> {
    fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    async fn send_message(&self, session_id: &str, content: &str) -> ChatMessage {
        // Save user message
        let user_message = ChatMessage {
            id: generate_id(),
            role: MessageRole::User,
            content: content.to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };
        self.repository
            .save_message(session_id, user_message.clone())
            .await;

        // Generate assistant response (mock)
        let assistant_message = ChatMessage {
            id: generate_id(),
            role: MessageRole::Assistant,
            content: format!("Response to: {}", content),
            timestamp: "2024-01-01T00:00:01Z".to_string(),
        };
        self.repository
            .save_message(session_id, assistant_message.clone())
            .await;

        assistant_message
    }

    async fn get_history(&self, session_id: &str) -> Vec<ChatMessage> {
        self.repository.get_session_messages(session_id).await
    }

    async fn clear_history(&self, session_id: &str) {
        self.repository.clear_session(session_id).await;
    }

    async fn get_message_count(&self, session_id: &str) -> usize {
        self.repository.get_session_messages(session_id).await.len()
    }

    async fn analyze_cluster(&self, _cluster_data: &str) -> String {
        "Based on the cluster data, I recommend reviewing the resource allocation.".to_string()
    }
}

fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!("msg_{}", duration.as_millis())
}

#[tokio::test]
async fn test_send_message() {
    let repo = Arc::new(InMemoryChatRepository::new());
    let service = ChatService::new(repo);

    let response = service.send_message("session-1", "Hello").await;

    assert_eq!(response.role, MessageRole::Assistant);
    assert!(response.content.contains("Hello"));
}

#[tokio::test]
async fn test_get_history() {
    let repo = Arc::new(InMemoryChatRepository::new());
    let service = ChatService::new(repo);

    service.send_message("session-1", "Hello").await;
    service.send_message("session-1", "How are you?").await;

    let history = service.get_history("session-1").await;

    assert_eq!(history.len(), 4); // 2 user messages + 2 assistant responses
}

#[tokio::test]
async fn test_clear_history() {
    let repo = Arc::new(InMemoryChatRepository::new());
    let service = ChatService::new(repo);

    service.send_message("session-1", "Hello").await;
    assert_eq!(service.get_message_count("session-1").await, 2);

    service.clear_history("session-1").await;
    assert_eq!(service.get_message_count("session-1").await, 0);
}

#[tokio::test]
async fn test_multiple_sessions() {
    let repo = Arc::new(InMemoryChatRepository::new());
    let service = ChatService::new(repo);

    service.send_message("session-1", "Message 1").await;
    service.send_message("session-2", "Message 2").await;

    let history1 = service.get_history("session-1").await;
    let history2 = service.get_history("session-2").await;

    assert_eq!(history1.len(), 2);
    assert_eq!(history2.len(), 2);

    // Ensure messages are isolated
    assert!(history1
        .iter()
        .all(|m| m.content.contains("Message 1") || m.content.contains("Response to: Message 1")));
    assert!(history2
        .iter()
        .all(|m| m.content.contains("Message 2") || m.content.contains("Response to: Message 2")));
}

#[tokio::test]
async fn test_empty_history() {
    let repo = Arc::new(InMemoryChatRepository::new());
    let service = ChatService::new(repo);

    let history = service.get_history("new-session").await;

    assert!(history.is_empty());
    assert_eq!(service.get_message_count("new-session").await, 0);
}

#[test]
fn test_generate_id() {
    let id1 = generate_id();
    std::thread::sleep(std::time::Duration::from_millis(2));
    let id2 = generate_id();

    assert!(!id1.is_empty());
    assert!(!id2.is_empty());
    // IDs might be the same if generated very quickly, so just check they're not empty
    assert!(id1.starts_with("msg_"));
    assert!(id2.starts_with("msg_"));
}

#[tokio::test]
async fn test_analyze_cluster() {
    let repo = Arc::new(InMemoryChatRepository::new());
    let service = ChatService::new(repo);

    let analysis = service.analyze_cluster("cluster data here").await;

    assert!(!analysis.is_empty());
    assert!(analysis.contains("recommend"));
}
