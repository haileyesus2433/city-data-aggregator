mod config;
mod db;
mod handlers;
mod jwt;
mod middleware;
mod openapi;

use axum::{
    Router, middleware as axum_middleware,
    routing::{delete, get, post, put},
};
use common::tracing::init_tracing_pretty;
use std::net::SocketAddr;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing_pretty();

    let config = config::Config::from_env();
    let pool = db::create_pool(&config.database_url).await?;

    let state = handlers::AppState {
        pool: pool.clone(),
        jwt_secret: config.jwt_secret.clone(),
    };

    let app = create_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    info!("Auth service starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Auth service stopped");
    Ok(())
}

fn create_router(state: handlers::AppState) -> Router {
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(handlers::health))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/register", post(handlers::register));

    // Admin routes (require JWT + admin role)
    let admin_routes = Router::new()
        .route("/api/admin/users", get(handlers::list_users))
        .route("/api/admin/users", post(handlers::create_user))
        .route("/api/admin/users/{id}", get(handlers::get_user))
        .route("/api/admin/users/{id}", delete(handlers::delete_user))
        .route(
            "/api/admin/users/{id}/role",
            put(handlers::update_user_role),
        )
        .layer(axum_middleware::from_fn(middleware::require_admin))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    public_routes
        .merge(admin_routes)
        .merge(openapi::swagger_ui())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
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
