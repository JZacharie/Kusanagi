#[cfg(test)]
mod tests {
    use kusanagi::AdvancedCache;
    use std::time::Duration;

    #[tokio::test]
    async fn test_multiple_caches() {
        let cache1 = AdvancedCache::<String>::new(Duration::from_secs(60));
        let cache2 = AdvancedCache::<String>::new(Duration::from_secs(120));

        cache1
            .set("key1".to_string(), "value1".to_string(), None)
            .await;
        cache2
            .set("key2".to_string(), "value2".to_string(), None)
            .await;

        assert_eq!(cache1.get("key1").await, Some("value1".to_string()));
        assert_eq!(cache2.get("key2").await, Some("value2".to_string()));
        assert_eq!(cache1.get("key2").await, None);
        assert_eq!(cache2.get("key1").await, None);
    }

    #[tokio::test]
    async fn test_cache_overwrite() {
        let cache = AdvancedCache::<String>::new(Duration::from_secs(60));

        cache
            .set("key1".to_string(), "value1".to_string(), None)
            .await;
        assert_eq!(cache.get("key1").await, Some("value1".to_string()));

        cache
            .set("key1".to_string(), "value2".to_string(), None)
            .await;
        assert_eq!(cache.get("key1").await, Some("value2".to_string()));
    }

    #[tokio::test]
    async fn test_cache_large_data() {
        let cache = AdvancedCache::<String>::new(Duration::from_secs(60));
        let large_string = "x".repeat(10000);

        cache
            .set("large".to_string(), large_string.clone(), None)
            .await;
        assert_eq!(cache.get("large").await, Some(large_string));
    }

    #[tokio::test]
    async fn test_cache_many_entries() {
        let cache = AdvancedCache::<String>::new(Duration::from_secs(60));

        for i in 0..100 {
            cache
                .set(format!("key{}", i), format!("value{}", i), None)
                .await;
        }

        let stats = cache.stats().await;
        assert_eq!(stats.entries, 100);

        for i in 0..100 {
            assert_eq!(
                cache.get(&format!("key{}", i)).await,
                Some(format!("value{}", i))
            );
        }
    }

    #[tokio::test]
    async fn test_cache_delete_nonexistent() {
        let cache = AdvancedCache::<String>::new(Duration::from_secs(60));
        cache.delete("nonexistent").await;
        // Should not panic
    }

    #[tokio::test]
    async fn test_cache_clear_empty() {
        let cache = AdvancedCache::<String>::new(Duration::from_secs(60));
        cache.clear().await;
        let stats = cache.stats().await;
        assert_eq!(stats.entries, 0);
    }
}
