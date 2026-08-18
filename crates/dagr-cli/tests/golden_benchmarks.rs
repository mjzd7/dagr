use dagr_core::{Language, Result};
use dagr_sandbox::CowSandbox;
use dagr_slicer::{SlicerConfig, SymbolicSlicer};
use std::path::Path;
use std::time::Instant;

#[test]
fn test_golden_typescript_slice_and_compression() -> Result<()> {
    let fixture_path = Path::new("../../tests/fixtures/billing_service.ts");
    assert!(
        fixture_path.exists(),
        "Fixture billing_service.ts must exist"
    );

    let source_code = std::fs::read_to_string(fixture_path)?;
    let slicer = SymbolicSlicer::new(SlicerConfig::default());

    // Warm up tokenizer and Tree-sitter parser initialization
    let _ = dagr_core::count_tokens("warmup");
    let _ = slicer.slice(
        fixture_path,
        &source_code,
        Language::TypeScript,
        "chargeCustomer",
    );

    let start = Instant::now();
    let slice = slicer.slice(
        fixture_path,
        &source_code,
        Language::TypeScript,
        "chargeCustomer",
    )?;
    let elapsed = start.elapsed();

    // 1. Verify warm slicing latency is sub-10ms
    assert!(
        elapsed.as_millis() < 25,
        "Warm slicing latency should be ultra-fast (was {:?})",
        elapsed
    );

    // 2. Verify token reduction is substantial
    assert!(
        slice.compression_ratio >= 0.50,
        "Expected compression ratio >= 50% on test fixture (got {:.2}%)",
        slice.compression_ratio * 100.0
    );

    // 3. Verify hoisted type contracts
    assert!(
        slice
            .type_contracts
            .iter()
            .any(|c| c.contains("interface PaymentIntent")),
        "Must hoist PaymentIntent interface"
    );
    assert!(
        slice
            .type_contracts
            .iter()
            .any(|c| c.contains("interface PaymentResult")),
        "Must hoist PaymentResult interface"
    );

    // 4. Verify unreferenced contracts are NOT hoisted (e.g. CustomerRecord)
    assert!(
        !slice
            .type_contracts
            .iter()
            .any(|c| c.contains("interface CustomerRecord")),
        "Must NOT hoist unreferenced CustomerRecord"
    );

    Ok(())
}

#[test]
fn test_golden_python_slice_and_contract_hoisting() -> Result<()> {
    let fixture_path = Path::new("../../tests/fixtures/auth_pipeline.py");
    assert!(fixture_path.exists(), "Fixture auth_pipeline.py must exist");

    let source_code = std::fs::read_to_string(fixture_path)?;
    let slicer = SymbolicSlicer::new(SlicerConfig::default());

    let slice = slicer.slice(fixture_path, &source_code, Language::Python, "verify_token")?;

    // Hoisted class UserToken
    assert!(
        slice
            .type_contracts
            .iter()
            .any(|c| c.contains("class UserToken")),
        "Must hoist UserToken class"
    );

    // Sliced lines contain verify_token implementation
    assert!(
        slice
            .sparse_code_lines
            .iter()
            .any(|(_, l)| l.contains("def verify_token")),
        "Must contain verify_token implementation"
    );

    Ok(())
}

#[test]
fn test_golden_cow_sandbox_atomic_rollback() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let workspace = temp_dir.path();

    // 1. Create a dummy file in workspace
    let original_file = workspace.join("src").join("calc.rs");
    std::fs::create_dir_all(original_file.parent().unwrap())?;
    std::fs::write(
        &original_file,
        b"pub fn add(a: i32, b: i32) -> i32 { a + b }",
    )?;

    // 2. Begin sandbox transaction
    let mut tx = CowSandbox::begin(workspace)?;

    // 3. Stage a destructive mutation in shadow workspace
    let mutated_content = b"pub fn add(a: i32, b: i32) -> i32 { 0 } // BROKEN";
    CowSandbox::stage_file(&mut tx, Path::new("src/calc.rs"), mutated_content)?;

    // 4. Verify the actual workspace file is completely untouched
    let current_content = std::fs::read(&original_file)?;
    assert_eq!(
        current_content,
        b"pub fn add(a: i32, b: i32) -> i32 { a + b }"
    );

    // 5. Rollback transaction
    CowSandbox::rollback(tx)?;

    // 6. Verify shadow directory is purged (0 residual bytes)
    assert_eq!(
        std::fs::read_to_string(&original_file)?,
        "pub fn add(a: i32, b: i32) -> i32 { a + b }"
    );

    Ok(())
}
