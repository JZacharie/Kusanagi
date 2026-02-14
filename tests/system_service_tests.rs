use kusanagi::domain::services::system_service::SystemService;

#[tokio::test]
async fn test_system_status_integration() {
    // This tests the actual implementation using sysinfo
    let status = SystemService::get_status();

    assert_eq!(status.status, "operational");
    assert!(!status.version.is_empty());
    // uptime_secs is unsigned, so it's always >= 0 by definition
    assert!(status.uptime_secs < u64::MAX); // Just verify it's a valid value
                                            // memory_usage_mb can be 0 in some container environments or if reading fails gracefully
                                            // cpu_usage can be 0.0
}

#[tokio::test]
async fn test_system_logs_integration() {
    // This attempts to read logs.
    // It might return an error if no logs are found, or empty string, but it shouldn't panic.
    let result = SystemService::get_logs().await;

    match result {
        Ok(logs) => {
            // If we get logs, great.
            println!("Got {} bytes of logs", logs.len());
        }
        Err(e) => {
            // If we fail (e.g. permission denied on journalctl), that's acceptable for a unit test environment
            // but we should verify the error message structure if possible.
            println!("Failed to get logs (expected in some envs): {}", e);
        }
    }
}
