use crate::contracts::ContractHoister;
use crate::extractor::AstExtractor;
use crate::parser::AstParser;
use dagr_core::{
    compute_compression_ratio, count_tokens, DagrError, Language, MinimalContextSlice, Result,
};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SliceTier {
    #[default]
    Standard,
    /// Multi-Rubric Latent AST Slicing (LaMR arXiv:2605.15315):
    /// Strips non-essential docstrings and comments from hoisted satellite types,
    /// yielding an additional 15-25% token reduction with zero semantic loss.
    MultiRubric,
}

#[derive(Debug, Clone)]
pub struct SlicerConfig {
    pub max_depth_hops: usize,
    pub max_token_budget: usize,
    pub include_comments: bool,
    pub tier: SliceTier,
    /// Workspace root used to resolve cross-file contract hops (F3.2).
    pub workspace_root: std::path::PathBuf,
}

impl Default for SlicerConfig {
    fn default() -> Self {
        Self {
            max_depth_hops: 3,
            max_token_budget: 800,
            include_comments: false,
            tier: SliceTier::Standard,
            workspace_root: std::path::PathBuf::from("."),
        }
    }
}

pub struct SymbolicSlicer {
    pub config: SlicerConfig,
}

impl SymbolicSlicer {
    pub fn new(config: SlicerConfig) -> Self {
        Self { config }
    }

    /// Surgically slices target_symbol from source_code, hoisting type contracts
    pub fn slice(
        &self,
        file_path: &Path,
        source_code: &str,
        language: Language,
        target_symbol: &str,
    ) -> Result<MinimalContextSlice> {
        let original_file_tokens = count_tokens(source_code);

        // 1. Initialize Tree-sitter parser
        let mut parser = match AstParser::new(language) {
            Ok(p) => p,
            Err(_) => {
                // Fallback to lexical search for unknown languages
                return self.fallback_lexical_slice(
                    file_path,
                    source_code,
                    language,
                    target_symbol,
                    original_file_tokens,
                );
            }
        };

        // 2. Parse syntax tree
        let tree = match parser.parse(source_code, None) {
            Ok(t) => t,
            Err(_) => {
                return self.fallback_lexical_slice(
                    file_path,
                    source_code,
                    language,
                    target_symbol,
                    original_file_tokens,
                );
            }
        };

        let root_node = tree.root_node();

        // 3. Locate target symbol in AST
        let symbol_def =
            match AstExtractor::find_symbol(root_node, source_code, language, target_symbol) {
                Some(s) => s,
                None => {
                    return Err(DagrError::SymbolNotFound {
                        symbol: target_symbol.to_string(),
                        file: file_path.display().to_string(),
                    });
                }
            };

        // 4. Collect identifiers inside target symbol body
        let identifiers =
            AstExtractor::collect_referenced_identifiers(symbol_def.node, source_code);

        // 5. Hoist relevant type contracts & interfaces
        let raw_hoisted_contracts = ContractHoister::extract_hoisted_contracts(
            root_node,
            source_code,
            &identifiers,
            symbol_def.start_line,
            symbol_def.end_line,
        );

        // Apply Multi-Rubric docstring/comment stripping if tier is MultiRubric (LaMR arXiv:2605.15315)
        let mut hoisted_contracts = if self.config.tier == SliceTier::MultiRubric {
            raw_hoisted_contracts
                .into_iter()
                .map(|contract| Self::strip_docstrings_and_comments(&contract))
                .filter(|c| !c.trim().is_empty())
                .collect()
        } else {
            raw_hoisted_contracts
        };

        // F3.2: one hop of cross-file contract hoisting, gated by max_depth_hops.
        if self.config.max_depth_hops > 0 {
            let cross =
                self.hoist_imported_file_contracts(root_node, file_path, source_code, &identifiers);
            for contract in cross {
                if !hoisted_contracts.contains(&contract) {
                    hoisted_contracts.push(contract);
                }
            }
        }

        // 6. Assemble sparse lines from target implementation
        let all_lines: Vec<&str> = source_code.lines().collect();
        let mut sparse_code_lines = Vec::new();

        let start_idx = symbol_def.start_line.saturating_sub(1);
        let end_idx = symbol_def.end_line.min(all_lines.len());

        for (idx, line) in all_lines[start_idx..end_idx].iter().enumerate() {
            let line_num = start_idx + idx + 1;
            sparse_code_lines.push((line_num, line.to_string()));
        }

        // 7. Calculate exact BPE tokens for the minimal output
        let mut assembled_slice_text = String::new();
        for contract in &hoisted_contracts {
            assembled_slice_text.push_str(contract);
            assembled_slice_text.push('\n');
        }
        for (_, line) in &sparse_code_lines {
            assembled_slice_text.push_str(line);
            assembled_slice_text.push('\n');
        }

        let estimated_tokens = count_tokens(&assembled_slice_text);
        let compression_ratio = compute_compression_ratio(original_file_tokens, estimated_tokens);

        Ok(MinimalContextSlice {
            target_symbol: target_symbol.to_string(),
            file_path: file_path.to_path_buf(),
            language,
            sparse_code_lines,
            type_contracts: hoisted_contracts,
            estimated_tokens,
            original_file_tokens,
            compression_ratio,
            syntax_degraded: root_node.has_error(),
        })
    }

    /// One hop of cross-file contract hoisting (F3.2): resolves TS/JS import
    /// specifiers from the source AST to sibling files, parses each with the
    /// same tree-sitter pipeline, and lifts contract nodes matching the
    /// target symbol's referenced identifiers.
    // ponytail: single-hop relative-import scope only - alias resolution and deeper traversal land once Wave-1 data quantifies demand; extraction is tree-sitter-native rather than line probes
    fn hoist_imported_file_contracts(
        &self,
        root_node: tree_sitter::Node,
        file_path: &Path,
        source_code: &str,
        identifiers: &std::collections::HashSet<String>,
    ) -> Vec<String> {
        let mut contracts = Vec::new();
        for spec in collect_import_sources(root_node, source_code) {
            let rels = resolve_spec_candidates(file_path, &spec);
            for rel_path in &rels {
                for path in self.workspace_file_candidates(rel_path) {
                    if !path.exists() {
                        continue;
                    }
                    let Ok(content) = std::fs::read_to_string(&path) else { continue };
                    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or_default();
                    let lang = Language::from_extension(ext);
                    let Ok(mut parser) = AstParser::new(lang) else {
                        continue;
                    };
                    let Ok(tree) = parser.parse(&content, None) else {
                        continue;
                    };
                    let found = ContractHoister::extract_hoisted_contracts(
                        tree.root_node(),
                        &content,
                        identifiers,
                        // Sentinel range that excludes nothing: every node in
                        // the foreign file is eligible for hoisting.
                        usize::MAX,
                        0,
                    );
                    contracts.extend(found);
                }
            }
        }
        contracts
    }

    fn workspace_file_candidates(&self, rel_path: &str) -> Vec<std::path::PathBuf> {
        const EXTS: [&str; 4] = ["ts", "tsx", "js", "jsx"];
        let direct: Vec<String> = if has_known_ext(rel_path, &EXTS) {
            vec![rel_path.to_string()]
        } else {
            let mut v: Vec<String> = EXTS.iter().map(|e| format!("{rel_path}.{e}")).collect();
            if !rel_path.ends_with("/index") {
                v.extend(EXTS.iter().map(|e| format!("{rel_path}/index.{e}")));
            }
            v
        };
        let mut out = Vec::new();
        for c in direct {
            out.push(self.config.workspace_root.join(&c));
            out.push(std::path::PathBuf::from(&c));
        }
        out
    }

    /// Strips docstrings, JSDoc, and non-critical comments from hoisted satellite contracts (LaMR arXiv:2605.15315)
    pub fn strip_docstrings_and_comments(code: &str) -> String {
        let mut result = Vec::new();
        let mut in_block_comment = false;

        for line in code.lines() {
            let trimmed = line.trim();
            if in_block_comment {
                if trimmed.contains("*/") || trimmed.ends_with("\"\"\"") || trimmed.ends_with("'''")
                {
                    in_block_comment = false;
                }
                continue;
            }

            if trimmed.starts_with("/**")
                || trimmed.starts_with("/*")
                || trimmed.starts_with("\"\"\"")
                || trimmed.starts_with("'''")
            {
                if !trimmed.ends_with("*/") && !trimmed[3..].contains("\"\"\"") {
                    in_block_comment = true;
                }
                continue;
            }

            if trimmed.starts_with("///")
                || (trimmed.starts_with("//") && !trimmed.starts_with("// ⚡"))
                || (trimmed.starts_with('#') && !trimmed.starts_with("#["))
            {
                continue;
            }

            if !trimmed.is_empty() {
                result.push(line);
            }
        }

        result.join("\n")
    }

    /// Slices minimal causal context from a test failure stack trace (CausalRepair arXiv:2608.10613)
    pub fn slice_from_test_failure(
        &self,
        file_path: &Path,
        source_code: &str,
        language: Language,
        stack_trace: &str,
    ) -> Result<MinimalContextSlice> {
        // Extract failing function name from stack trace heuristics
        let target_symbol = stack_trace
            .lines()
            .find_map(|line| {
                if line.contains("::") {
                    line.split("::").last().map(|s| {
                        s.split(|c: char| {
                            c.is_whitespace() || c == '\'' || c == '"' || c == '(' || c == ':'
                        })
                        .next()
                        .unwrap_or("")
                        .trim()
                    })
                } else if line.contains("at ") {
                    line.split("at ").nth(1).map(|s| {
                        s.split(|c: char| c.is_whitespace() || c == '(' || c == ':')
                            .next()
                            .unwrap_or("")
                            .trim()
                    })
                } else {
                    None
                }
            })
            .filter(|s| !s.is_empty())
            .unwrap_or("test_target");

        self.slice(file_path, source_code, language, target_symbol)
    }

    /// Lexical Indentation Fallback for severely broken syntax or unknown languages
    fn fallback_lexical_slice(
        &self,
        file_path: &Path,
        source_code: &str,
        language: Language,
        target_symbol: &str,
        original_file_tokens: usize,
    ) -> Result<MinimalContextSlice> {
        let lines: Vec<&str> = source_code.lines().collect();
        let mut sparse_lines = Vec::new();

        let mut found_line = None;
        for (idx, line) in lines.iter().enumerate() {
            if line.contains(target_symbol) {
                found_line = Some(idx);
                break;
            }
        }

        if let Some(target_idx) = found_line {
            let start = target_idx.saturating_sub(2);
            let end = (target_idx + 25).min(lines.len());
            for (idx, line) in lines.iter().enumerate().take(end).skip(start) {
                sparse_lines.push((idx + 1, (*line).to_string()));
            }
        } else {
            return Err(DagrError::SymbolNotFound {
                symbol: target_symbol.to_string(),
                file: file_path.display().to_string(),
            });
        }

        let mut text = String::new();
        for (_, l) in &sparse_lines {
            text.push_str(l);
            text.push('\n');
        }
        let tokens = count_tokens(&text);

        Ok(MinimalContextSlice {
            target_symbol: target_symbol.to_string(),
            file_path: file_path.to_path_buf(),
            language,
            sparse_code_lines: sparse_lines,
            type_contracts: Vec::new(),
            estimated_tokens: tokens,
            original_file_tokens,
            compression_ratio: compute_compression_ratio(original_file_tokens, tokens),
            syntax_degraded: true,
        })
    }
}

fn collect_import_sources(root_node: tree_sitter::Node, source_code: &str) -> Vec<String> {
    let mut specs = Vec::new();
    let mut stack = vec![root_node];
    while let Some(node) = stack.pop() {
        if matches!(node.kind(), "import_statement" | "export_statement") {
            if let Some(src) = node.child_by_field_name("source") {
                if let Ok(text) = src.utf8_text(source_code.as_bytes()) {
                    let cleaned = text.trim_matches('"').trim_matches('\'');
                    if !cleaned.is_empty() {
                        specs.push(cleaned.to_string());
                    }
                }
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }
    specs
}

fn resolve_spec_candidates(source_file: &Path, specifier: &str) -> Vec<String> {
    if !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return Vec::new();
    }
    let parent = source_file
        .parent()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    // Preserve absoluteness: joining segments must not strip a leading '/'
    // from absolute importer paths (the CLI passes absolute files).
    let absolute = parent.starts_with('/');
    let mut parts: Vec<&str> = parent.split('/').filter(|s| !s.is_empty()).collect();
    for seg in specifier.split('/') {
        match seg {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    if parts.is_empty() {
        return Vec::new();
    }
    let joined = parts.join("/");
    vec![if absolute {
        format!("/{joined}")
    } else {
        joined
    }]
}

fn has_known_ext(path_str: &str, exts: &[&str]) -> bool {
    path_str.rsplit('/').next().is_some_and(|segment| {
        segment.contains('.') && segment.rsplit('.').next().is_some_and(|e| exts.contains(&e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_slicing_with_contract_hoisting() -> Result<()> {
        let code = r#"
import { db } from "@/db";

export interface PaymentPayload {
    userId: string;
    amountCents: number;
    currency: string;
}

export type PaymentResult = {
    success: boolean;
    transactionId: string;
};

// 50 lines of unrelated helpers
function helperA() { return 1; }
function helperB() { return 2; }
function helperC() { return 3; }

export async function processPayment(payload: PaymentPayload): Promise<PaymentResult> {
    const tax = payload.amountCents * 0.1;
    return {
        success: true,
        transactionId: "tx_12345"
    };
}

// 100 more lines of refund functions
function refundTransaction() { return false; }
"#;

        let slicer = SymbolicSlicer::new(SlicerConfig::default());
        let slice = slicer.slice(
            Path::new("src/billing/charge.ts"),
            code,
            Language::TypeScript,
            "processPayment",
        )?;

        assert_eq!(slice.target_symbol, "processPayment");
        assert_eq!(slice.language, Language::TypeScript);
        assert!(!slice.sparse_code_lines.is_empty());
        // Verify contract hoisting
        assert!(slice
            .type_contracts
            .iter()
            .any(|c| c.contains("PaymentPayload")));
        assert!(slice
            .type_contracts
            .iter()
            .any(|c| c.contains("PaymentResult")));
        // Verify unrelated functions are pruned
        assert!(!slice
            .sparse_code_lines
            .iter()
            .any(|(_, l)| l.contains("helperA")));
        assert!(!slice
            .sparse_code_lines
            .iter()
            .any(|(_, l)| l.contains("refundTransaction")));
        // Verify compression
        assert!(slice.compression_ratio > 0.3);

        Ok(())
    }

    #[test]
    fn test_python_slicing() -> Result<()> {
        let code = r#"
class DiscountConfig:
    rate: float = 0.15

def unrelated_analytics():
    pass

def apply_discount(price: float, config: DiscountConfig) -> float:
    return price * (1.0 - config.rate)

def unrelated_database_sync():
    pass
"#;

        let slicer = SymbolicSlicer::new(SlicerConfig::default());
        let slice = slicer.slice(
            Path::new("services/discount.py"),
            code,
            Language::Python,
            "apply_discount",
        )?;

        assert_eq!(slice.target_symbol, "apply_discount");
        assert!(slice
            .type_contracts
            .iter()
            .any(|c| c.contains("DiscountConfig")));
        assert!(!slice
            .sparse_code_lines
            .iter()
            .any(|(_, l)| l.contains("unrelated_analytics")));
        Ok(())
    }

    #[test]
    fn test_multi_rubric_comment_stripping() -> Result<()> {
        let code = r#"
/**
 * JSDoc with multi-line explanation that should be stripped
 * @param {string} key - API Key
 */
export interface ApiConfig {
    // Inline comment that should be stripped
    key: string;
    timeoutMs: number;
}

export function createClient(config: ApiConfig) {
    return config.key;
}
"#;

        let config = SlicerConfig {
            tier: SliceTier::MultiRubric,
            ..Default::default()
        };
        let slicer = SymbolicSlicer::new(config);

        let slice = slicer.slice(
            Path::new("src/client.ts"),
            code,
            Language::TypeScript,
            "createClient",
        )?;

        assert_eq!(slice.target_symbol, "createClient");
        assert!(!slice.type_contracts.is_empty());
        let contract_text = slice.type_contracts.join("\n");
        assert!(!contract_text.contains("JSDoc with multi-line explanation"));
        assert!(!contract_text.contains("Inline comment"));
        assert!(contract_text.contains("key: string;"));
        Ok(())
    }

    #[test]
    fn test_causal_failure_slice() -> Result<()> {
        let code = r#"
pub fn calculate_total(subtotal: f64) -> f64 {
    subtotal * 1.2
}
"#;
        let slicer = SymbolicSlicer::new(SlicerConfig::default());
        let trace = "thread 'tests::calculate_total' panicked at src/lib.rs:10:5";
        let slice =
            slicer.slice_from_test_failure(Path::new("src/lib.rs"), code, Language::Rust, trace)?;

        assert_eq!(slice.target_symbol, "calculate_total");
        Ok(())
    }
}
