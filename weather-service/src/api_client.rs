use crate::cache::WeatherCache;
use common::errors::AppError;
use common::http_client::HttpClient;
use common::models::WeatherData;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::Instant;
use tracing::{info, instrument, warn};

#[derive(Debug, Serialize, Deserialize)]
struct OpenMeteoResponse {
    current: CurrentWeather,
}

#[derive(Debug, Serialize, Deserialize)]
struct CurrentWeather {
    temperature_2m: f64,
    relative_humidity_2m: Option<f64>,
    wind_speed_10m: Option<f64>,
    weather_code: Option<u32>,
}

pub struct OpenMeteoClient {
    http_client: HttpClient,
    cache: Arc<WeatherCache>,
    base_url: String,
    rate_limiter: Arc<Semaphore>,
    last_request_time: Arc<tokio::sync::Mutex<Option<Instant>>>,
    min_request_interval: Duration,
}

impl OpenMeteoClient {
    pub fn new(cache: Arc<WeatherCache>, base_url: String, rate_limit_per_minute: u32) -> Self {
        let permits = rate_limit_per_minute.max(1) as usize;
        let min_request_interval =
            Duration::from_millis(60_000 / rate_limit_per_minute.max(1) as u64);
        Self {
            http_client: HttpClient::default(),
            cache,
            base_url,
            rate_limiter: Arc::new(Semaphore::new(permits)),
            last_request_time: Arc::new(tokio::sync::Mutex::new(None)),
            min_request_interval,
        }
    }

    #[instrument(skip(self), fields(city = %city))]
    pub async fn get_weather(&self, city: &str) -> Result<WeatherData, AppError> {
        // Check cache first
        if let Some(cached) = self.cache.get(city).await {
            info!(city = %city, "Cache hit");
            return Ok(cached);
        }

        // Rate limiting: acquire permit
        let _permit = self
            .rate_limiter
            .acquire()
            .await
            .map_err(|e| AppError::internal(format!("Rate limiter error: {}", e)))?;

        // Debounce: ensure minimum time between requests
        self.debounce().await;

        info!(city = %city, "Fetching weather from API");

        // Build URL with city coordinates (simplified - better solution would be to use geocoding, but for now we'll use this)
        let url = format!(
            "{}?latitude={}&longitude={}&current=temperature_2m,relative_humidity_2m,wind_speed_10m,weather_code",
            self.base_url,
            self.get_latitude(city),
            self.get_longitude(city)
        );

        let response: OpenMeteoResponse = self.http_client.get_json(&url).await?;

        let weather = WeatherData {
            temperature: response.current.temperature_2m,
            condition: self.weather_code_to_condition(response.current.weather_code.unwrap_or(0)),
            humidity: response.current.relative_humidity_2m,
            wind_speed: response.current.wind_speed_10m,
        };

        // Cache the result
        self.cache.set(city.to_string(), weather.clone()).await;

        Ok(weather)
    }

    async fn debounce(&self) {
        let mut last_request = self.last_request_time.lock().await;
        if let Some(last) = *last_request {
            let elapsed = last.elapsed();
            if elapsed < self.min_request_interval {
                let wait_time = self.min_request_interval - elapsed;
                warn!(wait_ms = wait_time.as_millis(), "Debouncing request");
                tokio::time::sleep(wait_time).await;
            }
        }
        *last_request = Some(Instant::now());
    }

    fn get_latitude(&self, city: &str) -> f64 {
        match city.to_lowercase().as_str() {
            "london" => 51.5074,
            "tokyo" => 35.6762,
            "new york" | "new+york" => 40.7128,
            "paris" => 48.8566,
            "berlin" => 52.5200,
            "moscow" => 55.7558,
            "beijing" => 39.9042,
            "sydney" => -33.8688,
            "rio de janeiro" | "rio+de+janeiro" => -22.9068,
            "cairo" => 30.0444,
            _ => 0.0, // Default fallback
        }
    }

    fn get_longitude(&self, city: &str) -> f64 {
        match city.to_lowercase().as_str() {
            "london" => -0.1278,
            "tokyo" => 139.6503,
            "new york" | "new+york" => -74.0060,
            "paris" => 2.3522,
            "berlin" => 13.4050,
            "moscow" => 37.6173,
            "beijing" => 116.4074,
            "sydney" => 151.2093,
            "rio de janeiro" | "rio+de+janeiro" => -43.1729,
            "cairo" => 31.2357,
            _ => 0.0,
        }
    }

    fn weather_code_to_condition(&self, code: u32) -> String {
        match code {
            0 => "Clear sky",
            1..=3 => "Partly cloudy",
            45 | 48 => "Foggy",
            51 | 53 | 55 => "Drizzle",
            61 | 63 | 65 => "Rain",
            71 | 73 | 75 => "Snow",
            80..=82 => "Rain showers",
            85 | 86 => "Snow showers",
            95 => "Thunderstorm",
            96 | 99 => "Thunderstorm with hail",
            _ => "Unknown",
        }
        .to_string()
    }
}
