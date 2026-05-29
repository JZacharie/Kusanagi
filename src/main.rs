// Kusanagi - Axum Entry Point
// Migration from Actix-web to Axum

use tokio::net::TcpListener;
use tracing::info;

// State - from library
use kusanagi::state::AppState;

// Build timestamp
const BUILD_TIMESTAMP: &str = env!("BUILD_TIMESTAMP");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables from .env if present
    match dotenvy::dotenv() {
        Ok(path) => info!("✅ Loaded environment from: {:?}", path),
        Err(_) => info!("ℹ️ No .env file found, using system environment variables"),
    }

    // Install rustls crypto provider (required for rustls 0.23+)
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Setup logging
    kusanagi::infrastructure::logging::setup_logging()?;

    let version = env!("CARGO_PKG_VERSION");
    info!("🚀 Kusanagi Axum Migration");
    info!("📅 Version: {}", version);
    info!("⏰ Build Time: {}", BUILD_TIMESTAMP);

    // Get bind address
    let host = std::env::var("KUSANAGI_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("KUSANAGI_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let bind_addr = format!("{}:{}", host, port);

    info!("🌐 Server binding to: {}", bind_addr);

    // Create application state
    let state = AppState::new().await?;

    // Start Cilium background refresh task
    if let Some(client) = &state.kube_client {
        let cilium_service = kusanagi::domain::services::cilium_service::CiliumService::new(
            client.as_ref().clone(),
            state.cilium_cache.clone(),
        );
        cilium_service.start_background_refresh();
    }

    // Start MQTT client if configured
    if let Ok(mqtt_host) = std::env::var("MQTT_HOST") {
        let mqtt_port = std::env::var("MQTT_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(1883);
        let mqtt_user = std::env::var("MQTT_USER").ok();
        let mqtt_pass = std::env::var("MQTT_PASSWORD").ok(); // was MQTT_PASS — mismatch with .env

        info!(
            "📡 Starting MQTT client: {}:{} (user: {})",
            mqtt_host,
            mqtt_port,
            if mqtt_user.is_some() { "[CONFIGURED]" } else { "[NONE]" }
        );

        kusanagi::domain::services::mqtt_service::start_mqtt_client(
            state.mqtt_state.clone(),
            mqtt_host,
            mqtt_port,
            mqtt_user,
            mqtt_pass,
            state.namespace.clone(),
        );
    } else {
        info!("ℹ️ MQTT_HOST not set — MQTT client disabled");
    }

    // Build router using the routes module
    let app = kusanagi::interfaces::http::routes::configure_routes(state);

    // Start server
    let listener = TcpListener::bind(&bind_addr).await?;
    info!("✅ Server ready at http://{}", bind_addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}
