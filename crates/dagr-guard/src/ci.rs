//! CI/CD Pull Request Architecture Boundary Scanner & GitHub Workflow Integrator

use crate::checker::{ArchitectureGuard, Violation};
use dagr_core::Result;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CiGuardReport {
    pub total_files_checked: usize,
    pub clean_files: usize,
    pub violations: Vec<Violation>,
    pub latency_ms: f64,
}

impl CiGuardReport {
    /// Inspects only the files changed in a Pull Request or Git Diff against .dagr/rules.yaml
    pub fn check_pr_diff(
        workspace_root: &Path,
        base_ref: Option<&str>,
        head_ref: Option<&str>,
    ) -> Result<Self> {
        let start = Instant::now();
        let guard = ArchitectureGuard::load(workspace_root)?;

        let changed_files = Self::get_changed_files(workspace_root, base_ref, head_ref);
        let supported_exts = ["ts", "tsx", "js", "jsx", "py", "rs", "go"];

        let mut violations = Vec::new();
        let mut checked_count: usize = 0;

        for file_rel in changed_files {
            let full_path = workspace_root.join(&file_rel);
            if !full_path.exists() || !full_path.is_file() {
                continue;
            }

            let ext = full_path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if !supported_exts.contains(&ext) {
                continue;
            }

            checked_count += 1;
            if let Ok(content) = std::fs::read_to_string(&full_path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if let Some(imported) = ArchitectureGuard::extract_imported_module(trimmed) {
                        if let Some(v) = guard.check_import(&file_rel, &imported) {
                            violations.push(v);
                        }
                    }
                }
            }
        }

        let elapsed = start.elapsed().as_micros() as f64 / 1000.0;
        let clean_files = checked_count.saturating_sub(violations.len());

        Ok(CiGuardReport {
            total_files_checked: checked_count,
            clean_files,
            violations,
            latency_ms: elapsed,
        })
    }

    /// Resilient Git Diff extraction with fallback for shallow clones
    fn get_changed_files(
        workspace_root: &Path,
        base_ref: Option<&str>,
        head_ref: Option<&str>,
    ) -> Vec<String> {
        let base = base_ref.unwrap_or("origin/main");
        let head = head_ref.unwrap_or("HEAD");

        // Try primary git diff
        if let Ok(output) = Command::new("git")
            .current_dir(workspace_root)
            .args(["diff", "--name-only", &format!("{}...{}", base, head)])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let files: Vec<String> = stdout
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
                if !files.is_empty() {
                    return files;
                }
            }
        }

        // Fallback 1: git diff against single base commit (HEAD~1..HEAD)
        if let Ok(output) = Command::new("git")
            .current_dir(workspace_root)
            .args(["diff", "--name-only", "HEAD~1", "HEAD"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let files: Vec<String> = stdout
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
                if !files.is_empty() {
                    return files;
                }
            }
        }

        // Fallback 2: Staged & working tree changes
        if let Ok(output) = Command::new("git")
            .current_dir(workspace_root)
            .args(["status", "--porcelain"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return stdout
                    .lines()
                    .filter_map(|l| {
                        let trimmed = l.trim();
                        if trimmed.len() > 3 {
                            Some(trimmed[3..].trim().to_string())
                        } else {
                            None
                        }
                    })
                    .collect();
            }
        }

        Vec::new()
    }

    /// Formats structured GitHub Markdown summary table for $GITHUB_STEP_SUMMARY & PR review comments
    pub fn to_markdown_summary(&self) -> String {
        let mut md = String::new();
        md.push_str("### ⚡ DAGR Architecture & Layer Boundary Guardrail Report\n\n");

        if self.violations.is_empty() {
            md.push_str("✅ **Status: Clean & Protected**\n\n");
            md.push_str(&format!(
                "- **Files Inspected:** `{}` changed file(s)\n- **Evaluation Latency:** `{:.2}ms`\n- **Violations Detected:** `0`\n\n",
                self.total_files_checked, self.latency_ms
            ));
            md.push_str("> Clean architecture layer boundaries enforced. All imports satisfy `.dagr/rules.yaml`.\n");
        } else {
            md.push_str("❌ **Status: Architectural Violations Detected**\n\n");
            md.push_str(&format!(
                "- **Files Inspected:** `{}` changed file(s)\n- **Evaluation Latency:** `{:.2}ms`\n- **Violations Detected:** `{}`\n\n",
                self.total_files_checked, self.latency_ms, self.violations.len()
            ));

            md.push_str("| Rule Violated | Source File | Illegal Import | Policy Description |\n");
            md.push_str("| :--- | :--- | :--- | :--- |\n");

            for v in &self.violations {
                md.push_str(&format!(
                    "| **{}** | `{}` | `{}` | {} |\n",
                    v.rule_name, v.source_file, v.imported_module, v.message
                ));
            }

            md.push_str("\n> 💡 **Remediation:** Please route dependencies through appropriate domain service interfaces rather than importing lower-tier modules directly.\n");
        }

        md
    }

    /// Emits standard GitHub Actions workflow commands (::error:: annotations)
    pub fn emit_github_workflow_commands(&self) {
        for v in &self.violations {
            println!(
                "::error file={},title=Architecture Violation::Rule '{}' violated: Cannot import '{}'. {}",
                v.source_file, v.rule_name, v.imported_module, v.message
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_guard_report_markdown_clean() {
        let report = CiGuardReport {
            total_files_checked: 12,
            clean_files: 12,
            violations: Vec::new(),
            latency_ms: 0.18,
        };

        let md = report.to_markdown_summary();
        assert!(md.contains("✅ **Status: Clean & Protected**"));
        assert!(md.contains("`12` changed file(s)"));
    }

    #[test]
    fn test_ci_guard_report_markdown_with_violations() {
        let report = CiGuardReport {
            total_files_checked: 5,
            clean_files: 4,
            violations: vec![Violation {
                rule_name: "UI-to-DB Isolation".into(),
                source_file: "src/components/Button.tsx".into(),
                imported_module: "@/db/client".into(),
                message: "UI cannot import DB directly".into(),
            }],
            latency_ms: 0.25,
        };

        let md = report.to_markdown_summary();
        assert!(md.contains("❌ **Status: Architectural Violations Detected**"));
        assert!(md.contains("UI-to-DB Isolation"));
        assert!(md.contains("src/components/Button.tsx"));
    }
}
