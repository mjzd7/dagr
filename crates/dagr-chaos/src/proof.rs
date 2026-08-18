use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProofOfCorrectness {
    pub proof_id: String,
    pub commit_sha: String,
    pub timestamp: u64,
    pub status: String,               // "VERIFIED_GREEN" | "QUARANTINED"
    pub token_compression_ratio: f32, // e.g. 0.965 (96.5%)
    pub boundary_violations_count: usize,
    pub chaos_resilience_score: f32,     // 1.0 = perfect under faults
    pub cryptographic_signature: String, // Blake3 HMAC chained signature
}

pub struct ProofGenerator;

impl ProofGenerator {
    /// Assembles verified metrics and signs a cryptographic Proof-of-Correctness badge
    pub fn generate_proof(
        commit_sha: &str,
        token_compression_ratio: f32,
        boundary_violations_count: usize,
        chaos_resilience_score: f32,
        secret_key: &str,
    ) -> ProofOfCorrectness {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let status = if boundary_violations_count == 0 && chaos_resilience_score >= 0.95 {
            "VERIFIED_GREEN".to_string()
        } else {
            "QUARANTINED".to_string()
        };

        // Create deterministic audit payload string
        let raw_payload = format!(
            "{}:{}:{}:{}:{:.3}:{:.3}",
            commit_sha,
            timestamp,
            status,
            boundary_violations_count,
            token_compression_ratio,
            chaos_resilience_score
        );

        // Sign with Blake3 keyed hash
        let key_bytes = blake3::hash(secret_key.as_bytes());
        let signature = blake3::keyed_hash(key_bytes.as_bytes(), raw_payload.as_bytes())
            .to_hex()
            .to_string();

        let proof_id = format!("proof_{}", &signature[..16]);

        ProofOfCorrectness {
            proof_id,
            commit_sha: commit_sha.to_string(),
            timestamp,
            status,
            token_compression_ratio,
            boundary_violations_count,
            chaos_resilience_score,
            cryptographic_signature: signature,
        }
    }

    /// Verifies the cryptographic signature of an existing proof
    pub fn verify_signature(proof: &ProofOfCorrectness, secret_key: &str) -> bool {
        let raw_payload = format!(
            "{}:{}:{}:{}:{:.3}:{:.3}",
            proof.commit_sha,
            proof.timestamp,
            proof.status,
            proof.boundary_violations_count,
            proof.token_compression_ratio,
            proof.chaos_resilience_score
        );

        let key_bytes = blake3::hash(secret_key.as_bytes());
        let expected_sig = blake3::keyed_hash(key_bytes.as_bytes(), raw_payload.as_bytes())
            .to_hex()
            .to_string();

        proof.cryptographic_signature == expected_sig
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_of_correctness_lifecycle() {
        let secret = "dagr_enterprise_master_secret";
        let proof = ProofGenerator::generate_proof("b7f83e291a84f", 0.971, 0, 1.0, secret);

        assert_eq!(proof.status, "VERIFIED_GREEN");
        assert!(proof.proof_id.starts_with("proof_"));
        assert!(ProofGenerator::verify_signature(&proof, secret));

        // Test signature tampering detection
        let mut tampered = proof.clone();
        tampered.token_compression_ratio = 0.500;
        assert!(!ProofGenerator::verify_signature(&tampered, secret));
    }
}
