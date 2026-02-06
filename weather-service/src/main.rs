mod aggregator;
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
use tokio_util::sync::CancellationToken;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing_pretty();

    let config = config::Config::from_env();
    let cancellation_token = CancellationToken::new();

    let cache = Arc::new(cache::WeatherCache::with_ttl(config.cache_ttl_seconds));
    let api_client = Arc::new(api_client::OpenMeteoClient::new(
        cache.clone(),
        config.open_meteo_url.clone(),
        config.rate_limit_per_minute,
    ));
    let aggregator = Arc::new(aggregator::Aggregator::new(
        api_client.clone(),
        config.time_service_url.clone(),
        cancellation_token.clone(),
    ));

    let state = handlers::AppState {
        client: api_client,
        aggregator,
    };

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/api/weather/{city}", get(handlers::get_weather))
        .route("/api/aggregate", get(handlers::aggregate))
        .merge(openapi::swagger_ui())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Weather service starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(cancellation_token))
        .await?;

    info!("Weather service stopped");
    Ok(())
}

async fn shutdown_signal(cancellation_token: CancellationToken) {
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

    // Cancel all in-flight requests
    cancellation_token.cancel();
    warn!("Cancelled in-flight requests, shutting down gracefully...");
}
