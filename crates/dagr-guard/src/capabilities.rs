use dagr_core::{DagrError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
    ReadAST,
    SliceSymbol,
    MutateCoWShadow,
    ExecuteSubprocess,
    PublishMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub grant_id: Uuid,
    pub run_id: Uuid,
    pub tenant_id: String,
    pub permissions: Vec<Permission>,
    pub expires_at_utc: u64,
    pub signature: [u8; 32],
}

impl CapabilityGrant {
    pub fn new_signed(
        run_id: Uuid,
        tenant_id: &str,
        permissions: Vec<Permission>,
        ttl_seconds: u64,
        hmac_secret: &[u8],
    ) -> Self {
        let grant_id = Uuid::new_v4();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let expires_at_utc = now + ttl_seconds;

        let key = Self::derive_key(hmac_secret);
        let mut hasher = blake3::Hasher::new_keyed(&key);
        hasher.update(grant_id.as_bytes());
        hasher.update(run_id.as_bytes());
        hasher.update(tenant_id.as_bytes());
        hasher.update(&expires_at_utc.to_le_bytes());
        let signature = *hasher.finalize().as_bytes();

        Self {
            grant_id,
            run_id,
            tenant_id: tenant_id.to_string(),
            permissions,
            expires_at_utc,
            signature,
        }
    }

    fn derive_key(secret: &[u8]) -> [u8; 32] {
        let mut key = [0u8; 32];
        let hash = blake3::hash(secret);
        key.copy_from_slice(hash.as_bytes());
        key
    }

    pub fn verify(&self, hmac_secret: &[u8]) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now > self.expires_at_utc {
            return Err(DagrError::RuleViolation(format!(
                "Capability grant {} expired {} seconds ago",
                self.grant_id,
                now - self.expires_at_utc
            )));
        }

        let key = Self::derive_key(hmac_secret);
        let mut hasher = blake3::Hasher::new_keyed(&key);

        hasher.update(self.grant_id.as_bytes());
        hasher.update(self.run_id.as_bytes());
        hasher.update(self.tenant_id.as_bytes());
        hasher.update(&self.expires_at_utc.to_le_bytes());
        let expected_sig = *hasher.finalize().as_bytes();

        if self.signature != expected_sig {
            return Err(DagrError::RuleViolation(
                "Capability token cryptographic signature mismatch".into(),
            ));
        }

        Ok(())
    }

    pub fn has_permission(&self, perm: &Permission) -> bool {
        self.permissions.contains(perm)
    }
}

/// Zero-Trust Credential Broker
pub struct CredentialBroker {
    secrets: Mutex<HashMap<String, String>>,
}

impl Default for CredentialBroker {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialBroker {
    pub fn new() -> Self {
        Self {
            secrets: Mutex::new(HashMap::new()),
        }
    }

    pub fn register_secret(&self, handle_uri: &str, raw_secret: &str) {
        let mut map = self.secrets.lock().unwrap();
        map.insert(handle_uri.to_string(), raw_secret.to_string());
    }

    pub fn resolve_handle(
        &self,
        grant: &CapabilityGrant,
        handle_uri: &str,
        hmac_secret: &[u8],
    ) -> Result<String> {
        grant.verify(hmac_secret)?;

        if !grant.has_permission(&Permission::ExecuteSubprocess) {
            return Err(DagrError::RuleViolation(format!(
                "Capability grant {} lacks ExecuteSubprocess permission required to resolve credential handles",
                grant.grant_id
            )));
        }

        let map = self.secrets.lock().unwrap();
        map.get(handle_uri).cloned().ok_or_else(|| {
            DagrError::NotFound(format!(
                "Credential handle '{}' not found in broker",
                handle_uri
            ))
        })
    }
}
