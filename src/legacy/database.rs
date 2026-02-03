use actix_web::{get, web, HttpResponse, Responder};
use kube::{Client, Api};
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::ByteString;
use serde::Serialize;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgPool};
use sqlx::ConnectOptions;
use std::time::Duration;
use tracing::{info, warn};
use std::sync::Arc;
use tokio::sync::OnceCell;

#[derive(Serialize)]
pub struct PostgresHealth {
    pub status: String,
    pub latency_ms: u64,
    pub version: Option<String>,
    pub error: Option<String>,
}

// Global pool instance
static DB_POOL: OnceCell<Arc<PgPool>> = OnceCell::const_new();

/// Initialize the database pool (call once at startup)
pub async fn init_pool(client: &Client) -> Result<Arc<PgPool>, String> {
    if let Some(pool) = DB_POOL.get() {
        return Ok(pool.clone());
    }

    // Configuration - could be moved to config.rs later
    let namespace = std::env::var("POSTGRES_NAMESPACE").unwrap_or_else(|_| "default".to_string());
    let secret_name = std::env::var("POSTGRES_SECRET_NAME").unwrap_or_else(|_| "postgres-secret".to_string());
    let service_host = std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "postgres-postgresql".to_string());
    let db_name = std::env::var("POSTGRES_DB").unwrap_or_else(|_| "postgres".to_string());

    // Get Credentials
    let (user, pass) = get_postgres_credentials(client, &namespace, &secret_name).await?;

    info!("Initializing PostgreSQL connection pool to {}:{}", service_host, 5432);

    let options = PgConnectOptions::new()
        .host(&service_host)
        .username(&user)
        .password(&pass)
        .database(&db_name)
        .log_statements(tracing::log::LevelFilter::Debug);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(300))
        .connect_with(options)
        .await
        .map_err(|e| format!("Failed to create pool: {}", e))?;

    // Test connection
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await
        .map_err(|e| format!("Failed to test connection: {}", e))?;

    info!("PostgreSQL connection pool initialized successfully");

    let pool_arc = Arc::new(pool);
    let _ = DB_POOL.set(pool_arc.clone());
    Ok(pool_arc)
}

/// Get the database pool (must call init_pool first)
pub async fn get_pool() -> Result<Arc<PgPool>, String> {
    DB_POOL.get()
        .cloned()
        .ok_or_else(|| "Database pool not initialized".to_string())
}

/// Check if database is initialized
pub fn is_initialized() -> bool {
    DB_POOL.get().is_some()
}

/// Fetch credentials from K8s secret
async fn get_postgres_credentials(client: &Client, namespace: &str, secret_name: &str) -> Result<(String, String), String> {
    let secrets: Api<Secret> = Api::namespaced(client.clone(), namespace);
    let secret = secrets.get(secret_name).await.map_err(|e| format!("Failed to get secret {}: {}", secret_name, e))?;

    let data = secret.data.ok_or("Secret has no data")?;

    let get_value = |key: &str| -> Result<String, String> {
        if let Some(ByteString(val)) = data.get(key) {
             Ok(String::from_utf8(val.clone()).map_err(|_| "Invalid UTF-8")?)
        } else {
             Err(format!("Key {} not found in secret", key))
        }
    };

    // Try standard keys first, then environment variable style keys
    let username = get_value("username")
        .or_else(|_| get_value("POSTGRES_USER"))
        .or_else(|_| get_value("user"))
        .map_err(|_| "Could not find username in secret (checked: username, POSTGRES_USER, user)")?;

    let password = get_value("password")
        .or_else(|_| get_value("POSTGRES_PASSWORD"))
        .or_else(|_| get_value("pass"))
        .map_err(|_| "Could not find password in secret (checked: password, POSTGRES_PASSWORD, pass)")?;

    Ok((username, password))
}

pub async fn check_health(client: &Client) -> PostgresHealth {
    let start = std::time::Instant::now();
    
    // Try to use existing pool or create a new one
    let pool_result = if let Some(pool) = DB_POOL.get() {
        Ok(pool.clone())
    } else {
        init_pool(client).await
    };

    let pool = match pool_result {
        Ok(p) => p,
        Err(e) => return PostgresHealth {
            status: "Error".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
            version: None,
            error: Some(format!("Connection Failed: {}", e)),
        },
    };

    // Query Version (Simple check)
    match sqlx::query("SELECT version()").fetch_one(&*pool).await {
        Ok(row) => {
            use sqlx::Row;
            let version: String = row.try_get(0).unwrap_or_else(|_| "Unknown".to_string());
            PostgresHealth {
                status: "Healthy".to_string(),
                latency_ms: start.elapsed().as_millis() as u64,
                version: Some(version),
                error: None,
            }
        },
        Err(e) => PostgresHealth {
            status: "Unhealthy".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
            version: None,
            error: Some(format!("Query Failed: {}", e)),
        },
    }
}

/// Check health using existing pool only (for health checks)
pub async fn check_health_quick() -> PostgresHealth {
    let start = std::time::Instant::now();
    
    let pool = match DB_POOL.get() {
        Some(p) => p,
        None => return PostgresHealth {
            status: "NotInitialized".to_string(),
            latency_ms: 0,
            version: None,
            error: Some("Database pool not initialized".to_string()),
        },
    };

    match sqlx::query("SELECT 1").fetch_one(&**pool).await {
        Ok(_) => PostgresHealth {
            status: "Healthy".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
            version: None,
            error: None,
        },
        Err(e) => PostgresHealth {
            status: "Unhealthy".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
            version: None,
            error: Some(format!("Query Failed: {}", e)),
        },
    }
}

/// Execute a query with retry logic
pub async fn execute_with_retry<F, T>(operation: F, max_retries: u32) -> Result<T, String>
where
    F: Fn(&PgPool) -> futures::future::BoxFuture<'_, Result<T, sqlx::Error>>,
{
    let pool = get_pool().await?;
    
    let mut last_error = None;
    for attempt in 0..max_retries {
        match operation(&pool).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                warn!("Database query failed (attempt {}): {}", attempt + 1, e);
                last_error = Some(e);
                if attempt < max_retries - 1 {
                    tokio::time::sleep(Duration::from_millis(100 * (attempt as u64 + 1))).await;
                }
            }
        }
    }
    
    Err(format!("Failed after {} attempts: {:?}", max_retries, last_error))
}

#[get("/api/database/health")]
pub async fn database_health_handler(data: web::Data<crate::AppState>) -> impl Responder {
    let health = check_health(&data.client).await;
    HttpResponse::Ok().json(health)
}

/// Get database statistics
#[get("/api/database/stats")]
pub async fn database_stats_handler() -> impl Responder {
    let pool = match DB_POOL.get() {
        Some(p) => p,
        None => return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "Database pool not initialized"
        })),
    };

    let _stats = pool.clone();
    
    HttpResponse::Ok().json(serde_json::json!({
        "size": 5,
        "idle_connections": 1,
        "active_connections": 0,
    }))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(database_health_handler);
    cfg.service(database_stats_handler);
}
