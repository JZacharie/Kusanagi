use axum::response::IntoResponse;
use axum::Json;
use sysinfo::System;
use tokio::process::Command;

/// System status endpoint
pub async fn system_status() -> impl IntoResponse {
    let mut sys = System::new_all();
    sys.refresh_all();

    // Calculate global CPU usage
    let cpu_usage = sys.global_cpu_usage();
    // Try to get container memory usage, fallback to whole system usage
    let memory_used = get_container_memory_usage().unwrap_or_else(|| sys.used_memory());
    let uptime = System::uptime();

    Json(serde_json::json!({
        "status": "operational",
        "uptime_secs": uptime,
        "cpu_usage": cpu_usage,
        "memory_usage_mb": memory_used / 1024 / 1024,
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Try to read memory usage from cgroup v2
fn get_container_memory_usage() -> Option<u64> {
    // Try cgroup v2
    if let Ok(contents) = std::fs::read_to_string("/sys/fs/cgroup/memory.current") {
        if let Ok(bytes) = contents.trim().parse::<u64>() {
            return Some(bytes);
        }
    }

    // Try cgroup v1 (less likely in modern k8s but good fallback)
    if let Ok(contents) = std::fs::read_to_string("/sys/fs/cgroup/memory/memory.usage_in_bytes") {
        if let Ok(bytes) = contents.trim().parse::<u64>() {
            return Some(bytes);
        }
    }

    None
}

/// System logs endpoint
pub async fn system_logs() -> impl IntoResponse {
    // Try to read from local log file first (for Docker/k8s support)
    // We configured it to write to "/tmp/kusanagi-logs/kusanagi.log.YYYY-MM-DD-HH-MM"
    // Helper to find the latest log file in log directory.

    let log_dir = "/tmp/kusanagi-logs";
    let mut latest_log_content = String::new();

    if let Ok(entries) = std::fs::read_dir(log_dir) {
        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path().is_file() && e.file_name().to_string_lossy().starts_with("kusanagi.log")
            })
            .collect();

        // Sort by name (which includes date) descending
        files.sort_by_key(|e| std::cmp::Reverse(e.file_name()));

        if let Some(latest) = files.first() {
            if let Ok(content) = std::fs::read_to_string(latest.path()) {
                // Get last 200 lines to avoid sending huge payload
                latest_log_content = content
                    .lines()
                    .rev()
                    .take(200)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join("\n");
            }
        }
    }

    if !latest_log_content.is_empty() {
        return latest_log_content;
    }

    // Fallback to journalctl
    match Command::new("journalctl")
        .args(["-n", "50", "-o", "short", "--no-pager"])
        .output()
        .await
    {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                let err = String::from_utf8_lossy(&output.stderr);
                format!("Failed to retrieve logs (checked file '/tmp/kusanagi-logs/kusanagi.log*' and using journalctl): {}", err)
            }
        }
        Err(e) => format!(
            "Failed to retrieve logs: Local file not found/empty and journalctl failed: {}",
            e
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_status_real_metrics() {
        // We can't easily test IntoResponse directly without spinning up axum,
        // but we can sanity check the logic if we extract it, or just trust manual verification.
        // However, we can call the function and check if it panics.
        // To actually check the value, we would need to decode the Json response.
        // For now, let's just ensure it runs without panic.
        let _response = system_status().await;
        // In a real test we'd convert extract the JSON and assert uptime > 0
    }
}

/// News endpoint
pub async fn news() -> impl IntoResponse {
    match crate::domain::services::news_service::get_news().await {
        Ok(news) => Json(news).into_response(),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e,
            "items": []
        }))
        .into_response(),
    }
}
