use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use epochzone::config::AppConfig;
use epochzone::db::init_db;
use epochzone::routes::create_router;
use epochzone::AppState;

#[tokio::main]
async fn main() {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "epochzone=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration from environment
    let config = AppConfig::from_env();
    tracing::info!("CORS allowed origins: {:?}", config.cors_allowed_origins);

    // Initialize database
    let db = init_db(&config.database_url).await;
    tracing::info!("Database initialized at: {}", config.database_url);

    let state = AppState {
        db,
        config: Arc::new(config),
    };

    let app = create_router(state);

    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
