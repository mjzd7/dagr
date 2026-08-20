//! Distributed Blake3 Remote Monorepo AST Cache Client & Protocol

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoteCacheConfig {
    pub endpoint_url: String,
    pub timeout_ms: u64,
    pub enabled: bool,
}

impl Default for RemoteCacheConfig {
    fn default() -> Self {
        Self {
            endpoint_url: "http://127.0.0.1:4444".to_string(),
            timeout_ms: 15,
            enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CachedAstRecord {
    pub content_hash: String,
    pub file_path: String,
    pub language: String,
    pub symbols: Vec<String>,
    pub contracts: Vec<String>,
    pub token_count: usize,
    pub cached_at: i64,
}

/// Computes a deterministic Blake3 content hash for zero-stale cache keys
pub fn hash_file_content(content: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(content.as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// In-memory cache store for the local/remote daemon
#[derive(Debug, Default, Clone)]
pub struct AstCacheStore {
    store: Arc<Mutex<HashMap<String, CachedAstRecord>>>,
}

impl AstCacheStore {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, content_hash: &str) -> Option<CachedAstRecord> {
        let lock = self.store.lock().ok()?;
        lock.get(content_hash).cloned()
    }

    pub fn put(&self, record: CachedAstRecord) {
        if let Ok(mut lock) = self.store.lock() {
            lock.insert(record.content_hash.clone(), record);
        }
    }

    pub fn count(&self) -> usize {
        self.store.lock().map(|l| l.len()).unwrap_or(0)
    }

    pub fn clear(&self) {
        if let Ok(mut lock) = self.store.lock() {
            lock.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_content_hashing() {
        let code1 = "export function pay() { return true; }";
        let code2 = "export function pay() { return true; }";
        let code3 = "export function pay() { return false; }";

        let h1 = hash_file_content(code1);
        let h2 = hash_file_content(code2);
        let h3 = hash_file_content(code3);

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_ast_cache_store_crud() {
        let store = AstCacheStore::new();
        let hash = hash_file_content("test code");

        let record = CachedAstRecord {
            content_hash: hash.clone(),
            file_path: "src/test.ts".into(),
            language: "typescript".into(),
            symbols: vec!["pay".into()],
            contracts: vec!["interface Payment".into()],
            token_count: 42,
            cached_at: 1724108400,
        };

        store.put(record.clone());
        assert_eq!(store.count(), 1);

        let retrieved = store.get(&hash);
        assert_eq!(retrieved, Some(record));

        store.clear();
        assert_eq!(store.count(), 0);
    }
}
