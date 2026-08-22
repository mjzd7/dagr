use dagr_guard::RuleConfig;
use std::path::Path;

fn write_config(root: &Path, yaml: &str) {
    let dir = root.join(".dagr");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("rules.yaml"), yaml).unwrap();
}

fn err_of(root: &Path) -> String {
    match RuleConfig::load_or_default(root) {
        Ok(_) => panic!("expected strict-schema parse error"),
        Err(e) => format!("{e}"),
    }
}

/// L1 regression: the historical field-session config used a top-level
/// `rules:` key (instead of `boundaries:`) and previously parsed as a
/// zero-rule config whose guard always reported PASS. It must now be a hard error.
#[test]
fn legacy_top_level_wrong_shape_rejected() {
    let temp = tempfile::tempdir().unwrap();
    write_config(
        temp.path(),
        r#"version: "1.0"
rules:
  - source: "packages/core/src/content-filter/**"
    disallow:
      - "packages/core/src/db/**"
    reason: "content-filter must stay storage-free"
"#,
    );
    let msg = err_of(temp.path());
    assert!(msg.contains("unknown field"), "error was: {msg}");
    assert!(msg.contains("`rules`"), "error must name offending key: {msg}");
}

/// Nested strictness: a typo'd key inside a boundary entry (`disallow:` was
/// the legacy spelling of `cannot_import:`) must be rejected, not dropped.
#[test]
fn nested_boundary_unknown_key_rejected() {
    let temp = tempfile::tempdir().unwrap();
    write_config(
        temp.path(),
        r#"version: "1.0"
boundaries:
  - name: "UI-to-DB Boundary"
    from: "src/ui/**"
    cannot_import:
      - "src/db/**"
    disallow:
      - "src/infra/**"
"#,
    );
    let msg = err_of(temp.path());
    assert!(msg.contains("unknown field"), "error was: {msg}");
    assert!(
        msg.contains("`disallow`"),
        "error must name offending nested key: {msg}"
    );
}

/// Typo'd keys inside `limits` previously vanished silently (e.g.
/// `max_file_line` instead of `max_file_lines`) leaving limits unenforced.
#[test]
fn limits_unknown_key_rejected() {
    let temp = tempfile::tempdir().unwrap();
    write_config(
        temp.path(),
        r#"version: "1.0"
limits:
  max_file_line: 300
"#,
    );
    let msg = err_of(temp.path());
    assert!(msg.contains("unknown field"), "error was: {msg}");
}

#[test]
fn valid_minimal_schema_parses_with_default_message() {
    let temp = tempfile::tempdir().unwrap();
    write_config(
        temp.path(),
        r#"version: "1.0"
boundaries:
  - name: "UI-to-DB Boundary"
    from: "src/ui/**"
    cannot_import:
      - "src/db/**"
"#,
    );
    let config = RuleConfig::load_or_default(temp.path()).unwrap();
    assert_eq!(config.boundaries.len(), 1);
    assert_eq!(config.boundaries[0].from, "src/ui/**");
    assert_eq!(
        config.boundaries[0].message,
        "Architectural layer boundary violation detected"
    );
}

#[test]
fn valid_full_schema_roundtrips() {
    let temp = tempfile::tempdir().unwrap();
    write_config(
        temp.path(),
        r#"version: "1.0"
project_name: my-monorepo
boundaries:
  - name: "Core Independence"
    from: "crates/*-core/**"
    cannot_import:
      - "tokio"
    message: "Core crates stay runtime-free."
limits:
  max_file_lines: 300
  max_function_lines: 40
  disallow_eval: false
security:
  sanitize_prompt_injections: false
  strip_control_tokens:
    - "[INST]"
"#,
    );
    let config = RuleConfig::load_or_default(temp.path()).unwrap();
    assert_eq!(config.project_name.as_deref(), Some("my-monorepo"));
    assert_eq!(config.limits.max_file_lines, Some(300));
    assert_eq!(config.limits.max_function_lines, Some(40));
    assert_eq!(config.limits.disallow_eval, Some(false));
    assert!(!config.security.sanitize_prompt_injections);
    assert_eq!(config.security.strip_control_tokens, vec!["[INST]"]);
}

/// Intentional behavior preserved: absent file = built-in preset (this is why
/// a *missing* file enforces MORE than the old malformed file did).
#[test]
fn missing_file_falls_back_to_clean_architecture_preset() {
    let temp = tempfile::tempdir().unwrap();
    let config = RuleConfig::load_or_default(temp.path()).unwrap();
    assert_eq!(config.preset.as_deref(), Some("clean-architecture"));
    assert!(!config.boundaries.is_empty());
}

/// Pins the semantic distinction: a syntactically-valid config with zero
/// boundaries is user intent, not corruption — allowed, unlike malformed files.
#[test]
fn explicit_zero_boundaries_is_intentional_and_allowed() {
    let temp = tempfile::tempdir().unwrap();
    write_config(temp.path(), "version: \"1.0\"\n");
    let config = RuleConfig::load_or_default(temp.path()).unwrap();
    assert!(config.boundaries.is_empty());
}

/// N2 regression: an uncompilable 'from' glob previously became a silently
/// dead rule — check_import skipped it via `if let Ok(Pattern::new(..))`.
#[test]
fn invalid_from_glob_rejected_with_rule_name() {
    let temp = tempfile::tempdir().unwrap();
    write_config(
        temp.path(),
        r#"version: "1.0"
boundaries:
  - name: "Filter Isolation"
    from: "[unclosed"
    cannot_import:
      - "packages/core/src/db/**"
"#,
    );
    let msg = err_of(temp.path());
    assert!(
        msg.contains("Filter Isolation"),
        "error must name the offending rule: {msg}"
    );
    assert!(
        msg.contains("[unclosed"),
        "error must name the offending pattern: {msg}"
    );
}

#[test]
fn invalid_cannot_import_glob_rejected_with_rule_name() {
    let temp = tempfile::tempdir().unwrap();
    write_config(
        temp.path(),
        r#"version: "1.0"
boundaries:
  - name: "UI-to-DB Boundary"
    from: "src/ui/**"
    cannot_import:
      - "src/db/**"
      - "[unclosed"
"#,
    );
    let msg = err_of(temp.path());
    assert!(
        msg.contains("UI-to-DB Boundary"),
        "error must name the offending rule: {msg}"
    );
    assert!(
        msg.contains("[unclosed"),
        "error must name the offending pattern: {msg}"
    );
}

/// Presets are static known-good strings; validation must never reject them.
#[test]
fn built_in_presets_pass_pattern_validation() {
    assert!(RuleConfig::clean_architecture_preset()
        .validate_patterns()
        .is_ok());
    assert!(RuleConfig::nextjs_preset().validate_patterns().is_ok());
}
