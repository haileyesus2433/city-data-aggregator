use common::models::TimeData;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TimeCache {
    cache: Arc<RwLock<HashMap<String, TimeData>>>,
}

impl TimeCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, city: &str) -> Option<TimeData> {
        let cache = self.cache.read().await;
        cache.get(city).cloned()
    }

    pub async fn set(&self, city: String, data: TimeData) {
        let mut cache = self.cache.write().await;
        cache.insert(city, data);
    }
}

impl Default for TimeCache {
    fn default() -> Self {
        Self::new()
    }
}
