use axum::{
    extract::{Path, State},
    response::Json,
};
use common::errors::AppError;
use common::models::TimeData;
use std::sync::Arc;
use tracing::info;

use crate::api_client::WorldTimeApiClient;

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service health check")
    )
)]
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "time-service" }))
}

#[utoipa::path(
    get,
    path = "/api/time/{city}",
    params(
        ("city" = String, Path, description = "City name")
    ),
    responses(
        (status = 200, description = "Time data for the city", body = TimeData),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_time(
    State(client): State<Arc<WorldTimeApiClient>>,
    Path(city): Path<String>,
) -> Result<Json<TimeData>, AppError> {
    info!(city = %city, "Time request received");
    
    let time = client.get_time(&city).await?;
    
    Ok(Json(time))
}

