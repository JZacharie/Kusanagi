use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait::async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: &str, value: String);
    async fn delete(&self, key: &str);
}

pub struct InMemoryCache {
    data: Arc<RwLock<HashMap<String, String>>>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl Cache for InMemoryCache {
    async fn get(&self, key: &str) -> Option<String> {
        let data = self.data.read().await;
        data.get(key).cloned()
    }

    async fn set(&self, key: &str, value: String) {
        let mut data = self.data.write().await;
        data.insert(key.to_string(), value);
    }

    async fn delete(&self, key: &str) {
        let mut data = self.data.write().await;
        data.remove(key);
    }
}
