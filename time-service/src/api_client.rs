use crate::cache::TimeCache;
use common::errors::AppError;
use common::http_client::HttpClient;
use common::models::TimeData;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument, warn};

#[derive(Debug, Serialize, Deserialize)]
struct WorldTimeApiResponse {
    datetime: String,
    timezone: String,
    unixtime: i64,
}

pub struct WorldTimeApiClient {
    http_client: HttpClient,
    cache: Arc<TimeCache>,
    base_url: String,
}

impl WorldTimeApiClient {
    pub fn new(cache: Arc<TimeCache>, base_url: String) -> Self {
        Self {
            http_client: HttpClient::default(),
            cache,
            base_url,
        }
    }

    #[instrument(skip(self), fields(city = %city))]
    pub async fn get_time(&self, city: &str) -> Result<TimeData, AppError> {
        // Check cache first
        if let Some(cached) = self.cache.get(city).await {
            info!(city = %city, "Cache hit");
            return Ok(cached);
        }

        info!(city = %city, "Fetching time from API");

        let timezone = self.city_to_timezone(city);
        let encoded_timezone = urlencoding::encode(timezone);
        let url = format!("{}/{}", self.base_url, encoded_timezone);

        let response: WorldTimeApiResponse = self.http_client.get_json(&url).await?;

        let time_data = TimeData {
            datetime: response.datetime,
            timezone: response.timezone,
            unix_time: response.unixtime,
        };

        // Cache the result
        self.cache.set(city.to_string(), time_data.clone()).await;

        Ok(time_data)
    }

    pub async fn prefill_cache(&self) -> Result<(), AppError> {
        info!("Starting cache prefill for common cities");

        let cities = [
            "London",
            "Tokyo",
            "New York",
            "Paris",
            "Berlin",
            "Sydney",
            "Los Angeles",
            "Chicago",
            "Toronto",
            "Singapore",
        ];

        let mut cached = 0;
        let mut errors = 0;

        for city in cities {
            match self.get_time(city).await {
                Ok(_) => cached += 1,
                Err(e) => {
                    warn!(city = %city, error = %e, "Failed to fetch time during prefill");
                    errors += 1;
                }
            }
        }

        info!(cached, errors, "Cache prefill completed");

        if errors > 0 {
            warn!(errors, "Some cities failed to cache during prefill");
        }

        Ok(())
    }

    fn city_to_timezone(&self, city: &str) -> &str {
        match city.to_lowercase().as_str() {
            "london" => "Europe/London",
            "tokyo" => "Asia/Tokyo",
            "new york" | "new+york" => "America/New_York",
            "paris" => "Europe/Paris",
            "berlin" => "Europe/Berlin",
            "moscow" => "Europe/Moscow",
            "beijing" => "Asia/Shanghai",
            "sydney" => "Australia/Sydney",
            "rio de janeiro" | "rio+de+janeiro" => "America/Sao_Paulo",
            "cairo" => "Africa/Cairo",
            "los angeles" | "los+angeles" => "America/Los_Angeles",
            "chicago" => "America/Chicago",
            "toronto" => "America/Toronto",
            "mexico city" | "mexico+city" => "America/Mexico_City",
            "são paulo" | "sao+paulo" => "America/Sao_Paulo",
            "buenos aires" | "buenos+aires" => "America/Argentina/Buenos_Aires",
            "dubai" => "Asia/Dubai",
            "mumbai" => "Asia/Kolkata",
            "singapore" => "Asia/Singapore",
            "hong kong" | "hong+kong" => "Asia/Hong_Kong",
            _ => "UTC",
        }
    }
}
