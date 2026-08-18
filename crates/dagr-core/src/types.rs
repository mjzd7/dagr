use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    TypeScript,
    JavaScript,
    Python,
    Go,
    Rust,
    Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "ts" | "tsx" => Language::TypeScript,
            "js" | "jsx" | "mjs" | "cjs" => Language::JavaScript,
            "py" => Language::Python,
            "go" => Language::Go,
            "rs" => Language::Rust,
            _ => Language::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Struct,
    Interface,
    TypeAlias,
    Enum,
    Variable,
    Module,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SymbolSpan {
    pub file_path: PathBuf,
    pub start_line: usize,
    pub end_line: usize,
    pub start_col: usize,
    pub end_col: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeGraphNode {
    pub id: String,                    // Canonical URI: "repo://src/billing/charge.ts#processPayment"
    pub symbol_name: String,
    pub kind: SymbolKind,
    pub language: Language,
    pub span: SymbolSpan,
    pub docstring: Option<String>,
    pub blake3_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MinimalContextSlice {
    pub target_symbol: String,
    pub file_path: PathBuf,
    pub language: Language,
    pub sparse_code_lines: Vec<(usize, String)>, // (line_number, line_content)
    pub type_contracts: Vec<String>,             // Hoisted interfaces & type aliases
    pub estimated_tokens: usize,                 // Exact tiktoken BPE tokens
    pub original_file_tokens: usize,             // Full unpruned file tokens
    pub compression_ratio: f32,                  // e.g. 0.971 (97.1% reduction)
    pub syntax_degraded: bool,                   // True if error recovery/fallback was triggered
}
