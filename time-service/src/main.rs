mod api_client;
mod cache;
mod config;
mod handlers;
mod openapi;

use axum::{Router, routing::get};
use common::tracing::init_tracing_pretty;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing_pretty();

    let config = config::Config::from_env();
    let cache = Arc::new(cache::TimeCache::new());
    let api_client = Arc::new(api_client::WorldTimeApiClient::new(
        cache.clone(),
        config.world_time_api_url.clone(),
    ));

    // Prefill cache on startup
    info!("Prefilling time cache on startup...");
    if let Err(e) = api_client.prefill_cache().await {
        warn!(error = %e, "Cache prefill failed, continuing with empty cache");
    }
    info!("Cache prefill completed");

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/api/time/{city}", get(handlers::get_time))
        .merge(openapi::swagger_ui())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(api_client);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Time service starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Time service stopped");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received SIGINT, starting graceful shutdown...");
        },
        _ = terminate => {
            info!("Received SIGTERM, starting graceful shutdown...");
        },
    }

    warn!("Shutting down gracefully...");
}
