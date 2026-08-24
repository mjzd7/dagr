//! Agent identity registry: maps agent session ids to human owners with
//! optional time-boxed expiry. Backed by `.dagr/agents.json` (JSON rather
//! than TOML because serde_json is already a workspace dependency).
//!
//! ponytail: flat-file registry, no locking beyond atomic rename; upgrade
//! to SQLite-backed store when multi-writer contention actually occurs.

use crate::error::{DagrError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentRecord {
    pub id: String,
    pub owner: String,
    #[serde(default)]
    pub role: String,
    /// Unix seconds; None = no expiry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_unix: Option<u64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct RegistryFile {
    #[serde(default)]
    agents: Vec<AgentRecord>,
}

pub struct AgentRegistry {
    path: PathBuf,
}

impl AgentRegistry {
    pub fn load(workspace_root: &Path) -> Self {
        Self {
            path: workspace_root.join(".dagr").join("agents.json"),
        }
    }

    pub fn read(&self) -> Result<Vec<AgentRecord>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let raw = std::fs::read_to_string(&self.path).map_err(DagrError::Io)?;
        let file: RegistryFile =
            serde_json::from_str(&raw).map_err(|e| DagrError::Config(format!("agents.json: {e}")))?;
        Ok(file.agents)
    }

    fn write(&self, agents: &[AgentRecord]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(DagrError::Io)?;
        }
        let body = serde_json::to_string_pretty(&RegistryFile {
            agents: agents.to_vec(),
        })
        .map_err(|e| DagrError::Serialization(e.to_string()))?;
        let tmp = self.path.with_extension("json.tmp");
        std::fs::write(&tmp, body).map_err(DagrError::Io)?;
        std::fs::rename(&tmp, &self.path).map_err(DagrError::Io)?;
        Ok(())
    }

    pub fn register(&self, record: AgentRecord) -> Result<()> {
        if record.id.trim().is_empty() || record.owner.trim().is_empty() {
            return Err(DagrError::InvalidInput(
                "agent id and owner must be non-empty".into(),
            ));
        }
        let mut agents = self.read()?;
        if agents.iter().any(|a| a.id == record.id) {
            return Err(DagrError::Config(format!(
                "agent '{}' already registered",
                record.id
            )));
        }
        agents.push(record);
        self.write(&agents)
    }

    /// Removes the agent registration. Returns whether it existed.
    pub fn revoke(&self, id: &str) -> Result<bool> {
        let mut agents = self.read()?;
        let before = agents.len();
        agents.retain(|a| a.id != id);
        let removed = agents.len() != before;
        if removed {
            self.write(&agents)?;
        }
        Ok(removed)
    }

    /// An agent is active when registered and unexpired at `now_unix`.
    pub fn is_active(&self, id: &str, now_unix: u64) -> Result<bool> {
        Ok(self.read()?.into_iter().any(|a| {
            a.id == id && a.expires_at_unix.map(|e| e > now_unix).unwrap_or(true)
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_ws(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("dagr-reg-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn register_list_revoke_roundtrip() {
        let ws = temp_ws("round");
        let reg = AgentRegistry::load(&ws);
        reg.register(AgentRecord {
            id: "cursor-abc".into(),
            owner: "alice".into(),
            role: "builder".into(),
            expires_at_unix: Some(9_999_999_999),
        })
        .unwrap();
        assert_eq!(reg.read().unwrap().len(), 1);
        assert!(reg.is_active("cursor-abc", 1_000_000_000).unwrap());
        assert!(reg.revoke("cursor-abc").unwrap());
        assert!(!reg.revoke("cursor-abc").unwrap(), "second revoke is a no-op");
        assert!(!reg.is_active("cursor-abc", 1_000_000_000).unwrap());
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn duplicate_registration_rejected() {
        let ws = temp_ws("dup");
        let reg = AgentRegistry::load(&ws);
        let rec = |id: &str| AgentRecord {
            id: id.into(),
            owner: "bob".into(),
            role: String::new(),
            expires_at_unix: None,
        };
        reg.register(rec("a")).unwrap();
        assert!(reg.register(rec("a")).is_err());
        let _ = std::fs::remove_dir_all(&ws);
    }

    #[test]
    fn expiry_is_enforced_by_is_active() {
        let ws = temp_ws("exp");
        let reg = AgentRegistry::load(&ws);
        reg.register(AgentRecord {
            id: "shortlived".into(),
            owner: "carol".into(),
            role: String::new(),
            expires_at_unix: Some(100),
        })
        .unwrap();
        assert!(reg.is_active("shortlived", 99).unwrap());
        assert!(!reg.is_active("shortlived", 101).unwrap());
        let _ = std::fs::remove_dir_all(&ws);
    }
}
