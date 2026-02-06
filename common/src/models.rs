use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// City data aggregation response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CityData {
    pub city: String,
    pub weather: Option<WeatherData>,
    pub time: Option<TimeData>,
    pub errors: Vec<String>,
}

/// Weather information from external API
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct WeatherData {
    pub temperature: f64,
    pub condition: String,
    pub humidity: Option<f64>,
    pub wind_speed: Option<f64>,
}

/// Time information from external API
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TimeData {
    pub datetime: String,
    pub timezone: String,
    pub unix_time: i64,
}

/// Aggregate response for multiple cities
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct AggregateResponse {
    pub cities: Vec<CityData>,
    pub summary: ResponseSummary,
}

/// Summary of successful vs failed cities
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ResponseSummary {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
}

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub sub: String, // user_id
    pub exp: usize,  // expiration timestamp
    pub role: String,
    pub permissions: Vec<String>,
}

/// User creation request
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Option<String>,
}

/// User response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Login request
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// Login response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserResponse,
}
