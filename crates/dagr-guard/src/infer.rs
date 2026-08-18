use crate::rules::{LimitsConfig, RuleConfig, SecurityConfig};
use std::path::Path;

pub struct ArchitectureInferrer;

impl ArchitectureInferrer {
    /// Inspects the workspace root to infer the framework preset (Next.js, FastAPI, Rust, Clean Architecture)
    pub fn infer_preset(workspace_root: &Path) -> RuleConfig {
        let has_next = workspace_root.join("next.config.js").exists()
            || workspace_root.join("next.config.mjs").exists()
            || workspace_root.join("next.config.ts").exists();

        if has_next {
            return RuleConfig::nextjs_preset();
        }

        let has_python = workspace_root.join("pyproject.toml").exists()
            || workspace_root.join("requirements.txt").exists()
            || workspace_root.join("Pipfile").exists();

        if has_python {
            return RuleConfig {
                version: "1.0".into(),
                project_name: Some("python-app".into()),
                preset: Some("fastapi".into()),
                boundaries: RuleConfig::get_preset_boundaries("fastapi"),
                limits: LimitsConfig::default(),
                security: SecurityConfig::default(),
            };
        }

        let has_rust = workspace_root.join("Cargo.toml").exists();

        if has_rust {
            return RuleConfig {
                version: "1.0".into(),
                project_name: Some("rust-workspace".into()),
                preset: Some("rust".into()),
                boundaries: RuleConfig::get_preset_boundaries("rust"),
                limits: LimitsConfig::default(),
                security: SecurityConfig::default(),
            };
        }

        // Default to Clean Architecture preset
        RuleConfig::clean_architecture_preset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_rust_workspace() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("Cargo.toml"), b"[workspace]").unwrap();

        let config = ArchitectureInferrer::infer_preset(temp.path());
        assert_eq!(config.preset, Some("rust".into()));
        assert!(!config.boundaries.is_empty());
    }

    #[test]
    fn test_infer_python_project() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("requirements.txt"), b"fastapi").unwrap();

        let config = ArchitectureInferrer::infer_preset(temp.path());
        assert_eq!(config.preset, Some("fastapi".into()));
    }
}
