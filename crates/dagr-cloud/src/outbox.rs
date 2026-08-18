use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutboxEventRecord {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub repository_id: Uuid,
    pub event_type: String,
    pub aggregate_id: String,
    pub payload: serde_json::Value,
    pub idempotency_key: String,
    pub status: String,
}

impl OutboxEventRecord {
    pub fn new(
        organization_id: Uuid,
        repository_id: Uuid,
        event_type: &str,
        aggregate_id: &str,
        payload: serde_json::Value,
        idempotency_key: &str,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            organization_id,
            repository_id,
            event_type: event_type.to_string(),
            aggregate_id: aggregate_id.to_string(),
            payload,
            idempotency_key: idempotency_key.to_string(),
            status: "PENDING".to_string(),
        }
    }
}
