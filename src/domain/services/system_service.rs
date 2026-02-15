use serde::{Deserialize, Serialize};
use sysinfo::System;
use tokio::process::Command;
use utoipa::ToSchema;

use std::sync::OnceLock;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SystemStatus {
    pub status: String,
    pub uptime_secs: u64,
    pub start_time: String,
    pub cpu_usage: f32,
    pub memory_usage_mb: u64,
    pub version: String,
}

static START_TIME: OnceLock<Instant> = OnceLock::new();
static START_TIME_STR: OnceLock<String> = OnceLock::new();

pub struct SystemService;

impl SystemService {
    #[tracing::instrument(name = "system_get_status")]
    pub fn get_status() -> SystemStatus {
        metrics::counter!("system_status_check_total", 1);

        // Try to get system uptime from /proc/uptime for accuracy
        // Fallback to Instant-based calculation if unavailable
        let uptime = Self::get_system_uptime_secs().unwrap_or_else(|| {
            let now_instant = Instant::now();
            let start_instant = START_TIME.get_or_init(Instant::now);
            now_instant.duration_since(*start_instant).as_secs()
        });

        let start_time_str = START_TIME_STR.get_or_init(|| chrono::Utc::now().to_rfc3339());

        let mut sys = System::new_all();
        sys.refresh_all();

        let cpu_usage = sys.global_cpu_usage();
        // Try to get container memory usage, fallback to whole system usage
        let memory_used = Self::get_container_memory_usage().unwrap_or_else(|| sys.used_memory());

        SystemStatus {
            status: "operational".to_string(),
            uptime_secs: uptime,
            start_time: start_time_str.clone(),
            cpu_usage,
            memory_usage_mb: memory_used / 1024 / 1024,
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Read system uptime from /proc/uptime (Linux)
    /// This gives the real system uptime, not the process uptime
    fn get_system_uptime_secs() -> Option<u64> {
        // Try /proc/uptime first (most accurate for Linux systems)
        if let Ok(contents) = std::fs::read_to_string("/proc/uptime") {
            if let Some(first_part) = contents.split_whitespace().next() {
                if let Ok(uptime_secs_f) = first_part.parse::<f64>() {
                    return Some(uptime_secs_f as u64);
                }
            }
        }
        None
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
        if let Ok(contents) = std::fs::read_to_string("/sys/fs/cgroup/memory/memory.usage_in_bytes")
        {
            if let Ok(bytes) = contents.trim().parse::<u64>() {
                return Some(bytes);
            }
        }

        None
    }

    #[tracing::instrument(name = "system_get_logs")]
    pub async fn get_logs() -> Result<String, String> {
        metrics::counter!("system_logs_access_total", 1);
        // Try to read from local log file first (for Docker/k8s support)
        let log_dir_env =
            std::env::var("KUSANAGI_LOG_DIR").unwrap_or_else(|_| "/tmp/kusanagi-logs".to_string());
        let log_dir = log_dir_env.as_str();
        let mut latest_log_content = String::new();
        let debug_info = match std::fs::read_dir(log_dir) {
            Ok(entries) => {
                let mut files: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path().is_file()
                            && e.file_name().to_string_lossy().starts_with("kusanagi.log")
                    })
                    .collect();

                let info = format!("checked {} candidates in {}", files.len(), log_dir);

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
                info
            }
            Err(e) => {
                format!("failed to read directory {}: {}", log_dir, e)
            }
        };

        tracing::debug!("Log search result: {}", debug_info);

        if !latest_log_content.is_empty() {
            return Ok(latest_log_content);
        }

        // Fallback to journalctl (may not be available in Docker)
        match Command::new("journalctl")
            .args(["-n", "50", "-o", "short", "--no-pager"])
            .output()
            .await
        {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!("journalctl failed: {}", err);
                    // Return empty logs rather than error - application still works
                    Ok(format!(
                        "No local logs available ({}). journalctl not accessible in container environment.",
                        debug_info
                    ))
                }
            }
            Err(e) => {
                tracing::warn!("journalctl not available: {}", e);
                // Return informative message rather than error 500
                Ok(format!(
                    "No local logs available ({}). Log collection requires file logging to be configured or journalctl access.",
                    debug_info
                ))
            }
        }
    }
}
