//! Minimal Language-Server bridge (stdio JSON-RPC 2.0, hand-framed).
//!
//! Purpose: precise cross-file reference finding where tree-sitter's
//! identifier matching is approximate. Rust goes through rust-analyzer;
//! TypeScript needs a language server binary that is not assumed present —
//! detection degrades to None and callers skip LSP silently.
//!
//! ponytail: hand-rolled Content-Length framing instead of an lsp crate;
//! upgrade only if a second consumer needs more than initialize/references.

use dagr_core::{DagrError, Result};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};

/// A single reference hit, ready for JSON output.
#[derive(Debug, Clone)]
pub struct RefHit {
    pub file: String,
    pub line: usize,
}

pub struct LspBridge {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    next_id: i64,
    root_uri: String,
}

type ChildStdout = std::process::ChildStdout;

impl LspBridge {
    /// Finds a usable language server. Env overrides exist so tests can run
    /// against a scripted fake server: `DAGR_LSP_BIN` (+ optional
    /// space-separated `DAGR_LSP_ARGS`).
    pub fn detect(workspace_root: &Path) -> Option<Self> {
        let (bin, args): (String, Vec<String>) = match (
            std::env::var("DAGR_LSP_BIN"),
            std::env::var("DAGR_LSP_ARGS"),
        ) {
            (Ok(b), Ok(a)) => (b, a.split_whitespace().map(String::from).collect()),
            (Ok(b), Err(_)) => (b, vec![]),
            _ => {
                let ra = which_rust_analyzer()?;
                (ra, vec![])
            }
        };

        let mut child = Command::new(&bin)
            .args(&args)
            .current_dir(workspace_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;

        let stdin = child.stdin.take()?;
        let stdout = child.stdout.take()?;
        let mut bridge = Self {
            child,
            stdin,
            reader: BufReader::new(stdout),
            next_id: 0,
            root_uri: path_to_uri(workspace_root),
        };
        if bridge.initialize().is_err() {
            let _ = bridge.child.kill();
            return None;
        }
        Some(bridge)
    }

    fn send(&mut self, value: &Value) -> Result<()> {
        let body = serde_json::to_string(value)?;
        write!(self.stdin, "Content-Length: {}\r\n\r\n{}", body.len(), body)
            .map_err(|e| DagrError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }

    fn initialize(&mut self) -> Result<()> {
        let id = self.next_id();
        self.send(&json!({
            "jsonrpc": "2.0", "id": id, "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "rootUri": self.root_uri,
                "capabilities": {},
            }
        }))?;
        let resp = self.read_response(id)?;
        if resp.get("result").is_none() {
            return Err(DagrError::Config("LSP initialize failed".into()));
        }
        self.send(&json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}))?;
        Ok(())
    }

    fn next_id(&mut self) -> i64 {
        self.next_id += 1;
        self.next_id
    }

    fn peek_id(&self) -> i64 {
        self.next_id + 1
    }

    fn read_response(&mut self, want_id: i64) -> Result<Value> {
        loop {
            let msg = read_framed(&mut self.reader)?;
            let v: Value = serde_json::from_str(&msg)?;
            if v.get("id").and_then(|i| i.as_i64()) == Some(want_id) {
                return Ok(v);
            }
            // Notifications (publishDiagnostics etc.) are skipped.
        }
    }

    /// All references to the symbol at `file`:`line`(1-based)`col`(0-based),
    /// excluding the declaration site itself.
    pub fn references(
        &mut self,
        file: &Path,
        line: usize,
        col: usize,
        include_declaration: bool,
    ) -> Result<Vec<RefHit>> {
        let content = std::fs::read_to_string(file).unwrap_or_default();
        self.references_with_content(file, &content, line, col, include_declaration)
    }

    /// Same as [`references`] but supplies the document text explicitly —
    /// required for files deleted by the diff under review.
    pub fn references_with_content(
        &mut self,
        file: &Path,
        content: &str,
        line: usize,
        col: usize,
        include_declaration: bool,
    ) -> Result<Vec<RefHit>> {
        let uri = path_to_uri(file);
        self.send(&json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": {"textDocument": {
                "uri": uri, "languageId": "rust", "version": 1,
                "text": content,
            }}
        }))?;
        let id = self.next_id();
        self.send(&json!({
            "jsonrpc": "2.0", "id": id,
            "method": "textDocument/references",
            "params": {
                "textDocument": {"uri": uri},
                "position": {"line": line - 1, "character": col},
                "context": {"includeDeclaration": include_declaration},
            }
        }))?;
        let resp = self.read_response(id)?;
        let mut hits = Vec::new();
        if let Some(arr) = resp["result"].as_array() {
            for loc in arr {
                let uri_str = loc["uri"].as_str().unwrap_or_default();
                let l = loc["range"]["start"]["line"].as_u64().unwrap_or(0);
                hits.push(RefHit {
                    file: uri_to_path(uri_str),
                    line: (l + 1) as usize,
                });
            }
        }
        Ok(hits)
    }
}

impl Drop for LspBridge {
    fn drop(&mut self) {
        let shutdown_id = self.peek_id();
        let _ = self.send(&json!({"jsonrpc": "2.0", "id": shutdown_id, "method": "shutdown"}));
        let _ = self.send(&json!({"jsonrpc": "2.0", "method": "exit"}));
        let _ = self.child.wait();
    }
}

fn which_rust_analyzer() -> Option<String> {
    let out = Command::new("which").arg("rust-analyzer").output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
}

fn path_to_uri(p: &Path) -> String {
    format!("file://{}", p.display())
}

fn uri_to_path(uri: &str) -> String {
    uri.strip_prefix("file://").unwrap_or(uri).to_string()
}

/// Reads one `Content-Length`-framed message body.
fn read_framed(reader: &mut impl BufRead) -> Result<String> {
    let mut content_length: Option<usize> = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Err(DagrError::Config("LSP stream closed".into()));
        }
        let t = line.trim_end();
        if t.is_empty() {
            break;
        }
        if let Some(v) = t.strip_prefix("Content-Length:") {
            content_length = v.trim().parse().ok();
        }
    }
    let len = content_length
        .ok_or_else(|| DagrError::Config("LSP frame missing Content-Length".into()))?;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    String::from_utf8(buf).map_err(|e| DagrError::Config(format!("LSP utf8: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Mutex;

    // Env-var mutation is process-global; serialize bridge tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn fake_server_path() -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        p.pop(); // crates/
        p.pop(); // repo root
        p.push("tests/fixtures/fake-lsp.py");
        p
    }

    #[test]
    fn bridge_parses_framed_lsp_responses_from_fake_server() {
        let _guard = ENV_LOCK.lock().unwrap();
        assert!(fake_server_path().exists(), "fake server fixture missing");
        std::env::set_var("DAGR_LSP_BIN", "python3");
        std::env::set_var("DAGR_LSP_ARGS", fake_server_path().display().to_string());

        let ws = std::env::temp_dir();
        let mut bridge =
            LspBridge::detect(&ws).expect("bridge must initialize against scripted server");

        let hits = bridge
            .references_with_content(Path::new("/fake/def.rs"), "fn gone() {}", 1, 3, false)
            .expect("references query must succeed");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].file, "/fake/caller.rs");
        assert_eq!(hits[0].line, 10); // LSP is 0-based; we surface 1-based
        assert_eq!(hits[1].file, "/fake/other.rs");
        assert_eq!(hits[1].line, 42);

        drop(bridge);
        std::env::remove_var("DAGR_LSP_BIN");
        std::env::remove_var("DAGR_LSP_ARGS");
    }
}
