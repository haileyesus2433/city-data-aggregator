use common::models::WeatherData;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

struct CacheEntry {
    data: WeatherData,
    expires_at: Instant,
}

pub struct WeatherCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    ttl: Duration,
}

impl WeatherCache {
    pub fn with_ttl(ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    pub async fn get(&self, city: &str) -> Option<WeatherData> {
        let cache = self.cache.read().await;
        // Case-insensitive lookup
        if let Some(entry) = cache.get(&city.to_lowercase())
            && entry.expires_at > Instant::now()
        {
            return Some(entry.data.clone());
        }
        None
    }

    pub async fn set(&self, city: String, data: WeatherData) {
        let mut cache = self.cache.write().await;
        // Store with lowercase key for case-insensitive matching
        cache.insert(
            city.to_lowercase(),
            CacheEntry {
                data,
                expires_at: Instant::now() + self.ttl,
            },
        );
    }
}
