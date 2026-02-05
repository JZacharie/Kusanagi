use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub entries: usize,
    pub hits: u64,
    pub misses: u64,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            entries: 0,
            hits: 0,
            misses: 0,
        }
    }
}

#[async_trait::async_trait]
pub trait Cache<K = String, V = String>: Send + Sync {
    async fn get(&self, key: &K) -> Option<V>;
    async fn set(&self, key: K, value: V);
    async fn delete(&self, key: &K);
    async fn stats(&self) -> CacheStats;
}

#[derive(Default)]
pub struct InMemoryCache {
    data: Arc<RwLock<HashMap<String, String>>>,
    stats: Arc<RwLock<CacheStats>>,
}

impl InMemoryCache {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl Cache<String, String> for InMemoryCache {
    async fn get(&self, key: &String) -> Option<String> {
        let data = self.data.read().await;
        let result = data.get(key).cloned();
        
        let mut stats = self.stats.write().await;
        if result.is_some() {
            stats.hits += 1;
        } else {
            stats.misses += 1;
        }
        
        result
    }

    async fn set(&self, key: String, value: String) {
        let mut data = self.data.write().await;
        data.insert(key, value);
        
        let mut stats = self.stats.write().await;
        stats.entries = data.len();
    }

    async fn delete(&self, key: &String) {
        let mut data = self.data.write().await;
        data.remove(key);
        
        let mut stats = self.stats.write().await;
        stats.entries = data.len();
    }
    
    async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
}
