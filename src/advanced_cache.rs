use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};

#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

#[derive(Clone)]
pub struct AdvancedCache<T: Clone + Send + Sync + 'static> {
    data: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    default_ttl: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStats {
    pub entries: usize,
    pub expired: usize,
    pub memory_bytes: usize,
}

impl<T: Clone + Send + Sync + 'static> AdvancedCache<T> {
    pub fn new(default_ttl: Duration) -> Self {
        let cache = Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        };
        
        // Start cleanup task
        let cache_clone = cache.clone();
        tokio::spawn(async move {
            cache_clone.cleanup_loop().await;
        });
        
        cache
    }
    
    pub async fn get(&self, key: &str) -> Option<T> {
        let cache = self.data.read().await;
        if let Some(entry) = cache.get(key) {
            if entry.expires_at > Instant::now() {
                return Some(entry.value.clone());
            }
        }
        None
    }
    
    pub async fn set(&self, key: String, value: T, ttl: Option<Duration>) {
        let mut cache = self.data.write().await;
        let expires_at = Instant::now() + ttl.unwrap_or(self.default_ttl);
        cache.insert(key, CacheEntry { value, expires_at });
    }
    
    pub async fn delete(&self, key: &str) {
        let mut cache = self.data.write().await;
        cache.remove(key);
    }
    
    pub async fn clear(&self) {
        let mut cache = self.data.write().await;
        cache.clear();
    }
    
    pub async fn stats(&self) -> CacheStats {
        let cache = self.data.read().await;
        let now = Instant::now();
        
        let expired = cache.values()
            .filter(|entry| entry.expires_at <= now)
            .count();
        
        CacheStats {
            entries: cache.len(),
            expired,
            memory_bytes: cache.len() * std::mem::size_of::<CacheEntry<T>>(),
        }
    }
    
    async fn cleanup_expired(&self) -> usize {
        let mut cache = self.data.write().await;
        let now = Instant::now();
        let before = cache.len();
        cache.retain(|_, entry| entry.expires_at > now);
        before - cache.len()
    }
    
    async fn cleanup_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let removed = self.cleanup_expired().await;
            if removed > 0 {
                tracing::debug!("Cache cleanup: removed {} expired entries", removed);
            }
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Default for AdvancedCache<T> {
    fn default() -> Self {
        Self::new(Duration::from_secs(300)) // 5 minutes default TTL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_ttl() {
        let cache = AdvancedCache::new(Duration::from_millis(100));
        
        cache.set("key1".to_string(), "value1".to_string(), None).await;
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));
        
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(cache.get("key1").await, None);
    }

    #[tokio::test]
    async fn test_cache_custom_ttl() {
        let cache = AdvancedCache::new(Duration::from_secs(300));
        
        cache.set("key1".to_string(), "value1".to_string(), Some(Duration::from_millis(100))).await;
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));
        
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(cache.get("key1").await, None);
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let cache = AdvancedCache::new(Duration::from_millis(50));
        
        cache.set("key1".to_string(), "value1".to_string(), None).await;
        cache.set("key2".to_string(), "value2".to_string(), None).await;
        
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        let removed = cache.cleanup_expired().await;
        assert_eq!(removed, 2);
        
        let stats = cache.stats().await;
        assert_eq!(stats.entries, 0);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = AdvancedCache::new(Duration::from_secs(300));
        
        cache.set("key1".to_string(), "value1".to_string(), None).await;
        cache.set("key2".to_string(), "value2".to_string(), None).await;
        
        let stats = cache.stats().await;
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.expired, 0);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = AdvancedCache::new(Duration::from_secs(300));
        
        cache.set("key1".to_string(), "value1".to_string(), None).await;
        cache.set("key2".to_string(), "value2".to_string(), None).await;
        
        cache.clear().await;
        
        let stats = cache.stats().await;
        assert_eq!(stats.entries, 0);
    }
}
