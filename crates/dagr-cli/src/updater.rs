use colored::*;
use dagr_core::{DagrError, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct UpdateCache {
    pub last_checked_epoch: u64,
    pub latest_version: String,
}

pub struct AutoUpdater;

impl AutoUpdater {
    pub const REPO_OWNER: &'static str = "mjzd7";
    pub const REPO_NAME: &'static str = "dagr";
    pub const CURRENT_VERSION: &'static str = env!("CARGO_PKG_VERSION");
    pub const CHECK_INTERVAL_SECS: u64 = 43200; // 12 hours

    /// Resolves the cache file path for update checks
    fn get_cache_path() -> Option<PathBuf> {
        let home = std::env::var("HOME").ok().map(PathBuf::from)?;
        Some(home.join(".dagr").join("update_cache.json"))
    }

    /// Reads cached update status
    pub fn read_cache() -> Option<UpdateCache> {
        let path = Self::get_cache_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path).ok()?;
            let val: Value = serde_json::from_str(&content).ok()?;
            let epoch = val["last_checked_epoch"].as_u64()?;
            let ver = val["latest_version"].as_str()?.to_string();
            Some(UpdateCache {
                last_checked_epoch: epoch,
                latest_version: ver,
            })
        } else {
            None
        }
    }

    /// Writes update cache
    pub fn write_cache(latest_version: &str) {
        if let Some(path) = Self::get_cache_path() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let epoch = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let json_val = json!({
                "last_checked_epoch": epoch,
                "latest_version": latest_version
            });
            if let Ok(content) = serde_json::to_string(&json_val) {
                let _ = std::fs::write(path, content);
            }
        }
    }

    /// Checks if a newer version is available and prints an unobtrusive notification banner
    pub fn notify_if_update_available() {
        let now_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let cached = Self::read_cache();
        let should_check = match &cached {
            Some(c) => now_epoch.saturating_sub(c.last_checked_epoch) > Self::CHECK_INTERVAL_SECS,
            None => true,
        };

        let latest_version = if should_check {
            // Quick non-blocking probe or fallback
            let version = Self::fetch_latest_version_quick()
                .unwrap_or_else(|| Self::CURRENT_VERSION.to_string());
            Self::write_cache(&version);
            version
        } else {
            cached
                .map(|c| c.latest_version)
                .unwrap_or_else(|| Self::CURRENT_VERSION.to_string())
        };

        if latest_version != Self::CURRENT_VERSION && !latest_version.is_empty() {
            eprintln!();
            eprintln!(
                "{}",
                "┌─────────────────────────────────────────────────────────────┐".yellow()
            );
            eprintln!(
                "{}  💡 {} v{} → v{}           {}",
                "│".yellow(),
                "Update available:".bold().yellow(),
                Self::CURRENT_VERSION.dimmed(),
                latest_version.bold().green(),
                "│".yellow()
            );
            eprintln!(
                "{}  Run '{}' to upgrade to the latest version    {}",
                "│".yellow(),
                "dagr update".bold().cyan(),
                "│".yellow()
            );
            eprintln!(
                "{}",
                "└─────────────────────────────────────────────────────────────┘".yellow()
            );
            eprintln!();
        }
    }

    /// Quick remote check with tight timeout to avoid delaying user commands
    fn fetch_latest_version_quick() -> Option<String> {
        let output = std::process::Command::new("curl")
            .args([
                "-fsSL",
                "--max-time",
                "1",
                &format!(
                    "https://raw.githubusercontent.com/{}/{}/main/Cargo.toml",
                    Self::REPO_OWNER,
                    Self::REPO_NAME
                ),
            ])
            .output()
            .ok()?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout);
            for line in content.lines() {
                if line.trim().starts_with("version =") {
                    let parts: Vec<&str> = line.split('"').collect();
                    if parts.len() >= 2 {
                        return Some(parts[1].to_string());
                    }
                }
            }
        }
        None
    }

    /// Executes the self-update procedure
    pub fn self_update(_force: bool) -> Result<()> {
        eprintln!(
            "⚡ Checking for DAGR hypervisor updates from github.com/{}/{}...",
            Self::REPO_OWNER,
            Self::REPO_NAME
        );

        let current_exe = std::env::current_exe()?;
        let is_cargo_installed = current_exe
            .to_str()
            .map(|s| s.contains(".cargo"))
            .unwrap_or(false);

        eprintln!("   Current Version : v{}", Self::CURRENT_VERSION);
        eprintln!("   Installed Binary: {:?}", current_exe);

        if is_cargo_installed {
            eprintln!("⚡ Updating DAGR via Cargo git source...");
            let status = std::process::Command::new("cargo")
                .args([
                    "install",
                    "--git",
                    &format!(
                        "https://github.com/{}/{}.git",
                        Self::REPO_OWNER,
                        Self::REPO_NAME
                    ),
                    "dagr",
                    "--force",
                ])
                .status();

            match status {
                Ok(s) if s.success() => {
                    eprintln!("✅ Binary successfully updated via Cargo!");
                }
                _ => {
                    eprintln!(
                        "⚠️  Cargo git update failed. Invoking universal installer script..."
                    );
                    Self::run_universal_script_update()?;
                }
            }
        } else {
            Self::run_universal_script_update()?;
        }

        // Update local cache to current version
        Self::write_cache(Self::CURRENT_VERSION);

        // Re-sync MCP configurations and skills for all 30+ clients
        eprintln!("🔌 Refreshing Model Context Protocol (MCP) tool configurations...");
        let _ = crate::handle_mcp_install("all", None);

        eprintln!("📦 Refreshing Agent Skills across all IDEs and agents...");
        let _ = crate::handle_skills_install("all");

        eprintln!("\n✅ DAGR has been successfully updated to the latest version!");
        Ok(())
    }

    fn run_universal_script_update() -> Result<()> {
        let script_url = format!(
            "https://raw.githubusercontent.com/{}/{}/main/scripts/install.sh",
            Self::REPO_OWNER,
            Self::REPO_NAME
        );

        eprintln!(
            "⚡ Downloading and applying universal update from {}...",
            script_url
        );
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("curl -fsSL {} | bash", script_url))
            .status()
            .map_err(DagrError::Io)?;

        if !status.success() {
            return Err(DagrError::Io(std::io::Error::other(
                "Universal updater script exited with failure",
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_version_constant() {
        assert_eq!(AutoUpdater::CURRENT_VERSION, "0.1.0");
    }

    #[test]
    fn test_cache_roundtrip() {
        AutoUpdater::write_cache("0.2.0");
        let cached = AutoUpdater::read_cache();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().latest_version, "0.2.0");
        // Reset back to current
        AutoUpdater::write_cache(AutoUpdater::CURRENT_VERSION);
    }
}
