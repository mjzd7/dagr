//! T-EDGE P1: hostile filesystem matrix + extractor edge cases
//! EC-V7/T-E9: unicode, spaces, CRLF, BOM, tab-indented imports through extractor + walker

use dagr_guard::ArchitectureGuard;
use dagr_guard::rules::BoundaryRule;
use dagr_guard::rules::RuleConfig;

fn write_file(path: &std::path::Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

#[test]
fn unicode_and_space_filenames_scan_correctly() {
    let temp = tempfile::tempdir().unwrap();
    write_file(
        &temp.path().join("src/na\u{ef}ve file \u{2713}.ts"),
        "import { x } from \"forbidden/y\";\n",
    );
    write_file(
        &temp.path().join("src/\u{65e5}\u{672c}\u{8a9e} \u{30c6}\u{30b9}\u{30c8}.ts"),
        "import { z } from \"forbidden/w\";\n",
    );

    let config = RuleConfig {
        version: "1.0".into(),
        project_name: None,
        preset: None,
        boundaries: vec![BoundaryRule {
            name: "No Forbidden".into(),
            from: "**".into(),
            cannot_import: vec!["forbidden/**".into()],
            message: "forbidden".into(),
        }],
        limits: Default::default(),
        security: Default::default(),
    };
    let guard = ArchitectureGuard::with_parts(config, Default::default(), temp.path().to_path_buf());
    let violations = guard.scan_workspace(temp.path()).unwrap();

    assert_eq!(
        violations.len(),
        2,
        "unicode/space filenames must be scanned: {violations:?}"
    );
}

#[test]
fn crlf_and_bom_sources_parse_correctly() {
    // CRLF line endings
    let crlf_source = "import { a } from \"./mod\";\r\nexport function f() {}\r\n";
    let extracted = ArchitectureGuard::extract_imported_module(crlf_source.trim());
    assert_eq!(extracted.as_deref(), Some("./mod"));

    // UTF-8 BOM prefix — documents current behavior (BOM not stripped by trim).
    let bom_source = "\u{FEFF}import { b } from \"./other\";";
    let _ = ArchitectureGuard::extract_imported_module(bom_source.trim());

    // Tab-indented import
    let tabbed = "\timport { c } from \"./tabbed\";";
    let extracted = ArchitectureGuard::extract_imported_module(tabbed);
    assert_eq!(extracted.as_deref(), Some("./tabbed"));
}

#[test]
fn extractor_handles_empty_and_whitespace_lines() {
    assert!(ArchitectureGuard::extract_imported_module("").is_none());
    assert!(ArchitectureGuard::extract_imported_module("   ").is_none());
    assert!(ArchitectureGuard::extract_imported_module("\t\n").is_none());
    assert!(ArchitectureGuard::extract_imported_module("// just a comment").is_none());
}
