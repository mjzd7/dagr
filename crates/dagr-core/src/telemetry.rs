//! Persistent SQLite Telemetry and Lifetime ROI Analytics Engine

use crate::error::{DagrError, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Standard blended cost per 1 Million tokens ($3.00 for Sonnet 3.5 / GPT-4o tier)
pub const BLENDED_USD_PER_MILLION_TOKENS: f64 = 3.00;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetryEvent {
    pub id: String,
    pub timestamp: i64,
    pub client_id: String,
    pub event_type: String,
    pub file_path: Option<String>,
    pub symbol_name: Option<String>,
    pub raw_tokens: usize,
    pub sliced_tokens: usize,
    pub tokens_saved: usize,
    pub latency_us: u64,
    pub status: String,
    pub extra_json: Option<String>,
}

impl TelemetryEvent {
    pub fn new_slice(
        client_id: &str,
        file_path: &str,
        symbol_name: &str,
        raw_tokens: usize,
        sliced_tokens: usize,
        latency_us: u64,
    ) -> Self {
        let tokens_saved = raw_tokens.saturating_sub(sliced_tokens);
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: now_ms,
            client_id: client_id.to_string(),
            event_type: "slice".to_string(),
            file_path: Some(file_path.to_string()),
            symbol_name: Some(symbol_name.to_string()),
            raw_tokens,
            sliced_tokens,
            tokens_saved,
            latency_us,
            status: "success".to_string(),
            extra_json: None,
        }
    }

    pub fn new_guard_check(client_id: &str, violations_count: usize, latency_us: u64) -> Self {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        let status = if violations_count == 0 {
            "passed".to_string()
        } else {
            "violation".to_string()
        };

        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: now_ms,
            client_id: client_id.to_string(),
            event_type: "guard_check".to_string(),
            file_path: None,
            symbol_name: None,
            raw_tokens: 0,
            sliced_tokens: 0,
            tokens_saved: 0,
            latency_us,
            status,
            extra_json: Some(
                serde_json::json!({ "violations_count": violations_count }).to_string(),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeWindow {
    Last24Hours,
    Last7Days,
    Last30Days,
    Lifetime,
}

impl TimeWindow {
    pub fn start_timestamp_ms(&self) -> i64 {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        match self {
            TimeWindow::Last24Hours => now_ms.saturating_sub(86_400_000),
            TimeWindow::Last7Days => now_ms.saturating_sub(7 * 86_400_000),
            TimeWindow::Last30Days => now_ms.saturating_sub(30 * 86_400_000),
            TimeWindow::Lifetime => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySummary {
    pub total_events: usize,
    pub total_slices: usize,
    pub total_raw_tokens: usize,
    pub total_sliced_tokens: usize,
    pub total_tokens_saved: usize,
    pub overall_compression_ratio: f64,
    pub estimated_usd_saved: f64,
    pub avg_latency_us: f64,
    pub violations_prevented: usize,
    pub sandboxed_runs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientBreakdown {
    pub client_id: String,
    pub events_count: usize,
    pub tokens_saved: usize,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub date: String,
    pub tokens_saved: usize,
    pub events_count: usize,
    pub usd_saved: f64,
}

pub struct TelemetryStore {
    conn: Connection,
    pub db_path: PathBuf,
}

impl TelemetryStore {
    pub fn open(workspace_root: &Path) -> Result<Self> {
        let db_dir = workspace_root.join(".dagr");
        std::fs::create_dir_all(&db_dir)?;
        let db_path = db_dir.join("index.db");

        let conn = Connection::open(&db_path).map_err(|e| {
            DagrError::Storage(format!(
                "Failed to open SQLite database at {:?}: {}",
                db_path, e
            ))
        })?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;
             PRAGMA synchronous = NORMAL;

             CREATE TABLE IF NOT EXISTS telemetry_events (
                 id TEXT PRIMARY KEY,
                 timestamp INTEGER NOT NULL,
                 client_id TEXT NOT NULL,
                 event_type TEXT NOT NULL,
                 file_path TEXT,
                 symbol_name TEXT,
                 raw_tokens INTEGER NOT NULL DEFAULT 0,
                 sliced_tokens INTEGER NOT NULL DEFAULT 0,
                 tokens_saved INTEGER NOT NULL DEFAULT 0,
                 latency_us INTEGER NOT NULL DEFAULT 0,
                 status TEXT NOT NULL DEFAULT 'success',
                 extra_json TEXT
             );

             CREATE INDEX IF NOT EXISTS idx_telemetry_time ON telemetry_events(timestamp);
             CREATE INDEX IF NOT EXISTS idx_telemetry_client ON telemetry_events(client_id);
             CREATE INDEX IF NOT EXISTS idx_telemetry_type ON telemetry_events(event_type);",
        )
        .map_err(|e| DagrError::Storage(format!("Failed to initialize telemetry schema: {}", e)))?;

        Ok(Self { conn, db_path })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| DagrError::Storage(e.to_string()))?;

        conn.execute_batch(
            "CREATE TABLE telemetry_events (
                 id TEXT PRIMARY KEY,
                 timestamp INTEGER NOT NULL,
                 client_id TEXT NOT NULL,
                 event_type TEXT NOT NULL,
                 file_path TEXT,
                 symbol_name TEXT,
                 raw_tokens INTEGER NOT NULL DEFAULT 0,
                 sliced_tokens INTEGER NOT NULL DEFAULT 0,
                 tokens_saved INTEGER NOT NULL DEFAULT 0,
                 latency_us INTEGER NOT NULL DEFAULT 0,
                 status TEXT NOT NULL DEFAULT 'success',
                 extra_json TEXT
             );",
        )
        .map_err(|e| DagrError::Storage(e.to_string()))?;

        Ok(Self {
            conn,
            db_path: PathBuf::from(":memory:"),
        })
    }

    /// Records an execution event into the telemetry ledger (Fail-safe, does not block)
    pub fn record_event(&self, event: &TelemetryEvent) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO telemetry_events (
                id, timestamp, client_id, event_type, file_path, symbol_name,
                raw_tokens, sliced_tokens, tokens_saved, latency_us, status, extra_json
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )?;

        stmt.execute(params![
            event.id,
            event.timestamp,
            event.client_id,
            event.event_type,
            event.file_path,
            event.symbol_name,
            event.raw_tokens as i64,
            event.sliced_tokens as i64,
            event.tokens_saved as i64,
            event.latency_us as i64,
            event.status,
            event.extra_json,
        ])?;

        Ok(())
    }

    /// Aggregates lifetime summary metrics across a specific time window
    pub fn get_summary(&self, window: TimeWindow) -> Result<TelemetrySummary> {
        let start_ms = window.start_timestamp_ms();

        let mut stmt = self.conn.prepare(
            "SELECT 
                COUNT(*),
                COALESCE(SUM(CASE WHEN event_type = 'slice' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(raw_tokens), 0),
                COALESCE(SUM(sliced_tokens), 0),
                COALESCE(SUM(tokens_saved), 0),
                COALESCE(AVG(latency_us), 0.0),
                COALESCE(SUM(CASE WHEN event_type = 'guard_check' AND status = 'violation' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN event_type = 'sandbox_run' THEN 1 ELSE 0 END), 0)
             FROM telemetry_events 
             WHERE timestamp >= ?1",
        )?;

        let row = stmt.query_row(params![start_ms], |row| {
            let total_events: i64 = row.get(0)?;
            let total_slices: i64 = row.get(1)?;
            let raw_tokens: i64 = row.get(2)?;
            let sliced_tokens: i64 = row.get(3)?;
            let tokens_saved: i64 = row.get(4)?;
            let avg_latency: f64 = row.get(5)?;
            let violations: i64 = row.get(6)?;
            let sandboxed_runs: i64 = row.get(7)?;

            let compression = if raw_tokens > 0 {
                1.0 - (sliced_tokens as f64 / raw_tokens as f64)
            } else {
                0.0
            };

            let usd_saved = (tokens_saved as f64 / 1_000_000.0) * BLENDED_USD_PER_MILLION_TOKENS;

            Ok(TelemetrySummary {
                total_events: total_events as usize,
                total_slices: total_slices as usize,
                total_raw_tokens: raw_tokens as usize,
                total_sliced_tokens: sliced_tokens as usize,
                total_tokens_saved: tokens_saved as usize,
                overall_compression_ratio: compression,
                estimated_usd_saved: (usd_saved * 100.0).round() / 100.0,
                avg_latency_us: avg_latency,
                violations_prevented: violations as usize,
                sandboxed_runs: sandboxed_runs as usize,
            })
        })?;

        Ok(row)
    }

    /// Fetches the recent N telemetry events
    pub fn get_recent_events(&self, limit: usize) -> Result<Vec<TelemetryEvent>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, client_id, event_type, file_path, symbol_name,
                    raw_tokens, sliced_tokens, tokens_saved, latency_us, status, extra_json
             FROM telemetry_events 
             ORDER BY timestamp DESC 
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(TelemetryEvent {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                client_id: row.get(2)?,
                event_type: row.get(3)?,
                file_path: row.get(4)?,
                symbol_name: row.get(5)?,
                raw_tokens: row.get::<_, i64>(6)? as usize,
                sliced_tokens: row.get::<_, i64>(7)? as usize,
                tokens_saved: row.get::<_, i64>(8)? as usize,
                latency_us: row.get::<_, i64>(9)? as u64,
                status: row.get(10)?,
                extra_json: row.get(11)?,
            })
        })?;

        let mut events = Vec::new();
        for r in rows {
            events.push(r?);
        }

        Ok(events)
    }

    /// Groups token savings by client platform (Cursor, Claude Code, Antigravity, etc.)
    pub fn get_client_breakdown(&self) -> Result<Vec<ClientBreakdown>> {
        let mut stmt = self.conn.prepare(
            "SELECT client_id, COUNT(*), COALESCE(SUM(tokens_saved), 0)
             FROM telemetry_events
             GROUP BY client_id
             ORDER BY SUM(tokens_saved) DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let client_id: String = row.get(0)?;
            let events_count: i64 = row.get(1)?;
            let tokens_saved: i64 = row.get(2)?;
            Ok((client_id, events_count as usize, tokens_saved as usize))
        })?;

        let mut results = Vec::new();
        let mut total_saved = 0;

        for r in rows {
            let item = r?;
            total_saved += item.2;
            results.push(item);
        }

        let breakdown = results
            .into_iter()
            .map(|(client_id, events_count, tokens_saved)| {
                let percentage = if total_saved > 0 {
                    (tokens_saved as f64 / total_saved as f64) * 100.0
                } else {
                    0.0
                };
                ClientBreakdown {
                    client_id,
                    events_count,
                    tokens_saved,
                    percentage: (percentage * 10.0).round() / 10.0,
                }
            })
            .collect();

        Ok(breakdown)
    }

    /// Computes daily token savings time-series for sparklines and area charts
    pub fn get_daily_velocity(&self, days: usize) -> Result<Vec<TimeSeriesPoint>> {
        let start_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64
            - (days as i64 * 86_400_000);

        let mut stmt = self.conn.prepare(
            "SELECT 
                DATE(timestamp / 1000, 'unixepoch') as day,
                COALESCE(SUM(tokens_saved), 0),
                COUNT(*)
             FROM telemetry_events
             WHERE timestamp >= ?1
             GROUP BY day
             ORDER BY day ASC",
        )?;

        let rows = stmt.query_map(params![start_ms], |row| {
            let date: String = row.get(0)?;
            let tokens_saved: i64 = row.get(1)?;
            let events_count: i64 = row.get(2)?;
            let usd = (tokens_saved as f64 / 1_000_000.0) * BLENDED_USD_PER_MILLION_TOKENS;

            Ok(TimeSeriesPoint {
                date,
                tokens_saved: tokens_saved as usize,
                events_count: events_count as usize,
                usd_saved: (usd * 100.0).round() / 100.0,
            })
        })?;

        let mut series = Vec::new();
        for r in rows {
            series.push(r?);
        }

        Ok(series)
    }

    /// Exports all telemetry events as standard JSON string
    pub fn export_json(&self) -> Result<String> {
        let events = self.get_recent_events(100_000)?;
        serde_json::to_string_pretty(&events).map_err(|e| DagrError::Storage(e.to_string()))
    }

    /// Exports all telemetry events as standard CSV string
    pub fn export_csv(&self) -> Result<String> {
        let events = self.get_recent_events(100_000)?;
        let mut csv = String::from("id,timestamp,client_id,event_type,file_path,symbol_name,raw_tokens,sliced_tokens,tokens_saved,latency_us,status\n");

        for e in events {
            csv.push_str(&format!(
                "{},{},\"{}\",\"{}\",\"{}\",\"{}\",{},{},{},{},\"{}\"\n",
                e.id,
                e.timestamp,
                e.client_id,
                e.event_type,
                e.file_path.unwrap_or_default(),
                e.symbol_name.unwrap_or_default(),
                e.raw_tokens,
                e.sliced_tokens,
                e.tokens_saved,
                e.latency_us,
                e.status
            ));
        }

        Ok(csv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_lifecycle_and_aggregations() -> Result<()> {
        let store = TelemetryStore::open_in_memory()?;

        // Record 3 slicing events
        let ev1 = TelemetryEvent::new_slice("cursor", "src/auth.rs", "verifyToken", 2000, 200, 250);
        let ev2 = TelemetryEvent::new_slice("claudecode", "src/db.rs", "getUser", 6000, 500, 420);
        let ev3 = TelemetryEvent::new_slice("cursor", "src/api.rs", "handleReq", 3000, 300, 180);
        let ev4 = TelemetryEvent::new_guard_check("cursor", 2, 80);

        store.record_event(&ev1)?;
        store.record_event(&ev2)?;
        store.record_event(&ev3)?;
        store.record_event(&ev4)?;

        let summary = store.get_summary(TimeWindow::Lifetime)?;
        assert_eq!(summary.total_events, 4);
        assert_eq!(summary.total_slices, 3);
        assert_eq!(summary.total_raw_tokens, 11000);
        assert_eq!(summary.total_sliced_tokens, 1000);
        assert_eq!(summary.total_tokens_saved, 10000);
        assert!(summary.overall_compression_ratio > 0.9);
        assert_eq!(summary.violations_prevented, 1);
        assert!(summary.estimated_usd_saved > 0.0);

        let clients = store.get_client_breakdown()?;
        assert_eq!(clients.len(), 2);
        assert_eq!(clients[0].client_id, "claudecode"); // 5500 saved
        assert_eq!(clients[1].client_id, "cursor"); // 4500 saved

        let recent = store.get_recent_events(10)?;
        assert_eq!(recent.len(), 4);

        let json = store.export_json()?;
        assert!(json.contains("verifyToken"));

        let csv = store.export_csv()?;
        assert!(csv.contains("verifyToken"));

        Ok(())
    }
}
