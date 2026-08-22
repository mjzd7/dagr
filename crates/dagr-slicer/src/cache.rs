use dagr_core::MinimalContextSlice;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct CacheEntry<T> {
    data: T,
    inserted_at: Instant,
}

/// Dual positive and negative query cache with TTL expiration
pub struct SlicerQueryCache {
    positive_cache: Mutex<HashMap<[u8; 32], CacheEntry<MinimalContextSlice>>>,
    negative_cache: Mutex<HashMap<[u8; 32], CacheEntry<String>>>,
    ttl: Duration,
}

impl SlicerQueryCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            positive_cache: Mutex::new(HashMap::new()),
            negative_cache: Mutex::new(HashMap::new()),
            ttl,
        }
    }

    pub fn default_15m() -> Self {
        Self::new(Duration::from_secs(15 * 60))
    }

    fn compute_key(file_path: &str, symbol: &str) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(file_path.as_bytes());
        hasher.update(b"::");
        hasher.update(symbol.as_bytes());
        *hasher.finalize().as_bytes()
    }

    pub fn get_positive(&self, file_path: &str, symbol: &str) -> Option<MinimalContextSlice> {
        let key = Self::compute_key(file_path, symbol);
        let mut cache = self.positive_cache.lock().unwrap();

        if let Some(entry) = cache.get(&key) {
            if entry.inserted_at.elapsed() <= self.ttl {
                return Some(entry.data.clone());
            } else {
                cache.remove(&key);
            }
        }
        None
    }

    pub fn set_positive(&self, file_path: &str, symbol: &str, slice: MinimalContextSlice) {
        let key = Self::compute_key(file_path, symbol);
        let mut cache = self.positive_cache.lock().unwrap();
        cache.insert(
            key,
            CacheEntry {
                data: slice,
                inserted_at: Instant::now(),
            },
        );
    }

    pub fn get_negative(&self, file_path: &str, symbol: &str) -> Option<String> {
        let key = Self::compute_key(file_path, symbol);
        let mut cache = self.negative_cache.lock().unwrap();

        if let Some(entry) = cache.get(&key) {
            if entry.inserted_at.elapsed() <= self.ttl {
                return Some(entry.data.clone());
            } else {
                cache.remove(&key);
            }
        }
        None
    }

    pub fn set_negative(&self, file_path: &str, symbol: &str, error_msg: String) {
        let key = Self::compute_key(file_path, symbol);
        let mut cache = self.negative_cache.lock().unwrap();
        cache.insert(
            key,
            CacheEntry {
                data: error_msg,
                inserted_at: Instant::now(),
            },
        );
    }
}
