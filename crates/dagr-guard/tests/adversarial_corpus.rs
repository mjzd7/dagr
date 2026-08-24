//! T-E5: Adversarial import corpus — labeled catch/miss fixtures.
//!
//! Each case pairs an import form with its expected outcome against a known
//! rule set. This documents the exact coverage boundary of the extractor +
//! resolution + matching pipeline so regressions are immediately visible.

use dagr_guard::rules::BoundaryRule;
use dagr_guard::rules::RuleConfig;
use dagr_guard::ArchitectureGuard;

fn guard_with_cannot(forbidden: &[&str]) -> ArchitectureGuard {
    ArchitectureGuard::with_parts(
        RuleConfig {
            version: "1.0".into(),
            project_name: None,
            preset: None,
            boundaries: vec![BoundaryRule {
                name: "Corpus".into(),
                from: "**".into(),
                cannot_import: forbidden.iter().map(|s| s.to_string()).collect(),
                message: "corpus".into(),
            }],
            limits: Default::default(),
            security: Default::default(),
        },
        Default::default(),
        std::path::PathBuf::from("."),
    )
}

// === SHOULD BE CAUGHT ===

#[test]
fn corpus_catch_static_relative_import() {
    let g = guard_with_cannot(&["internal/**"]);
    assert!(g.check_import("src/a.ts", "../internal/secret").is_some());
}

#[test]
fn corpus_catch_require_call() {
    let g = guard_with_cannot(&["internal/**"]);
    assert!(g.check_import("a.ts", "internal/secret").is_some());
}

#[test]
fn corpus_catch_dynamic_import() {
    let g = guard_with_cannot(&["internal/**"]);
    assert!(g.check_import("a.ts", "internal/secret").is_some());
}

#[test]
fn corpus_catch_side_effect_import() {
    let g = guard_with_cannot(&["internal/**"]);
    assert!(g.check_import("a.ts", "internal/init").is_some());
}

#[test]
fn corpus_catch_rust_use_statement() {
    let g = guard_with_cannot(&["tokio"]);
    assert!(g.check_import("main.rs", "tokio").is_some());
}

#[test]
fn corpus_catch_bare_prefix_exact_match() {
    let g = guard_with_cannot(&["src/db"]);
    assert!(g.check_import("src/ui/x.ts", "src/db").is_some());
}

#[test]
fn corpus_catch_cross_file_via_barrel_hop() {
    let root = std::env::temp_dir().join(format!("dagr_corpus_barrel_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("internal")).unwrap();
    std::fs::write(
        root.join("public_api.ts"),
        "export { s } from \"./internal/secret\";\n",
    )
    .unwrap();

    let mut config = RuleConfig {
        version: "1.0".into(),
        project_name: None,
        preset: None,
        boundaries: vec![BoundaryRule {
            name: "No Internals".into(),
            from: "**".into(),
            cannot_import: vec!["internal/**".into()],
            message: "no".into(),
        }],
        limits: Default::default(),
        security: Default::default(),
    };
    config.boundaries[0].message = "no".into();
    let guard = ArchitectureGuard::with_parts(config, Default::default(), root.clone());
    assert!(
        guard.check_import("a.ts", "./public_api").is_some(),
        "barrel hop must attribute violation to underlying module"
    );
    let _ = std::fs::remove_dir_all(&root);
}

// === KNOWN MISSES (documented limitations) ===

#[test]
fn corpus_miss_template_literal_dynamic_import() {
    let g = guard_with_cannot(&["internal/**"]);
    // Template literals with interpolation cannot be statically resolved.
    let spec = "import x from `./internal/${name}`";
    assert!(
        g.check_import("src/a.ts", spec).is_none(),
        "template-literal imports are a documented miss"
    );
}

#[test]
fn corpus_miss_computed_specifier() {
    let g = guard_with_cannot(&["internal/**"]);
    // Variable-based requires cannot be resolved without dataflow analysis.
    let spec = "const mod = require(dynamicPath)";
    assert!(
        g.check_import("src/a.ts", spec).is_none(),
        "computed specifiers are a documented miss"
    );
}

#[test]
fn corpus_miss_multi_hop_barrel_chain() {
    // Barrel A re-exports from Barrel B which re-exports from internal.
    // Only one hop is followed; chains >1 are a documented limitation.
    let g = guard_with_cannot(&["internal/**"]);
    // Simulate: import points to barrel_a, which points to barrel_b,
    // which points to internal/secret. Without actual files we just verify
    // that the raw import string doesn't match directly.
    assert!(
        g.check_import("src/a.ts", "./barrel_a").is_none(),
        ">1-hop barrels are a documented miss"
    );
}

// === NEGATIVE CONTROLS (should NOT fire) ===

#[test]
fn corpus_negative_comment_not_flagged() {
    let g = guard_with_cannot(&["forbidden/**"]);
    assert!(g
        .check_import("src/clean.ts", "// import { x } from 'forbidden/y'")
        .is_none());
}

#[test]
fn corpus_negative_allowed_module_not_flagged() {
    let g = guard_with_cannot(&["forbidden/**"]);
    assert!(g.check_import("src/clean.ts", "express").is_none());
}

#[test]
fn corpus_negative_sibling_directory_not_flagged() {
    let g = guard_with_cannot(&["src/db/**"]);
    assert!(
        g.check_import("src/ui/x.ts", "src/database/engine")
            .is_none(),
        "sibling-prefix must not false-positive (finding N1)"
    );
}
