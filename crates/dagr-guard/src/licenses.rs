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

pub fn check_declared_licenses(workspace_root: &Path, allowed: &[String]) -> Vec<LicenseViolation> {
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
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"x\"\nlicense = \"GPL-3.0\"\n",
        )
        .unwrap();
        let v = check_declared_licenses(&dir, &allow(DEFAULT_ALLOWLIST));
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].kind, "disallowed");
        assert_eq!(v[0].found.as_deref(), Some("GPL-3.0"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_declaration_is_a_violation() {
        let dir = temp_ws("missing");
        std::fs::write(
            dir.join("package.json"),
            r#"{"name":"y","version":"1.0.0"}"#,
        )
        .unwrap();
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
        std::fs::write(
            sub.join("Cargo.toml"),
            "[package]\nlicense.workspace = true\n",
        )
        .unwrap();

        let v = check_declared_licenses(&dir, &allow(DEFAULT_ALLOWLIST));
        assert!(v.is_empty(), "expected clean: {v:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn node_modules_and_target_are_not_scanned() {
        let dir = temp_ws("skipdirs");
        std::fs::create_dir_all(dir.join("node_modules/pkg")).unwrap();
        std::fs::create_dir_all(dir.join("target/pkg")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nlicense = \"MIT\"\n").unwrap();
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

// ---------------------------------------------------------------------------
// Transitive dependency licenses (SBOM-lite)
//
// ponytail: shells out to `cargo metadata` instead of adding a TOML/cargo
// crate; upgrade only if lockfile-less environments make the subprocess
// unreliable.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepLicenseViolation {
    pub name: String,
    pub version: String,
    pub kind: String, // "missing" | "disallowed"
    pub found: Option<String>,
}

/// Practical default: everything above plus licenses overwhelmingly used by
/// the Rust/JS ecosystems' infrastructure crates.
pub const DEFAULT_RUNTIME_ALLOWLIST_EXTRAS: &[&str] = &[
    "Unicode-3.0",
    "Zlib",
    "MPL-2.0",
    "BSL-1.0",
    "Apache-2.0 WITH LLVM-exception",
];

fn full_allowlist(extra: &[String]) -> Vec<String> {
    let mut v: Vec<String> = DEFAULT_ALLOWLIST
        .iter()
        .chain(DEFAULT_RUNTIME_ALLOWLIST_EXTRAS)
        .map(|s| s.to_string())
        .collect();
    v.extend(extra.iter().cloned());
    v.sort();
    v.dedup();
    v
}

/// Minimal SPDX expression evaluator supporting parentheses, AND/OR,
/// `+` suffixes, WITH-exceptions, and the `/` shorthand for OR.
/// Semantics: OR requires one allowed branch; AND requires every branch.
///
/// ponytail: hand-rolled recursive descent instead of an spdx crate.
pub fn spdx_allowed(expr: &str, allow: &[String]) -> bool {
    let normalized = expr.replace('/', " OR ");
    let mut tokens: Vec<String> = normalized
        .replace('(', " ( ")
        .replace(')', " ) ")
        .split_whitespace()
        .map(String::from)
        .collect();
    tokens.reverse(); // pop() from the front via pop-back trickery below
    tokens.reverse();
    let mut toks = tokens;
    parse_or(&mut toks, allow)
}

fn pop_front(toks: &mut Vec<String>) -> Option<String> {
    if toks.is_empty() {
        None
    } else {
        Some(toks.remove(0))
    }
}

fn peek_front(toks: &[String]) -> Option<&String> {
    toks.first()
}

fn parse_or(toks: &mut Vec<String>, allow: &[String]) -> bool {
    let mut ok = parse_and(toks, allow);
    while peek_front(toks).map(|t| t == "OR").unwrap_or(false) {
        pop_front(toks);
        ok |= parse_and(toks, allow);
    }
    ok
}

fn parse_and(toks: &mut Vec<String>, allow: &[String]) -> bool {
    let mut ok = parse_unit(toks, allow);
    while peek_front(toks).map(|t| t == "AND").unwrap_or(false) {
        pop_front(toks);
        ok &= parse_unit(toks, allow);
    }
    ok
}

fn parse_unit(toks: &mut Vec<String>, allow: &[String]) -> bool {
    let tok = match pop_front(toks) {
        Some(t) => t,
        None => return false,
    };
    if tok == "(" {
        let inner = parse_or(toks, allow);
        // consume closing paren
        let _ = pop_front(toks);
        return inner;
    }
    let mut id = tok;
    if peek_front(toks).map(|t| t == "WITH").unwrap_or(false) {
        pop_front(toks);
        if let Some(exc) = pop_front(toks) {
            id = format!("{id} WITH {exc}");
        }
    }
    let base = id.trim_end_matches('+');
    allow.iter().any(|a| a.eq_ignore_ascii_case(base))
}

/// Resolves the full dependency graph via `cargo metadata` and checks every
/// non-workspace package's declared license against the allowlist.
pub fn check_dependency_licenses(
    workspace_root: &Path,
    allowed_extra: &[String],
) -> Result<Vec<DepLicenseViolation>, String> {
    let output = std::process::Command::new("cargo")
        .current_dir(workspace_root)
        .args(["metadata", "--format-version", "1"])
        .output()
        .map_err(|e| format!("failed to spawn cargo: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| e.to_string())?;

    let ws_members: Vec<String> = v["workspace_members"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|x| x.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let allow = full_allowlist(allowed_extra);
    let mut out = Vec::new();
    for pkg in v["packages"].as_array().into_iter().flatten() {
        let id = pkg["id"].as_str().unwrap_or_default();
        if ws_members.iter().any(|m| m == id) {
            continue;
        }
        let name = pkg["name"].as_str().unwrap_or("?").to_string();
        let version = pkg["version"].as_str().unwrap_or("?").to_string();
        let license = pkg["license"]
            .as_str()
            .map(String::from)
            .filter(|s| !s.is_empty());
        match license {
            None => out.push(DepLicenseViolation {
                name,
                version,
                kind: "missing".into(),
                found: None,
            }),
            Some(l) => {
                if !spdx_allowed(&l, &allow) {
                    out.push(DepLicenseViolation {
                        name,
                        version,
                        kind: "disallowed".into(),
                        found: Some(l),
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

#[cfg(test)]
mod dep_tests {
    use super::*;

    #[test]
    fn self_scan_finds_no_violations_in_this_workspace() {
        let ws = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let extra: Vec<String> = DEFAULT_RUNTIME_ALLOWLIST_EXTRAS
            .iter()
            .map(|s| s.to_string())
            .collect();
        let v = check_dependency_licenses(&ws, &extra).expect("cargo metadata must succeed");
        assert!(
            v.is_empty(),
            "workspace deps must pass the practical allowlist: {v:#?}"
        );
    }

    #[test]
    fn spdx_or_expressions_pass_when_one_side_is_allowed() {
        // Directly exercise the split logic through a synthetic scan of a
        // workspace whose metadata we cannot fake — so test the matcher.
        let allow = full_allowlist(&[]);
        let expr = "Apache-2.0 OR MIT";
        assert!(expr
            .split(" OR ")
            .any(|p| allow.iter().any(|a| a.eq_ignore_ascii_case(p.trim()))));
        let bad = "GPL-3.0-only";
        assert!(!bad
            .split(" OR ")
            .any(|p| allow.iter().any(|a| a.eq_ignore_ascii_case(p.trim()))));
        // Slash-form dual licensing must pass like its OR-form twin.
        assert!(spdx_allowed("MIT/Apache-2.0", &allow));
        // Parenthesized AND groups evaluate both branches.
        assert!(spdx_allowed(
            "(MIT OR Apache-2.0) AND Unicode-3.0",
            &full_allowlist(
                DEFAULT_RUNTIME_ALLOWLIST_EXTRAS
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .as_slice(),
            )
        ));
        assert!(!spdx_allowed(
            "(MIT OR GPL-3.0-only) AND GPL-3.0-only",
            &allow
        ));
    }
}
