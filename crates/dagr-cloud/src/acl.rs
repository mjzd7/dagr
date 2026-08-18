use dagr_core::{CodeGraphNode, Language, Result, SymbolKind, SymbolSpan};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EdgeType {
    Calls,
    Imports,
    Inherits,
    MutatesSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeGraphEdge {
    pub source_id: String,
    pub target_id: String,
    pub edge_type: EdgeType,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodeGraphDelta {
    pub repository: String,
    pub commit_sha: String,
    pub upserted_nodes: Vec<CodeGraphNode>,
    pub deleted_node_ids: Vec<String>,
    pub upserted_edges: Vec<CodeGraphEdge>,
}

pub struct AntiCorruptionLayer;

impl AntiCorruptionLayer {
    /// Normalizes raw file contents into unified AST nodes & edges
    pub fn parse_file_delta(
        repository: &str,
        file_path: &str,
        content: &str,
    ) -> Result<Vec<CodeGraphNode>> {
        let ext = std::path::Path::new(file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let language = Language::from_extension(ext);

        let mut nodes = Vec::new();
        let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();

        // Scan lines for top-level definitions
        for (idx, line) in content.lines().enumerate() {
            if let Some(name) = Self::extract_function_name(line) {
                if name.is_empty() {
                    continue;
                }
                let canonical_id = format!("repo://{}/{}#{}", repository, file_path, name);

                nodes.push(CodeGraphNode {
                    id: canonical_id,
                    symbol_name: name.to_string(),
                    kind: SymbolKind::Function,
                    language,
                    span: SymbolSpan {
                        file_path: PathBuf::from(file_path),
                        start_line: idx + 1,
                        end_line: idx + 1,
                        start_col: 0,
                        end_col: line.len(),
                    },
                    docstring: None,
                    blake3_hash: content_hash.clone(),
                });
            }
        }

        Ok(nodes)
    }

    fn extract_function_name(line: &str) -> Option<&str> {
        let trimmed = line.trim();
        let prefixes = [
            "export function ",
            "export async function ",
            "function ",
            "def ",
            "pub fn ",
            "fn ",
        ];
        for prefix in prefixes {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                return rest.split('(').next().map(|s| s.trim());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acl_file_delta_normalization() -> Result<()> {
        let code = "export function processPayment(id: string) {\n  return true;\n}\n\nexport function refundPayment() {}\n";
        let nodes = AntiCorruptionLayer::parse_file_delta("mjzd7/dagr", "src/billing.ts", code)?;

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].symbol_name, "processPayment");
        assert_eq!(
            nodes[0].id,
            "repo://mjzd7/dagr/src/billing.ts#processPayment"
        );
        assert_eq!(nodes[1].symbol_name, "refundPayment");
        Ok(())
    }
}
