use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::interval;

pub struct PerfMonitor {
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    pub api_calls: AtomicU64,
    pub k8s_queries: AtomicU64,
}

impl PerfMonitor {
    pub fn new() -> Self {
        Self {
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            api_calls: AtomicU64::new(0),
            k8s_queries: AtomicU64::new(0),
        }
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_api_call(&self) {
        self.api_calls.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_k8s_query(&self) {
        self.k8s_queries.fetch_add(1, Ordering::Relaxed);
    }

    pub fn stats(&self) -> PerfStats {
        PerfStats {
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
            api_calls: self.api_calls.load(Ordering::Relaxed),
            k8s_queries: self.k8s_queries.load(Ordering::Relaxed),
        }
    }

    pub async fn start_logging(self: std::sync::Arc<Self>) {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let stats = self.stats();
            tracing::info!(
                "📊 Perf: cache_hit_rate={:.1}%, api_calls={}, k8s_queries={}",
                stats.cache_hit_rate(),
                stats.api_calls,
                stats.k8s_queries
            );
        }
    }
}

#[derive(Debug)]
pub struct PerfStats {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub api_calls: u64,
    pub k8s_queries: u64,
}

impl PerfStats {
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f64 / total as f64) * 100.0
        }
    }
}
