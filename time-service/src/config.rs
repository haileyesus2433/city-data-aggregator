use std::env;

pub struct Config {
    pub port: u16,
    pub world_time_api_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3003),
            world_time_api_url: env::var("WORLD_TIME_API_URL")
                .unwrap_or_else(|_| "http://worldtimeapi.org/api/timezone".to_string()),
        }
    }
}

