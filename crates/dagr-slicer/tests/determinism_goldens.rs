//! EC-V2/T-E6: determinism — identical slices must produce byte-identical JSON.

use dagr_slicer::{SlicerConfig, SymbolicSlicer};
use std::path::Path;

#[test]
fn slice_output_is_deterministic_across_calls() {
    let root = std::env::temp_dir().join(format!("dagr_det_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();

    let source = r#"
export interface Config { host: string; port: number; }
export interface Result { ok: boolean; data?: string; }

export async function handleRequest(cfg: Config): Promise<Result> {
  const res = await fetch(`http://${cfg.host}:${cfg.port}`);
  return { ok: res.ok, data: undefined };
}
"#;
    std::fs::write(root.join("src/service.ts"), source).unwrap();

    let slicer = SymbolicSlicer::new(SlicerConfig {
        workspace_root: root.clone(),
        ..SlicerConfig::default()
    });

    let s1 = slicer
        .slice(
            Path::new("src/service.ts"),
            source,
            dagr_core::Language::from_extension("ts"),
            "handleRequest",
        )
        .unwrap();
    let s2 = slicer
        .slice(
            Path::new("src/service.ts"),
            source,
            dagr_core::Language::from_extension("ts"),
            "handleRequest",
        )
        .unwrap();

    let j1 = serde_json::to_string(&s1).unwrap();
    let j2 = serde_json::to_string(&s2).unwrap();
    assert_eq!(j1, j2, "slice output must be deterministic");

    assert_eq!(s1.sparse_code_lines, s2.sparse_code_lines);
    assert_eq!(s1.type_contracts, s2.type_contracts);

    drop(slicer);
    let _ = std::fs::remove_dir_all(&root);
}
