use axum::{
    extract::{Path, State},
    response::Json,
};
use axum_extra::extract::Query;
use common::errors::AppError;
use common::models::{AggregateResponse, WeatherData};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

use crate::aggregator::Aggregator;
use crate::api_client::OpenMeteoClient;

#[derive(Clone)]
pub struct AppState {
    pub client: Arc<OpenMeteoClient>,
    pub aggregator: Arc<Aggregator>,
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service health check")
    )
)]
pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "weather-service" }))
}

#[utoipa::path(
    get,
    path = "/api/weather/{city}",
    params(
        ("city" = String, Path, description = "City name")
    ),
    responses(
        (status = 200, description = "Weather data for the city", body = WeatherData),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_weather(
    State(state): State<AppState>,
    Path(city): Path<String>,
) -> Result<Json<WeatherData>, AppError> {
    info!(city = %city, "Weather request received");

    let weather = state.client.get_weather(&city).await?;

    Ok(Json(weather))
}

#[derive(Deserialize)]
pub struct AggregateQuery {
    #[serde(default)]
    pub city: Vec<String>,
}

#[utoipa::path(
    get,
    path = "/api/aggregate",
    params(
        ("city" = Vec<String>, Query, description = "Cities to aggregate data for (1-20)")
    ),
    responses(
        (status = 200, description = "Aggregated city data", body = AggregateResponse),
        (status = 400, description = "Invalid request - must provide 1-20 cities")
    ),
    tag = "aggregate"
)]
pub async fn aggregate(
    State(state): State<AppState>,
    Query(params): Query<AggregateQuery>,
) -> Result<Json<AggregateResponse>, AppError> {
    info!(count = params.city.len(), "Aggregate request received");

    let response = state.aggregator.aggregate(params.city).await?;

    Ok(Json(response))
}
