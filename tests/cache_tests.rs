#[cfg(test)]
mod tests {
    use kusanagi::cache::{Cache, InMemoryCache};

    #[tokio::test]
    async fn test_cache_set_get() {
        let cache = InMemoryCache::new();

        cache.set("key1", "value1".to_string()).await;
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

        cache.set("key1", "value1".to_string()).await;
        cache.delete("key1").await;
        let result = cache.get("key1").await;

        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let cache = InMemoryCache::new();

        cache.set("key1", "value1".to_string()).await;
        cache.set("key2", "value2".to_string()).await;

        let stats = cache.stats().await;
        assert_eq!(stats.entries, 2);
    }
}
