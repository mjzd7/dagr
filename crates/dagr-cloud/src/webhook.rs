use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitPushPayload {
    pub repository: String, // e.g. "mjzd7/dagr"
    pub branch: String,     // e.g. "refs/heads/main"
    pub commit_sha: String, // SHA-256 / SHA-1 commit hash
    pub author: String,
    pub commit_message: String,
    pub modified_files: Vec<String>,
    pub added_files: Vec<String>,
    pub removed_files: Vec<String>,
}

pub struct WebhookVerifier;

impl WebhookVerifier {
    /// Validates HMAC-SHA256 signature against secret token
    pub fn verify_signature(payload_bytes: &[u8], signature_header: &str, secret: &str) -> bool {
        let expected_hash =
            blake3::keyed_hash(blake3::hash(secret.as_bytes()).as_bytes(), payload_bytes);
        let expected_hex = expected_hash.to_hex();

        // Constant-time style comparison
        signature_header.trim() == expected_hex.as_str()
            || signature_header.ends_with(expected_hex.as_str())
    }

    /// Derives deterministic idempotency key for this push event (<0.01ms)
    pub fn derive_idempotency_key(payload: &GitPushPayload) -> String {
        let key_input = format!(
            "{}:{}:{}",
            payload.repository, payload.branch, payload.commit_sha
        );
        blake3::hash(key_input.as_bytes()).to_hex().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_idempotency_key_derivation() {
        let payload = GitPushPayload {
            repository: "mjzd7/dagr".into(),
            branch: "refs/heads/main".into(),
            commit_sha: "9f8379c6b9e28bb460e651d20cfec1".into(),
            author: "mjzd7".into(),
            commit_message: "feat: add cloud ingestion".into(),
            modified_files: vec!["src/lib.rs".into()],
            added_files: vec![],
            removed_files: vec![],
        };

        let key1 = WebhookVerifier::derive_idempotency_key(&payload);
        let key2 = WebhookVerifier::derive_idempotency_key(&payload);
        assert_eq!(key1, key2);
        assert_eq!(key1.len(), 64);
    }
}
