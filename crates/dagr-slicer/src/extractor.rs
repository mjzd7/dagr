use dagr_core::Language;
use std::collections::HashSet;
use tree_sitter::Node;

#[derive(Debug, Clone)]
pub struct SymbolDef<'a> {
    pub name: String,
    pub node: Node<'a>,
    pub start_line: usize,
    pub end_line: usize,
}

pub struct AstExtractor;

impl AstExtractor {
    /// Traverses the AST root to find the node definition matching `target_symbol`
    pub fn find_symbol<'a>(
        root_node: Node<'a>,
        source: &'a str,
        _language: Language,
        target_symbol: &str,
    ) -> Option<SymbolDef<'a>> {
        let mut stack = vec![root_node];

        while let Some(node) = stack.pop() {
            if Self::is_definition_node(node) {
                if let Some(name) = Self::get_node_name(node, source) {
                    if name == target_symbol {
                        return Some(SymbolDef {
                            name: name.to_string(),
                            node,
                            start_line: node.start_position().row + 1,
                            end_line: node.end_position().row + 1,
                        });
                    }
                }
            }

            // Traverse child nodes
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }

        None
    }

    /// Recursively collects all identifier names referenced inside a target AST node
    pub fn collect_referenced_identifiers<'a>(node: Node<'a>, source: &'a str) -> HashSet<String> {
        let mut identifiers = HashSet::new();
        let mut stack = vec![node];

        while let Some(current) = stack.pop() {
            let kind = current.kind();
            if kind == "identifier" || kind == "type_identifier" || kind == "property_identifier" {
                if let Ok(text) = current.utf8_text(source.as_bytes()) {
                    identifiers.insert(text.to_string());
                }
            }

            let mut cursor = current.walk();
            for child in current.children(&mut cursor) {
                stack.push(child);
            }
        }

        identifiers
    }

    fn is_definition_node(node: Node) -> bool {
        match node.kind() {
            // TypeScript / JavaScript / Go / Rust / Python definition node kinds
            "function_declaration"
            | "method_definition"
            | "class_declaration"
            | "interface_declaration"
            | "type_alias_declaration"
            | "export_statement"
            | "function_definition"
            | "class_definition"
            | "method_declaration"
            | "type_declaration"
            | "function_item"
            | "struct_item"
            | "enum_item"
            | "trait_item"
            | "impl_item" => true,
            _ => false,
        }
    }

    fn get_node_name<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
        // Direct name child node
        if let Some(name_node) = node.child_by_field_name("name") {
            return name_node.utf8_text(source.as_bytes()).ok();
        }

        // Special handling for export statements in TS/JS
        if node.kind() == "export_statement" {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if let Some(name_node) = child.child_by_field_name("name") {
                    return name_node.utf8_text(source.as_bytes()).ok();
                }
            }
        }

        None
    }
}
