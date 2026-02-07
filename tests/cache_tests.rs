#[cfg(test)]
mod tests {
    use kusanagi::cache::{InMemoryCache, Cache};

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = InMemoryCache::new();
        
        cache.set("key1".to_string(), "value1".to_string()).await;
        let result = cache.get("key1").await;
        
        assert_eq!(result, Some("value1".to_string()));
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = InMemoryCache::new();
        let result = cache.get("nonexistent").await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cache_delete() {
        let cache = InMemoryCache::new();
        
        cache.set("key1".to_string(), "value1".to_string()).await;
        cache.delete("key1").await;
        let result = cache.get("key1").await;
        
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = InMemoryCache::new();
        
        cache.set("key1".to_string(), "value1".to_string()).await;
        cache.set("key2".to_string(), "value2".to_string()).await;
        
        let stats = cache.stats().await;
        assert_eq!(stats.entries, 2);
    }

    #[tokio::test]
    async fn test_cache_concurrent_access() {
        let cache = InMemoryCache::new();
        
        let cache1 = cache.clone();
        let cache2 = cache.clone();
        
        let handle1 = tokio::spawn(async move {
            for i in 0..100 {
                cache1.set(format!("key{}", i), format!("value{}", i)).await;
            }
        });
        
        let handle2 = tokio::spawn(async move {
            for i in 0..100 {
                cache2.get(&format!("key{}", i)).await;
            }
        });
        
        handle1.await.unwrap();
        handle2.await.unwrap();
        
        let stats = cache.stats().await;
        assert!(stats.entries <= 100);
    }
}
