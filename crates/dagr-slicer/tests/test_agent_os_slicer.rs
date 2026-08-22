use dagr_core::{Language, MinimalContextSlice, SymbolKind};
use dagr_slicer::{ASTPageFaultHandler, PositionAwareAssembler, SlicerQueryCache};
use std::path::PathBuf;
use std::time::Duration;

#[test]
fn test_ast_page_fault_scanner_and_contract_hoisting() {
    let handler = ASTPageFaultHandler::new();
    let code = r#"
        pub fn process_order(repo: &OrderRepository, auth: &AuthService) -> Result<PaymentToken> {
            let user = repo.find_user()?;
            auth.verify(&user)?;
            Ok(PaymentToken::new())
        }
    "#;

    let unresolved = handler.scan_unresolved_symbols(code, Language::Rust);
    assert!(unresolved.contains(&"OrderRepository".to_string()));
    assert!(unresolved.contains(&"AuthService".to_string()));
    assert!(unresolved.contains(&"PaymentToken".to_string()));

    let hoisted = handler.synthesize_contract(
        "OrderRepository",
        &PathBuf::from("src/domain/repo.rs"),
        SymbolKind::Struct,
        "pub struct OrderRepository { pub pool: PgPool }",
    );
    assert_eq!(hoisted.symbol_name, "OrderRepository");
    assert!(hoisted.token_cost > 0);
    assert!(hoisted
        .signature_slice
        .contains("pub struct OrderRepository"));
}

#[test]
fn test_position_aware_prompt_assembly() {
    let assembler = PositionAwareAssembler::new();
    let handler = ASTPageFaultHandler::new();

    let contract = handler.synthesize_contract(
        "BillingEngine",
        &PathBuf::from("src/billing.rs"),
        SymbolKind::Interface,
        "pub trait BillingEngine { fn charge(&self); }",
    );

    let active_slice = MinimalContextSlice {
        target_symbol: "execute_charge".into(),
        file_path: PathBuf::from("src/main.rs"),
        language: Language::Rust,
        sparse_code_lines: vec![(10, "pub fn execute_charge() {}".into())],
        type_contracts: vec![],
        estimated_tokens: 20,
        original_file_tokens: 500,
        compression_ratio: 0.96,
        syntax_degraded: false,
    };

    let assembled = assembler.assemble(
        "You are an AI coding assistant.",
        &["UI must not import Database directly".into()],
        &[contract],
        &["User: fix payment issue".into()],
        &active_slice,
        "Implement charge retry logic",
    );

    assert!(assembled.total_tokens > 0);
    assert!(assembled
        .final_prompt
        .contains("=== ARCHITECTURAL BOUNDARY RULES ==="));
    assert!(assembled
        .final_prompt
        .contains("=== HOISTED SYMBOL TYPE CONTRACTS ==="));
    assert!(assembled
        .final_prompt
        .contains("=== TARGET AST SLICE (PRIMARY CODE CONTEXT) ==="));
}

#[test]
fn test_slicer_positive_and_negative_query_cache() {
    let cache = SlicerQueryCache::new(Duration::from_secs(60));

    assert!(cache.get_positive("src/lib.rs", "my_func").is_none());
    assert!(cache.get_negative("src/lib.rs", "my_func").is_none());

    // Record positive
    let slice = MinimalContextSlice {
        target_symbol: "my_func".into(),
        file_path: PathBuf::from("src/lib.rs"),
        language: Language::Rust,
        sparse_code_lines: vec![(1, "fn my_func() {}".into())],
        type_contracts: vec![],
        estimated_tokens: 10,
        original_file_tokens: 100,
        compression_ratio: 0.9,
        syntax_degraded: false,
    };
    cache.set_positive("src/lib.rs", "my_func", slice.clone());
    assert_eq!(
        cache
            .get_positive("src/lib.rs", "my_func")
            .unwrap()
            .target_symbol,
        "my_func"
    );

    // Record negative
    cache.set_negative(
        "src/bad.rs",
        "broken_symbol",
        "Symbol not found in AST".into(),
    );
    assert_eq!(
        cache.get_negative("src/bad.rs", "broken_symbol").unwrap(),
        "Symbol not found in AST"
    );
}
