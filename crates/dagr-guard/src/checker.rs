use crate::rules::RuleConfig;
use crate::sanitizer::ZeroTrustSanitizer;
use dagr_core::Result;
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Violation {
    pub rule_name: String,
    pub source_file: String,
    pub imported_module: String,
    pub message: String,
}

pub struct ArchitectureGuard {
    pub config: RuleConfig,
}

impl ArchitectureGuard {
    /// Loads rule configuration for the given workspace
    pub fn load(workspace_root: &Path) -> Result<Self> {
        let config = RuleConfig::load_or_default(workspace_root)?;
        Ok(Self { config })
    }

    /// Evaluates if a single import violates any boundary rule (<0.05ms)
    pub fn check_import(&self, source_file: &str, imported_module: &str) -> Option<Violation> {
        for rule in &self.config.boundaries {
            if let Ok(from_pattern) = Pattern::new(&rule.from) {
                if from_pattern.matches(source_file) {
                    for forbidden in &rule.cannot_import {
                        if let Ok(forbid_pattern) = Pattern::new(forbidden) {
                            if forbid_pattern.matches(imported_module)
                                || imported_module.starts_with(forbidden.trim_end_matches("/**"))
                            {
                                return Some(Violation {
                                    rule_name: rule.name.clone(),
                                    source_file: source_file.to_string(),
                                    imported_module: imported_module.to_string(),
                                    message: rule.message.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Batch checks a list of imports for a file
    pub fn check_file_imports(&self, source_file: &str, imports: &[String]) -> Vec<Violation> {
        let mut violations = Vec::new();
        for import in imports {
            if let Some(violation) = self.check_import(source_file, import) {
                violations.push(violation);
            }
        }
        violations
    }

    /// Sanitizes docstrings or user comments
    pub fn sanitize_comment(&self, comment: &str) -> String {
        ZeroTrustSanitizer::sanitize(comment, &self.config.security)
    }

    /// Recursively scans workspace source files for boundary violations
    pub fn scan_workspace(&self, workspace_root: &Path) -> Result<Vec<Violation>> {
        let mut violations = Vec::new();
        Self::walk_and_check(workspace_root, workspace_root, self, &mut violations)?;
        Ok(violations)
    }

    fn walk_and_check(
        root: &Path,
        current: &Path,
        guard: &ArchitectureGuard,
        violations: &mut Vec<Violation>,
    ) -> Result<()> {
        if current.is_dir() {
            let name = current.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name == ".git"
                || name == "node_modules"
                || name == "target"
                || name == ".dagr"
                || name == ".next"
                || name == "dist"
            {
                return Ok(());
            }
            if let Ok(entries) = std::fs::read_dir(current) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    Self::walk_and_check(root, &path, guard, violations)?;
                }
            }
        } else if current.is_file() {
            let ext = current.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ["ts", "tsx", "js", "jsx", "py", "rs", "go"].contains(&ext) {
                let rel_path = current
                    .strip_prefix(root)
                    .unwrap_or(current)
                    .display()
                    .to_string();
                if let Ok(content) = std::fs::read_to_string(current) {
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if let Some(imported) = Self::extract_imported_module(trimmed) {
                            if let Some(v) = guard.check_import(&rel_path, &imported) {
                                violations.push(v);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn extract_imported_module(line: &str) -> Option<String> {
        // TypeScript/JavaScript: import ... from '...' / from "..."
        if let Some(pos) = line.find("from ") {
            let rest = &line[pos + 5..].trim();
            let quote = rest.chars().next()?;
            if quote == '\'' || quote == '"' {
                let after_quote = &rest[1..];
                if let Some(end_quote) = after_quote.find(quote) {
                    return Some(after_quote[..end_quote].to_string());
                }
            }
        }
        // Python: from ... import ...
        if let Some(stripped) = line.strip_prefix("from ") {
            let rest = stripped.trim();
            if let Some(pkg) = rest.split_whitespace().next() {
                return Some(pkg.to_string());
            }
        }
        // Python: import ...
        if let Some(stripped) = line.strip_prefix("import ") {
            let rest = stripped.trim();
            if let Some(pkg) = rest.split_whitespace().next() {
                return Some(pkg.trim_matches(&['\'', '"', ';'][..]).to_string());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boundary_violation_detection() {
        let config = RuleConfig::clean_architecture_preset();
        let guard = ArchitectureGuard { config };

        // 1. Violation: UI importing DB
        let violation = guard.check_import("src/ui/Button.tsx", "src/db/client");
        assert!(violation.is_some());
        let v = violation.unwrap();
        assert_eq!(v.rule_name, "UI-to-DB Boundary");

        // 2. Allowed: UI importing Hooks
        let allowed = guard.check_import("src/ui/Button.tsx", "src/ui/hooks/useClick");
        assert!(allowed.is_none());

        // 3. Violation: Domain importing Express
        let domain_violation = guard.check_import("src/domain/User.ts", "express");
        assert!(domain_violation.is_some());
    }
}
