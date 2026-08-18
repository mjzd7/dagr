use std::collections::HashSet;
use tree_sitter::Node;

pub struct ContractHoister;

impl ContractHoister {
    /// Extracts type definitions and interface contracts for identifiers referenced in the target slice
    pub fn extract_hoisted_contracts<'a>(
        root_node: Node<'a>,
        source: &'a str,
        referenced_identifiers: &HashSet<String>,
        target_start_line: usize,
        target_end_line: usize,
    ) -> Vec<String> {
        let mut contracts = Vec::new();
        let mut stack = vec![root_node];

        while let Some(node) = stack.pop() {
            let start_line = node.start_position().row + 1;
            let end_line = node.end_position().row + 1;

            // Skip nodes inside the target symbol itself
            if start_line >= target_start_line && end_line <= target_end_line {
                continue;
            }

            if Self::is_contract_node(node) {
                if let Some(name) = Self::get_contract_name(node, source) {
                    if referenced_identifiers.contains(name) {
                        if let Ok(contract_text) = node.utf8_text(source.as_bytes()) {
                            contracts.push(contract_text.trim().to_string());
                        }
                    }
                }
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }

        contracts
    }

    fn is_contract_node(node: Node) -> bool {
        match node.kind() {
            // TypeScript
            "interface_declaration" | "type_alias_declaration" | "enum_declaration" => true,
            // Rust
            "struct_item" | "enum_item" | "type_item" | "trait_item" => true,
            // Go
            "type_declaration" | "type_spec" => true,
            // Python / Classes used as types
            "class_definition" => true,
            _ => false,
        }
    }

    fn get_contract_name<'a>(node: Node<'a>, source: &'a str) -> Option<&'a str> {
        if let Some(name_node) = node.child_by_field_name("name") {
            return name_node.utf8_text(source.as_bytes()).ok();
        }
        None
    }
}
