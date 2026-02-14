// Infrastructure - Logging configuration
// Extracted from main.rs for better organization

use anyhow::Result;
use tracing::info;

/// Setup logging with file appender and rotation
pub fn setup_logging() -> Result<()> {
    // Use /tmp as it's usually writable in containers, or allow override
    let log_dir_env =
        std::env::var("KUSANAGI_LOG_DIR").unwrap_or_else(|_| "/tmp/kusanagi-logs".to_string());
    let log_dir = log_dir_env.as_str();

    // Check if we can write to the log directory
    let file_appender = match std::fs::create_dir_all(log_dir) {
        Ok(_) => {
            // Create a placeholder file to ensure the directory is not empty
            let init_file = std::path::Path::new(log_dir).join("kusanagi.log.0000-init");
            if let Err(e) = std::fs::write(&init_file, "Initializing Kusanagi logs...\n") {
                if e.kind() == std::io::ErrorKind::ReadOnlyFilesystem
                    || e.raw_os_error() == Some(30)
                {
                    eprintln!(
                        "ℹ️ Log directory '{}' is read-only. File logging disabled.",
                        log_dir
                    );
                } else {
                    eprintln!(
                        "⚠️ Failed to create init log file in '{}': {}. File logging disabled.",
                        log_dir, e
                    );
                }
                None
            } else {
                let appender = tracing_appender::rolling::minutely(log_dir, "kusanagi.log");
                Some(tracing_appender::non_blocking(appender))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::ReadOnlyFilesystem || e.raw_os_error() == Some(30) {
                eprintln!(
                    "ℹ️ Log directory '{}' is read-only. File logging disabled.",
                    log_dir
                );
            } else {
                eprintln!(
                    "⚠️ Failed to create log directory '{}': {}. File logging disabled.",
                    log_dir, e
                );
            }
            None
        }
    };

    let (non_blocking, _guard) = match file_appender {
        Some((nb, guard)) => (Some(nb), Some(guard)),
        None => (None, None),
    };

    // Spawn background task to clean up old logs ONLY if file logging is enabled
    if _guard.is_some() {
        let log_dir_for_cleanup = log_dir_env.clone();
        tokio::spawn(async move {
            // Wait a bit before first cleanup
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

            loop {
                if let Ok(entries) = std::fs::read_dir(&log_dir_for_cleanup) {
                    let now = std::time::SystemTime::now();
                    let retention_period = std::time::Duration::from_secs(15 * 60); // 15 minutes

                    for entry in entries.filter_map(|e| e.ok()) {
                        let path = entry.path();
                        if path.is_file() {
                            // Check if file name starts with kusanagi.log
                            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                                if name.starts_with("kusanagi.log") {
                                    // Check modification time
                                    if let Ok(metadata) = std::fs::metadata(&path) {
                                        if let Ok(modified) = metadata.modified() {
                                            if let Ok(age) = now.duration_since(modified) {
                                                if age > retention_period {
                                                    if let Err(e) = std::fs::remove_file(&path) {
                                                        eprintln!(
                                                            "Failed to delete old log {}: {}",
                                                            name, e
                                                        );
                                                    } else {
                                                        // Use println instead of tracing to avoid recursive logging issues
                                                        println!("Purged old log file: {}", name);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Check every minute
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });
    }

    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "kusanagi=debug,tower_http=debug,axum=debug".into());

    let stdout_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    // Create the file layer as an Option
    let file_layer = non_blocking.map(|nb| {
        tracing_subscriber::fmt::layer()
            .with_writer(nb)
            .with_ansi(false)
    });

    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    info!("✅ Logging initialized");
    Ok(())
}
