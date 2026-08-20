pub mod acl;
pub mod auth;
pub mod client;
pub mod graph_writer;
pub mod outbox;
pub mod webhook;

pub use acl::{AntiCorruptionLayer, CodeGraphDelta, CodeGraphEdge, EdgeType};
pub use auth::OrgCredentials;
pub use client::{CloudSyncClient, SyncResult, ZeroPiiTelemetryPacket};
pub use graph_writer::CypherQueryBuilder;
pub use outbox::OutboxEventRecord;
pub use webhook::{GitPushPayload, WebhookVerifier};

#[cfg(test)]
mod tests {
    use super::*;
    use dagr_core::Result;
    use uuid::Uuid;

    #[test]
    fn test_cloud_pipeline_end_to_end() -> Result<()> {
        // 1. Simulate Webhook Ingestion
        let payload = GitPushPayload {
            repository: "mjzd7/dagr".into(),
            branch: "refs/heads/main".into(),
            commit_sha: "a1b2c3d4e5f6".into(),
            author: "mjzd7".into(),
            commit_message: "feat: add payment processor".into(),
            modified_files: vec!["src/billing.ts".into()],
            added_files: vec![],
            removed_files: vec![],
        };

        let idempotency_key = WebhookVerifier::derive_idempotency_key(&payload);
        assert_eq!(idempotency_key.len(), 64);

        // 2. Normalize Diff via Anti-Corruption Layer (ACL)
        let source_code = "export function chargeCreditCard(amount: number) { return true; }";
        let nodes = AntiCorruptionLayer::parse_file_delta(
            &payload.repository,
            "src/billing.ts",
            source_code,
        )?;
        assert_eq!(nodes.len(), 1);

        // 3. Format Cypher Ingestion Query for Memgraph
        let (cypher, params) = CypherQueryBuilder::build_node_upsert_query(&nodes[0]);
        assert!(cypher.contains("MERGE (s:Symbol {id: $id})"));
        assert_eq!(params["name"], "chargeCreditCard");

        // 4. Create Transactional Outbox Event for Debezium CDC
        let org_id = Uuid::new_v4();
        let repo_id = Uuid::new_v4();
        let outbox = OutboxEventRecord::new(
            org_id,
            repo_id,
            "CommitIngested",
            &payload.commit_sha,
            serde_json::json!({ "total_symbols": nodes.len() }),
            &idempotency_key,
        );

        assert_eq!(outbox.status, "PENDING");
        assert_eq!(outbox.event_type, "CommitIngested");

        Ok(())
    }
}
