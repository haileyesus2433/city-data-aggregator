use common::errors::AppError;
use common::http_client::HttpClient;
use common::models::{AggregateResponse, CityData, ResponseSummary, TimeData, WeatherData};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, error, info, instrument, warn};

use crate::api_client::OpenMeteoClient;

pub struct Aggregator {
    weather_client: Arc<OpenMeteoClient>,
    time_service_url: String,
    http_client: Arc<HttpClient>,
    semaphore: Arc<Semaphore>,
    cancellation_token: CancellationToken,
}

impl Aggregator {
    pub fn new(
        weather_client: Arc<OpenMeteoClient>,
        time_service_url: String,
        cancellation_token: CancellationToken,
    ) -> Self {
        Self {
            weather_client,
            time_service_url,
            http_client: Arc::new(HttpClient::default()),
            semaphore: Arc::new(Semaphore::new(10)), // Max 10 in-flight city tasks
            cancellation_token,
        }
    }

    #[instrument(skip(self), fields(city_count = cities.len()))]
    pub async fn aggregate(&self, cities: Vec<String>) -> Result<AggregateResponse, AppError> {
        if cities.is_empty() || cities.len() > 20 {
            return Err(AppError::validation("Must provide between 1 and 20 cities"));
        }

        info!(count = cities.len(), "Starting aggregation for cities");

        // Spawn all city tasks concurrently, respecting the semaphore limit
        let mut handles = Vec::with_capacity(cities.len());

        for city in cities {
            let semaphore = self.semaphore.clone();
            let weather_client = self.weather_client.clone();
            let time_service_url = self.time_service_url.clone();
            let http_client = self.http_client.clone();
            let cancel = self.cancellation_token.clone();

            let handle = tokio::spawn(
                async move {
                    // Check for cancellation before starting
                    if cancel.is_cancelled() {
                        return CityData {
                            city,
                            weather: None,
                            time: None,
                            errors: vec!["Request cancelled".to_string()],
                        };
                    }

                    // Acquire semaphore permit (this queues if more than 10 are in-flight)
                    let _permit = match semaphore.acquire().await {
                        Ok(p) => p,
                        Err(_) => {
                            return CityData {
                                city,
                                weather: None,
                                time: None,
                                errors: vec!["Semaphore closed".to_string()],
                            };
                        }
                    };

                    // Process city with cancellation support
                    tokio::select! {
                        result = process_city(&city, &weather_client, &time_service_url, &http_client) => result,
                        _ = cancel.cancelled() => {
                            CityData {
                                city,
                                weather: None,
                                time: None,
                                errors: vec!["Request cancelled".to_string()],
                            }
                        }
                    }
                }
                .in_current_span(),
            );

            handles.push(handle);
        }

        // Collect all results
        let mut city_results = Vec::with_capacity(handles.len());
        let mut successful = 0;
        let mut failed = 0;

        for handle in handles {
            match handle.await {
                Ok(city_data) => {
                    if city_data.errors.is_empty() {
                        successful += 1;
                    } else {
                        failed += 1;
                    }
                    city_results.push(city_data);
                }
                Err(e) => {
                    error!(error = %e, "Task join error");
                    failed += 1;
                }
            }
        }

        let total = city_results.len();

        info!(total, successful, failed, "Aggregation completed");

        Ok(AggregateResponse {
            cities: city_results,
            summary: ResponseSummary {
                total,
                successful,
                failed,
            },
        })
    }
}

#[instrument(skip(weather_client, http_client), fields(city = %city))]
async fn process_city(
    city: &str,
    weather_client: &OpenMeteoClient,
    time_service_url: &str,
    http_client: &HttpClient,
) -> CityData {
    let mut errors = Vec::new();
    let mut weather: Option<WeatherData> = None;
    let mut time: Option<TimeData> = None;

    // Fetch weather and time in parallel with timeout
    let weather_future = fetch_weather(city, weather_client);
    let time_future = fetch_time(city, time_service_url, http_client);

    let (weather_result, time_result) = tokio::join!(weather_future, time_future);

    match weather_result {
        Ok(w) => weather = Some(w),
        Err(e) => {
            warn!(city = %city, error = %e, "Weather fetch failed");
            errors.push(format!("Weather: {}", e));
        }
    }

    match time_result {
        Ok(t) => time = Some(t),
        Err(e) => {
            warn!(city = %city, error = %e, "Time fetch failed");
            errors.push(format!("Time: {}", e));
        }
    }

    CityData {
        city: city.to_string(),
        weather,
        time,
        errors,
    }
}

async fn fetch_weather(
    city: &str,
    weather_client: &OpenMeteoClient,
) -> Result<WeatherData, AppError> {
    timeout(Duration::from_secs(10), weather_client.get_weather(city))
        .await
        .map_err(|_| AppError::timeout(format!("Weather fetch for {} timed out", city)))?
}

async fn fetch_time(
    city: &str,
    time_service_url: &str,
    http_client: &HttpClient,
) -> Result<TimeData, AppError> {
    let url = format!("{}/api/time/{}", time_service_url, city);

    #[derive(Deserialize)]
    struct TimeResponse {
        datetime: String,
        timezone: String,
        unix_time: i64,
    }

    timeout(Duration::from_secs(10), async {
        let response: TimeResponse = http_client.get_json(&url).await?;
        Ok(TimeData {
            datetime: response.datetime,
            timezone: response.timezone,
            unix_time: response.unix_time,
        })
    })
    .await
    .map_err(|_| AppError::timeout(format!("Time fetch for {} timed out", city)))?
}
