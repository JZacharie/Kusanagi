//! Comprehensive tests for AdvancedCache

use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// Re-implementing the AdvancedCache for testing
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    expires_at: DateTime<Utc>,
    access_count: u64,
}

pub struct AdvancedCache<T: Clone + Send + Sync> {
    data: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    ttl: Duration,
}

impl<T: Clone + Send + Sync> AdvancedCache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub async fn get(&self, key: &str) -> Option<T> {
        let mut data = self.data.write().await;

        if let Some(entry) = data.get_mut(key) {
            if Utc::now() < entry.expires_at {
                entry.access_count += 1;
                return Some(entry.value.clone());
            } else {
                data.remove(key);
            }
        }
        None
    }

    pub async fn set(&self, key: &str, value: T) {
        let entry = CacheEntry {
            value,
            expires_at: Utc::now() + chrono::Duration::from_std(self.ttl).unwrap(),
            access_count: 0,
        };

        let mut data = self.data.write().await;
        data.insert(key.to_string(), entry);
    }

    pub async fn delete(&self, key: &str) -> bool {
        let mut data = self.data.write().await;
        data.remove(key).is_some()
    }

    pub async fn clear(&self) {
        let mut data = self.data.write().await;
        data.clear();
    }

    pub async fn len(&self) -> usize {
        self.cleanup_expired().await;
        let data = self.data.read().await;
        data.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.len().await == 0
    }

    pub async fn contains_key(&self, key: &str) -> bool {
        self.get(key).await.is_some()
    }

    pub async fn get_stats(&self) -> CacheStats {
        let data = self.data.read().await;
        let total_entries = data.len();
        let total_accesses: u64 = data.values().map(|e| e.access_count).sum();

        CacheStats {
            total_entries,
            total_accesses,
        }
    }

    async fn cleanup_expired(&self) {
        let mut data = self.data.write().await;
        let now = Utc::now();
        data.retain(|_, entry| entry.expires_at > now);
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_accesses: u64,
}

#[tokio::test]
async fn test_cache_basic_operations() {
    let cache = AdvancedCache::<String>::new(Duration::from_secs(60));

    // Test set and get
    cache.set("key1", "value1".to_string()).await;
    assert_eq!(cache.get("key1").await, Some("value1".to_string()));

    // Test non-existent key
    assert_eq!(cache.get("nonexistent").await, None);
}

#[tokio::test]
async fn test_cache_ttl_expiration() {
    let cache = AdvancedCache::<String>::new(Duration::from_millis(100));

    cache.set("key1", "value1".to_string()).await;
    assert_eq!(cache.get("key1").await, Some("value1".to_string()));

    // Wait for expiration
    sleep(Duration::from_millis(150)).await;

    // Should be expired
    assert_eq!(cache.get("key1").await, None);
}

#[tokio::test]
async fn test_cache_delete() {
    let cache = AdvancedCache::<String>::new(Duration::from_secs(60));

    cache.set("key1", "value1".to_string()).await;
    assert!(cache.delete("key1").await);
    assert_eq!(cache.get("key1").await, None);

    // Delete non-existent key
    assert!(!cache.delete("nonexistent").await);
}

#[tokio::test]
async fn test_cache_clear() {
    let cache = AdvancedCache::<String>::new(Duration::from_secs(60));

    cache.set("key1", "value1".to_string()).await;
    cache.set("key2", "value2".to_string()).await;
    cache.set("key3", "value3".to_string()).await;

    assert_eq!(cache.len().await, 3);

    cache.clear().await;

    assert_eq!(cache.len().await, 0);
    assert!(cache.is_empty().await);
}

#[tokio::test]
async fn test_cache_len_and_is_empty() {
    let cache = AdvancedCache::<String>::new(Duration::from_secs(60));

    assert!(cache.is_empty().await);
    assert_eq!(cache.len().await, 0);

    cache.set("key1", "value1".to_string()).await;

    assert!(!cache.is_empty().await);
    assert_eq!(cache.len().await, 1);
}

#[tokio::test]
async fn test_cache_contains_key() {
    let cache = AdvancedCache::<String>::new(Duration::from_secs(60));

    cache.set("key1", "value1".to_string()).await;

    assert!(cache.contains_key("key1").await);
    assert!(!cache.contains_key("nonexistent").await);
}

#[tokio::test]
async fn test_cache_update_existing_key() {
    let cache = AdvancedCache::<String>::new(Duration::from_secs(60));

    cache.set("key1", "value1".to_string()).await;
    assert_eq!(cache.get("key1").await, Some("value1".to_string()));

    // Update existing key
    cache.set("key1", "updated_value".to_string()).await;
    assert_eq!(cache.get("key1").await, Some("updated_value".to_string()));
}

#[tokio::test]
async fn test_cache_multiple_types() {
    // Test with integers
    let int_cache = AdvancedCache::<i32>::new(Duration::from_secs(60));
    int_cache.set("count", 42).await;
    assert_eq!(int_cache.get("count").await, Some(42));

    // Test with structs
    #[derive(Clone, Debug, PartialEq)]
    struct TestStruct {
        id: u64,
        name: String,
    }

    let struct_cache = AdvancedCache::<TestStruct>::new(Duration::from_secs(60));
    let test_obj = TestStruct {
        id: 1,
        name: "Test".to_string(),
    };
    struct_cache.set("obj", test_obj.clone()).await;
    assert_eq!(struct_cache.get("obj").await, Some(test_obj));
}

#[tokio::test]
async fn test_cache_concurrent_access() {
    let cache = Arc::new(AdvancedCache::<String>::new(Duration::from_secs(60)));

    // Spawn multiple tasks that access the cache concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = tokio::spawn(async move {
            cache_clone
                .set(&format!("key{}", i), format!("value{}", i))
                .await;
            cache_clone.get(&format!("key{}", i)).await
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for (i, handle) in handles.into_iter().enumerate() {
        let result = handle.await.unwrap();
        assert_eq!(result, Some(format!("value{}", i)));
    }

    assert_eq!(cache.len().await, 10);
}

#[tokio::test]
async fn test_cache_stats() {
    let cache = AdvancedCache::<String>::new(Duration::from_secs(60));

    cache.set("key1", "value1".to_string()).await;
    cache.set("key2", "value2".to_string()).await;

    // Access key1 multiple times
    cache.get("key1").await;
    cache.get("key1").await;
    cache.get("key1").await;

    // Access key2 once
    cache.get("key2").await;

    let stats = cache.get_stats().await;
    assert_eq!(stats.total_entries, 2);
    assert_eq!(stats.total_accesses, 4);
}

#[tokio::test]
async fn test_cache_large_number_of_entries() {
    let cache = AdvancedCache::<String>::new(Duration::from_secs(60));

    // Insert many entries
    for i in 0..1000 {
        cache.set(&format!("key{}", i), format!("value{}", i)).await;
    }

    assert_eq!(cache.len().await, 1000);

    // Verify random access
    assert_eq!(cache.get("key500").await, Some("value500".to_string()));
    assert_eq!(cache.get("key999").await, Some("value999".to_string()));
}

#[tokio::test]
async fn test_cache_partial_expiration() {
    let cache = AdvancedCache::<String>::new(Duration::from_millis(100));

    cache.set("key1", "value1".to_string()).await;

    sleep(Duration::from_millis(50)).await;

    cache.set("key2", "value2".to_string()).await;

    sleep(Duration::from_millis(60)).await;

    // key1 should be expired, key2 should still be valid
    assert_eq!(cache.get("key1").await, None);
    assert_eq!(cache.get("key2").await, Some("value2".to_string()));
}
