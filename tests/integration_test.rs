use kusanagi::{Config, InMemoryCache, cache::Cache};

#[tokio::test]
async fn test_cache_integration() {
    let cache = InMemoryCache::new();
    
    // Test multiple operations
    cache.set("user:1", "Alice".to_string()).await;
    cache.set("user:2", "Bob".to_string()).await;
    
    assert_eq!(cache.get("user:1").await, Some("Alice".to_string()));
    assert_eq!(cache.get("user:2").await, Some("Bob".to_string()));
    
    cache.delete("user:1").await;
    assert_eq!(cache.get("user:1").await, None);
    
    let stats = cache.stats().await;
    assert_eq!(stats.entries, 1);
}

#[test]
fn test_config_creation() {
    let config = Config::default();
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.mqtt.port, 1883);
}
