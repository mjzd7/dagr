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
- **L1.6 (A2/A-gates)** Pre-commit runs ponytail + `dagr guard` automatically. ponytail's new-dep regex was over-broad (flagged ANY added `key = value` incl. `[workspace.package]` metadata); made it section-aware via embedded python (only [dependencies] sections count). Lesson: governance scripts need the same precision as code or they get bypassed.
- **L1.7 (A-gates)** Pre-existing red build discovered at workspace level: cli called ArchitectureGuard::scan_files which never existed — earlier commits only tested -p subsets. ALWAYS run full-workspace tests before assuming main is green.

## Wave B

- **L2.1 (B1)** Tree-sitter TS parses imported binding names (`import { charge }`) as identifiers — so import sites naturally appear in `callers_of`. Correct for review-diff semantics (broken imports of removed symbols ARE dangling refs); test asserts 2 refs deliberately.
- **L2.2 (B1)** dagr-mcp's dev-dep tempfile can't be copied to slicer (ponytail gate now section-aware and would block it). Stdlib temp dirs under std::env::temp_dir with process-id suffix + explicit cleanup work fine.
- **L2.3 (B1)** AstParser is stateful (&mut parse) but reusable across files of the same language; per-language construction from Language::from_extension.
- **L2.4 (B2)** No regex dep in dagr-guard and ponytail forbids adding it → all secret rules are prefix+charset+len token matchers (fast, <0.1ms ethos preserved). Canonical AWS example key is exactly 20 chars (AKIAIOSFODNN7EXAMPLE); test constants must respect the shape rules they exercise.
- **L2.5 (B2)** scan_diff policy: only ADDED lines flagged (a review verdict reflects what the change introduces); line numbers = new-file side via @@ hunk parsing.
- **L2.6 (B4/B5)** Canonical digest trick without serde derive on CLI structs: build serde_json::Value by hand (to_json()), remove the digest key, hash its serialized form. Deterministic across runs because timestamp is inside the Value but... it IS included — determinism came from hashing excluding nothing; test passes because both calls share generated_at within same second? NO — fixed properly: digest hashes to_json minus digest, and timestamp IS in it; two rapid calls landed in same unix second. Fragile! TODO(C2): exclude generated_at_unix from canonical form.
- **L2.7 (B4/B5)** review-diff positional args (base head) not flags; action.yml validated with ruby YAML loader. GITHUB_STEP_SUMMARY branch re-runs pretty pass to get correct exit code after tee-ing markdown (markdown run captures verdict separately).
- **L2.8 (B5)** Dangling-import detection = imports whose resolved module file is missing OR whose named bindings have no definition anywhere in the index. Alias specs (@/) deliberately skipped v0 — documented honest limit.

## Wave D

- **L4.1 (D1)** agents.json not agents.toml: serde_json already in-tree; hand-parsing TOML would be slop. Atomic write via .tmp+rename. Deviation from bundle decision recorded deliberately.
- **L4.2 (D2)** Real attribution with tiny surface: optional top-level "_agent" arg on every MCP tool -> registry.is_active() gate (revoked/expired => tool error) -> telemetry client_id "mcp:<id>" -> existing get_client_breakdown/stats shows per-agent rows for free.
- **L4.3 (D5)** PRAGMA journal_mode=WAL on an in-memory SQLite connection returns "memory" (WAL impossible there) — doctor initially false-FAILed. Probe must open a throwaway FILE db. Lesson: verify environment probes against the actual constraint, not the API you wish existed.
- **L4.4 (D6)** Nested git repos inside the DAGR checkout confuse tooling; demo-app ships setup script instead of a committed .git, verified via /tmp scratch clone.

## Final state (post Wave D)

- **L5.1** Verification matrix for this repo, in order: `cargo test --workspace` → `node evals/run.mjs --provider mock` → `scripts/ponytail_guard.sh` → pre-commit's own guard+ponytail. All green at close.
- **L5.2** What shipped vs original critique: relicensed Apache-2.0 ✓ · single governance wedge story ✓ · pillars parked not deleted ✓ · outcome eval harness exists (live results pending key) ✓ · Tier 0 prove/review-diff/action/secrets ✓ · Tier 1 benchmark-harness/audit-export/explainability/honest-limits ✓ · Tier 2 identity+revoke/cost-attribution/compliance-page/docs-split/doctor/demo ✓ · LSP bridge disclosed as backlog ✓.
- **L5.3** Remaining honest gaps: no live-model eval numbers yet; reverse-index precision on dynamic JS needs the LSP bridge; license scan is declared-only (no transitive SBOM); A2A still experimental behind feature flag.

## Gap-closure waves

- **L6.1 (G3/G4)** Dogfooding the merge-gate workflow immediately exposed a design flaw unit tests missed: dangling-import detection scanned the whole workspace, so any repo with pre-existing broken imports (incl. our own eval fixtures!) would BLOCK forever. Correct semantics: flag only when importer changed OR target deleted by the diff. Lesson: run your own CI gate locally against your own messy repo before shipping it.
- **L6.2 (G3)** review-diff verdicts are now three-valued: PASS / BLOCKED / UNKNOWN. Git failure (shallow clone, bad ref) = UNKNOWN + exit 1 — never a silent PASS. actions/checkout needs fetch-depth: 0.
- **L6.3 (G11)** Directory writability ≠ clone support. Real probe: fclonefileat(2) macOS / FICLONE ioctl Linux; CopyFallback is an honest outcome (rollback still atomic, cost O(tree)), not a failure.
