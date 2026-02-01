use actix_web::{get, web, HttpResponse, Responder};
use kube::{Client, Api};
use k8s_openapi::api::core::v1::Secret;
use k8s_openapi::ByteString;
use serde::Serialize;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::ConnectOptions;
use std::time::Duration;
use tracing::{info, error, warn};

#[derive(Serialize)]
pub struct PostgresHealth {
    pub status: String,
    pub latency_ms: u64,
    pub version: Option<String>,
    pub error: Option<String>,
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
    
    // Configuration - could be moved to config.rs later
    let namespace = "default"; // Or "kusanagi" depending on setup
    let secret_name = "postgres-secret"; // Common helm chart name
    let service_host = "postgres-postgresql"; // Common service name. Or use external DNS.
    let db_name = "postgres"; 

    // 1. Get Credentials
    let (user, pass) = match get_postgres_credentials(client, namespace, secret_name).await {
        Ok(creds) => creds,
        Err(e) => return PostgresHealth {
            status: "Error".to_string(),
            latency_ms: 0,
            version: None,
            error: Some(format!("Credential Error: {}", e)),
        },
    };

    // 2. Connect
    let options = PgConnectOptions::new()
        .host(service_host)
        .username(&user)
        .password(&pass)
        .database(db_name)
        .log_statements(tracing::log::LevelFilter::Debug);

    let pool_res = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(3))
        .connect_with(options)
        .await;

    let pool = match pool_res {
        Ok(p) => p,
        Err(e) => return PostgresHealth {
            status: "Unhealthy".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
            version: None,
            error: Some(format!("Connection Failed: {}", e)),
        },
    };

    // 3. Query Version (Simple check)
    match sqlx::query("SELECT version()").fetch_one(&pool).await {
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

#[get("/api/database/health")]
pub async fn database_health_handler(data: web::Data<crate::AppState>) -> impl Responder {
    let health = check_health(&data.client).await;
    HttpResponse::Ok().json(health)
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(database_health_handler);
}
