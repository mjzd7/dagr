//! Incremental Background File Watcher for Real-Time AST Re-Indexing

use colored::Colorize;
use dagr_core::{
    CodeGraphNode, DagrError, Language, LocalIndexStore, Result, SymbolKind, SymbolSpan,
};
use dagr_slicer::{AstExtractor, AstParser};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Instant;

pub struct WorkspaceWatcher {
    pub workspace_root: PathBuf,
}

impl WorkspaceWatcher {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    /// Starts watching workspace files and incrementally updates SQLite AST index on save
    pub fn watch(&self) -> Result<()> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(tx, Config::default())
            .map_err(|e| DagrError::Io(std::io::Error::other(e)))?;

        watcher
            .watch(&self.workspace_root, RecursiveMode::Recursive)
            .map_err(|e| DagrError::Io(std::io::Error::other(e)))?;

        eprintln!(
            "\n{}",
            "⚡ DAGR Incremental AST File Watcher active".bold().cyan()
        );
        eprintln!("   Watching: {:?}", self.workspace_root);
        eprintln!("   Press Ctrl+C to stop.\n");

        let supported_exts = ["ts", "tsx", "js", "jsx", "py", "rs", "go"];

        for res in rx {
            match res {
                Ok(event) => {
                    if let EventKind::Modify(_) | EventKind::Create(_) = event.kind {
                        for path in event.paths {
                            let path_str = path.to_string_lossy();
                            if path_str.contains("/.git/")
                                || path_str.contains("/target/")
                                || path_str.contains("/node_modules/")
                                || path_str.contains("/.dagr/")
                            {
                                continue;
                            }

                            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                                if supported_exts.contains(&ext) && path.is_file() {
                                    self.reindex_file(&path);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Watcher error: {:?}", e);
                }
            }
        }

        Ok(())
    }

    fn reindex_file(&self, path: &Path) {
        let start = Instant::now();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let language = Language::from_extension(ext);
        if language == Language::Unknown {
            return;
        }

        if let Ok(content) = std::fs::read_to_string(path) {
            let hash = blake3::hash(content.as_bytes()).to_hex().to_string();

            if let Ok(mut store) = LocalIndexStore::open(&self.workspace_root) {
                if let Ok(cached) = store.is_file_cached(&path.to_string_lossy(), &hash) {
                    if cached {
                        return;
                    }
                }

                if let Ok(mut parser) = AstParser::new(language) {
                    if let Ok(tree) = parser.parse(&content, None) {
                        let symbols =
                            AstExtractor::extract_all_symbols(tree.root_node(), &content, language);
                        let count = symbols.len();

                        let mut nodes = Vec::new();
                        for sym in symbols {
                            let node = CodeGraphNode {
                                id: format!("{}#{}", path.to_string_lossy(), sym.name),
                                symbol_name: sym.name,
                                kind: SymbolKind::Function,
                                language,
                                span: SymbolSpan {
                                    file_path: path.to_path_buf(),
                                    start_line: sym.start_line,
                                    end_line: sym.end_line,
                                    start_col: 0,
                                    end_col: 0,
                                },
                                docstring: None,
                                blake3_hash: hash.clone(),
                            };
                            nodes.push(node);
                        }

                        let _ = store.store_symbols(&path.to_string_lossy(), &nodes);
                        let _ = store.update_file_cache(&path.to_string_lossy(), &hash);
                        let elapsed_ms = start.elapsed().as_micros() as f64 / 1000.0;
                        eprintln!(
                            "   {} Re-indexed {}: {} symbols ({:.2}ms)",
                            "⚡ [WATCH]".green().bold(),
                            path.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .cyan(),
                            count,
                            elapsed_ms
                        );
                    }
                }
            }
        }
    }
}
