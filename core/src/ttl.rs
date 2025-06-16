use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};

#[derive(Debug)]
pub struct ExpirationTable {
    pub expirations: RwLock<HashMap<String, u64>>,
}

impl ExpirationTable {
    pub fn new() -> Self {
        Self {
            expirations: RwLock::new(HashMap::new()),
        }
    }

    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn set(&self, key: String, ttl: Duration) {
        let expire_at = Self::current_timestamp() + ttl.as_secs();
        self.expirations.write().insert(key, expire_at);
    }

    pub fn is_expired(&self, key: &str) -> bool {
        if let Some(&timestamp) = self.expirations.read().get(key) {
            return Self::current_timestamp() >= timestamp;
        }
        false
    }

    pub fn get_expired_keys(&self) -> Vec<String> {
        let now = Self::current_timestamp();
        let expirations = self.expirations.read();
        expirations
            .iter()
            .filter(|(_, expiry)| now >= **expiry)
            .map(|(k, _)| k.clone())
            .collect()
    }


    pub fn remove(&self, key: &str) {
        self.expirations.write().remove(key);
    }
}
