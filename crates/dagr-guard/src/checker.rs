use crate::rules::RuleConfig;
use crate::sanitizer::ZeroTrustSanitizer;
use dagr_core::Result;
use glob::Pattern;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Violation {
    pub rule_name: String,
    pub source_file: String,
    pub imported_module: String,
    pub message: String,
}

/// Public test/harness surface for the lexical relative-specifier resolver.
pub fn checker_relative_candidates(source_file: &str, specifier: &str) -> Vec<String> {
    resolve_relative_candidates(source_file, specifier)
}

/// Segment-boundary prefix match: kills sibling-prefix false positives
/// (`src/db/**` vs `src/db-migration/x`) while bare prefixes like `src/db`
/// still catch `src/db/client` (finding N1).
fn module_under_prefix(module: &str, pattern: &str) -> bool {
    let prefix = pattern.trim_end_matches("/**").trim_end_matches('/');
    module == prefix
        || (module.len() > prefix.len()
            && module.starts_with(prefix)
            && module.as_bytes()[prefix.len()] == b'/')
}

fn has_known_extension(candidate: &str, exts: &[&str]) -> bool {
    candidate.rsplit('/').next().is_some_and(|segment| {
        segment.contains('.')
            && segment
                .rsplit('.')
                .next()
                .is_some_and(|e| exts.contains(&e))
    })
}

/// Lexically resolves TS/JS-style relative specifiers (`./x`, `../x`) against
/// the importing file's directory into workspace-relative candidates, so
/// boundary rules written as absolute globs also catch relative-import
/// evasions (finding L2). Pure string math — no filesystem access.
// ponytail: lexical-only resolution (no FS probing) - directory imports rely on the /index candidate alone; add extension probing only if rules ever need file-exact matching
fn resolve_relative_candidates(source_file: &str, specifier: &str) -> Vec<String> {
    if !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return Vec::new();
    }
    let mut parts: Vec<&str> = source_file.split('/').collect();
    parts.pop();
    for seg in specifier.split('/') {
        match seg {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            seg => parts.push(seg),
        }
    }
    if parts.is_empty() {
        return Vec::new();
    }
    let joined = parts.join("/");
    vec![joined.clone(), format!("{joined}/index")]
}

fn quoted_content(s: &str) -> Option<String> {
    let q = s.chars().next()?;
    if q != '\'' && q != '"' {
        return None;
    }
    let end = s[1..].find(q)?;
    let module = &s[1..1 + end];
    (!module.is_empty()).then(|| module.to_string())
}

fn probe_call_argument(line: &str, marker: &str) -> Option<String> {
    let start = line.find(marker)? + marker.len();
    quoted_content(&line[start..])
}

pub struct ArchitectureGuard {
    pub config: RuleConfig,
    pub alias_map: crate::alias::AliasMap,
    pub(crate) workspace_root: PathBuf,
    pub(crate) barrel_cache: Mutex<HashMap<String, Option<Vec<String>>>>,
}

impl ArchitectureGuard {
    /// Loads rule configuration and the workspace alias map (tsconfig/jsconfig)
    pub fn load(workspace_root: &Path) -> Result<Self> {
        let config = RuleConfig::load_or_default(workspace_root)?;
        let alias_map = crate::alias::AliasMap::load(workspace_root);
        Ok(Self {
            config,
            alias_map,
            workspace_root: workspace_root.to_path_buf(),
            barrel_cache: Mutex::new(HashMap::new()),
        })
    }

    /// Full-control constructor for harnesses/tests that supply their own
    /// config, alias map and workspace root.
    pub fn with_parts(
        config: RuleConfig,
        alias_map: crate::alias::AliasMap,
        workspace_root: PathBuf,
    ) -> Self {
        Self {
            config,
            alias_map,
            workspace_root,
            barrel_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Evaluates if a single import violates any boundary rule (<0.05ms)
    pub fn check_import(&self, source_file: &str, imported_module: &str) -> Option<Violation> {
        let candidates = self.candidate_set(source_file, imported_module);
        if let Some(v) = self.first_violation(source_file, imported_module, &candidates) {
            return Some(v);
        }

        // One-hop barrel expansion runs only when nothing matched directly, so
        // clean scans stay IO-free; per-candidate results are cached for the
        // guard's lifetime.
        let mut via_barrels = Vec::new();
        for cand in &candidates {
            via_barrels.extend(self.barrel_reexports(cand));
        }
        if via_barrels.is_empty() {
            return None;
        }
        self.first_violation(source_file, imported_module, &via_barrels)
    }

    fn candidate_set(&self, source_file: &str, imported_module: &str) -> Vec<String> {
        let mut candidates: Vec<String> = vec![imported_module.to_string()];
        candidates.extend(resolve_relative_candidates(source_file, imported_module));
        if !self.alias_map.is_empty() {
            candidates.extend(self.alias_map.candidates(imported_module));
        }
        candidates
    }

    fn first_violation(
        &self,
        source_file: &str,
        imported_module: &str,
        specs: &[String],
    ) -> Option<Violation> {
        for rule in &self.config.boundaries {
            if let Ok(from_pattern) = Pattern::new(&rule.from) {
                if from_pattern.matches(source_file) {
                    for forbidden in &rule.cannot_import {
                        if let Ok(forbid_pattern) = Pattern::new(forbidden) {
                            if specs.iter().any(|cand| {
                                forbid_pattern.matches(cand) || module_under_prefix(cand, forbidden)
                            }) {
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

    /// Cached one-hop `export ... from "..."` targets for a resolved candidate;
    /// empty when the candidate is not a readable barrel.
    fn barrel_reexports(&self, candidate: &str) -> Vec<String> {
        let mut cache = self.barrel_cache.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(hit) = cache.get(candidate) {
            return hit.clone().unwrap_or_default();
        }
        let targets = self.read_barrel_reexports(candidate);
        cache.insert(
            candidate.to_string(),
            (!targets.is_empty()).then_some(targets.clone()),
        );
        targets
    }

    fn read_barrel_reexports(&self, candidate: &str) -> Vec<String> {
        const EXTS: [&str; 4] = ["ts", "tsx", "js", "jsx"];
        let mut file_candidates: Vec<String> = if has_known_extension(candidate, &EXTS) {
            vec![candidate.to_string()]
        } else {
            let mut v: Vec<String> = EXTS.iter().map(|e| format!("{candidate}.{e}")).collect();
            if !candidate.ends_with("/index") {
                v.extend(EXTS.iter().map(|e| format!("{candidate}/index.{e}")));
            }
            v
        };

        for rel in file_candidates.drain(..) {
            let Ok(content) = std::fs::read_to_string(self.workspace_root.join(&rel)) else {
                continue;
            };
            let mut out = Vec::new();
            for line in content.lines() {
                // Follow re-exports only: a barrel's own private imports must
                // not taint importers (that would transitive-flag everything).
                let trimmed = line.trim_start();
                if !trimmed.starts_with("export") || !trimmed.contains(" from ") {
                    continue;
                }
                if let Some(spec) = Self::extract_imported_module(trimmed) {
                    out.extend(resolve_relative_candidates(&rel, &spec));
                    if !self.alias_map.is_empty() {
                        out.extend(self.alias_map.candidates(&spec));
                    }
                }
            }
            if !out.is_empty() {
                return out;
            }
        }
        Vec::new()
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

    /// Scans an explicit list of workspace-relative files (`--staged` mode).
    pub fn scan_files(&self, workspace_root: &Path, files: &[String]) -> Result<Vec<Violation>> {
        const SCANNED_EXTS: [&str; 7] = ["ts", "tsx", "js", "jsx", "py", "rs", "go"];
        let mut violations = Vec::new();
        for rel in files {
            let abs = workspace_root.join(rel);
            if !abs.is_file() {
                continue;
            }
            let ext = abs.extension().and_then(|s| s.to_str()).unwrap_or("");
            if !SCANNED_EXTS.contains(&ext) {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&abs) else {
                continue;
            };
            for line in content.lines() {
                if let Some(imported) = Self::extract_imported_module(line.trim()) {
                    if let Some(v) = self.check_import(rel, &imported) {
                        violations.push(v);
                    }
                }
            }
        }
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
                || name == "build"
                || name == "out"
                || name == ".output"
                || name == ".turbo"
                || name == ".venv"
                || name == "venv"
                || name == "__pycache__"
                || name == "vendor"
                || name == "coverage"
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
                // Forward slashes on every platform: rules.yaml globs and
                // violation reports are written/parsed with `/`.
                let rel_path = current
                    .strip_prefix(root)
                    .unwrap_or(current)
                    .display()
                    .to_string()
                    .replace('\\', "/");
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

    /// Line-level import extractor across dialects: TS/JS static + side-effect
    /// imports, dynamic `import()` / `require()` calls, re-exports, Python,
    /// Rust `use` paths, Go single-line and block lines. Comment lines never
    /// yield phantom imports (findings N3, H-R1, H-GO1).
    // ponytail: string-probe extraction, not tree-sitter AST (parsers live in dagr-slicer); upgrade when any dialect misfires during field-wave testing
    pub fn extract_imported_module(line: &str) -> Option<String> {
        // Final hygiene gate: a legitimate specifier never embeds its own
        // delimiter quotes; anything carrying them is soup, not an import.
        // Final hygiene gate: a legitimate specifier never embeds its own
        // delimiter quotes; anything carrying them is soup, not an import.
        Self::extract_imported_module_inner(line).filter(|m| !m.contains('"') && !m.contains('\''))
    }

    fn extract_imported_module_inner(line: &str) -> Option<String> {
        let trimmed = line.trim();

        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with('#')
        {
            return None;
        }

        if let Some(rest) = trimmed.strip_prefix("use ") {
            let path = rest.trim_end().trim_end_matches(';').trim();
            let base = path.split("::{").next()?.trim();
            let base = base.split(" as ").next()?.trim();
            if base.is_empty()
                || base.contains(['"', '\'', '\n'])
                || !base
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphabetic() || c == '_')
            {
                return None;
            }
            return Some(base.split("::").collect::<Vec<&str>>().join("/"));
        }

        if let Some(rest) = trimmed.strip_prefix("from ") {
            if let Some(pkg) = rest.split_whitespace().next() {
                return Some(pkg.to_string());
            }
        }

        if let Some(pos) = trimmed.find("from ") {
            let rest = trimmed[pos + 5..].trim_start();
            if let Some(module) = quoted_content(rest) {
                return Some(module);
            }
        }

        if let Some(rest) = trimmed.strip_prefix("import ") {
            let rest = rest.trim_start();
            if let Some(module) = quoted_content(rest).or_else(|| {
                rest.split_whitespace().next().map(|pkg| {
                    pkg.trim_matches(|c| c == ';' || c == '\'' || c == '"')
                        .to_string()
                })
            }) {
                return Some(module);
            }
        }

        if let Some(module) = probe_call_argument(trimmed, "import(")
            .or_else(|| probe_call_argument(trimmed, "require("))
        {
            return Some(module);
        }

        if trimmed.starts_with('"') && !trimmed.ends_with(';') {
            return quoted_content(trimmed);
        }

        let mut tokens = trimmed.split_whitespace();
        if let (Some(alias), Some(path)) = (tokens.next(), tokens.next()) {
            if tokens.next().is_none()
                && !trimmed.contains('=')
                && !trimmed.ends_with(';')
                && alias
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
                && (path.starts_with('"') || path.starts_with('\''))
            {
                return quoted_content(path);
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
        let guard = ArchitectureGuard {
            config,
            alias_map: Default::default(),
            workspace_root: PathBuf::from("."),
            barrel_cache: Mutex::new(HashMap::new()),
        };

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

    fn guard_with(from: &str, forbidden: &str) -> ArchitectureGuard {
        ArchitectureGuard {
            alias_map: Default::default(),
            workspace_root: PathBuf::from("."),
            barrel_cache: Mutex::new(HashMap::new()),
            config: RuleConfig {
                version: "1.0".into(),
                project_name: None,
                preset: None,
                boundaries: vec![crate::rules::BoundaryRule {
                    name: "UI-to-DB".into(),
                    from: from.into(),
                    cannot_import: vec![forbidden.to_string()],
                    message: "no db".into(),
                }],
                limits: Default::default(),
                security: Default::default(),
            },
        }
    }

    #[test]
    fn sibling_directory_prefix_is_not_a_violation() {
        let guard = guard_with("src/ui/**", "src/db/**");
        assert!(guard.check_import("src/ui/A.ts", "src/db/client").is_some());
        assert!(guard
            .check_import("src/ui/A.ts", "src/db-migration/client")
            .is_none());
    }

    #[test]
    fn bare_prefix_requires_segment_boundary() {
        let guard = guard_with("src/ui/**", "src/db");
        assert!(guard.check_import("src/ui/A.ts", "src/db").is_some());
        assert!(guard.check_import("src/ui/A.ts", "src/db/client").is_some());
        assert!(guard
            .check_import("src/ui/A.ts", "src/database/engine")
            .is_none());
    }

    #[test]
    fn parent_relative_import_resolves_to_absolute_pattern() {
        let guard = guard_with(
            "packages/core/src/content-filter/**",
            "packages/core/src/db/**",
        );
        assert!(guard
            .check_import(
                "packages/core/src/content-filter/text-filter.ts",
                "../db/client"
            )
            .is_some());
        assert!(guard
            .check_import("packages/core/src/content-filter/text-filter.ts", "../db")
            .is_some());
    }

    #[test]
    fn current_dir_and_parent_relative_imports_resolve() {
        let guard = guard_with("apps/web/app/**", "apps/web/app/lib/secrets/**");
        assert!(guard
            .check_import("apps/web/app/page.tsx", "./lib/secrets/api")
            .is_some());

        let guard = guard_with("apps/web/app/**", "apps/web/lib/secrets/**");
        assert!(guard
            .check_import("apps/web/app/page.tsx", "../lib/secrets/api")
            .is_some());
    }

    #[test]
    fn non_relative_specifiers_skip_resolution_entirely() {
        let guard = guard_with("src/domain/**", "src/db/**");
        assert!(guard.check_import("src/domain/user.ts", "src/db").is_some());
        assert!(guard
            .check_import("src/domain/user.ts", "vendor/../db/client")
            .is_none());
    }

    fn barrel_guard(root: &std::path::Path, forbidden: &str) -> ArchitectureGuard {
        ArchitectureGuard {
            alias_map: Default::default(),
            workspace_root: root.to_path_buf(),
            barrel_cache: Mutex::new(HashMap::new()),
            config: RuleConfig {
                version: "1.0".into(),
                project_name: None,
                preset: None,
                boundaries: vec![crate::rules::BoundaryRule {
                    name: "No Internals".into(),
                    from: "apps/**".into(),
                    cannot_import: vec![forbidden.into()],
                    message: "internals stay internal".into(),
                }],
                limits: Default::default(),
                security: Default::default(),
            },
        }
    }

    #[test]
    fn barrel_reexport_attributes_violation_one_hop() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("internal")).unwrap();
        std::fs::write(
            temp.path().join("public_api.ts"),
            "export { stash } from \"./internal/secret\";\n",
        )
        .unwrap();

        let guard = barrel_guard(temp.path(), "internal/**");
        let v = guard
            .check_import("apps/web/a.ts", "../../public_api")
            .expect("re-exported module must be attributed through the barrel");
        assert_eq!(
            v.imported_module, "../../public_api",
            "violation must report the original specifier"
        );

        std::fs::write(
            temp.path().join("public_api.ts"),
            "export const stash = 1;\n",
        )
        .unwrap();
        let fresh = barrel_guard(temp.path(), "internal/**");
        assert!(
            fresh
                .check_import("apps/web/a.ts", "../../public_api")
                .is_none(),
            "barrel without re-exports must stay clean"
        );
    }

    #[test]
    fn plain_modules_never_produce_barrel_hits() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(
            temp.path().join("utils.ts"),
            "export const help = () => 1;\nimport { x } from \"./internal/nope\";\n",
        )
        .unwrap();
        let guard = barrel_guard(temp.path(), "internal/**");
        assert!(guard.check_import("apps/b.ts", "../utils").is_none());
    }
}

#[test]
fn rust_use_statements_are_visible() {
    assert_eq!(
        ArchitectureGuard::extract_imported_module("use tokio::runtime::Runtime;"),
        Some("tokio/runtime/Runtime".to_string())
    );
    assert_eq!(
        ArchitectureGuard::extract_imported_module("use std::io::{self, Write};"),
        Some("std/io".to_string())
    );
    assert_eq!(
        ArchitectureGuard::extract_imported_module("use crate::db::client as dbc;"),
        Some("crate/db/client".to_string())
    );
}

#[test]
fn require_and_dynamic_import_calls_are_extracted() {
    assert_eq!(
        ArchitectureGuard::extract_imported_module("const db = require(\"../db/client\");"),
        Some("../db/client".to_string())
    );
    assert_eq!(
        ArchitectureGuard::extract_imported_module("const m = await import('./heavy');"),
        Some("./heavy".to_string())
    );
}

#[test]
fn side_effect_import_is_extracted() {
    assert_eq!(
        ArchitectureGuard::extract_imported_module("import \"./polyfill\";"),
        Some("./polyfill".to_string())
    );
}

#[test]
fn go_block_import_lines_extract_paths() {
    assert_eq!(
        ArchitectureGuard::extract_imported_module("\"fmt\""),
        Some("fmt".to_string())
    );
    assert_eq!(
        ArchitectureGuard::extract_imported_module("mysql \"github.com/go-sql-driver/mysql\""),
        Some("github.com/go-sql-driver/mysql".to_string())
    );
}

#[test]
fn comment_lines_never_yield_phantom_imports() {
    assert_eq!(
        ArchitectureGuard::extract_imported_module("// import { x } from \"../evil\""),
        None
    );
    assert_eq!(
        ArchitectureGuard::extract_imported_module("# import os"),
        None
    );
    assert_eq!(
        ArchitectureGuard::extract_imported_module("/* from somewhere */"),
        None
    );
    assert_eq!(
        ArchitectureGuard::extract_imported_module(" * see docs from elsewhere"),
        None
    );
}
