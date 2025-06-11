use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct ExpirationTable {
    pub expirations: RwLock<HashMap<String, Instant>>,
}

impl ExpirationTable {
    pub fn new() -> Self {
        Self {
            expirations: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_expiration(&self, key: String, ttl: Duration) {
        let expire_at = Instant::now() + ttl;
        self.expirations.write().insert(key, expire_at);
    }

    pub fn is_expired(&self, key: &str) -> bool {
        if let Some(&instant) = self.expirations.read().get(key) {
            if Instant::now() >= instant {
                return true;
            }
        }
        false
    }

    pub fn get_expired_keys(&self) -> Vec<String> {
        let now = Instant::now();
        let expirations = self.expirations.read();
        expirations
            .iter()
            .filter(|(_, expiry)| now >= **expiry)  // ✅ double deref
            .map(|(k, _)| k.clone())
            .collect()
    }

    pub fn remove(&self, key: &str) {
        self.expirations.write().remove(key);
    }
}
