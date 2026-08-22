use crate::error::{DagrError, Result};
use crate::event_store::RunId;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    Live,
    Replay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectRecord {
    pub effect_id: Uuid,
    pub run_id: RunId,
    pub step_index: u32,
    pub effect_type: String,
    pub input_blake3: [u8; 32],
    pub output_payload: Vec<u8>,
    pub timestamp_utc: u64,
}

/// Persistent Journal of non-deterministic side-effects
pub struct EffectJournal {
    conn: Mutex<Connection>,
}

impl EffectJournal {
    pub fn new(conn: Connection) -> Result<Self> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE IF NOT EXISTS effect_journal (
                 effect_id TEXT PRIMARY KEY,
                 run_id TEXT NOT NULL,
                 step_index INTEGER NOT NULL,
                 effect_type TEXT NOT NULL,
                 input_blake3 BLOB NOT NULL,
                 output_payload BLOB NOT NULL,
                 timestamp_utc INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_effect_run_step ON effect_journal(run_id, step_index);",
        )
        .map_err(|e| DagrError::Internal(format!("Failed to initialize effect journal: {}", e)))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| DagrError::Internal(format!("Failed to open in-memory SQLite: {}", e)))?;
        Self::new(conn)
    }

    pub fn record_effect(&self, record: &EffectRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO effect_journal (effect_id, run_id, step_index, effect_type, input_blake3, output_payload, timestamp_utc)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                record.effect_id.to_string(),
                record.run_id.to_string(),
                record.step_index as i64,
                record.effect_type,
                &record.input_blake3[..],
                &record.output_payload[..],
                record.timestamp_utc as i64,
            ]
        ).map_err(|e| DagrError::Internal(format!("Failed to record effect: {}", e)))?;

        Ok(())
    }

    pub fn fetch_replay_effect(
        &self,
        run_id: RunId,
        step_index: u32,
        expected_hash: &[u8; 32],
    ) -> Result<EffectRecord> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT effect_id, run_id, step_index, effect_type, input_blake3, output_payload, timestamp_utc
             FROM effect_journal
             WHERE run_id = ?1 AND step_index = ?2"
        ).map_err(|e| DagrError::Internal(e.to_string()))?;

        let mut rows = stmt
            .query(params![run_id.to_string(), step_index as i64])
            .map_err(|e| DagrError::Internal(e.to_string()))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| DagrError::Internal(e.to_string()))?
        {
            let effect_id_str: String =
                row.get(0).map_err(|e| DagrError::Internal(e.to_string()))?;
            let run_id_str: String = row.get(1).map_err(|e| DagrError::Internal(e.to_string()))?;
            let step: i64 = row.get(2).map_err(|e| DagrError::Internal(e.to_string()))?;
            let eff_type: String = row.get(3).map_err(|e| DagrError::Internal(e.to_string()))?;
            let stored_hash: Vec<u8> =
                row.get(4).map_err(|e| DagrError::Internal(e.to_string()))?;
            let payload: Vec<u8> = row.get(5).map_err(|e| DagrError::Internal(e.to_string()))?;
            let ts: i64 = row.get(6).map_err(|e| DagrError::Internal(e.to_string()))?;

            if stored_hash != expected_hash {
                return Err(DagrError::Internal(format!(
                    "Replay Drift Detected at step {}: expected hash does not match recorded journal hash",
                    step_index
                )));
            }

            let mut hash_arr = [0u8; 32];
            hash_arr.copy_from_slice(&stored_hash);

            Ok(EffectRecord {
                effect_id: Uuid::parse_str(&effect_id_str).unwrap_or_else(|_| Uuid::new_v4()),
                run_id: RunId(Uuid::parse_str(&run_id_str).unwrap_or_else(|_| Uuid::new_v4())),
                step_index: step as u32,
                effect_type: eff_type,
                input_blake3: hash_arr,
                output_payload: payload,
                timestamp_utc: ts as u64,
            })
        } else {
            Err(DagrError::NotFound(format!(
                "No recorded effect found for run {} at step {}",
                run_id, step_index
            )))
        }
    }
}

/// Deterministic Replay Cursor executing live effects vs serving from journal
pub struct ReplayCursor {
    journal: EffectJournal,
    mode: ExecutionMode,
}

impl ReplayCursor {
    pub fn new(journal: EffectJournal, mode: ExecutionMode) -> Self {
        Self { journal, mode }
    }

    pub fn execute_or_replay<F>(
        &self,
        run_id: RunId,
        step_index: u32,
        effect_type: &str,
        input_data: &[u8],
        live_fn: F,
    ) -> Result<Vec<u8>>
    where
        F: FnOnce() -> Result<Vec<u8>>,
    {
        let input_hash = *blake3::hash(input_data).as_bytes();

        match self.mode {
            ExecutionMode::Live => {
                let output = live_fn()?;
                let record = EffectRecord {
                    effect_id: Uuid::new_v4(),
                    run_id,
                    step_index,
                    effect_type: effect_type.to_string(),
                    input_blake3: input_hash,
                    output_payload: output.clone(),
                    timestamp_utc: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                };
                self.journal.record_effect(&record)?;
                Ok(output)
            }
            ExecutionMode::Replay => {
                let record = self
                    .journal
                    .fetch_replay_effect(run_id, step_index, &input_hash)?;
                Ok(record.output_payload)
            }
        }
    }
}
