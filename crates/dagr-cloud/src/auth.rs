//! DAGR Cloud Multi-Tenant Organization Authentication & Credentials Manager

use dagr_core::{DagrError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrgCredentials {
    pub org_id: String,
    pub org_name: String,
    pub api_key: String,
    pub cloud_url: String,
    pub authenticated_at: i64,
}

impl OrgCredentials {
    pub fn credentials_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map_err(|_| DagrError::Config("Could not determine user HOME directory".into()))?;

        let dir = PathBuf::from(home).join(".dagr");
        std::fs::create_dir_all(&dir)?;
        Ok(dir.join("credentials.json"))
    }

    /// Saves credentials securely to ~/.dagr/credentials.json
    pub fn save(&self) -> Result<()> {
        let path = Self::credentials_path()?;
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| DagrError::Config(format!("Serialization error: {}", e)))?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Loads active credentials from ~/.dagr/credentials.json if present
    pub fn load() -> Result<Option<Self>> {
        let path = Self::credentials_path()?;
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path)?;
        let creds: Self = serde_json::from_str(&content)
            .map_err(|e| DagrError::Config(format!("Malformed credentials file: {}", e)))?;
        Ok(Some(creds))
    }

    /// Clears active credentials upon logout
    pub fn clear() -> Result<()> {
        let path = Self::credentials_path()?;
        if path.exists() {
            let _ = std::fs::remove_file(path);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_serialization() {
        let creds = OrgCredentials {
            org_id: "org_acme_corp".into(),
            org_name: "Acme Corporation".into(),
            api_key: "dagr_live_sec_123456789".into(),
            cloud_url: "https://api.dagr.dev".into(),
            authenticated_at: 1724108400,
        };

        let json = serde_json::to_string(&creds).unwrap();
        let deserialized: OrgCredentials = serde_json::from_str(&json).unwrap();
        assert_eq!(creds, deserialized);
    }
}
