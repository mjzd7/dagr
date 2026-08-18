use crate::contracts::ContractHoister;
use crate::extractor::AstExtractor;
use crate::parser::AstParser;
use dagr_core::{
    compute_compression_ratio, count_tokens, DagrError, Language, MinimalContextSlice, Result,
};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SlicerConfig {
    pub max_depth_hops: usize,
    pub max_token_budget: usize,
    pub include_comments: bool,
}

impl Default for SlicerConfig {
    fn default() -> Self {
        Self {
            max_depth_hops: 3,
            max_token_budget: 800,
            include_comments: false,
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
                return self.fallback_lexical_slice(file_path, source_code, language, target_symbol, original_file_tokens);
            }
        };

        // 2. Parse syntax tree
        let tree = match parser.parse(source_code, None) {
            Ok(t) => t,
            Err(_) => {
                return self.fallback_lexical_slice(file_path, source_code, language, target_symbol, original_file_tokens);
            }
        };

        let root_node = tree.root_node();

        // 3. Locate target symbol in AST
        let symbol_def = match AstExtractor::find_symbol(root_node, source_code, language, target_symbol) {
            Some(s) => s,
            None => {
                return Err(DagrError::SymbolNotFound {
                    symbol: target_symbol.to_string(),
                    file: file_path.display().to_string(),
                });
            }
        };

        // 4. Collect identifiers inside target symbol body
        let identifiers = AstExtractor::collect_referenced_identifiers(symbol_def.node, source_code);

        // 5. Hoist relevant type contracts & interfaces
        let hoisted_contracts = ContractHoister::extract_hoisted_contracts(
            root_node,
            source_code,
            &identifiers,
            symbol_def.start_line,
            symbol_def.end_line,
        );

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
            for idx in start..end {
                sparse_lines.push((idx + 1, lines[idx].to_string()));
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
        assert!(slice.type_contracts.iter().any(|c| c.contains("PaymentPayload")));
        assert!(slice.type_contracts.iter().any(|c| c.contains("PaymentResult")));
        // Verify unrelated functions are pruned
        assert!(!slice.sparse_code_lines.iter().any(|(_, l)| l.contains("helperA")));
        assert!(!slice.sparse_code_lines.iter().any(|(_, l)| l.contains("refundTransaction")));
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
        assert!(slice.type_contracts.iter().any(|c| c.contains("DiscountConfig")));
        assert!(!slice.sparse_code_lines.iter().any(|(_, l)| l.contains("unrelated_analytics")));
        Ok(())
    }
}
