use crate::rules::RuleConfig;
use std::path::Path;

pub struct ArchitectureInferrer;

impl ArchitectureInferrer {
    /// Inspects the workspace root to infer the framework preset (Next.js, FastAPI, Clean Architecture)
    pub fn infer_preset(workspace_root: &Path) -> RuleConfig {
        let has_next = workspace_root.join("next.config.js").exists()
            || workspace_root.join("next.config.mjs").exists()
            || workspace_root.join("next.config.ts").exists()
            || workspace_root.join("app").exists();

        if has_next {
            return RuleConfig::nextjs_preset();
        }

        // Default to Clean Architecture preset
        RuleConfig::clean_architecture_preset()
    }
}
