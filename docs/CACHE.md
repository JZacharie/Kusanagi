# Kusanagi Cache System

## Overview

The unified caching system provides a generic, type-safe caching layer with TTL support. It replaces scattered cache implementations across modules with a centralized solution.

## Features

- ✅ **Generic**: Works with any `Clone + Send + Sync` type
- ✅ **TTL Support**: Automatic expiration of cached entries
- ✅ **Statistics**: Cache hit/miss tracking
- ✅ **Configurable**: TTL values from configuration file
- ✅ **Thread-Safe**: Concurrent access with `RwLock`

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Cache Layer                             │
├─────────────────────────────────────────────────────────────┤
│  TypedCache<T>          InMemoryCache<K, V>    CacheEntry<V>│
│  (Domain-specific)      (HashMap + RwLock)     (Value + TTL)│
├─────────────────────────────────────────────────────────────┤
│  Caches (Registry)                                           │
│  ├── metrics: TypedCache<PrometheusMetrics>                  │
│  ├── alerts: TypedCache<AlertsResponse>                      │
│  ├── news: TypedCache<Vec<NewsItem>>                         │
│  ├── cilium_flows: TypedCache<HubbleFlowsResponse>           │
│  └── k8s_resources: TypedCache<Value>                        │
└─────────────────────────────────────────────────────────────┘
```

## Usage

### Basic Cache Operations

```rust
use crate::cache::{Cache, InMemoryCache};
use std::time::Duration;

// Create a cache
let cache: InMemoryCache<String, i32> = InMemoryCache::new();

// Set a value with TTL
cache.set("key".to_string(), 42, Duration::from_secs(60)).await;

// Get a value
if let Some(value) = cache.get(&"key".to_string()).await {
    println!("Cached: {}", value);
}

// Set a permanent value (never expires)
cache.set_permanent("config".to_string(), settings).await;

// Remove a value
cache.remove(&"key".to_string()).await;

// Clear all entries
cache.clear().await;
```

### Get or Insert Pattern

```rust
use crate::error::Result;

let result = cache
    .get_or_insert("expensive_key".to_string(), || async {
        // This closure is only called if the key is not in cache
        let data = fetch_from_database().await?;
        Ok::<_, KusanagiError>(data)
    })
    .await?;
```

### Typed Cache (Domain-Specific)

```rust
use crate::cache::TypedCache;
use crate::prometheus::PrometheusMetrics;

// Create a typed cache with 60-second TTL
let metrics_cache: TypedCache<PrometheusMetrics> = 
    TypedCache::in_memory("metrics", 60);

// Use the cache
metrics_cache.set("cluster", metrics, Duration::from_secs(60)).await;

if let Some(metrics) = metrics_cache.get("cluster").await {
    println!("CPU: {}%", metrics.cpu_usage_percent);
}

// Get statistics
let stats = metrics_cache.stats().await;
println!("Hit rate: {:.1}%", stats.hit_rate() * 100.0);
```

### Global Cache Registry

```rust
use crate::cache::Caches;

// Initialize all caches from configuration
let caches = Caches::new();

// Access specific caches
caches.metrics.set("cluster", metrics, ttl).await;

// Get stats for all caches
let all_stats = caches.stats().await;
for (name, stats) in all_stats {
    println!("{}: {:.1}% hit rate", name, stats.hit_rate() * 100.0);
}

// Clear all caches
caches.clear_all().await;
```

## Configuration

### Cache TTL Settings

```toml
[cache]
default_ttl_secs = 300      # 5 minutes
news_ttl_mins = 30          # 30 minutes
prometheus_ttl_secs = 60    # 1 minute
cilium_ttl_secs = 60        # 1 minute
```

### Environment Variables

```bash
export KUSANAGI_CACHE_DEFAULT_TTL_SECS=300
export KUSANAGI_CACHE_NEWS_TTL_MINS=30
export KUSANAGI_CACHE_PROMETHEUS_TTL_SECS=60
export KUSANAGI_CACHE_CILIUM_TTL_SECS=60
```

## Cache Statistics

The cache tracks detailed statistics:

```rust
pub struct CacheStats {
    pub hits: u64,        // Number of cache hits
    pub misses: u64,      // Number of cache misses
    pub entries: usize,   // Current number of entries
    pub evicted: u64,     // Number of expired entries removed
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64;   // 0.0 to 1.0
    pub fn miss_rate(&self) -> f64;  // 0.0 to 1.0
    pub fn total_requests(&self) -> u64;
}
```

## Migration Guide

### Before (Old Pattern)

```rust
// In each module...
lazy_static! {
    static ref METRICS_CACHE: Arc<RwLock<Option<(Metrics, Instant)>>> = 
        Arc::new(RwLock::new(None));
}

pub async fn get_cached_metrics() -> Result<Metrics> {
    // Check cache
    {
        let cache = METRICS_CACHE.read().await;
        if let Some((ref metrics, timestamp)) = *cache {
            if timestamp.elapsed() < Duration::from_secs(60) {
                return Ok(metrics.clone());
            }
        }
    }
    
    // Fetch and cache
    let metrics = fetch_metrics().await?;
    let mut cache = METRICS_CACHE.write().await;
    *cache = Some((metrics.clone(), Instant::now()));
    Ok(metrics)
}
```

### After (New Pattern)

```rust
use crate::cache::{Cache, InMemoryCache};

lazy_static! {
    static ref METRICS_CACHE: InMemoryCache<String, Metrics> = 
        InMemoryCache::from_config("metrics", 60);
}

pub async fn get_cached_metrics() -> Result<Metrics> {
    // Try cache first
    if let Some(metrics) = METRICS_CACHE.get(&"cluster".to_string()).await {
        return Ok(metrics);
    }
    
    // Fetch and cache
    let metrics = fetch_metrics().await?;
    METRICS_CACHE.set("cluster".to_string(), metrics.clone(), Duration::from_secs(60)).await;
    Ok(metrics)
}

// Or even simpler with get_or_insert:
pub async fn get_cached_metrics() -> Result<Metrics> {
    METRICS_CACHE
        .get_or_insert("cluster".to_string(), || async {
            fetch_metrics().await
        })
        .await
}
```

## Best Practices

### 1. Choose Appropriate TTL

```rust
// Short TTL for frequently changing data
const METRICS_TTL: u64 = 30;  // 30 seconds

// Longer TTL for stable data
const CONFIG_TTL: u64 = 3600;  // 1 hour

// Very long TTL for static data
const STATIC_TTL: u64 = 86400;  // 24 hours
```

### 2. Use Meaningful Cache Keys

```rust
// Good: Include all relevant parameters
let key = format!("pods:{}:{}", namespace, status);

// Good: Use structured keys
let key = format!("metrics:{cluster}:{node}");
```

### 3. Handle Cache Errors Gracefully

```rust
// Don't fail the request if caching fails
match cache.set(key, value, ttl).await {
    Ok(_) => tracing::debug!("Cached successfully"),
    Err(e) => tracing::warn!("Failed to cache: {}", e),
}
```

### 4. Monitor Cache Performance

```rust
let stats = cache.stats().await;
tracing::info!(
    "Cache hit rate: {:.1}% ({} hits, {} misses)",
    stats.hit_rate() * 100.0,
    stats.hits,
    stats.misses
);
```

## Performance Characteristics

| Operation | Time Complexity | Notes |
|-----------|----------------|-------|
| `get` | O(1) | Read lock, occasional cleanup |
| `set` | O(1) | Write lock |
| `remove` | O(1) | Write lock |
| `clear` | O(n) | Write lock, drops all entries |
| `cleanup` | O(n) | Removes expired entries |

## Future Enhancements

- [ ] **Redis Backend**: Distributed caching support
- [ ] **LRU Eviction**: Remove least recently used entries when size limit reached
- [ ] **Compression**: Compress large cached values
- [ ] **Metrics Export**: Prometheus metrics for cache performance
- [ ] **Distributed Lock**: Prevent cache stampede on popular keys

## Testing

Run cache-specific tests:

```bash
cargo test cache::
```

All tests:

```bash
cargo test
```
