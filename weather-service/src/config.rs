use std::env;

pub struct Config {
    pub port: u16,
    pub open_meteo_url: String,
    pub cache_ttl_seconds: u64,
    pub rate_limit_per_minute: u32,
    pub time_service_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3002),
            open_meteo_url: env::var("OPEN_METEO_URL")
                .unwrap_or_else(|_| "https://api.open-meteo.com/v1/forecast".to_string()),
            cache_ttl_seconds: env::var("CACHE_TTL_SECONDS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300), // 5 minutes default
            rate_limit_per_minute: env::var("RATE_LIMIT_PER_MINUTE")
                .ok()
                .and_then(|r| r.parse().ok())
                .unwrap_or(60),
            time_service_url: env::var("TIME_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3003".to_string()),
        }
    }
}

