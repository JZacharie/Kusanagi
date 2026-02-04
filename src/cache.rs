//! Unified caching system for Kusanagi
//!
//! This module provides a generic, type-safe caching layer with TTL support.
//! It replaces the scattered cache implementations across modules with a
//! centralized, configurable solution.
//!
//! # Features
//!
//! - **Generic**: Works with any `Clone + Send + Sync` type
//! - **TTL Support**: Automatic expiration of cached entries

use crate::error::KusanagiError;
// - **Multiple Backends**: In-memory (current) and Redis (future)
// - **Metrics**: Cache hit/miss statistics
// - **Configurable**: TTL values from config file
//
// # Example
//
// ```rust
// use crate::cache::{Cache, InMemoryCache};
// use std::time::Duration;
//
// let cache = InMemoryCache::new();
// cache.set("key", "value", Duration::from_secs(60)).await;
//
// if let Some(value) = cache.get(&"key").await {
//     println!("Cached: {}", value);
// }
// ```

use crate::config;
use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, trace};

/// Trait for cache implementations
///
/// This trait defines the interface for all cache backends.
/// It is generic over the key and value types.
#[async_trait]
pub trait Cache<K, V>: Send + Sync
where
    K: Clone + Eq + Hash + Send + Sync,
    V: Clone + Send + Sync,
{
    /// Get a value from the cache
    ///
    /// Returns `Some(value)` if the key exists and hasn't expired,
    /// `None` otherwise.
    async fn get(&self, key: &K) -> Option<V>;

    /// Set a value in the cache with TTL
    ///
    /// The value will expire after the specified duration.
    async fn set(&self, key: K, value: V, ttl: Duration);

    /// Set a value in the cache without TTL (never expires)
    async fn set_permanent(&self, key: K, value: V);

    /// Remove a value from the cache
    async fn remove(&self, key: &K);

    /// Check if a key exists in the cache (and hasn't expired)
    async fn contains(&self, key: &K) -> bool;

    /// Clear all entries from the cache
    async fn clear(&self);

    /// Get cache statistics
    async fn stats(&self) -> CacheStats;

    /// Get the number of entries in the cache
    async fn len(&self) -> usize;

    /// Check if the cache is empty
    async fn is_empty(&self) -> bool;
}

/// Cache statistics
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of cache hits
    pub hits: u64,
    /// Total number of cache misses
    pub misses: u64,
    /// Number of entries currently in cache
    pub entries: usize,
    /// Number of expired entries removed
    pub evicted: u64,
}

impl CacheStats {
    /// Calculate hit rate (0.0 to 1.0)
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Calculate miss rate (0.0 to 1.0)
    pub fn miss_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.misses as f64 / total as f64
        }
    }

    /// Total number of requests
    pub fn total_requests(&self) -> u64 {
        self.hits + self.misses
    }
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    /// The cached value
    value: V,
    /// When this entry was created
    created_at: Instant,
    /// Time-to-live (None = permanent)
    ttl: Option<Duration>,
    /// Number of times this entry was accessed (for future analytics)
    _access_count: u64,
}

impl<V> CacheEntry<V> {
    /// Create a new cache entry with TTL
    fn new(value: V, ttl: Option<Duration>) -> Self {
        Self {
            value,
            created_at: Instant::now(),
            ttl,
            _access_count: 0,
        }
    }

    /// Check if this entry has expired
    fn is_expired(&self) -> bool {
        match self.ttl {
            Some(ttl) => self.created_at.elapsed() > ttl,
            None => false, // Permanent entries never expire
        }
    }

    /// Get the remaining TTL (for future use)
    #[allow(dead_code)]
    fn remaining_ttl(&self) -> Option<Duration> {
        self.ttl.map(|ttl| {
            let elapsed = self.created_at.elapsed();
            if elapsed > ttl {
                Duration::from_secs(0)
            } else {
                ttl - elapsed
            }
        })
    }

    /// Record an access (for future analytics)
    #[allow(dead_code)]
    fn record_access(&mut self) {
        self._access_count += 1;
    }
}

/// In-memory cache implementation
///
/// This is the default cache implementation using a HashMap with RwLock.
/// It provides O(1) lookups and automatic expiration.
pub struct InMemoryCache<K, V> {
    /// The cache storage
    store: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
    /// Default TTL for entries
    default_ttl: Duration,
    /// Cache name (for logging/metrics)
    name: String,
}

impl<K, V> InMemoryCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + Debug,
    V: Clone + Send + Sync + Debug,
{
    /// Create a new in-memory cache
    pub fn new() -> Self {
        Self::with_name("unnamed")
    }

    /// Create a new cache with a name
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            default_ttl: Duration::from_secs(300), // 5 minutes default
            name: name.into(),
        }
    }

    /// Create a cache with a specific default TTL
    pub fn with_ttl(name: impl Into<String>, ttl: Duration) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            default_ttl: ttl,
            name: name.into(),
        }
    }

    /// Create a cache from configuration
    pub fn from_config(name: impl Into<String>, ttl_secs: u64) -> Self {
        Self::with_ttl(name, Duration::from_secs(ttl_secs))
    }

    /// Get the cache name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the default TTL
    pub fn default_ttl(&self) -> Duration {
        self.default_ttl
    }

    /// Set the default TTL
    pub fn set_default_ttl(&mut self, ttl: Duration) {
        self.default_ttl = ttl;
    }

    /// Perform cleanup of expired entries
    ///
    /// This removes all expired entries from the cache.
    /// It's called automatically on reads, but can be called manually
    /// for proactive cleanup.
    pub async fn cleanup(&self) -> usize {
        let mut store = self.store.write().await;
        let before = store.len();
        store.retain(|key, entry| {
            let expired = entry.is_expired();
            if expired {
                trace!(cache = %self.name, ?key, "Removing expired entry");
            }
            !expired
        });
        let removed = before - store.len();
        
        if removed > 0 {
            debug!(cache = %self.name, removed, "Cleaned up expired entries");
            let mut stats = self.stats.write().await;
            stats.evicted += removed as u64;
        }
        
        removed
    }

    /// Get or insert a value
    ///
    /// If the key exists and hasn't expired, returns the cached value.
    /// Otherwise, calls the provided function to compute the value,
    /// caches it, and returns it.
    pub async fn get_or_insert<F, Fut>(&self, key: K, f: F) -> Result<V>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<V>>,
    {
        // Try to get from cache first
        if let Some(value) = self.get(&key).await {
            return Ok(value);
        }

        // Compute the value
        let value = f().await?;

        // Cache it
        self.set(key, value.clone(), self.default_ttl).await;

        Ok(value)
    }

    /// Get or insert with custom TTL
    pub async fn get_or_insert_with_ttl<F, Fut>(
        &self,
        key: K,
        ttl: Duration,
        f: F,
    ) -> Result<V>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<V>>,
    {
        if let Some(value) = self.get(&key).await {
            return Ok(value);
        }

        let value = f().await?;
        self.set(key, value.clone(), ttl).await;

        Ok(value)
    }
}

impl<K, V> Default for InMemoryCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + Debug,
    V: Clone + Send + Sync + Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for InMemoryCache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + Debug,
    V: Clone + Send + Sync + Debug,
{
    async fn get(&self, key: &K) -> Option<V> {
        // First, try to get without write lock
        {
            let store = self.store.read().await;
            if let Some(entry) = store.get(key) {
                if !entry.is_expired() {
                    // Cache hit
                    let mut stats = self.stats.write().await;
                    stats.hits += 1;
                    drop(stats); // Release lock early

                    // Clone the value while still holding read lock
                    let value = entry.value.clone();
                    trace!(cache = %self.name, ?key, "Cache hit");
                    return Some(value);
                }
            }
        }

        // Check if we need to clean up an expired entry
        let mut store = self.store.write().await;
        if let Some(entry) = store.get(key) {
            if entry.is_expired() {
                store.remove(key);
                debug!(cache = %self.name, ?key, "Removed expired entry on read");
            }
        }
        drop(store);

        // Cache miss
        let mut stats = self.stats.write().await;
        stats.misses += 1;
        stats.entries = self.store.read().await.len();
        trace!(cache = %self.name, ?key, "Cache miss");

        None
    }

    async fn set(&self, key: K, value: V, ttl: Duration) {
        let entry = CacheEntry::new(value, Some(ttl));
        let mut store = self.store.write().await;
        store.insert(key, entry);
        let entries = store.len();
        drop(store);

        let mut stats = self.stats.write().await;
        stats.entries = entries;
        trace!(cache = %self.name, "Entry cached with TTL");
    }

    async fn set_permanent(&self, key: K, value: V) {
        let entry = CacheEntry::new(value, None);
        let mut store = self.store.write().await;
        store.insert(key, entry);
        let entries = store.len();
        drop(store);

        let mut stats = self.stats.write().await;
        stats.entries = entries;
        trace!(cache = %self.name, "Permanent entry cached");
    }

    async fn remove(&self, key: &K) {
        let mut store = self.store.write().await;
        store.remove(key);
        let entries = store.len();
        drop(store);

        let mut stats = self.stats.write().await;
        stats.entries = entries;
        trace!(cache = %self.name, ?key, "Entry removed");
    }

    async fn contains(&self, key: &K) -> bool {
        let store = self.store.read().await;
        if let Some(entry) = store.get(key) {
            if !entry.is_expired() {
                return true;
            }
        }
        false
    }

    async fn clear(&self) {
        let mut store = self.store.write().await;
        store.clear();
        drop(store);

        let mut stats = self.stats.write().await;
        stats.entries = 0;
        debug!(cache = %self.name, "Cache cleared");
    }

    async fn stats(&self) -> CacheStats {
        let stats = self.stats.read().await;
        let mut result = *stats;
        result.entries = self.store.read().await.len();
        result
    }

    async fn len(&self) -> usize {
        self.store.read().await.len()
    }

    async fn is_empty(&self) -> bool {
        self.store.read().await.is_empty()
    }
}

/// Typed cache wrapper for domain-specific caches
///
/// This provides a type-safe wrapper around InMemoryCache,
/// with domain-specific methods and configuration.
pub struct TypedCache<T> {
    inner: Arc<InMemoryCache<String, T>>,
    name: String,
}

impl<T: Clone + Send + Sync + Debug + 'static> TypedCache<T> {
    /// Create a new typed cache with in-memory backend
    pub fn in_memory(name: impl Into<String>, ttl_secs: u64) -> Self {
        let name_str = name.into();
        let cache = InMemoryCache::from_config(name_str.clone(), ttl_secs);
        Self {
            inner: Arc::new(cache),
            name: name_str,
        }
    }

    /// Get a value by key
    pub async fn get(&self, key: &str) -> Option<T> {
        self.inner.get(&key.to_string()).await
    }

    /// Set a value with TTL
    pub async fn set(&self, key: impl Into<String>, value: T, ttl: Duration) {
        self.inner.set(key.into(), value, ttl).await;
    }

    /// Set a value with default TTL from config
    pub async fn set_with_default_ttl(&self, key: impl Into<String>, value: T) {
        let ttl = Duration::from_secs(config::get().cache.default_ttl_secs);
        self.inner.set(key.into(), value, ttl).await;
    }

    /// Remove a value
    pub async fn remove(&self, key: &str) {
        self.inner.remove(&key.to_string()).await;
    }

    /// Get or compute a value
    pub async fn get_or_insert<F, Fut>(&self, key: impl Into<String>, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T>> + Send,
    {
        let key = key.into();
        self.inner.get_or_insert(key, f).await
    }

    /// Get cache stats
    pub async fn stats(&self) -> CacheStats {
        self.inner.stats().await
    }

    /// Get the cache name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Clear all entries
    pub async fn clear(&self) {
        self.inner.clear().await;
    }
}

/// Predefined cache instances for common use cases
pub struct Caches {
    /// Prometheus metrics cache
    pub metrics: TypedCache<crate::legacy::prometheus::PrometheusMetrics>,
    /// Alertmanager alerts cache
    pub alerts: TypedCache<crate::legacy::alertmanager::AlertsResponse>,
    /// News feed cache
    pub news: TypedCache<Vec<crate::legacy::newsfeed::NewsItem>>,
    /// Cilium flows cache
    pub cilium_flows: TypedCache<crate::legacy::cilium::HubbleFlowsResponse>,
    /// Kubernetes resources cache
    pub k8s_resources: TypedCache<serde_json::Value>,
}

impl Caches {
    /// Initialize all caches from configuration
    pub fn new() -> Self {
        let cfg = config::get();

        Self {
            metrics: TypedCache::in_memory("metrics", cfg.cache.prometheus_ttl_secs),
            alerts: TypedCache::in_memory("alerts", cfg.cache.default_ttl_secs),
            news: TypedCache::in_memory("news", cfg.cache.news_ttl_mins * 60),
            cilium_flows: TypedCache::in_memory("cilium_flows", cfg.cache.cilium_ttl_secs),
            k8s_resources: TypedCache::in_memory("k8s_resources", cfg.cache.default_ttl_secs),
        }
    }

    /// Get stats for all caches
    pub async fn stats(&self) -> HashMap<String, CacheStats> {
        let mut stats = HashMap::new();
        stats.insert(self.metrics.name().to_string(), self.metrics.stats().await);
        stats.insert(self.alerts.name().to_string(), self.alerts.stats().await);
        stats.insert(self.news.name().to_string(), self.news.stats().await);
        stats.insert(self.cilium_flows.name().to_string(), self.cilium_flows.stats().await);
        stats.insert(self.k8s_resources.name().to_string(), self.k8s_resources.stats().await);
        stats
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        self.metrics.clear().await;
        self.alerts.clear().await;
        self.news.clear().await;
        self.cilium_flows.clear().await;
        self.k8s_resources.clear().await;
    }
}

impl Default for Caches {
    fn default() -> Self {
        Self::new()
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Cache Entry Tests ====================

    #[test]
    fn test_cache_entry_new() {
        let entry = CacheEntry::new("value", Some(Duration::from_secs(60)));
        assert_eq!(entry.value, "value");
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_permanent() {
        let entry = CacheEntry::<&str>::new("value", None);
        assert!(!entry.is_expired());
        // Should never expire
        std::thread::sleep(Duration::from_millis(10));
        assert!(!entry.is_expired());
    }

    #[test]
    fn test_cache_entry_expired() {
        let entry = CacheEntry::new("value", Some(Duration::from_millis(1)));
        assert!(!entry.is_expired());
        std::thread::sleep(Duration::from_millis(10));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_entry_remaining_ttl() {
        let entry = CacheEntry::new("value", Some(Duration::from_secs(60)));
        let remaining = entry.remaining_ttl().unwrap();
        assert!(remaining > Duration::from_secs(59));
        assert!(remaining <= Duration::from_secs(60));
    }

    #[test]
    fn test_cache_entry_expired_remaining_ttl() {
        let entry = CacheEntry::new("value", Some(Duration::from_millis(1)));
        std::thread::sleep(Duration::from_millis(10));
        let remaining = entry.remaining_ttl().unwrap();
        assert_eq!(remaining, Duration::from_secs(0));
    }

    // ==================== CacheStats Tests ====================

    #[test]
    fn test_cache_stats_default() {
        let stats = CacheStats::default();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.evicted, 0);
    }

    #[test]
    fn test_cache_stats_hit_rate() {
        let stats = CacheStats {
            hits: 75,
            misses: 25,
            ..Default::default()
        };
        assert_eq!(stats.hit_rate(), 0.75);
        assert_eq!(stats.miss_rate(), 0.25);
    }

    #[test]
    fn test_cache_stats_hit_rate_empty() {
        let stats = CacheStats::default();
        assert_eq!(stats.hit_rate(), 0.0);
        assert_eq!(stats.miss_rate(), 0.0);
    }

    #[test]
    fn test_cache_stats_total_requests() {
        let stats = CacheStats {
            hits: 10,
            misses: 5,
            ..Default::default()
        };
        assert_eq!(stats.total_requests(), 15);
    }

    // ==================== InMemoryCache Tests ====================

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache: InMemoryCache<String, i32> = InMemoryCache::new();

        // Set and get
        cache.set("key".to_string(), 42, Duration::from_secs(60)).await;
        assert_eq!(cache.get(&"key".to_string()).await, Some(42));

        // Contains
        assert!(cache.contains(&"key".to_string()).await);
        assert!(!cache.contains(&"other".to_string()).await);

        // Remove
        cache.remove(&"key".to_string()).await;
        assert_eq!(cache.get(&"key".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache: InMemoryCache<String, i32> = InMemoryCache::new();

        // Set with very short TTL
        cache.set("key".to_string(), 42, Duration::from_millis(10)).await;
        assert_eq!(cache.get(&"key".to_string()).await, Some(42));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(cache.get(&"key".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_cache_permanent() {
        let cache: InMemoryCache<String, i32> = InMemoryCache::new();

        cache.set_permanent("key".to_string(), 42).await;
        
        // Should not expire
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(cache.get(&"key".to_string()).await, Some(42));
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache: InMemoryCache<String, i32> = InMemoryCache::new();

        cache.set("key1".to_string(), 1, Duration::from_secs(60)).await;
        cache.set("key2".to_string(), 2, Duration::from_secs(60)).await;

        assert_eq!(cache.len().await, 2);
        
        cache.clear().await;
        
        assert_eq!(cache.len().await, 0);
        assert!(cache.is_empty().await);
    }

    #[tokio::test]
    async fn test_cache_stats_tracking() {
        let cache: InMemoryCache<String, i32> = InMemoryCache::new();

        // Miss
        let _ = cache.get(&"key".to_string()).await;
        
        // Set
        cache.set("key".to_string(), 42, Duration::from_secs(60)).await;
        
        // Hit
        let _ = cache.get(&"key".to_string()).await;
        let _ = cache.get(&"key".to_string()).await;

        let stats = cache.stats().await;
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.entries, 1);
    }

    #[tokio::test]
    async fn test_cache_get_or_insert() {
        let cache: InMemoryCache<String, i32> = InMemoryCache::new();
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicI32::new(0));

        let result = cache
            .get_or_insert("key".to_string(), || async {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok::<_, KusanagiError>(42)
            })
            .await
            .unwrap();

        assert_eq!(result, 42);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);

        // Second call should use cache
        let result2 = cache
            .get_or_insert("key".to_string(), || async {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok::<_, KusanagiError>(999)
            })
            .await
            .unwrap();

        assert_eq!(result2, 42); // Should return cached value
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1); // Function not called again
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let cache: InMemoryCache<String, i32> = InMemoryCache::new();

        cache.set("expired".to_string(), 1, Duration::from_millis(1)).await;
        cache.set("valid".to_string(), 2, Duration::from_secs(60)).await;

        tokio::time::sleep(Duration::from_millis(10)).await;

        let removed = cache.cleanup().await;
        assert_eq!(removed, 1);
        assert_eq!(cache.len().await, 1);
    }

    #[tokio::test]
    async fn test_cache_with_ttl() {
        let cache: InMemoryCache<String, i32> = InMemoryCache::with_ttl("test", Duration::from_secs(120));
        assert_eq!(cache.default_ttl(), Duration::from_secs(120));
    }

    #[tokio::test]
    async fn test_cache_is_empty() {
        let cache: InMemoryCache<String, i32> = InMemoryCache::new();
        assert!(cache.is_empty().await);

        cache.set("key".to_string(), 42, Duration::from_secs(60)).await;
        assert!(!cache.is_empty().await);
    }

    // ==================== TypedCache Tests ====================

    #[tokio::test]
    async fn test_typed_cache() {
        let cache: TypedCache<i32> = TypedCache::in_memory("test", 60);

        cache.set("key", 42, Duration::from_secs(60)).await;
        assert_eq!(cache.get("key").await, Some(42));

        let stats = cache.stats().await;
        assert!(stats.hit_rate() > 0.0 || stats.miss_rate() > 0.0);
    }

    // ==================== Integration Tests ====================

    #[tokio::test]
    async fn test_concurrent_access() {
        use std::sync::Arc;
        use tokio::task;

        let cache: Arc<InMemoryCache<String, i32>> = Arc::new(InMemoryCache::new());
        let mut handles = vec![];

        // Spawn multiple tasks accessing the cache
        for i in 0..10 {
            let cache = Arc::clone(&cache);
            let handle = task::spawn(async move {
                cache.set(format!("key{}", i), i, Duration::from_secs(60)).await;
                cache.get(&format!("key{}", i)).await
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_some());
        }

        assert_eq!(cache.len().await, 10);
    }

    #[tokio::test]
    async fn test_cache_name() {
        let cache: InMemoryCache<String, i32> = InMemoryCache::with_name("my_cache");
        assert_eq!(cache.name(), "my_cache");
    }

    #[tokio::test]
    async fn test_typed_cache_name() {
        let cache: TypedCache<i32> = TypedCache::in_memory("typed_test", 60);
        assert_eq!(cache.name(), "typed_test");
    }
}
