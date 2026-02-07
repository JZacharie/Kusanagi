use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CacheStats {
    pub entries: usize,
    pub hits: u64,
    pub misses: u64,
}

#[async_trait::async_trait]
pub trait Cache: Send + Sync {
    async fn get(&self, key: &str) -> Option<String>;
    async fn set(&self, key: &str, value: String);
    async fn delete(&self, key: &str);
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
impl Cache for InMemoryCache {
    async fn get(&self, key: &str) -> Option<String> {
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

    async fn set(&self, key: &str, value: String) {
        let mut data = self.data.write().await;
        data.insert(key.to_string(), value);

        let mut stats = self.stats.write().await;
        stats.entries = data.len();
    }

    async fn delete(&self, key: &str) {
        let mut data = self.data.write().await;
        data.remove(key);

        let mut stats = self.stats.write().await;
        stats.entries = data.len();
    }

    async fn stats(&self) -> CacheStats {
        self.stats.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = InMemoryCache::new();
        cache.set("key1", "value1".to_string()).await;
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = InMemoryCache::new();
        assert_eq!(cache.get("nonexistent").await, None);
    }

    #[tokio::test]
    async fn test_cache_delete() {
        let cache = InMemoryCache::new();
        cache.set("key1", "value1".to_string()).await;
        cache.delete("key1").await;
        assert_eq!(cache.get("key1").await, None);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = InMemoryCache::new();
        cache.set("key1", "value1".to_string()).await;
        cache.get("key1").await;
        cache.get("missing").await;

        let stats = cache.stats().await;
        assert_eq!(stats.entries, 1);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }
}
