#[cfg(test)]
mod tests {
    use kusanagi::perf_monitor::PerfMonitor;

    #[test]
    fn test_perf_monitor_new() {
        let monitor = PerfMonitor::new();
        let stats = monitor.stats();
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);
        assert_eq!(stats.api_calls, 0);
        assert_eq!(stats.k8s_queries, 0);
    }

    #[test]
    fn test_perf_monitor_record() {
        let monitor = PerfMonitor::new();
        
        monitor.record_cache_hit();
        monitor.record_cache_hit();
        monitor.record_cache_miss();
        monitor.record_api_call();
        monitor.record_k8s_query();
        
        let stats = monitor.stats();
        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.api_calls, 1);
        assert_eq!(stats.k8s_queries, 1);
    }

    #[test]
    fn test_cache_hit_rate() {
        let monitor = PerfMonitor::new();
        
        monitor.record_cache_hit();
        monitor.record_cache_hit();
        monitor.record_cache_hit();
        monitor.record_cache_miss();
        
        let stats = monitor.stats();
        assert_eq!(stats.cache_hit_rate(), 75.0);
    }

    #[test]
    fn test_cache_hit_rate_zero() {
        let monitor = PerfMonitor::new();
        let stats = monitor.stats();
        assert_eq!(stats.cache_hit_rate(), 0.0);
    }
}
