use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

/// Structured error types for the microservices
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("HTTP error: {status} - {message}")]
    HttpError { status: u16, message: String },

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON parse error: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Authorization error: {0}")]
    AuthorizationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl AppError {
    pub fn http(status: u16, message: impl Into<String>) -> Self {
        Self::HttpError {
            status,
            message: message.into(),
        }
    }

    pub fn timeout(message: impl Into<String>) -> Self {
        Self::TimeoutError(message.into())
    }

    pub fn database(message: impl Into<String>) -> Self {
        Self::DatabaseError(message.into())
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::AuthError(message.into())
    }

    pub fn authorization(message: impl Into<String>) -> Self {
        Self::AuthorizationError(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError(message.into())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::TimeoutError(_) => StatusCode::GATEWAY_TIMEOUT,
            AppError::HttpError { status, .. } => {
                StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY)
            }
            AppError::NetworkError(_) => StatusCode::BAD_GATEWAY,
            AppError::ParseError(_) => StatusCode::BAD_REQUEST,
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::AuthError(_) => StatusCode::UNAUTHORIZED,
            AppError::AuthorizationError(_) => StatusCode::FORBIDDEN,
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(ErrorResponse {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}
