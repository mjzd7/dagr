# DAGR — Learnings Ledger

Everything worth remembering during the completion plan (Waves A–D). Append-only per task.

---

## Pre-flight (repo recon)

- **L0.1** Workspace = 8 crates: `dagr-core`, `dagr-slicer`, `dagr-guard`, `dagr-sandbox`, `dagr-mcp`, `dagr-cli`, `dagr-cloud`, `dagr-chaos`. All license fields currently `BSL-1.1` (workspace + crates).
- **L0.2** A2A surface is tiny: ~14 refs in `dagr-mcp/src/tools.rs`, ~5 in `server.rs` → feature-gating is cheap.
- **L0.3** No `evals/` dir exists; token studies live in `docs/findings/wave*.md` (next.js/vite/deno/vscode/rust-analyzer/tokio) — input metrics only.
- **L0.4** `.ponytail.md` governance = **zero new dependencies**; violations fail pre-commit (`scripts/ponytail_guard.sh`) and CI (`ponytail-guard.yml`). Every simplification must be tagged `// ponytail: <what>, upgrade when <trigger>`.
- **L0.5** AGENTS.md acceptance: `cargo test --workspace` exit 0, `dagr guard` 0 violations, no lingering `.dagr/shadow/` temp dirs.
- **L0.6** Root `action.yml` exists (GitHub Action); `packages/dagr` is the npm wrapper; `dashboard/` is a Next.js telemetry app.
- **L0.7** Working tree had a stray deleted `original_state.txt` at start — field-test artifact, restored before Wave A commit.

## Wave A

- **L1.1 (A1)** All 8 crate manifests inherit `license.workspace = true` — one-line workspace change covers everything. `cargo metadata --no-deps | jq '.packages[].license'` is the verification oracle.
- **L1.2 (A1)** Stale BSL text also lived in CONTRIBUTING.md (live legal terms — fixed) and TODO.md (historical milestone log — left as history). npm wrapper package.json said MIT, not BSL — aligned to Apache-2.0.
- **L1.3** PROGRESS.md write via tool reported success but file was absent on first attempt; rewriting worked. Always `ls` after writing tracked files in this repo.
- **L1.4 (A2)** Feature-gating pattern that worked: `let mut tools = vec![...]; #[cfg(feature="a2a")] tools.extend([...]); tools` for list_tools; `#[cfg]` directly on match arms and handler fns; field+init+import (`uuid::Uuid` only used by a2a path) all need gates or you get dead-code warnings.
- **L1.5 (A2)** server.rs has an exact-allowlist tool test ("EC-P7") — additions/renames are deliberate reviewed events. Any Wave B/C MCP tool addition MUST update that allowlist test in both cfg variants.
