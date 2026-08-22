use crate::error::{DagrError, Result};
use crate::event_store::RunId;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantinedItem {
    pub run_id: RunId,
    pub step_index: u32,
    pub error_message: String,
    pub failure_count: usize,
    pub quarantined_at_utc: u64,
    pub raw_context_snippet: Option<String>,
}

pub struct QuarantineManager {
    dlq_file_path: PathBuf,
}

impl QuarantineManager {
    pub fn new(dlq_path: impl AsRef<Path>) -> Self {
        Self {
            dlq_file_path: dlq_path.as_ref().to_path_buf(),
        }
    }

    pub fn default_local() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        let dlq_dir = PathBuf::from(home).join(".dagr");
        let _ = std::fs::create_dir_all(&dlq_dir);
        Self::new(dlq_dir.join("dlq.jsonl"))
    }

    pub fn record_quarantine(
        &self,
        run_id: RunId,
        step_index: u32,
        error: &DagrError,
        failure_count: usize,
        context_snippet: Option<&str>,
    ) -> Result<()> {
        let item = QuarantinedItem {
            run_id,
            step_index,
            error_message: error.to_string(),
            failure_count,
            quarantined_at_utc: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            raw_context_snippet: context_snippet.map(|s| s.to_string()),
        };

        let json_line = serde_json::to_string(&item)
            .map_err(|e| DagrError::Internal(format!("Failed to serialize DLQ item: {}", e)))?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.dlq_file_path)
            .map_err(|e| {
                DagrError::Internal(format!(
                    "Failed to open DLQ file {:?}: {}",
                    self.dlq_file_path, e
                ))
            })?;

        writeln!(file, "{}", json_line)
            .map_err(|e| DagrError::Internal(format!("Failed to append to DLQ: {}", e)))?;

        Ok(())
    }
}
