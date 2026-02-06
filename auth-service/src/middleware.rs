use axum::{
    body::Body,
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use common::errors::AppError;
use common::models::Claims;
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::handlers::AppState;

/// Middleware to validate JWT token and extract claims
pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::auth("Missing Authorization header"))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::auth("Invalid Authorization header format"));
    }

    let token = &auth_header[7..];
    let decoding_key = DecodingKey::from_secret(state.jwt_secret.as_ref());
    let validation = Validation::default();

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
        .map_err(|e| AppError::auth(format!("Invalid token: {}", e)))?;

    // Insert claims into request extensions for handlers to access
    request.extensions_mut().insert(token_data.claims);

    Ok(next.run(request).await)
}

/// Middleware to require admin role
pub async fn require_admin(request: Request<Body>, next: Next) -> Result<Response, AppError> {
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| AppError::authorization("No claims found in request"))?
        .clone();

    if claims.role != "admin" {
        return Err(AppError::authorization(
            "Admin role required for this endpoint",
        ));
    }

    Ok(next.run(request).await)
}
