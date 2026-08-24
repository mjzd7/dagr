//! Governance surface: `dagr prove` (signed audit receipts) and
//! `dagr review-diff` (merge gate with PASS/BLOCKED verdicts).
//!
//! Composes existing engines — guard (policy), secrets/licenses (scanning),
//! slicer reverse-index (blast radius), sandbox (verification) — into
//! CI-consumable artifacts. No new engines here by design.
//!
//! ponytail: risk-score weights are documented constants + env overrides
//! instead of rules.yaml keys (strict fail-closed schema stays untouched);
//! upgrade when calibration data from evals/ justifies per-project weights.

use dagr_core::{DagrError, Result};
use dagr_guard::{
    check_declared_licenses, ArchitectureGuard, LicenseViolation, SecretFinding, SecretScanner,
};
use dagr_sandbox::CowSandbox;
use dagr_slicer::{ImportRef, ReverseIndex};
use std::path::{Path, PathBuf};

pub const RECEIPT_SCHEMA_VERSION: u8 = 1;
pub const VERDICT_SCHEMA_VERSION: u8 = 1;

const W_DANGLING_IMPORT: u32 = 40;
const W_GUARD_VIOLATION: u32 = 25;

fn weight(name: &str, default: u32) -> u32 {
    std::env::var(format!("DAGR_RISK_W_{}", name))
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

// ---------------------------------------------------------------------------
// B4: proof receipts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TestOutcome {
    pub command: String,
    pub success: bool,
    pub stderr_tail: String,
}

#[derive(Debug, Clone)]
pub struct ProofReceipt {
    pub schema_version: u8,
    pub workspace: String,
    pub tool_version: String,
    pub generated_at_unix: u64,
    pub rules_enforced: usize,
    pub files_indexed: usize,
    pub guard_violations: usize,
    pub secret_findings: Vec<SecretFinding>,
    pub license_violations: Vec<LicenseViolation>,
    pub tests: Option<TestOutcome>,
    /// Blake3 over the canonical JSON of every field above except itself.
    pub digest: String,
}

impl ProofReceipt {
    pub fn generate(workspace_root: &Path, test_command: Option<&str>) -> Result<Self> {
        let guard = ArchitectureGuard::load(workspace_root)?;
        let rules_enforced = guard.config.boundaries.len();
        let guard_violations = guard.scan_workspace(workspace_root)?.len();

        let scanner = SecretScanner::new();
        let mut secret_findings = Vec::new();
        let indexed_sources = collect_source_texts(workspace_root);
        for (rel, body) in &indexed_sources {
            for f in scanner.scan_text(body) {
                secret_findings.push(SecretFinding {
                    line: f.line,
                    snippet_hash: f.snippet_hash,
                    kind: f.kind,
                });
            }
            let _ = rel;
        }

        let allowlist: Vec<String> = dagr_guard::DEFAULT_ALLOWLIST
            .iter()
            .map(|s| s.to_string())
            .collect();
        let license_violations = check_declared_licenses(workspace_root, &allowlist);

        let tests = test_command.map(|cmd| run_in_sandbox(workspace_root, cmd));

        let files_indexed = indexed_sources.len();
        let mut receipt_no_digest = ProofReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION,
            workspace: workspace_root.display().to_string(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at_unix: unix_now(),
            rules_enforced,
            files_indexed,
            guard_violations,
            secret_findings,
            license_violations,
            tests,
            digest: String::new(),
        };
        let mut canonical_value = receipt_no_digest.to_json();
        {
            let obj = canonical_value.as_object_mut().unwrap();
            obj.remove("digest");
            obj.remove("generated_at_unix");
        }
        let canonical = serde_json::to_string(&canonical_value)?;
        receipt_no_digest.digest =
            blake3::hash(canonical.as_bytes()).to_hex()[..32].to_string();
        Ok(receipt_no_digest)
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "workspace": self.workspace,
            "tool_version": self.tool_version,
            "generated_at_unix": self.generated_at_unix,
            "rules_enforced": self.rules_enforced,
            "files_indexed": self.files_indexed,
            "guard_violations": self.guard_violations,
            "secret_findings": self.secret_findings.iter().map(|f| serde_json::json!({
                "kind": f.kind, "line": f.line, "snippet_hash": f.snippet_hash
            })).collect::<Vec<_>>(),
            "license_violations": self.license_violations.iter().map(|v| serde_json::json!({
                "manifest": v.manifest, "kind": v.kind,
                "found": v.found
            })).collect::<Vec<_>>(),
            "tests": self.tests.as_ref().map(|t| serde_json::json!({
                "command": t.command, "success": t.success, "stderr_tail": t.stderr_tail
            })),
            "digest": self.digest,
        })
    }

    pub fn to_markdown(&self) -> String {
        let status = if self.guard_violations == 0
            && self.secret_findings.is_empty()
            && self.license_violations.is_empty()
            && self.tests.as_ref().map(|t| t.success).unwrap_or(true)
        {
            "✅ VERIFIED"
        } else {
            "❌ FINDINGS PRESENT"
        };
        let tests_line = match &self.tests {
            Some(t) => format!(
                "| Tests | {} (`{}`) |\n",
                if t.success { "pass" } else { "FAIL" },
                t.command
            ),
            None => String::new(),
        };
        format!(
            "<!-- dagr proof receipt -->\n\
             ## 🛡️ dagr proof receipt {}\n\n\
             | Field | Value |\n|---|---|\n\
             | Proof | `{}` |\n\
             | Tool | dagr v{} |\n\
             | Rules enforced | {} |\n\
             | Guard violations | {} |\n\
             | Secrets found | {} |\n\
             | License violations | {} |\n\
             {}\
             > Reproduce with `dagr prove` on this commit.\n",
            status,
            self.digest,
            self.tool_version,
            self.rules_enforced,
            self.guard_violations,
            self.secret_findings.len(),
            self.license_violations.len(),
            tests_line,
        )
    }
}

fn run_in_sandbox(workspace_root: &Path, cmd: &str) -> TestOutcome {
    let outcome = (|| -> Result<(bool, String)> {
        let tx = CowSandbox::begin(workspace_root)?;
        let result = CowSandbox::verify(&tx, cmd)?;
        CowSandbox::rollback(tx)?;
        Ok((
            result.success,
            tail_lines(&result.stderr, 3),
        ))
    })();
    match outcome {
        Ok((success, stderr_tail)) => TestOutcome {
            command: cmd.to_string(),
            success,
            stderr_tail,
        },
        Err(e) => TestOutcome {
            command: cmd.to_string(),
            success: false,
            stderr_tail: format!("sandbox error: {e}"),
        },
    }
}

fn tail_lines(s: &str, n: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join("\n")
}

// ---------------------------------------------------------------------------
// B5: review-diff merge gate
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanglingImport {
    pub importer_file: String,
    pub import_line: usize,
    pub module: String,
    /// None = module path does not exist; Some(missing) = named binding absent.
    pub missing_binding: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileRisk {
    pub file: String,
    pub risk_score: u32,
    pub reasons: Vec<String>,
    pub test_coverage_hint: bool,
}

pub const VERDICT_PASS: &str = "PASS";
pub const VERDICT_BLOCKED: &str = "BLOCKED";
/// Git diff could not be determined (shallow clone, missing ref). Fail-closed.
pub const VERDICT_UNKNOWN: &str = "UNKNOWN";

#[derive(Debug, Clone, Default)]
pub struct ReviewVerdict {
    pub schema_version: u8,
    pub verdict: String,
    /// Human-readable explanation when verdict is UNKNOWN.
    pub note: Option<String>,
    pub base: String,
    pub head: String,
    pub files_changed: usize,
    pub guard_violation_count: usize,
    pub secret_count: usize,
    pub dangling_imports: Vec<DanglingImport>,
    pub files: Vec<FileRisk>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailOn {
    Blocked,
    Never,
}

impl ReviewVerdict {
    pub fn generate(
        workspace_root: &Path,
        base: &str,
        head: &str,
    ) -> Result<Self> {
        let guard = ArchitectureGuard::load(workspace_root)?;
        let changed = match git_changed_files(workspace_root, base, head) {
            Ok(c) => c,
            Err(e) => {
                return Ok(ReviewVerdict {
                    schema_version: VERDICT_SCHEMA_VERSION,
                    verdict: VERDICT_UNKNOWN.to_string(),
                    note: Some(format!(
                        "could not determine diff {base}...{head}: {e}. \
                         A shallow clone or missing ref blocks review — fetch \
                         full history (actions/checkout fetch-depth: 0)."
                    )),
                    base: base.to_string(),
                    head: head.to_string(),
                    ..Default::default()
                });
            }
        };

        let mut guard_violation_count = 0usize;
        let mut violations_per_file: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for rel in &changed {
            let abs = workspace_root.join(rel);
            let Ok(content) = std::fs::read_to_string(&abs) else {
                continue;
            };
            let mut count = 0usize;
            for line in content.lines() {
                if let Some(imported) = ArchitectureGuard::extract_imported_module(line.trim()) {
                    if guard.check_import(rel, &imported).is_some() {
                        count += 1;
                    }
                }
            }
            if count > 0 {
                violations_per_file.insert(rel.clone(), count);
                guard_violation_count += count;
            }
        }

        let scanner = SecretScanner::new();
        let mut secret_count = 0usize;
        let mut secrets_per_file: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        for rel in &changed {
            let abs = workspace_root.join(rel);
            let Ok(content) = std::fs::read_to_string(&abs) else {
                continue;
            };
            let count = scanner.scan_text(&content).len();
            if count > 0 {
                secrets_per_file.insert(rel.clone(), count);
                secret_count += count;
            }
        }

        let index = ReverseIndex::build(workspace_root)?;
        let scope = DiffScope::load(workspace_root, base, head)?;
        let dangling = detect_dangling_imports(&index, workspace_root, &scope);

        let w_dangling = weight("DANGLING_IMPORT", W_DANGLING_IMPORT);
        let w_guard = weight("GUARD_VIOLATION", W_GUARD_VIOLATION);

        let mut files = Vec::new();
        for rel in &changed {
            let dangle_hits: usize = dangling
                .iter()
                .filter(|d| &d.importer_file == rel)
                .count();
            let guard_hits = violations_per_file.get(rel).copied().unwrap_or(0);
            let secret_hits = secrets_per_file.get(rel).copied().unwrap_or(0);

            let score =
                dangle_hits as u32 * w_dangling
                + guard_hits as u32 * w_guard
                + secret_hits as u32 * 100;
            let mut reasons = Vec::new();
            if dangle_hits > 0 {
                reasons.push(format!("{dangle_hits} dangling import(s)"));
            }
            if guard_hits > 0 {
                reasons.push(format!("{guard_hits} boundary violation(s)"));
            }
            if secret_hits > 0 {
                reasons.push(format!("{secret_hits} secret(s)"));
            }
            files.push(FileRisk {
                file: rel.clone(),
                risk_score: score,
                reasons,
                test_coverage_hint: has_test_sibling(workspace_root, rel),
            });
        }
        files.sort_by(|a, b| b.risk_score.cmp(&a.risk_score));

        let blocked = secret_count > 0 || guard_violation_count > 0 || !dangling.is_empty();
        Ok(ReviewVerdict {
            schema_version: VERDICT_SCHEMA_VERSION,
            verdict: if blocked {
                VERDICT_BLOCKED.to_string()
            } else {
                VERDICT_PASS.to_string()
            },
            note: None,
            base: base.to_string(),
            head: head.to_string(),
            files_changed: changed.len(),
            guard_violation_count,
            secret_count,
            dangling_imports: dangling,
            files,
        })
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "verdict": self.verdict,
            "note": self.note,
            "base": self.base,
            "head": self.head,
            "files_changed": self.files_changed,
            "guard_violation_count": self.guard_violation_count,
            "secret_count": self.secret_count,
            "dangling_imports": self.dangling_imports.iter().map(|d| serde_json::json!({
                "importer_file": d.importer_file,
                "import_line": d.import_line,
                "module": d.module,
                "missing_binding": d.missing_binding,
            })).collect::<Vec<_>>(),
            "files": self.files.iter().map(|f| serde_json::json!({
                "file": f.file,
                "risk_score": f.risk_score,
                "reasons": f.reasons,
                "test_coverage_hint": f.test_coverage_hint,
            })).collect::<Vec<_>>(),
        })
    }

    pub fn to_markdown(&self) -> String {
        let icon = match self.verdict.as_str() {
            VERDICT_PASS => "✅",
            VERDICT_BLOCKED => "⛔",
            _ => "❔",
        };
        let mut md = format!(
            "{} **verdict: {}** — {} file(s) changed vs {}...{}\n\n",
            icon, self.verdict, self.files_changed, self.base, self.head
        );
        if let Some(note) = &self.note {
            md.push_str(&format!("> ⚠️ {note}\n\n"));
        }
        if !self.dangling_imports.is_empty() {
            md.push_str("### Dangling imports\n");
            for d in &self.dangling_imports {
                let detail = match &d.missing_binding {
                    Some(b) => format!("missing binding `{b}`"),
                    None => "module path does not exist".to_string(),
                };
                md.push_str(&format!(
                    "- `{}`:{} imports `{}` — {}\n",
                    d.importer_file, d.import_line, d.module, detail
                ));
            }
            md.push('\n');
        }
        md.push_str("| File | Risk | Coverage hint | Reasons |\n|---|---|---|---|\n");
        for f in &self.files {
            md.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                f.file,
                f.risk_score,
                if f.test_coverage_hint { "✓" } else { "—" },
                if f.reasons.is_empty() {
                    "clean".to_string()
                } else {
                    f.reasons.join(", ")
                }
            ));
        }
        md
    }
}

#[derive(Default)]
struct DiffScope {
    /// Files added/modified in this diff.
    changed: std::collections::HashSet<String>,
    /// Files deleted by this diff.
    deleted: std::collections::HashSet<String>,
}

impl DiffScope {
    fn load(ws: &Path, base: &str, head: &str) -> Result<Self> {
        Ok(Self {
            changed: git_changed_files(ws, base, head)?.into_iter().collect(),
            deleted: git_deleted_files(ws, base, head)?.into_iter().collect(),
        })
    }

    fn touches(&self, imp_file: &str) -> bool {
        self.changed.contains(imp_file) || self.deleted.contains(imp_file)
    }

    fn deletes_module(&self, importer_dir: &Path, spec: &str) -> bool {
        use std::path::Component;
        let joined: PathBuf = importer_dir
            .join(spec)
            .components()
            .filter(|c| !matches!(c, Component::CurDir))
            .collect();
        const CANDIDATES: [&str; 4] = [".ts", ".tsx", "/index.ts", "/index.tsx"];
        CANDIDATES.iter().any(|c| {
            self.deleted.contains(&format!("{}{}", joined.display(), c))
        })
    }
}

/// Detects imports whose target module file no longer exists, plus TS named
/// bindings that no longer resolve anywhere in the index. Scoped to the diff:
/// only imports whose importer changed or whose target was deleted are
/// verdict-relevant — pre-existing breakage elsewhere is not this PR's fault.
fn detect_dangling_imports(
    index: &ReverseIndex,
    ws: &Path,
    scope: &DiffScope,
) -> Vec<DanglingImport> {
    let mut out = Vec::new();
    for imp in index_imports(index) {
        if imp.module.starts_with('@') || !imp.module.starts_with('.') {
            continue;
        }
        let importer_dir = PathBuf::from(&imp.file)
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default();

        // Cheap scope pre-check on the raw spec before resolution.
        let possibly_relevant =
            scope.touches(&imp.file) || scope.deletes_module(&importer_dir, &imp.module);
        if !possibly_relevant {
            continue;
        }

        let resolved = resolve_ts_module(ws, &importer_dir, &imp.module);
        match resolved {
            ModuleResolution::Missing => out.push(DanglingImport {
                importer_file: imp.file.clone(),
                import_line: imp.line,
                module: imp.module.clone(),
                missing_binding: None,
            }),
            ModuleResolution::Found(target_rel) => {
                for binding in index.bindings_imported_from(&imp.file, imp.line) {
                    let defined_in_target = index
                        .definitions_of(binding)
                        .iter()
                        .any(|d| d.file == target_rel);
                    if !defined_in_target {
                        out.push(DanglingImport {
                            importer_file: imp.file.clone(),
                            import_line: imp.line,
                            module: imp.module.clone(),
                            missing_binding: Some(binding.to_string()),
                        });
                    }
                }
            }
        }
    }
    out.sort_by(|a, b| (&a.importer_file, a.import_line).cmp(&(&b.importer_file, b.import_line)));
    out.dedup_by(|a, b| {
        a.importer_file == b.importer_file
            && a.import_line == b.import_line
            && a.missing_binding == b.missing_binding
    });
    out
}

enum ModuleResolution {
    Missing,
    Found(String),
}

fn resolve_ts_module(ws: &Path, importer_dir: &Path, spec: &str) -> ModuleResolution {
    let joined = importer_dir.join(spec);
    let base = joined.strip_prefix(ws).unwrap_or(&joined).to_path_buf();
    const CANDIDATES: [&str; 4] = [".ts", ".tsx", "/index.ts", "/index.tsx"];
    for c in CANDIDATES {
        let cand = PathBuf::from(format!("{}{}", base.display(), c));
        if ws.join(&cand).is_file() {
            return ModuleResolution::Found(cand.display().to_string());
        }
    }
    ModuleResolution::Missing
}

fn index_imports(_index: &ReverseIndex) -> Vec<ImportRef> {
    _index.all_imports()
}

fn has_test_sibling(ws: &Path, rel: &str) -> bool {
    let stem = match rel.rsplit_once('.') {
        Some((s, _)) => s.to_string(),
        None => rel.to_string(),
    };
    let candidates = [
        format!("{stem}.test.ts"),
        format!("{stem}.test.tsx"),
        format!("{stem}_test.rs"),
        format!("{stem}.spec.ts"),
    ];
    candidates.iter().any(|c| ws.join(c).is_file())
        || ws.join("tests").join(stem).exists()
}

fn git_changed_files(ws: &Path, base: &str, head: &str) -> Result<Vec<String>> {
    use std::process::Command;
    let range = format!("{base}...{head}");
    let output = Command::new("git")
        .current_dir(ws)
        .args(["diff", "--name-only", &range])
        .output()
        .map_err(|e| DagrError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    if !output.status.success() {
        return Err(DagrError::Config(format!(
            "git exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

fn git_deleted_files(ws: &Path, base: &str, head: &str) -> Result<Vec<String>> {
    use std::process::Command;
    let range = format!("{base}...{head}");
    let output = Command::new("git")
        .current_dir(ws)
        .args(["diff", "--name-only", "--diff-filter=D", &range])
        .output()
        .map_err(|e| DagrError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    if !output.status.success() {
        return Err(DagrError::Config(format!(
            "git exited {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

fn collect_source_texts(ws: &Path) -> Vec<(String, String)> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
        let skip = [
            ".git", "node_modules", "target", ".dagr", ".next", "dist", "build", "out", ".output",
            ".turbo", ".venv", "venv", "__pycache__", "vendor", "coverage",
        ];
        if let Ok(entries) = std::fs::read_dir(dir) {
            for e in entries.flatten() {
                let p = e.path();
                let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
                if p.is_dir() {
                    if !skip.contains(&name) {
                        walk(&p, root, out);
                    }
                } else if matches!(
                    p.extension().and_then(|s| s.to_str()),
                    Some("ts") | Some("tsx") | Some("js") | Some("rs")
                ) {
                    let rel = p.strip_prefix(root).unwrap_or(&p).display().to_string();
                    if let Ok(body) = std::fs::read_to_string(&p) {
                        out.push((rel, body));
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    walk(ws, ws, &mut out);
    out
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, files: &[(&str, &str)]) {
        for (rel, src) in files {
            let p = dir.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, src).unwrap();
        }
    }

    #[test]
    fn proof_receipt_is_deterministic_excluding_timestamp_and_renders_hash() {
        let dir = std::env::temp_dir().join(format!("dagr-proof-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write(
            &dir,
            &[(".dagr/rules.yaml", "version: \"1.0\"\nboundaries:\n  - name: \"UI-DB\"\n    from: \"src/ui/**\"\n    cannot_import:\n      - \"src/db/**\"\n")],
        );
        let r1 = ProofReceipt::generate(&dir, None).unwrap();
        let r2 = ProofReceipt::generate(&dir, None).unwrap();

        assert_eq!(r1.digest, r2.digest, "digest must ignore timestamp drift");
        assert_eq!(r1.schema_version, 1);
        assert_eq!(r1.rules_enforced, 1);
        assert!(r1.to_markdown().contains(&r1.digest));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn proof_receipt_flags_planted_secret_and_license() {
        let dir = std::env::temp_dir().join(format!("dagr-proof-bad-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write(
            &dir,
            &[
                (
                    "Cargo.toml",
                    "[package]\nname=\"bad\"\nlicense = \"GPL-3.0-only\"\n",
                ),
                (
                    "src/x.ts",
                    "export const k = \"ghp_abcdefghijklmnopqrstuvwxyz0123456789\";\n",
                ),
            ],
        );
        let r = ProofReceipt::generate(&dir, None).unwrap();
        assert_eq!(r.secret_findings.len(), 1);
        assert!(!r.license_violations.is_empty());
        assert!(r.to_markdown().contains("FINDINGS PRESENT"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn review_diff_blocks_on_broken_import_after_deletion() {
        let dir = std::env::temp_dir().join(format!("dagr-rdiff-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();

        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&dir)
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@t")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@t")
                .output()
                .expect("git")
        };

        assert!(git(&["init", "-q"]).status.success());
        write(
            &dir,
            &[
                ("src/a.ts", "export function charge(): number {\n  return 1;\n}\n"),
                ("src/b.ts", "import { charge } from \"./a\";\nexport const total = charge();\n"),
            ],
        );
        assert!(git(&["add", "."]).status.success());
        assert!(git(&["commit", "-q", "-m", "base"]).status.success());

        std::fs::remove_file(dir.join("src/a.ts")).unwrap();
        assert!(git(&["add", "."]).status.success());
        assert!(git(&["commit", "-q", "-m", "drop a"]).status.success());

        let verdict = ReviewVerdict::generate(&dir, "HEAD~1", "HEAD").unwrap();
        assert_eq!(verdict.verdict, "BLOCKED", "{:#?}", verdict.dangling_imports);
        assert!(!verdict.dangling_imports.is_empty());

        let md = verdict.to_markdown();
        assert!(md.contains("Dangling imports"));
        assert!(md.contains("BLOCKED"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn review_diff_passes_clean_change() {
        let dir = std::env::temp_dir().join(format!("dagr-rdiff-ok-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();

        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&dir)
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@t")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@t")
                .output()
                .expect("git")
        };

        assert!(git(&["init", "-q"]).status.success());
        write(
            &dir,
            &[("src/calc.ts", "export function add(a: number, b: number): number {\n  return a + b;\n}\n")],
        );
        assert!(git(&["add", "."]).status.success());
        assert!(git(&["commit", "-q", "-m", "base"]).status.success());

        write(
            &dir,
            &[("src/calc.ts", "export function add(a: number, b: number): number {\n  return a + b;\n}\nexport function sub(a: number, b: number): number {\n  return a - b;\n}\n")],
        );
        assert!(git(&["add", "."]).status.success());
        assert!(git(&["commit", "-q", "-m", "feat sub"]).status.success());

        let verdict = ReviewVerdict::generate(&dir, "HEAD~1", "HEAD").unwrap();
        assert_eq!(verdict.verdict, "PASS", "{:#?}", verdict);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod unknown_verdict_tests {
    use super::*;

    #[test]
    fn missing_base_ref_yields_unknown_never_pass() {
        let dir = std::env::temp_dir().join(format!("dagr-unk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();

        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&dir)
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@t")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@t")
                .output()
                .expect("git")
        };
        assert!(git(&["init", "-q"]).status.success());
        std::fs::write(dir.join("src/ok.ts"), "export const a = 1;\n").unwrap();
        assert!(git(&["add", "."]).status.success());
        assert!(git(&["commit", "-q", "-m", "base"]).status.success());

        let verdict = ReviewVerdict::generate(&dir, "origin/main", "HEAD").unwrap();
        assert_eq!(verdict.verdict, VERDICT_UNKNOWN);
        assert!(verdict.note.as_deref().unwrap().contains("fetch"));
        assert_eq!(verdict.files_changed, 0);

        let md = verdict.to_markdown();
        assert!(md.contains("❔"));
        assert!(md.contains("⚠️"));

        let json = verdict.to_json();
        assert_eq!(json["verdict"], "UNKNOWN");
        assert!(json["note"].is_string());

        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod scoping_tests {
    use super::*;

    /// Regression for the dogfooding finding: pre-existing dangling imports
    /// in untouched files must not fail an unrelated diff.
    #[test]
    fn pre_existing_dangling_import_does_not_block_unrelated_diff() {
        let dir = std::env::temp_dir().join(format!("dagr-scope-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();

        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&dir)
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@t")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@t")
                .output()
                .expect("git")
        };
        assert!(git(&["init", "-q"]).status.success());
        // broken.ts imports a module that never exists — committed as-is.
        std::fs::write(dir.join("src/broken.ts"), "import { x } from \"./missing\";\nexport const y = x;\n").unwrap();
        std::fs::write(dir.join("src/clean.ts"), "export const clean = 1;\n").unwrap();
        assert!(git(&["add", "."]).status.success());
        assert!(git(&["commit", "-q", "-m", "base (with pre-existing breakage)"]).status.success());

        // The diff only touches clean.ts — unrelated to the broken import.
        std::fs::write(dir.join("src/clean.ts"), "export const clean = 2;\n").unwrap();
        assert!(git(&["add", "."]).status.success());
        assert!(git(&["commit", "-q", "-m", "touch clean only"]).status.success());

        let v = ReviewVerdict::generate(&dir, "HEAD~1", "HEAD").unwrap();
        assert_eq!(v.verdict, VERDICT_PASS, "{:#?}", v.dangling_imports);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Deletion side-effects still block even when importers are untouched:
    /// deleting a module that a surviving file imports stays BLOCKED.
    #[test]
    fn deleted_module_still_blocks_untouched_importer() {
        let dir = std::env::temp_dir().join(format!("dagr-scope2-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).unwrap();

        let git = |args: &[&str]| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(&dir)
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@t")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@t")
                .output()
                .expect("git")
        };
        assert!(git(&["init", "-q"]).status.success());
        std::fs::write(dir.join("src/a.ts"), "export function charge(): number {\n  return 1;\n}\n").unwrap();
        std::fs::write(dir.join("src/b.ts"), "import { charge } from \"./a\";\nexport const total = charge();\n").unwrap();
        assert!(git(&["add", "."]).status.success());
        assert!(git(&["commit", "-q", "-m", "base"]).status.success());

        std::fs::remove_file(dir.join("src/a.ts")).unwrap();
        assert!(git(&["add", "-A"]).status.success());
        assert!(git(&["commit", "-q", "-m", "delete a"]).status.success());

        let v = ReviewVerdict::generate(&dir, "HEAD~1", "HEAD").unwrap();
        assert_eq!(v.verdict, VERDICT_BLOCKED, "{:#?}", v.dangling_imports);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
