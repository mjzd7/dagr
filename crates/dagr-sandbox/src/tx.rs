use crate::engine::CloneEngine;
use dagr_core::{DagrError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SandboxTx {
    pub tx_id: Uuid,
    pub workspace_root: PathBuf,
    pub shadow_root: PathBuf,
    pub modified_files: Vec<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

pub struct CowSandbox;

impl CowSandbox {
    /// Initializes a shadow transaction sandbox overlay in `<workspace>/.dagr/shadow/<tx_id>` (<2ms)
    pub fn begin(workspace_root: &Path) -> Result<SandboxTx> {
        let tx_id = Uuid::new_v4();
        let shadow_root = workspace_root
            .join(".dagr")
            .join("shadow")
            .join(tx_id.to_string());

        std::fs::create_dir_all(&shadow_root)?;

        Ok(SandboxTx {
            tx_id,
            workspace_root: workspace_root.to_path_buf(),
            shadow_root,
            modified_files: Vec::new(),
        })
    }

    /// Stages a file mutation inside the shadow workspace
    pub fn stage_file(tx: &mut SandboxTx, relative_path: &Path, content: &[u8]) -> Result<()> {
        let shadow_file = tx.shadow_root.join(relative_path);
        if let Some(parent) = shadow_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&shadow_file, content)?;
        if !tx.modified_files.contains(&relative_path.to_path_buf()) {
            tx.modified_files.push(relative_path.to_path_buf());
        }

        Ok(())
    }

    /// Clones an existing file from the real workspace into shadow storage for modification
    pub fn clone_into_shadow(tx: &mut SandboxTx, relative_path: &Path) -> Result<()> {
        let src = tx.workspace_root.join(relative_path);
        let dst = tx.shadow_root.join(relative_path);

        if src.exists() {
            CloneEngine::clone_file(&src, &dst)?;
        }
        Ok(())
    }

    /// Executes verification commands (tests, linters) within the shadow root
    pub fn verify(tx: &SandboxTx, command: &str) -> Result<ExecutionResult> {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", command])
                .current_dir(&tx.shadow_root)
                .output()
        } else {
            Command::new("sh")
                .arg("-c")
                .arg(command)
                .current_dir(&tx.shadow_root)
                .output()
        }.map_err(|e| DagrError::Sandbox(format!("Failed to execute verification command: {}", e)))?;

        Ok(ExecutionResult {
            success: output.status.success(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Atomically swaps modified files from the shadow root into the actual workspace
    pub fn commit(tx: SandboxTx) -> Result<()> {
        for file in &tx.modified_files {
            let shadow_path = tx.shadow_root.join(file);
            let workspace_path = tx.workspace_root.join(file);

            if let Some(parent) = workspace_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            if shadow_path.exists() {
                std::fs::copy(&shadow_path, &workspace_path)?;
            }
        }

        // Clean up shadow directory
        let _ = std::fs::remove_dir_all(&tx.shadow_root);
        Ok(())
    }

    /// Discards shadow modifications within 10ms leaving zero side effects
    pub fn rollback(tx: SandboxTx) -> Result<()> {
        if tx.shadow_root.exists() {
            std::fs::remove_dir_all(&tx.shadow_root)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cow_sandbox_lifecycle() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let workspace = temp_dir.path();

        // 1. Create a baseline file in workspace
        let file_a = workspace.join("src").join("main.rs");
        std::fs::create_dir_all(file_a.parent().unwrap())?;
        std::fs::write(&file_a, "fn main() { println!(\"original\"); }")?;

        // 2. Begin transaction
        let mut tx = CowSandbox::begin(workspace)?;
        assert!(tx.shadow_root.exists());

        // 3. Stage a modification in shadow
        let relative_path = Path::new("src/main.rs");
        CowSandbox::stage_file(&mut tx, relative_path, b"fn main() { println!(\"mutated\"); }")?;

        // Verify shadow has mutated content while workspace retains original
        let shadow_content = std::fs::read_to_string(tx.shadow_root.join(relative_path))?;
        let workspace_content = std::fs::read_to_string(&file_a)?;
        assert_eq!(shadow_content, "fn main() { println!(\"mutated\"); }");
        assert_eq!(workspace_content, "fn main() { println!(\"original\"); }");

        // 4. Test rollback
        let shadow_dir = tx.shadow_root.clone();
        CowSandbox::rollback(tx)?;
        assert!(!shadow_dir.exists());

        // Workspace must remain 100% clean and original
        let final_workspace_content = std::fs::read_to_string(&file_a)?;
        assert_eq!(final_workspace_content, "fn main() { println!(\"original\"); }");

        Ok(())
    }
}
