//! T-EDGE battery from HYPERPLAN scrutiny:
//! - EC-V1/T-E3: property-style fuzz of extract_imported_module + resolver
//!   (deterministic seeded; no-panic, quoted-substring, no-`..`-output)
//! - EC-S4/T-E2: traversal-read proof — `../`-heavy specifiers must not read
//!   outside the workspace root via barrel/hoister paths.

use dagr_guard::rules::RuleConfig;
use dagr_guard::ArchitectureGuard;

const ALPHABET: [&str; 12] = [
    "import", "from", "export", "use", "require", "(", ")", "\"", "'", "..", "/", "{x}",
];

fn lcg_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state >> 16
}

fn gen_line(seed: u64, len_hint: usize) -> String {
    let mut st = seed | 1;
    let mut out = String::new();
    for _ in 0..(len_hint % 24 + 1) {
        out.push_str(ALPHABET[(lcg_next(&mut st) as usize) % ALPHABET.len()]);
    }
    out
}

#[test]
fn fuzz_extractor_never_panics_and_respects_invariants() {
    // Structured mutations around real forms plus pure alphabet soup.
    let templates = [
        "import { A } from \"{}\";",
        "const x = require('{}');",
        "await import(\"{}\");",
        "use {}::item;",
        "from {} import y",
        "export * from '{}';",
        "{}",
    ];
    for i in 0..600u64 {
        let mut st = i | 1;
        let filler_len = (lcg_next(&mut st) % 20) as usize;
        let filler = gen_line(i.wrapping_mul(7919), filler_len);
        let line = if i % 3 == 0 {
            templates[(i as usize) % templates.len()].replace("{}", &filler)
        } else {
            filler
        };

        // Invariant 1: never panics (this call IS the test body).
        let extracted = ArchitectureGuard::extract_imported_module(&line);

        // Invariant 2: an extracted module is never empty and contains no
        // quote characters or newlines.
        if let Some(m) = extracted {
            assert!(!m.is_empty(), "empty extraction from {line:?}");
            assert!(
                !m.contains('"') && !m.contains('\''),
                "quote leaked: {line:?}"
            );
            assert!(!m.contains('\n'), "newline leaked: {line:?}");
        }

        // Invariant 3: resolver never panics and never emits `..` segments.
        let cands = dagr_guard::checker_relative_candidates("src/a.ts", &line);
        for c in cands {
            assert!(
                !c.split('/').any(|seg| seg == ".."),
                "resolver emitted `..` for {line:?}: {c}"
            );
        }
    }
}

#[test]
fn long_and_degenerate_inputs_stay_bounded() {
    let long_spec = "../".repeat(2_000) + "deep";
    let long_line = format!("import x from \"{long_spec}\";");
    assert!(ArchitectureGuard::extract_imported_module(&long_line).is_some());
    // Resolver clamps `..` at the root and keeps remaining segments —
    // the contract is no `..` in output, not emptiness.
    let cands = dagr_guard::checker_relative_candidates("a.ts", &long_spec);
    assert!(
        cands.iter().all(|c| !c.split('/').any(|seg| seg == "..")),
        "no `..` may survive clamping: {cands:?}"
    );

    let huge = "x".repeat(100_000);
    assert!(ArchitectureGuard::extract_imported_module(&huge).is_none());
}

/// EC-S4: a decoy file OUTSIDE the workspace root containing a forbidden
/// re-export must never influence guard results.
#[test]
fn barrel_reader_never_escapes_workspace_root() {
    let outer = tempfile::tempdir().unwrap();
    let inner = tempfile::tempdir_in(&outer).unwrap();

    // Decoy lives NEXT TO the workspace root, reachable only via `..`.
    std::fs::write(
        outer.path().join("decoy_internal.ts"),
        "export { s } from \"./internal/secret\";\n",
    )
    .unwrap();

    std::fs::create_dir_all(inner.path().join("internal")).unwrap();
    std::fs::write(
        inner.path().join("public_api.ts"),
        "export const clean = 1;\n",
    )
    .unwrap();

    let config = RuleConfig {
        version: "1.0".into(),
        project_name: None,
        preset: None,
        boundaries: vec![dagr_guard::rules::BoundaryRule {
            name: "No Internals".into(),
            from: "apps/**".into(),
            cannot_import: vec!["internal/**".into()],
            message: "internal".into(),
        }],
        limits: Default::default(),
        security: Default::default(),
    };
    let guard =
        ArchitectureGuard::with_parts(config, Default::default(), inner.path().to_path_buf());

    // "../../decoy_internal" resolves to `decoy_internal` INSIDE the root's
    // lexical parent — the resolver clamps it to nothing outside, so the
    // decoy must NOT be read and NO violation may fire.
    assert!(guard
        .check_import("apps/web/a.ts", "../../decoy_internal")
        .is_none());
}

/// EC-F2: front-loaded walker ignore-list — planted violations inside
/// heavy/generated dirs (build/, vendor/, __pycache__/…) must be skipped,
/// while the identical violation in a scanned dir is caught.
#[test]
fn walker_front_loaded_ignore_list_skips_heavy_dirs() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(temp.path().join("src")).unwrap();
    std::fs::create_dir_all(temp.path().join("build")).unwrap();
    std::fs::create_dir_all(temp.path().join("vendor")).unwrap();
    let violating = "import { s } from \"forbidden/y\";\n";
    std::fs::write(temp.path().join("src/scanned.ts"), violating).unwrap();
    std::fs::write(temp.path().join("build/generated.ts"), violating).unwrap();
    std::fs::write(temp.path().join("vendor/vendored.ts"), violating).unwrap();

    let config = RuleConfig {
        version: "1.0".into(),
        project_name: None,
        preset: None,
        boundaries: vec![dagr_guard::rules::BoundaryRule {
            name: "No Forbidden".into(),
            from: "**".into(),
            cannot_import: vec!["forbidden/**".into()],
            message: "forbidden module".into(),
        }],
        limits: Default::default(),
        security: Default::default(),
    };
    let guard =
        ArchitectureGuard::with_parts(config, Default::default(), temp.path().to_path_buf());
    let violations = guard.scan_workspace(temp.path()).unwrap();

    assert_eq!(
        violations.len(),
        1,
        "only the scanned-dir file counts: {violations:?}"
    );
    assert_eq!(violations[0].source_file, "src/scanned.ts");
}
