//! Import-alias resolution (F2.3): maps tsconfig/jsconfig
//! `compilerOptions.paths` entries onto canonical workspace-relative
//! candidates so absolute boundary globs catch aliased imports.
//!
// ponytail: strict-JSON parsing with a hand-rolled JSONC stripper instead of a json5 crate; upgrade when a real-world tsconfig fails to parse

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Precomputed alias table; targets are workspace-relative templates
/// (still containing the source pattern's `*`) so match-time resolution
/// is pure string substitution.
#[derive(Debug, Default, Clone)]
pub struct AliasMap {
    wildcards: Vec<(String, String, String)>,
    exact: HashMap<String, Vec<String>>,
}

impl AliasMap {
    /// Loads aliases from `<root>/tsconfig.json`, falling back to
    /// `<root>/jsconfig.json`. Any read/parse failure degrades to an
    /// empty map (specifiers stay literal) rather than failing the guard.
    pub fn load(workspace_root: &Path) -> Self {
        for name in ["tsconfig.json", "jsconfig.json"] {
            if let Ok(raw) = std::fs::read_to_string(workspace_root.join(name)) {
                if let Some(map) = Self::parse(&raw) {
                    return map;
                }
            }
        }
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.wildcards.is_empty() && self.exact.is_empty()
    }

    /// Canonical workspace-relative candidates for an aliased specifier.
    pub fn candidates(&self, specifier: &str) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(targets) = self.exact.get(specifier) {
            out.extend(targets.iter().cloned());
        }
        for (prefix, suffix, template) in &self.wildcards {
            if specifier.len() >= prefix.len() + suffix.len()
                && specifier.starts_with(prefix.as_str())
                && specifier.ends_with(suffix.as_str())
            {
                let middle = &specifier[prefix.len()..specifier.len() - suffix.len()];
                out.push(template.replace('*', middle));
            }
        }
        out
    }

    fn parse(raw: &str) -> Option<Self> {
        let cfg: TsConfig = serde_json::from_str(&strip_jsonc(raw)).ok()?;
        let options = cfg.compiler_options?;
        let paths = options.paths?;

        // Targets must stay workspace-RELATIVE to match boundary globs, so the
        // base is folded lexically; absolute bases are unusable here and
        // degrade to an empty map.
        let base = match options.base_url.as_deref().map(str::trim) {
            None | Some("") => String::new(),
            Some(b) if !b.starts_with('/') => normalize_rel(b),
            _ => return None,
        };

        let mut map = AliasMap::default();
        for (key, entry) in paths {
            let targets = match entry {
                StringOrList::One(s) => vec![s],
                StringOrList::Many(list) => list,
            };
            for target in targets {
                let folded = normalize_join(&base, &target);
                if let Some((prefix, suffix)) = key.split_once('*') {
                    map.wildcards
                        .push((prefix.to_string(), suffix.to_string(), folded));
                } else {
                    map.exact.entry(key.clone()).or_default().push(folded);
                }
            }
        }
        (!map.wildcards.is_empty() || !map.exact.is_empty()).then_some(map)
    }
}

#[derive(Deserialize)]
struct TsConfig {
    #[serde(rename = "compilerOptions", alias = "compiler_options")]
    compiler_options: Option<CompilerOptions>,
}

#[derive(Deserialize)]
struct CompilerOptions {
    #[serde(rename = "baseUrl")]
    base_url: Option<String>,
    paths: Option<HashMap<String, StringOrList>>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrList {
    One(String),
    Many(Vec<String>),
}

/// Joins and collapses `.`/`..` segments lexically; glob metacharacters
/// like `*` pass through untouched.
fn normalize_join(base: &str, relative: &str) -> String {
    normalize_rel(&format!("{base}/{relative}"))
}

fn normalize_rel(spec: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for segment in spec.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

fn strip_jsonc(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    let mut in_string = false;

    while i < bytes.len() {
        let c = bytes[i];
        if in_string {
            out.push(c as char);
            if c == b'\\' && i + 1 < bytes.len() {
                out.push(bytes[i + 1] as char);
                i += 2;
                continue;
            }
            if c == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' => {
                in_string = true;
                out.push('"');
                i += 1;
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'/' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if i + 1 < bytes.len() && bytes[i + 1] == b'*' => {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
            }
            b',' => {
                let mut j = i + 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && (bytes[j] == b'}' || bytes[j] == b']') {
                    i += 1;
                } else {
                    out.push(',');
                    i += 1;
                }
            }
            _ => {
                out.push(c as char);
                i += 1;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard_from(root: &Path, from: &str, forbidden: &str) -> super::super::ArchitectureGuard {
        use super::super::{rules::RuleConfig, ArchitectureGuard};
        ArchitectureGuard {
            alias_map: AliasMap::load(root),
            workspace_root: root.to_path_buf(),
            barrel_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
            config: RuleConfig {
                version: "1.0".into(),
                project_name: None,
                preset: None,
                boundaries: vec![crate::rules::BoundaryRule {
                    name: "No Secrets".into(),
                    from: from.into(),
                    cannot_import: vec![forbidden.into()],
                    message: "aliased secrets are still secrets".into(),
                }],
                limits: Default::default(),
                security: Default::default(),
            },
        }
    }

    fn write_config(root: &Path, name: &str, json: &str) {
        std::fs::write(root.join(name), json).unwrap();
    }

    #[test]
    fn wildcard_alias_resolves_and_violates() {
        let temp = tempfile::tempdir().unwrap();
        write_config(
            temp.path(),
            "tsconfig.json",
            r#"{
              // Next.js-style aliases, comments + trailing comma included
              "compilerOptions": {
                "baseUrl": ".",
                "paths": {
                  "@/*": ["./src/*"],
                },
              },
            }"#,
        );
        let guard = guard_from(temp.path(), "apps/web/**", "src/db/**");
        assert!(
            guard
                .check_import("apps/web/app/page.tsx", "@/db/client")
                .is_some(),
            "aliased db import must violate"
        );
        assert!(
            guard
                .check_import("apps/web/app/page.tsx", "@/components/ui")
                .is_none(),
            "non-forbidden alias target stays clean"
        );
    }

    #[test]
    fn jsconfig_fallback_works() {
        let temp = tempfile::tempdir().unwrap();
        write_config(
            temp.path(),
            "jsconfig.json",
            r#"{"compilerOptions":{"baseUrl":".","paths":{"~/*":["app/*"]}}}"#,
        );
        let guard = guard_from(temp.path(), "**/*.tsx", "app/secrets/**");
        assert!(guard
            .check_import("web/Page.tsx", "~/secrets/tokens")
            .is_some());
    }

    #[test]
    fn exact_key_maps_without_wildcard() {
        let temp = tempfile::tempdir().unwrap();
        write_config(
            temp.path(),
            "tsconfig.json",
            r#"{"compilerOptions":{"baseUrl":".","paths":{"@auth":"src/auth/index.ts"}}}"#,
        );
        let guard = guard_from(temp.path(), "apps/**", "src/auth/**");
        assert!(guard.check_import("apps/x.ts", "@auth").is_some());
    }

    #[test]
    fn malformed_json_degrades_to_literal_matching() {
        let temp = tempfile::tempdir().unwrap();
        write_config(temp.path(), "tsconfig.json", "{ not json ,,");
        let guard = guard_from(temp.path(), "src/**", "src/db/**");
        assert!(guard.alias_map.is_empty());
        assert!(guard.check_import("src/a.ts", "@/db/client").is_none());
    }

    #[test]
    fn missing_configs_yield_empty_map() {
        let temp = tempfile::tempdir().unwrap();
        let map = AliasMap::load(temp.path());
        assert!(map.is_empty());
        assert!(map.candidates("@/x").is_empty());
    }

    /// EC-V4: unreadable config (permission-denied) must degrade to an empty
    /// map exactly like a missing file — never fail the guard.
    #[cfg(unix)]
    #[test]
    fn unreadable_tsconfig_degrades_to_empty() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("tsconfig.json");
        std::fs::write(
            &path,
            r#"{"compilerOptions":{"paths":{"@/*":["./src/*"]}}}"#,
        )
        .unwrap();
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o000);
        std::fs::set_permissions(&path, perms).unwrap();

        let map = AliasMap::load(temp.path());
        assert!(map.is_empty(), "unreadable config must degrade silently");
    }
}
