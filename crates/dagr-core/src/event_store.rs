use crate::error::{DagrError, Result};
use async_trait::async_trait;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Unique identifier for an agent run
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RunId(pub Uuid);

impl std::fmt::Display for RunId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Monotonically increasing fencing token protecting worker leases
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct FencingToken(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunState {
    pub run_id: RunId,
    pub tenant_id: String,
    pub status: RunStatus,
    pub current_step: u32,
    pub tokens_consumed: usize,
    pub max_tokens: usize,
    pub max_fencing_token: FencingToken,
    pub active_lease_holder: Option<String>,
    pub lease_expires_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEvent {
    pub event_id: Uuid,
    pub run_id: RunId,
    pub sequence_number: u64,
    pub fencing_token: FencingToken,
    pub payload: EventPayload,
    pub timestamp_utc: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayload {
    RunCreated {
        tenant_id: String,
        max_tokens: usize,
    },
    LeaseAcquired {
        worker_id: String,
        lease_duration_ms: u64,
    },
    StepScheduled {
        step_index: u32,
        target_symbol: String,
    },
    StepCompleted {
        step_index: u32,
        tokens_used: usize,
    },
    StepFailed {
        step_index: u32,
        error_message: String,
    },
    RunCompleted {
        status: RunStatus,
    },
}

/// Pure state transition fold function: S_t = fold(S_0, [e_1, ..., e_t])
pub fn fold_events(initial: Option<RunState>, events: &[RunEvent]) -> Result<RunState> {
    let mut state = initial.unwrap_or_else(|| RunState {
        run_id: RunId(Uuid::nil()),
        tenant_id: "default".into(),
        status: RunStatus::Pending,
        current_step: 0,
        tokens_consumed: 0,
        max_tokens: 100_000,
        max_fencing_token: FencingToken(0),
        active_lease_holder: None,
        lease_expires_at: None,
    });

    for event in events {
        if event.fencing_token > state.max_fencing_token {
            state.max_fencing_token = event.fencing_token;
        }

        match &event.payload {
            EventPayload::RunCreated {
                tenant_id,
                max_tokens,
            } => {
                state.run_id = event.run_id;
                state.tenant_id = tenant_id.clone();
                state.max_tokens = *max_tokens;
                state.status = RunStatus::Pending;
            }
            EventPayload::LeaseAcquired {
                worker_id,
                lease_duration_ms,
            } => {
                state.active_lease_holder = Some(worker_id.clone());
                state.lease_expires_at = Some(event.timestamp_utc + lease_duration_ms);
                state.status = RunStatus::Running;
            }
            EventPayload::StepScheduled { step_index, .. } => {
                state.current_step = *step_index;
            }
            EventPayload::StepCompleted { tokens_used, .. } => {
                state.tokens_consumed += tokens_used;
            }
            EventPayload::StepFailed { error_message, .. } => {
                state.status = RunStatus::Failed(error_message.clone());
            }
            EventPayload::RunCompleted { status } => {
                state.status = status.clone();
                state.active_lease_holder = None;
            }
        }
    }

    Ok(state)
}

#[async_trait]
pub trait EventStorePort: Send + Sync {
    async fn append_event(&self, event: RunEvent) -> Result<()>;
    async fn read_events(&self, run_id: RunId, from_seq: u64) -> Result<Vec<RunEvent>>;
    async fn acquire_lease(
        &self,
        run_id: RunId,
        worker_id: &str,
        ttl: Duration,
    ) -> Result<FencingToken>;
    async fn fold_state(&self, run_id: RunId) -> Result<RunState>;
}

/// SQLite-backed persistent Event Store with WAL Mode
pub struct SqliteEventStore {
    conn: Mutex<Connection>,
}

impl SqliteEventStore {
    pub fn new(conn: Connection) -> Result<Self> {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             CREATE TABLE IF NOT EXISTS run_events (
                 event_id TEXT PRIMARY KEY,
                 run_id TEXT NOT NULL,
                 seq_num INTEGER NOT NULL,
                 fencing_token INTEGER NOT NULL,
                 payload TEXT NOT NULL,
                 timestamp_utc INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_run_events_run_seq ON run_events(run_id, seq_num);

             CREATE TABLE IF NOT EXISTS run_leases (
                 run_id TEXT PRIMARY KEY,
                 current_fencing_token INTEGER NOT NULL,
                 worker_id TEXT NOT NULL,
                 expires_at INTEGER NOT NULL
             );",
        )
        .map_err(|e| {
            DagrError::Internal(format!("Failed to initialize event store schema: {}", e))
        })?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| DagrError::Internal(format!("Failed to open in-memory SQLite: {}", e)))?;
        Self::new(conn)
    }
}

#[async_trait]
impl EventStorePort for SqliteEventStore {
    async fn append_event(&self, event: RunEvent) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Check monotonic fencing token invariant
        let mut stmt = conn
            .prepare("SELECT current_fencing_token FROM run_leases WHERE run_id = ?1")
            .map_err(|e| DagrError::Internal(e.to_string()))?;
        let current_max_token: Option<u64> = stmt
            .query_row(params![event.run_id.to_string()], |row| row.get(0))
            .ok();

        if let Some(max_tok) = current_max_token {
            if event.fencing_token.0 < max_tok {
                return Err(DagrError::Internal(format!(
                    "Stale fencing token reject: event token {} < active lease token {}",
                    event.fencing_token.0, max_tok
                )));
            }
        }

        let payload_json = serde_json::to_string(&event.payload)
            .map_err(|e| DagrError::Internal(format!("Failed to serialize payload: {}", e)))?;

        conn.execute(
            "INSERT INTO run_events (event_id, run_id, seq_num, fencing_token, payload, timestamp_utc)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                event.event_id.to_string(),
                event.run_id.to_string(),
                event.sequence_number as i64,
                event.fencing_token.0 as i64,
                payload_json,
                event.timestamp_utc as i64,
            ]
        ).map_err(|e| DagrError::Internal(format!("Failed to append event: {}", e)))?;

        Ok(())
    }

    async fn read_events(&self, run_id: RunId, from_seq: u64) -> Result<Vec<RunEvent>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT event_id, run_id, seq_num, fencing_token, payload, timestamp_utc
             FROM run_events
             WHERE run_id = ?1 AND seq_num >= ?2
             ORDER BY seq_num ASC",
            )
            .map_err(|e| DagrError::Internal(e.to_string()))?;

        let rows = stmt
            .query_map(params![run_id.to_string(), from_seq as i64], |row| {
                let event_id_str: String = row.get(0)?;
                let run_id_str: String = row.get(1)?;
                let seq: i64 = row.get(2)?;
                let fence: i64 = row.get(3)?;
                let payload_str: String = row.get(4)?;
                let ts: i64 = row.get(5)?;

                Ok((event_id_str, run_id_str, seq, fence, payload_str, ts))
            })
            .map_err(|e| DagrError::Internal(e.to_string()))?;

        let mut events = Vec::new();
        for r in rows {
            let (event_id_str, run_id_str, seq, fence, payload_str, ts) =
                r.map_err(|e| DagrError::Internal(e.to_string()))?;
            let payload: EventPayload = serde_json::from_str(&payload_str)
                .map_err(|e| DagrError::Internal(format!("Corrupt event payload: {}", e)))?;

            events.push(RunEvent {
                event_id: Uuid::parse_str(&event_id_str).unwrap_or_else(|_| Uuid::new_v4()),
                run_id: RunId(Uuid::parse_str(&run_id_str).unwrap_or_else(|_| Uuid::new_v4())),
                sequence_number: seq as u64,
                fencing_token: FencingToken(fence as u64),
                payload,
                timestamp_utc: ts as u64,
            });
        }

        Ok(events)
    }

    async fn acquire_lease(
        &self,
        run_id: RunId,
        worker_id: &str,
        ttl: Duration,
    ) -> Result<FencingToken> {
        let conn = self.conn.lock().unwrap();
        let now_utc = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let expires_at = now_utc + ttl.as_millis() as u64;

        let mut stmt = conn.prepare(
            "SELECT current_fencing_token, worker_id, expires_at FROM run_leases WHERE run_id = ?1"
        ).map_err(|e| DagrError::Internal(e.to_string()))?;

        let existing: Option<(u64, String, u64)> = stmt
            .query_row(params![run_id.to_string()], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .ok();

        let new_token = match existing {
            Some((cur_token, holder, expiry)) => {
                if now_utc < expiry && holder != worker_id {
                    return Err(DagrError::Internal(format!(
                        "Lease for run {} already held by active worker {}",
                        run_id, holder
                    )));
                }
                let tok = cur_token + 1;
                conn.execute(
                    "UPDATE run_leases SET current_fencing_token = ?1, worker_id = ?2, expires_at = ?3 WHERE run_id = ?4",
                    params![tok as i64, worker_id, expires_at as i64, run_id.to_string()]
                ).map_err(|e| DagrError::Internal(e.to_string()))?;
                FencingToken(tok)
            }
            None => {
                let tok = 1;
                conn.execute(
                    "INSERT INTO run_leases (run_id, current_fencing_token, worker_id, expires_at) VALUES (?1, ?2, ?3, ?4)",
                    params![run_id.to_string(), tok as i64, worker_id, expires_at as i64]
                ).map_err(|e| DagrError::Internal(e.to_string()))?;
                FencingToken(tok)
            }
        };

        Ok(new_token)
    }

    async fn fold_state(&self, run_id: RunId) -> Result<RunState> {
        let events = self.read_events(run_id, 0).await?;
        if events.is_empty() {
            return Err(DagrError::NotFound(format!(
                "Run {} not found in event store",
                run_id
            )));
        }
        fold_events(None, &events)
    }
}
