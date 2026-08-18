use crate::acl::{CodeGraphEdge, EdgeType};
use dagr_core::CodeGraphNode;

pub struct CypherQueryBuilder;

impl CypherQueryBuilder {
    /// Generates Cypher queries to upsert a CodeGraphNode into Memgraph
    pub fn build_node_upsert_query(node: &CodeGraphNode) -> (String, serde_json::Value) {
        let cypher = "MERGE (s:Symbol {id: $id}) \
                      SET s.name = $name, \
                          s.file = $file, \
                          s.language = $language, \
                          s.hash = $hash \
                      RETURN s.id;"
            .to_string();

        let params = serde_json::json!({
            "id": node.id,
            "name": node.symbol_name,
            "file": node.span.file_path.display().to_string(),
            "language": format!("{:?}", node.language),
            "hash": node.blake3_hash
        });

        (cypher, params)
    }

    /// Generates Cypher queries to upsert an edge between two symbol nodes
    pub fn build_edge_upsert_query(edge: &CodeGraphEdge) -> (String, serde_json::Value) {
        let rel_name = match edge.edge_type {
            EdgeType::Calls => "CALLS",
            EdgeType::Imports => "IMPORTS",
            EdgeType::Inherits => "INHERITS",
            EdgeType::MutatesSchema => "MUTATES_SCHEMA",
        };

        let cypher = format!(
            "MATCH (src:Symbol {{id: $source_id}}) \
             MATCH (dst:Symbol {{id: $target_id}}) \
             MERGE (src)-[r:{}]->(dst) \
             SET r.weight = $weight \
             RETURN type(r);",
            rel_name
        );

        let params = serde_json::json!({
            "source_id": edge.source_id,
            "target_id": edge.target_id,
            "weight": edge.weight
        });

        (cypher, params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dagr_core::{Language, SymbolKind, SymbolSpan};
    use std::path::PathBuf;

    #[test]
    fn test_cypher_node_query_generation() {
        let node = CodeGraphNode {
            id: "repo://src/auth.ts#login".into(),
            symbol_name: "login".into(),
            kind: SymbolKind::Function,
            language: Language::TypeScript,
            span: SymbolSpan {
                file_path: PathBuf::from("src/auth.ts"),
                start_line: 1,
                end_line: 10,
                start_col: 0,
                end_col: 1,
            },
            docstring: None,
            blake3_hash: "blake_hash_123".into(),
        };

        let (cypher, params) = CypherQueryBuilder::build_node_upsert_query(&node);
        assert!(cypher.contains("MERGE (s:Symbol {id: $id})"));
        assert_eq!(params["name"], "login");
    }
}
