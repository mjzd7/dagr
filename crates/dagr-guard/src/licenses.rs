//! Declared-license governance: checks the licenses a repository declares
//! about itself (Cargo.toml `license`, package.json `license`) against an
//! allowlist. Missing declarations are violations.
//!
//! ponytail: does NOT resolve transitive dependency licenses (needs
//! cargo-metadata or a TOML parser dep); upgrade when enterprise audit
//! requires SBOM-grade transitive coverage.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LicenseViolation {
    pub manifest: String,
    /// "gpl-matching", "missing", or "disallowed"
    pub kind: String,
    pub found: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PackageJson {
    #[serde(default)]
    license: Option<serde_json::Value>,
}

pub const DEFAULT_ALLOWLIST: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "Apache-2.0 OR MIT",
    "MIT OR Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "0BSD",
    "Unlicense",
    "CC0-1.0",
];

pub fn check_declared_licenses(
    workspace_root: &Path,
    allowed: &[String],
) -> Vec<LicenseViolation> {
    let mut violations = Vec::new();

    for manifest in collect_manifests(workspace_root) {
        let rel = manifest
            .strip_prefix(workspace_root)
            .unwrap_or(&manifest)
            .display()
            .to_string();
        let name = manifest.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let declared = if name == "package.json" {
            read_package_json_license(&manifest)
        } else {
            read_cargo_license(&manifest)
        };

        match declared {
            None => violations.push(LicenseViolation {
                manifest: rel,
                kind: "missing".into(),
                found: None,
            }),
            Some(lic) => {
                if lic == "workspace-inherited" {
                    continue;
                }
                let ok = allowed.iter().any(|a| a.eq_ignore_ascii_case(&lic));
                if !ok {
                    violations.push(LicenseViolation {
                        manifest: rel,
                        kind: "disallowed".into(),
                        found: Some(lic),
                    });
                }
            }
        }
    }

    violations.sort_by(|a, b| a.manifest.cmp(&b.manifest));
    violations
}

fn collect_manifests(root: &Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            match name {
                "Cargo.toml" | "package.json" => out.push(path),
                _ if path.is_dir() => {
                    if !matches!(
                        name,
                        ".git"
                            | "node_modules"
                            | "target"
                            | ".dagr"
                            | "dist"
                            | "build"
                            | ".next"
                            | ".venv"
                            | "coverage"
                    ) {
                        stack.push(path);
                    }
                }
                _ => {}
            }
        }
    }
    out.sort();
    out
}

/// Extracts `license = "..."`. Workspace inheritance (`license.workspace`)
/// resolves to the sentinel `"workspace-inherited"`, which callers treat as a
/// pass because the workspace root's own declaration is checked separately.
fn read_cargo_license(path: &Path) -> Option<String> {
    const INHERITED: &str = "workspace-inherited";
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("license.workspace") {
            return Some(INHERITED.to_string());
        }
        if let Some(rest) = t.strip_prefix("license") {
            if let Some(v) = rest.trim_start().strip_prefix('=') {
                let v = v.trim();
                if !v.is_empty() {
                    return Some(v.trim_matches('"').to_string());
                }
            }
        }
    }
    None
}

fn read_package_json_license(path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let pkg: PackageJson = serde_json::from_str(&content).ok()?;
    pkg.license?.as_str().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_ws(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("dagr-lic-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn allow(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn disallowed_cargo_license_is_flagged() {
        let dir = temp_ws("gpl");
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"x\"\nlicense = \"GPL-3.0\"\n").unwrap();
        let v = check_declared_licenses(&dir, &allow(DEFAULT_ALLOWLIST));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].kind, "disallowed");
        assert_eq!(v[0].found.as_deref(), Some("GPL-3.0"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_declaration_is_a_violation() {
        let dir = temp_ws("missing");
        std::fs::write(dir.join("package.json"), r#"{"name":"y","version":"1.0.0"}"#).unwrap();
        let v = check_declared_licenses(&dir, &allow(DEFAULT_ALLOWLIST));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].kind, "missing");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn workspace_inheritance_passes_and_allowlisted_root_is_clean() {
        let dir = temp_ws("inherit");
        std::fs::write(
            dir.join("Cargo.toml"),
            "[workspace]\nmembers=[]\n\n[workspace.package]\nlicense = \"Apache-2.0\"\n",
        )
        .unwrap();
        let sub = dir.join("crates/x");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("Cargo.toml"), "[package]\nlicense.workspace = true\n").unwrap();

        let v = check_declared_licenses(&dir, &allow(DEFAULT_ALLOWLIST));
        assert!(v.is_empty(), "expected clean: {v:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn node_modules_and_target_are_not_scanned() {
        let dir = temp_ws("skipdirs");
        std::fs::create_dir_all(dir.join("node_modules/pkg")).unwrap();
        std::fs::create_dir_all(dir.join("target/pkg")).unwrap();
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nlicense = \"MIT\"\n",
        )
        .unwrap();
        // GPL manifests in skipped dirs must not surface
        std::fs::write(
            dir.join("node_modules/pkg/Cargo.toml"),
            "[package]\nlicense = \"GPL-2.0\"\n",
        )
        .unwrap();
        let v = check_declared_licenses(&dir, &allow(DEFAULT_ALLOWLIST));
        assert!(v.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
