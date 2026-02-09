use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use epochzone::routes::create_router;

#[tokio::main]
async fn main() {
    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "epochzone=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = create_router();
    
    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("🚀 Server listening on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
