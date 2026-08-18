use dagr_core::Result;
use std::path::Path;

pub struct SandboxJournal;

impl SandboxJournal {
    /// Scans `.dagr/shadow/` on startup and purges any abandoned shadow directories from previous crashed processes
    pub fn cleanup_orphaned_transactions(workspace_root: &Path) -> Result<usize> {
        let shadow_base = workspace_root.join(".dagr").join("shadow");
        if !shadow_base.exists() {
            return Ok(0);
        }

        let mut cleaned_count = 0;
        if let Ok(entries) = std::fs::read_dir(&shadow_base) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let _ = std::fs::remove_dir_all(entry.path());
                    cleaned_count += 1;
                }
            }
        }

        Ok(cleaned_count)
    }
}
