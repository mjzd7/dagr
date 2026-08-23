use dagr_core::{count_tokens, Language, SymbolKind};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HoistedSymbolContract {
    pub symbol_name: String,
    pub kind: SymbolKind,
    pub file_path: String,
    pub signature_slice: String,
    pub token_cost: usize,
}

pub struct ASTPageFaultHandler;

impl Default for ASTPageFaultHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ASTPageFaultHandler {
    pub fn new() -> Self {
        Self
    }

    /// Detects potential unresolved symbol references within a code snippet (identifiers that look like types, traits, or structs)
    pub fn scan_unresolved_symbols(&self, code_slice: &str, _lang: Language) -> Vec<String> {
        let mut candidates = HashSet::new();

        // Fast regex-free token scanner looking for capitalized identifiers (PascalCase/CamelCase)
        for word in code_slice.split(|c: char| !c.is_alphanumeric() && c != '_') {
            if word.len() > 2 {
                let first_char = word.chars().next().unwrap();
                if first_char.is_uppercase() {
                    // Filter common keywords
                    if !matches!(
                        word,
                        "Ok" | "Err"
                            | "Some"
                            | "None"
                            | "String"
                            | "Vec"
                            | "Option"
                            | "Result"
                            | "Self"
                            | "Box"
                            | "Arc"
                            | "Mutex"
                    ) {
                        candidates.insert(word.to_string());
                    }
                }
            }
        }

        candidates.into_iter().collect()
    }

    /// Synthesizes a minimal hoisted public contract for an unresolved symbol (avoiding full file dumps)
    pub fn synthesize_contract(
        &self,
        symbol_name: &str,
        file_path: &Path,
        kind: SymbolKind,
        raw_declaration: &str,
    ) -> HoistedSymbolContract {
        let signature_slice = format!(
            "// [HOISTED AST CONTRACT: {}]\n{}",
            symbol_name,
            raw_declaration.trim()
        );
        let token_cost = count_tokens(&signature_slice);

        HoistedSymbolContract {
            symbol_name: symbol_name.to_string(),
            kind,
            file_path: file_path.to_string_lossy().to_string(),
            signature_slice,
            token_cost,
        }
    }
}
