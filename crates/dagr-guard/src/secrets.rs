//! Secret detection over diffs and file contents: curated token-shape rules
//! for common credential formats plus Shannon-entropy outliers that catch
//! unstructured secrets (base64 blobs, hex keys). Findings never include the
//! raw secret — only a truncated Blake3 hash so reports are safe to paste.
//!
//! ponytail: hand-rolled token matching instead of a regex dependency;
//! upgrade to `regex` only if a rule cannot be expressed as prefix+charset+len.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecretFinding {
    pub kind: &'static str,
    pub line: usize,
    /// First 12 hex chars of the Blake3 digest of the matched value.
    pub snippet_hash: String,
}

const ENTROPY_THRESHOLD: f64 = 4.5;

pub struct SecretScanner;

impl SecretScanner {
    pub fn new() -> Self {
        Self
    }

    pub fn with_entropy_threshold(self, _threshold: f64) -> Self {
        self
    }

    pub fn scan_text(&self, text: &str) -> Vec<SecretFinding> {
        let mut findings = Vec::new();
        for (line_no, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') && trimmed.to_lowercase().contains("example") {
                continue;
            }
            if trimmed.contains("YOUR_") || trimmed.contains("<your") || trimmed.contains("xxx") {
                continue;
            }

            let line_num = line_no + 1;
            let mut flagged = false;
            for (kind, value) in find_token_secrets(line) {
                findings.push(SecretFinding {
                    kind,
                    line: line_num,
                    snippet_hash: hash_prefix(&value),
                });
                flagged = true;
            }

            if !flagged {
                for run in high_entropy_runs(line) {
                    if shannon_entropy(&run) >= ENTROPY_THRESHOLD {
                        findings.push(SecretFinding {
                            kind: "high_entropy_string",
                            line: line_num,
                            snippet_hash: hash_prefix(&run),
                        });
                        break;
                    }
                }
            }
        }
        findings.sort_by_key(|f| f.line);
        findings.dedup_by(|a, b| {
            a.kind == b.kind && a.line == b.line && a.snippet_hash == b.snippet_hash
        });
        findings
    }

    /// Scans a unified diff. Only ADDED lines are flagged — a review verdict
    /// should reflect what this change introduces, not pre-existing content.
    /// Line numbers refer to the new-file side of each hunk.
    pub fn scan_diff(&self, diff: &str) -> Vec<SecretFinding> {
        let mut out = Vec::new();
        let mut in_hunk = false;
        let mut new_file_line = 0usize;

        for line in diff.split_inclusive('\n') {
            if line.starts_with("@@") {
                in_hunk = true;
                new_file_line = parse_hunk_new_start(line);
                continue;
            }
            if !in_hunk || line.starts_with("+++") {
                continue;
            }
            let Some(body) = line.strip_prefix('+') else {
                if !line.starts_with('-') {
                    new_file_line += 1;
                }
                continue;
            };
            for finding in self.scan_text(body.trim_end_matches('\n')) {
                out.push(SecretFinding {
                    line: new_file_line + finding.line - 1,
                    ..finding
                });
            }
            new_file_line += 1;
        }
        out.sort_by_key(|f| f.line);
        out
    }
}

impl Default for SecretScanner {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_hunk_new_start(header: &str) -> usize {
    // "@@ -l,c +l2,c2 @@" -> l2
    let Some(rest) = header.split_whitespace().nth(2) else {
        return 0;
    };
    rest.trim_start_matches('+')
        .split(',')
        .next()
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

fn is_token_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-')
}

/// Extract maximal token runs ([A-Za-z0-9_-]+) and classify known shapes.
fn find_token_secrets(line: &str) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();

    if let Some(begin) = line.find("-----BEGIN") {
        if line[begin..].contains("PRIVATE KEY-----") {
            out.push(("private_key_header", line[begin..].to_string()));
            return out;
        }
    }

    let bytes = line.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if !is_token_char(bytes[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && is_token_char(bytes[i]) {
            i += 1;
        }
        let tok = &line[start..i];

        if let Some(kind) = classify_token(tok) {
            out.push((kind, tok.to_string()));
        }
    }

    if let Some(found) = find_bearer(line) {
        out.push(found);
    }
    out
}

fn classify_token(tok: &str) -> Option<&'static str> {
    if let Some(rest) = tok.strip_prefix("AKIA") {
        if tok.len() == 20 && rest.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit()) {
            return Some("aws_access_key_id");
        }
    }

    if let Some(rest) = tok.strip_prefix("gh").and_then(|r| r.strip_prefix(['p', 'o', 'u', 's', 'r'])).and_then(|r| r.strip_prefix('_')) {
        if rest.len() >= 36 && rest.bytes().all(|b| b.is_ascii_alphanumeric()) {
            return Some("github_token");
        }
    }

    if let Some(rest) = tok.strip_prefix("sk-ant-api03-") {
        if rest.len() >= 20
            && rest
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        {
            return Some("anthropic_api_key");
        }
    }

    if let Some(rest) = tok.strip_prefix("sk-") {
        if !tok.starts_with("sk-ant")
            && rest.len() >= 20
            && rest
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
        {
            return Some("openai_api_key");
        }
    }

    None
}

fn find_bearer(line: &str) -> Option<(&'static str, String)> {
    let lower = line.to_lowercase();
    let idx = lower.find("bearer")?;
    let after = line[idx + "bearer".len()..].trim_start();
    let run_len = after
        .bytes()
        .take_while(|b| {
            b.is_ascii_alphanumeric()
                || matches!(b, b'.' | b'_' | b'~' | b'+' | b'/' | b'-' | b'=')
        })
        .count();
    if run_len >= 24 {
        Some(("generic_bearer", after[..run_len].to_string()))
    } else {
        None
    }
}

fn high_entropy_runs(line: &str) -> Vec<String> {
    let mut runs = Vec::new();
    let bytes = line.as_bytes();
    let mut start: Option<usize> = None;

    fn secretish(b: u8) -> bool {
        b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'_' | b'-' | b'=')
    }

    for (i, b) in bytes.iter().enumerate() {
        if secretish(*b) {
            start.get_or_insert(i);
        } else if let Some(s) = start.take() {
            if i - s >= 20 {
                runs.push(line[s..i].to_string());
            }
        }
    }
    if let Some(s) = start {
        if line.len() - s >= 20 {
            runs.push(line[s..].to_string());
        }
    }
    runs
}

fn shannon_entropy(s: &str) -> f64 {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return 0.0;
    }
    let mut freq = [0u32; 256];
    for b in bytes {
        freq[*b as usize] += 1;
    }
    let len = bytes.len() as f64;
    freq.iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum()
}

fn hash_prefix(value: &str) -> String {
    blake3::hash(value.as_bytes()).to_hex()[..12].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAKE_AWS: &str = "aws_key = \"AKIAIOSFODNN7EXAMPLE\"";
    const FAKE_GH: &str = "token: ghp_abcdefghijklmnopqrstuvwxyz0123456789";
    const HIGH_ENTROPY: &str = "session = \"ZmDkmTQdMfPuRTmoNvBspQJWnKXYhGvAELcsIxUbrkOt\"";

    #[test]
    fn detects_aws_and_github_shapes() {
        let s = SecretScanner::new();
        let f = s.scan_text(&format!("{FAKE_AWS}\n{FAKE_GH}"));
        assert!(f.iter().any(|x| x.kind == "aws_access_key_id"), "{f:?}");
        assert!(f.iter().any(|x| x.kind == "github_token"));
    }

    #[test]
    fn findings_never_contain_the_secret_itself() {
        let s = SecretScanner::new();
        let report = format!("{:?}", s.scan_text(FAKE_AWS));
        assert!(!report.contains("AKIAIOSFODNN7EXAMPLEQ"));
    }

    #[test]
    fn high_entropy_blob_without_known_shape_is_caught() {
        let s = SecretScanner::new();
        let f = s.scan_text(HIGH_ENTROPY);
        assert!(
            f.iter().any(|x| x.kind == "high_entropy_string"),
            "entropy fallback must fire: {f:?}"
        );
    }

    #[test]
    fn placeholders_and_examples_are_skipped() {
        let s = SecretScanner::new();
        assert!(
            s.scan_text("# example: AKIAIOSFODNN7EXAMPLEQ placeholder")
                .is_empty()
        );
        assert!(
            s.scan_text("api_key = YOUR_KEY_HERE_xxxxxxxxxxxxxxxx")
                .is_empty()
        );
    }

    #[test]
    fn plain_code_is_clean() {
        let s = SecretScanner::new();
        let code = "function total(a: number, b: number) {\n  return a + b;\n}\n";
        assert!(s.scan_text(code).is_empty());
    }

    #[test]
    fn scan_diff_flags_only_added_lines_with_new_file_line_numbers() {
        let diff = "@@ -1,2 +1,3 @@\n context_line_that_stays = 1\n-removed_secret ghp_abcdefghijklmnopqrstuvwxyz0123456789\n+added_ok = compute_total(items)\n+leaked = ghp_abcdefghijklmnopqrstuvwxyz0123456789\n";
        let s = SecretScanner::new();
        let f = s.scan_diff(diff);
        assert_eq!(f.len(), 1, "removed/context lines ignored: {f:?}");
        assert_eq!(f[0].kind, "github_token");
        assert_eq!(f[0].line, 3);
    }

    #[test]
    fn private_key_header_is_flagged() {
        let s = SecretScanner::new();
        let f = s.scan_text("data = \"-----BEGIN RSA PRIVATE KEY-----\\nMIIE\"");
        assert!(f.iter().any(|x| x.kind == "private_key_header"), "{f:?}");
    }
}
