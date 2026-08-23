//! F3.2 acceptance: cross-file type-contract hoisting via relative imports.

use dagr_slicer::{SlicerConfig, SymbolicSlicer};
use std::path::Path;

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

#[test]
fn cross_file_contract_is_hoisted_one_hop() {
    let root = std::env::temp_dir().join(format!("dagr_f32_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let src = root.join("src");
    std::fs::create_dir_all(&src).unwrap();

    write(
        &src.join("payment_types.ts"),
        "export interface PaymentPayload {\n  userId: string;\n  amountCents: number;\n}\n",
    );
    write(
        &src.join("billing_service.ts"),
        r#"import { PaymentPayload } from "./payment_types";

export async function chargeCustomer(payload: PaymentPayload): Promise<boolean> {
  if (payload.amountCents <= 0) {
    return false;
  }
  return true;
}
"#,
    );

    let slicer = SymbolicSlicer::new(SlicerConfig {
        workspace_root: root.clone(),
        ..SlicerConfig::default()
    });
    let source = std::fs::read_to_string(src.join("billing_service.ts")).unwrap();
    let slice = slicer
        .slice(
            Path::new("src/billing_service.ts"),
            &source,
            dagr_core::Language::from_extension("ts"),
            "chargeCustomer",
        )
        .expect("slice succeeds");

    assert!(
        slice
            .type_contracts
            .iter()
            .any(|c| c.contains("interface PaymentPayload")),
        "PaymentPayload contract must be hoisted from the imported file; got: {:?}",
        slice.type_contracts
    );

    // Depth gate: max_depth_hops = 0 disables cross-file hoisting entirely.
    let slicer = SymbolicSlicer::new(SlicerConfig {
        workspace_root: root.clone(),
        max_depth_hops: 0,
        ..SlicerConfig::default()
    });
    let slice = slicer
        .slice(
            Path::new("src/billing_service.ts"),
            &source,
            dagr_core::Language::from_extension("ts"),
            "chargeCustomer",
        )
        .unwrap();
    assert!(
        !slice
            .type_contracts
            .iter()
            .any(|c| c.contains("PaymentPayload")),
        "depth=0 must not hoist cross-file contracts"
    );

    drop(slicer);
    let _ = std::fs::remove_dir_all(&root);
}

/// F3.2 v1.1: alias specifiers (`@/*`) hoist contracts through tsconfig
/// `paths` mappings, matching Next.js/Vite conventions.
#[test]
fn alias_import_resolves_through_tsconfig_paths() {
    let root = std::env::temp_dir().join(format!("dagr_f32_alias_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let src = root.join("src");
    std::fs::create_dir_all(&src).unwrap();

    std::fs::write(
        root.join("tsconfig.json"),
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@/*":["./src/*"]}}}"#,
    )
    .unwrap();
    write(
        &src.join("payment_types.ts"),
        "export interface RefundRequest {\n  orderId: string;\n}\n",
    );
    write(
        &src.join("refund_service.ts"),
        r#"import { RefundRequest } from "@/payment_types";

export async function refund(req: RefundRequest): Promise<boolean> {
  return req.orderId.length > 0;
}
"#,
    );

    let slicer = SymbolicSlicer::new(SlicerConfig {
        workspace_root: root.clone(),
        ..SlicerConfig::default()
    });
    let source = std::fs::read_to_string(src.join("refund_service.ts")).unwrap();
    let slice = slicer
        .slice(
            Path::new("src/refund_service.ts"),
            &source,
            dagr_core::Language::from_extension("ts"),
            "refund",
        )
        .expect("slice succeeds");

    assert!(
        slice
            .type_contracts
            .iter()
            .any(|c| c.contains("interface RefundRequest")),
        "aliased contract must be hoisted via tsconfig paths; got: {:?}",
        slice.type_contracts
    );

    drop(slicer);
    let _ = std::fs::remove_dir_all(&root);
}
