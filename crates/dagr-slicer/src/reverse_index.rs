//! Reverse symbol index: maps symbol names and module paths back to the
//! files/lines that reference them. Powers `dagr review-diff` dangling-symbol
//! detection ("2 removed symbols still referenced by 8 importers").
//!
//! ponytail: full-workspace rescan per invocation, no incremental cache;
//! upgrade when review-diff latency exceeds ~1s on real repos.

use crate::parser::AstParser;
use dagr_core::{DagrError, Language, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolRef {
    pub file: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportRef {
    pub file: String,
    pub line: usize,
    pub module: String,
}

#[derive(Default)]
struct FileFacts {
    definitions: Vec<(String, usize)>,
    identifiers: Vec<(String, usize)>,
    imports: Vec<ImportRef>,
    bindings: HashMap<(String, usize), Vec<String>>,
}

#[derive(Debug, Default)]
pub struct ReverseIndex {
    files: Vec<String>,
    definitions: HashMap<String, Vec<SymbolRef>>,
    usages: HashMap<String, Vec<SymbolRef>>,
    imports: Vec<ImportRef>,
    bindings: HashMap<(String, usize), Vec<String>>,
}

const SCANNED_EXTS: [&str; 3] = ["ts", "tsx", "rs"];

impl ReverseIndex {
    pub fn build(workspace_root: &Path) -> Result<Self> {
        let mut sources: Vec<(String, String)> = Vec::new();
        collect_sources(workspace_root, workspace_root, &mut sources);

        let mut index = Self::default();
        let mut facts_per_file: Vec<(String, FileFacts)> = Vec::with_capacity(sources.len());

        for (rel, src) in &sources {
            let lang = language_for(rel)?;
            let facts = extract_facts(rel, src, lang)?;
            index.files.push(rel.clone());
            facts_per_file.push((rel.clone(), facts));
        }

        // Pass 1: definitions
        for (rel, facts) in &facts_per_file {
            for (name, line) in &facts.definitions {
                index
                    .definitions
                    .entry(name.clone())
                    .or_default()
                    .push(SymbolRef {
                        file: rel.clone(),
                        line: *line,
                    });
            }
        }

        // Pass 2: identifier usages excluding the definition site itself
        for (rel, facts) in &facts_per_file {
            for (name, line) in &facts.identifiers {
                let is_own_def = facts
                    .definitions
                    .iter()
                    .any(|(n, dl)| n == name && dl == line);
                if is_own_def {
                    continue;
                }
                index
                    .usages
                    .entry(name.clone())
                    .or_default()
                    .push(SymbolRef {
                        file: rel.clone(),
                        line: *line,
                    });
            }
            index.imports.extend(facts.imports.iter().cloned());
            for (k, v) in &facts.bindings {
                index.bindings.insert(k.clone(), v.clone());
            }
        }

        Ok(index)
    }

    /// Every indexed import site across the workspace.
    pub fn all_imports(&self) -> Vec<ImportRef> {
        self.imports.clone()
    }

    /// Named bindings imported at one specific import statement.
    pub fn bindings_imported_from(&self, file: &str, line: usize) -> &[String] {
        self.bindings
            .get(&(file.to_string(), line))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn files(&self) -> &[String] {
        &self.files
    }

    /// Files/lines where `symbol` is defined.
    pub fn definitions_of(&self, symbol: &str) -> &[SymbolRef] {
        self.definitions
            .get(symbol)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Files/lines where `symbol` is referenced but not defined.
    pub fn callers_of(&self, symbol: &str) -> &[SymbolRef] {
        self.usages.get(symbol).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// A removed definition leaves references behind iff callers exist while
    /// definitions do not.
    pub fn is_dangling(&self, symbol: &str) -> bool {
        !self.callers_of(symbol).is_empty() && self.definitions_of(symbol).is_empty()
    }

    /// Imports whose module path ends with `module_suffix` (e.g. "db/client").
    pub fn importers_of_module(&self, module_suffix: &str) -> Vec<&ImportRef> {
        self.imports
            .iter()
            .filter(|imp| {
                imp.module == module_suffix || imp.module.ends_with(&format!("/{module_suffix}"))
            })
            .collect()
    }
}

fn language_for(rel: &str) -> Result<Language> {
    let ext = Path::new(rel)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    match ext {
        "ts" | "tsx" => Ok(Language::TypeScript),
        "rs" => Ok(Language::Rust),
        _ => Err(DagrError::UnsupportedLanguage(format!(
            "{rel}: only {SCANNED_EXTS:?} are indexed"
        ))),
    }
}

/// ponytail: mirrors guard's walk_and_check skip-list by value (guard cannot
/// be a dependency of slicer — CLI composes both); upgrade to shared crate
/// constant if the lists ever diverge.
fn collect_sources(root: &Path, current: &Path, out: &mut Vec<(String, String)>) {
    if current.is_dir() {
        let skip = [
            ".git",
            "node_modules",
            "target",
            ".dagr",
            ".next",
            "dist",
            "build",
            "out",
            ".output",
            ".turbo",
            ".venv",
            "venv",
            "__pycache__",
            "vendor",
            "coverage",
        ];
        let name = current.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if skip.contains(&name) {
            return;
        }
        if let Ok(entries) = std::fs::read_dir(current) {
            for entry in entries.flatten() {
                collect_sources(root, &entry.path(), out);
            }
        }
        return;
    }
    let ext = current.extension().and_then(|s| s.to_str()).unwrap_or("");
    if !SCANNED_EXTS.contains(&ext) {
        return;
    }
    let rel = current
        .strip_prefix(root)
        .unwrap_or(current)
        .display()
        .to_string()
        .replace('\\', "/");
    if let Ok(src) = std::fs::read_to_string(current) {
        out.push((rel, src));
    }
}

fn extract_facts(rel: &str, source: &str, lang: Language) -> Result<FileFacts> {
    let mut parser = AstParser::new(lang)?;
    let tree = parser.parse(source, None)?;
    let root = tree.root_node();

    let mut facts = FileFacts::default();
    let mut stack = vec![root];

    while let Some(node) = stack.pop() {
        let kind = node.kind();
        let line = node.start_position().row + 1;

        let parent_is_export = node
            .parent()
            .is_some_and(|par| par.kind() == "export_statement");
        if !parent_is_export {
            if let Some(name) = definition_name(node, source) {
                facts.definitions.push((name.to_string(), line));
            }
        }

        match kind {
            "identifier" | "type_identifier" | "property_identifier" => {
                if let Ok(text) = node.utf8_text(source.as_bytes()) {
                    facts.identifiers.push((text.to_string(), line));
                }
            }
            "import_statement" | "export_statement" => {
                if let Some(src_node) = node.child_by_field_name("source") {
                    if let Ok(raw) = src_node.utf8_text(source.as_bytes()) {
                        facts.imports.push(ImportRef {
                            file: rel.to_string(),
                            line,
                            module: raw.trim_matches(|c| c == '"' || c == '\'').to_string(),
                        });
                        let mut names = Vec::new();
                        let mut cursor = node.walk();
                        for desc in node.children(&mut cursor) {
                            collect_named_specifiers(desc, source, &mut names);
                        }
                        if !names.is_empty() {
                            facts.bindings.insert((rel.to_string(), line), names);
                        }
                    }
                }
            }
            "use_declaration" => {
                if let Ok(raw) = node.utf8_text(source.as_bytes()) {
                    let module = raw
                        .trim_start_matches("use ")
                        .trim_end_matches(';')
                        .split("::")
                        .collect::<Vec<_>>()
                        .join("::");
                    if !module.is_empty() {
                        facts.imports.push(ImportRef {
                            file: rel.to_string(),
                            line,
                            module,
                        });
                    }
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            stack.push(child);
        }
    }

    Ok(facts)
}

fn collect_named_specifiers(node: tree_sitter::Node, source: &str, out: &mut Vec<String>) {
    if node.kind() == "import_specifier" {
        if let Some(name_node) = node.child_by_field_name("name") {
            if let Ok(text) = name_node.utf8_text(source.as_bytes()) {
                out.push(text.to_string());
            }
        }
        return;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_named_specifiers(child, source, out);
    }
}

/// Definition names hide one level deep inside TS `export` wrappers
/// (`export const x`, `export function f`) — unwrap before naming.
fn definition_name<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> Option<&'a str> {
    let kind = node.kind();
    if is_definition(kind) {
        return node_name(node, source);
    }
    if kind == "export_statement" {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if is_definition(child.kind()) {
                return node_name_deep(child, source);
            }
        }
    }
    None
}

fn is_definition(kind: &str) -> bool {
    matches!(
        kind,
        "function_declaration"
            | "method_definition"
            | "class_declaration"
            | "interface_declaration"
            | "type_alias_declaration"
            | "lexical_declaration"
            | "variable_declaration"
            | "function_definition"
            | "type_declaration"
            | "function_item"
            | "struct_item"
            | "enum_item"
            | "trait_item"
    )
}

/// `name` field lookup that descends one level (lexical_declaration keeps
/// its identifier on the variable_declarator child).
fn node_name_deep<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> Option<&'a str> {
    let direct = node_name(node, source);
    if direct.is_some() {
        return direct;
    }
    let mut cursor = node.walk();
    let found = node
        .children(&mut cursor)
        .find_map(|child| node_name(child, source));
    found
}

fn node_name<'a>(node: tree_sitter::Node<'a>, source: &'a str) -> Option<&'a str> {
    if let Some(name_node) = node.child_by_field_name("name") {
        return name_node.utf8_text(source.as_bytes()).ok();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_ws(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("dagr-ridx-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn write_ws(dir: &Path, files: &[(&str, &str)]) {
        for (rel, src) in files {
            let p = dir.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, src).unwrap();
        }
    }

    const A_TS: &str =
        "export function charge(amountCents: number): number {\n  return amountCents;\n}\n";
    const B_TS: &str = "import { charge } from \"./a\";\n\nexport function checkout(): number {\n  return charge(100);\n}\n";

    #[test]
    fn finds_cross_file_callers_and_definitions() {
        let dir = temp_ws("cross");
        write_ws(&dir, &[("src/a.ts", A_TS), ("src/b.ts", B_TS)]);
        let idx = ReverseIndex::build(&dir).unwrap();

        assert_eq!(idx.definitions_of("charge").len(), 1);
        assert_eq!(idx.definitions_of("charge")[0].file, "src/a.ts");

        let callers = idx.callers_of("charge");
        assert_eq!(
            callers.len(),
            2,
            "import site + call site both count: a broken import of a \
             removed symbol must surface as a dangling reference"
        );
        assert!(callers.iter().all(|r| r.file == "src/b.ts"));

        assert!(!idx.is_dangling("charge"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn removed_definition_leaves_dangling_references() {
        let dir = temp_ws("dangle");
        write_ws(&dir, &[("src/b_only.ts", B_TS)]);
        let idx = ReverseIndex::build(&dir).unwrap();

        assert!(idx.definitions_of("charge").is_empty());
        assert!(!idx.callers_of("charge").is_empty());
        assert!(idx.is_dangling("charge"), "a.ts gone => charge dangles");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn importers_of_module_matches_suffix() {
        let dir = temp_ws("imp");
        write_ws(
            &dir,
            &[(
                "src/b2.ts",
                "import { pool } from \"../db/client\";\nexport const q = pool();\n",
            )],
        );
        let idx = ReverseIndex::build(&dir).unwrap();

        let importers = idx.importers_of_module("db/client");
        assert_eq!(importers.len(), 1);
        assert_eq!(importers[0].file, "src/b2.ts");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rust_struct_usage_is_indexed() {
        let dir = temp_ws("rust");
        write_ws(
            &dir,
            &[
                (
                    "src/models.rs",
                    "pub struct Payment {\n    pub cents: u64,\n}\n",
                ),
                (
                    "src/main.rs",
                    "mod models;\n\nfn total(p: Payment) -> u64 {\n    p.cents\n}\n",
                ),
            ],
        );
        let idx = ReverseIndex::build(&dir).unwrap();

        assert_eq!(idx.definitions_of("Payment")[0].file, "src/models.rs");
        let users = idx.callers_of("Payment");
        assert!(
            users.iter().any(|r| r.file == "src/main.rs"),
            "main.rs references Payment: {users:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod barrel_probe_tests {
    use super::*;

    #[test]
    fn probe_barrel_chain_indexing() {
        let d = std::env::temp_dir().join(format!("dagr-bprobe-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("src/db")).unwrap();
        std::fs::write(d.join("src/db/client.ts"), "export const pool = 1;\n").unwrap();
        std::fs::write(d.join("src/db/index.ts"), "export * from \"./client\";\n").unwrap();

        let idx = ReverseIndex::build(&d).unwrap();
        println!("files: {:?}", idx.files());
        for imp in idx.all_imports() {
            println!("import: {}:{} -> {}", imp.file, imp.line, imp.module);
        }
        println!(
            "bindings at db/index.ts:2: {:?}",
            idx.bindings_imported_from("src/db/index.ts", 1)
        );
        assert!(
            !idx.definitions_of("pool").is_empty(),
            "export const must register a definition"
        );
        let _ = std::fs::remove_dir_all(&d);
    }
}
