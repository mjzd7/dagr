# 🔧 DAGR Improvement Roadmap — Findings → Fixes Ledger

> **Living Fix Specification**
> **Companion:** [`TESTING_ROADMAP.md`](./TESTING_ROADMAP.md) (discovery program feeding this ledger)
> **Findings origin:** [`docs/LearningsDAGR-field-session.md`](./docs/LearningsDAGR-field-session.md) (in-repo canonical copy; L-series from field session 2026-08-22) + source audit (N-series, 2026-08-22)
> **Last synchronized:** August 2026

---

## 📌 Conventions

- `L<n>` — field finding from LearningsDAGR.md · `N<n>` — finding from source-code audit · `H<n>` — hypothesis under test.
- Each fix ticket: **Finding ref(s) → Root cause (file:line) → Fix approach → Acceptance criteria → Status**.
- Status: ☐ pending · ◐ partial · ✅ done (with date).

**Priority model:** `P0` fail-open/security/correctness traps · `P1` feature honesty & guard intelligence · `P2` DX/ecosystem/polish.

---

## 🟢 Phase F1 — Eliminate Fail-Open Traps (P0)

### ✅ F1.1 Strict rules.yaml schema validation `[L1, L9.2]`
- **Root cause:** `RuleConfig` (crates/dagr-guard/src/rules.rs:64) derives serde without `deny_unknown_fields`; `boundaries` carries `#[serde(default)]`. A yaml containing `version:` plus a typo'd list key (`rules:`) deserializes cleanly with an **empty boundary vec** → guard exits 0 forever. Meanwhile a *missing* file falls back to `clean_architecture_preset()` (rules.rs:83–84) — i.e., malformed config enforces **less** than absent config (inverted safety). The historical bad file also had nested wrong keys (`source:`/`disallow:`/`reason:` inside entries), so nested structs need strictness too.
- **Fix approach:** add `#[serde(deny_unknown_fields)]` to `RuleConfig` **and** `BoundaryRule`; enrich the `DagrError::Config` message to name offending keys and point to the README schema section; document the full schema in README (partial F4.2).
- **Acceptance criteria:**
  - [x] Legacy wrong-shape file (top-level `rules:`) → hard error listing `unknown field 'rules'`.
  - [x] Nested unknown key inside a boundary entry (`source:` instead of `from:`) → hard error.
  - [x] Valid minimal + full schemas still parse (defaults preserved).
  - [x] Missing-file preset fallback behavior unchanged (intentional).
  - [x] README documents the schema.
- **Status:** ✅ 2026-08-22 (tests: `crates/dagr-guard/tests/rules_schema_tests.rs`)

### ✅ F1.2 Dead-glob hard error `[N2]`
- **Root cause:** `check_import` swallows pattern compilation failures — `if let Ok(from_pattern) = Pattern::new(&rule.from)` and same for forbidden patterns (crates/dagr-guard/src/checker.rs:30,33). An invalid glob = rule that silently never fires. Third fail-open variant.
- **Fix approach:** validate all patterns at config load time (`RuleConfig::load_or_default` or a post-load `validate()` pass) and return `DagrError::Config` naming the rule + bad pattern; optionally keep runtime `debug_assert!`.
- **Acceptance criteria:** config containing an invalid glob (`"[unclosed"`) fails load with rule name in message; existing valid presets unaffected.
- **Status:** ✅ 2026-08-22 — `RuleConfig::validate_patterns()` wired into `load_or_default`; regression tests in `rules_schema_tests.rs`. Runtime matching left untouched (validation guarantees loaded patterns compile; direct-construction callers unchanged).

### ✅ F1.3 MCP strict argument validation `[L6, L9.3]`
- **Root cause:** tool handlers read arguments leniently; wrong/missing values silently coerce to empty → `dagr_verify_architecture` returns `valid: true`. Additionally, unknown tools returned an `isError` content envelope instead of a JSON-RPC error code (server.rs tools/call arm).
- **Fix approach:** every handler validates required fields via `ok_or_else` (as `handle_get_context_slice` already does correctly); reject unknown argument keys per tool input_schema; emit `-32602 Invalid params` from the server layer for argument errors while keeping `-32601` for unknown tools.
- **Acceptance criteria:** violating-case tests (missing arg, wrong type, unknown arg, unknown tool) all return proper JSON-RPC error codes; happy paths unchanged.
- **Status:** ✅ 2026-08-22 — all five lenient handlers hardened (`verify_architecture`, `execute_sandboxed`, `a2a_handshake`, `a2a_transfer_context`, `a2a_verify_peer_patch`). **Scope refinements recorded:** (1) missing/wrong-typed required args surface as MCP-standard `isError: true` tool content with precise messages; unknown tool names return JSON-RPC `-32602`. (2) Unknown *extra* argument keys are deliberately NOT rejected — strict on required presence + types, permissive on client-added extras (avoids breaking legitimate clients; typo'd names are still caught by the required check). Bonus kill from this fix: `execute_sandboxed` previously defaulted a missing command to `echo 'no command provided'` → silent success + sandbox commit; now a hard arg error before any side effect. Tests: 6 new cases in `server.rs::tests`.

### ✅ F1.4 Workspace pinning + response transparency `[L7]`
- **Root cause:** `McpServer::new(workspace_root)` receives whatever the CLI passes — process CWD (test hardcodes `PathBuf::from(".")`). IDE-launched servers resolve rules/telemetry from arbitrary CWDs and silently fall back to preset boundaries.
- **Fix approach:** honor `DAGR_WORKSPACE` env var and explicit `--workspace` flag on `mcp start` (precedence: flag > env > cwd); embed `{ "workspace": ..., "rules_source": "file"|"preset", "active_rules": n }` into every guard-type response; print a stderr banner when preset fallback triggers because no rules.yaml exists at the resolved root.
- **Acceptance criteria:** launching outside a repo yields visible fallback warning; response payloads identify their config provenance.
- **Status:** ✅ 2026-08-22 — `dagr mcp start --workspace <PATH>` (`-w`) flag added; `$DAGR_WORKSPACE` override via `resolve_workspace()` (precedence: flag > env > CWD; 3 unit tests in `crates/dagr-cli/tests/workspace_resolution.rs`); stderr preset-fallback banner at server launch; `verify_architecture` responses embed `workspace` / `rules_source` / `active_rules` (2 MCP tests incl. planted-violation catch).

---

## 🟡 Phase F2 — Guard Matching Intelligence (P1)

### ✅ F2.1 Segment-aware glob semantics (kill sibling-prefix false positives) `[N1]`
- **Root cause:** checker.rs:35 — `imported_module.starts_with(forbidden.trim_end_matches("/**"))`: pattern `src/db/**` trims to `src/db`, which prefix-matches `src/db-migration/client`. Documented in the field session as a known risk; confirmed real.
- **Fix approach:** drop the raw `starts_with` shortcut; rely solely on `glob::Pattern` matching with patterns normalized to include a trailing segment boundary, or implement segment-split matching (`pattern segments == candidate segments up to wildcard`).
- **Acceptance criteria:** `src/db/**` does NOT match `src/db-migration/x` but DOES match `src/db/x` and `src/db/sub/x`; regression test added.
- **Status:** ✅ 2026-08-22 — `module_under_prefix()` segment-boundary matcher replaces the raw `starts_with`; bare prefixes (`src/db`) still catch exact matches and `src/db/client`; glob matching untouched as primary path. Tests in `checker.rs::tests`.

### ✅ F2.2 Import resolution layer (relative → canonical workspace paths) `[L2]`
- **Root cause:** matching compares the literal string inside the import statement against workspace-relative glob patterns. Relative imports (`../db/client`) can never match absolute patterns (`packages/core/src/db/**`) — the exact failure observed in the field.
- **Fix approach:** introduce a resolution step between extraction and matching: given importing-file workspace path + relative import specifier, compute the candidate workspace-relative target path (handling `./`, `../`, extension probing `.ts/.tsx/.js/index.*`) and match against THAT, keeping raw-specifier matching as a secondary pass.
- **Acceptance criteria:** field-session scenario (`text-filter.ts` importing `../db/client` vs pattern `packages/core/src/db/**`) now detected; unit tests per dialect.
- **Status:** ✅ 2026-08-22 — `resolve_relative_candidates()` performs pure-lexical resolution (`./` and `../` only, no filesystem access, hot-path safe); `check_import` matches the raw specifier OR any canonical candidate (resolved path + `/index` variant). Alias/bare specifiers untouched (F2.3/F2.4). MCP `verify_architecture` inherits automatically via `check_file_imports`. 3 new tests.

### ✅ F2.3 Alias resolution (tsconfig/jsconfig paths)
- **Approach:** parse `tsconfig.json`/`jsconfig.json` `compilerOptions.paths` + `baseUrl`; map `@/lib/x` → workspace-relative candidates pre-match. Cache parsed maps per workspace.
- **Acceptance criteria:** alias-import violation scenario caught in a fixture repo with standard Next.js/Vite aliases.
- **Status:** ✅ 2026-08-22 — new `dagr-guard/src/alias.rs`: `AliasMap::load()` reads root `tsconfig.json` → `jsconfig.json` fallback; JSONC-tolerant (comment + trailing-comma stripping), accepts string-or-list path values, wildcard (`@/*`) and exact (`@auth`) keys; `baseUrl` folded lexically and targets kept **workspace-relative** (absolute bases degrade to literal matching). Graceful empty-map degrade on malformed configs. Wired into `check_import` candidates. Zero new dependencies (serde_json already present). Known v1 limitation: root-level configs only — per-package monorepo tsconfigs need nearest-config lookup from source_file (deferred to Wave-1 data). 5 fixture tests.

### ✅ F2.4 Importer coverage expansion `[N3, H-R1, H-GO1]`
- **Root cause:** `extract_imported_module` (checker.rs:121) handles TS/JS via `"from "` substring and Python prefixes only. Misses: `require("x")`, dynamic `import("x")`, side-effect `import "x";`, Rust `use` statements (**possibly all Rust imports invisible — see H-R1**), Go import blocks (single-line works accidentally via the Python branch). The substring probe also fires inside comments/strings.
- **Fix approach:** dialect-specific extractors (regex set or reuse tree-sitter queries already available via dagr-slicer); strip comments before line-probing.
- **Acceptance criteria:** per-dialect fixtures (TS variants, Rust `use`, Go blocks, comment traps) all yield correct extracted modules; Wave 3 confirms H-R1 resolution.
- **Status:** ✅ 2026-08-22 — extractor rewritten as ordered dialect probes: comment-line guard (kills phantom imports), Rust `use` paths (`::`→`/`, brace-group + `as` handling — **H-R1 confirmed at code level and closed**), TS/JS side-effect imports, dynamic `import()` / `require()` calls, re-export forms, Go single-line AND block lines (bare + aliased). Crate-level Rust rules (preset `tokio`) now fire via segment matching on `tokio/...` candidates. Known heuristic edge: bare two-token `alias "path"` lines outside import blocks could false-hit (guarded against `=` and trailing `;`); tree-sitter extraction remains the long-term upgrade path. 5 new fixture tests.

### ✅ F2.5 Barrel/re-export following (one hop)
- **Approach:** when an import resolves to an `index.ts`/mod.rs barrel, follow its re-exports one hop to attribute violations to the underlying module.
- **Status:** ✅ 2026-08-22 — architecture decision: **lazy per-miss barrel cache** (no load-time scan, no FS on clean paths). `ArchitectureGuard` gains `workspace_root` + `Mutex` cache; `check_import` is two-phase — direct candidate match first, and only on a miss does it probe plausible barrel files (`<cand>.{ts,tsx,js,jsx}` + `/index.*`, or as-is when the candidate already names a file), parse their **re-exports only** (`export … from` — private imports never taint importers), resolve specs against the barrel's own directory (+ alias map), cache per candidate, and retry matching once. Original specifier preserved in violations. 2 fixture tests incl. negative-twin.

---

## 🟡 Phase F3 — Slicer Honesty & Contracts (P1)

### ✅ F3.1 Make `--depth` truthful `[L4, L9.4]`
- **Root cause (confirmed harder than reported):** `SlicerConfig.max_depth_hops` (crates/dagr-slicer/src/slicer.rs:21) is **never read** anywhere in `slice()`; contract hoisting (`contracts.rs::extract_hoisted_contracts`) walks only the *same file's* AST. Cross-file hops do not exist — the flag is not merely weak intra-file, it is fully inert.
- **Decision point:** (a) implement multi-hop collection (parse imported files up to `max_depth_hops`, hoist referenced contracts) — larger feature, or (b) short-term honesty: CLI warns "`--depth` has no effect for single-file slices; cross-file contracts land with F3.2" and/or hides the flag. Recommend (b) now, (a) via F3.2.
- **Acceptance criteria:** no user-visible knob that does nothing without explanation; JSON output notes contract scope honestly.
- **Status:** ✅ 2026-08-22 — option (b) shipped: `-d/--depth` is now `Option<usize>` with clap help "Reserved for cross-file contract hoisting (F3.2); currently a no-op"; explicit use emits a stderr warning naming F3.2; MCP `max_depth_hops` schema description updated identically. `SlicerConfig.max_depth_hops` retained for F3.2 consumption. CLI parse test updated to `Some(4)`. Verified: warning fires only when the flag is passed; baseline stderr chatter identical pre/post change.

### ✅ F3.2 Cross-file type-contract hoisting (v1: single hop, relative imports)
- **Approach:** resolve import map (reuses F2.2 resolver), fetch + parse referenced files (Blake3-cached), hoist contracts across hops bounded by `max_depth_hops` and token budget.
- **Acceptance criteria:** billing_service-style fixture with external interface shows populated `type_contracts`; compression stays ≥85%; latency budget respected (<5ms warm cache).
- **Status:** ✅ 2026-08-22 (v1) — `SlicerConfig.workspace_root` added; `slice()` now walks the source AST (tree-sitter-native, not line probes) for TS/JS `import`/`export-from` specifiers, resolves relative candidates against the importer directory (absoluteness-preserving), probes sibling files (+ext/index variants), parses each via the same AstParser pipeline, and merges identifier-matched contracts from the foreign file with dedupe against same-file hoists. Gated by `max_depth_hops > 0`; sentinel range fix required in the hoister call. F3.1 surfaces updated to truth: clap/MCP descriptions + a warn only when depth>1 (v1 = one effective hop). Known v1 scope: relative imports only (aliases/multi-hop tagged for Wave data); parse-cache deferred. Wired into CLI (CWD) and MCP (workspace_root). Integration test: crossfile_contracts.rs (hoist + depth-gate negative).

---

## ⚪ Phase F4 — Ecosystem & Polish (P2)

### ✅ F4.1 `dagr mcp install --client opencode` `[L10]`
- Merge-write `~/.config/opencode/opencode.json` mcp object (`{"mcp": {"dagr": {"type": "local", "command": ["dagr","mcp","start"]}}}`), merge-not-clobber, idempotent. Note: takes effect on next opencode start; docs must repeat the CWD caveat from F1.4.
- **Status:** ✅ 2026-08-22 — path arm added (`~/.config/opencode/opencode.json`); dedicated `inject_dagr_config_opencode()` writes OpenCode's own schema (top-level `mcp` object, array-form command, `type: local`) while preserving sibling servers and foreign keys; `install()` branches per client shape. E2E-probed with isolated HOME: written config matches L10's required shape exactly. 2 unit tests (schema+merge+idempotency, path-arm resolution).

### ✅ F4.2 Schema documentation + JSON-Schema emitter `[L9.2]`
- README schema section ships with F1.1 ✅. Remaining: `dagr schema rules` emitting JSON Schema for editor autocomplete/validation.
- **Status:** ✅ 2026-08-22 — `dagr schema rules` emits draft-2020-12 JSON Schema mirroring RuleConfig's strict contract (`additionalProperties:false` at every level, required keys, preset enum). Hand-maintained static builder — no schemars dependency; an anti-drift test pins key-sets, requireds and the preset enum against drift. E2E-probed valid via python json.tool.

### ✅ F4.3 Exit-code hygiene under SIGPIPE `[L9.5]`
- Guard exit codes masked when stdout piped through `head`. Handle SIGPIPE explicitly / detect closed pipe and preserve meaningful exit semantics.
- **Status:** ✅ 2026-08-22 — `restore_sigpipe_default()` in main.rs restores SIG_DFL via `libc` (already vendored in-tree via dagr-sandbox — Ladder rung 5; the new dagr-cli declaration was a conscious, gate-flagged addition). CLI writers using `println!` now die with conventional SIGPIPE (exit 141) instead of masking errors behind exit 1; a belt-and-braces `Io::BrokenPipe → exit 0` branch covers fallible writers (MCP stdio loop). Note: full EPIPE-race e2e needs a >64KB output stream — deferred to harness item (T0).

### ☐ F4.4 Walker ignore-list audit `[H-W1]`
- Current skip set (checker.rs:83–89): `.git, node_modules, target, .dagr, .next, dist`. Likely missing common heavy dirs: `build`, `out`, `.output`, `.turbo`, `.venv`, `venv`, `__pycache__`, `vendor`, coverage dirs. Confirm via Wave 1/4 timing runs, then extend list (+ respect `.gitignore` optionally behind a flag).
- **Status:** ☐

---

## 🗺️ Execution Order & Rationale

1. **F1.1** ✅ — cheapest, highest-severity; done first (fail-open posture eliminated).
2. **F1.2** — same file, same day-class effort, closes the third fail-open variant.
3. **F1.3 + F1.4** — MCP trustworthiness pair; unblocks reliable IDE-based testing waves.
4. **F2.1** — small diff, kills documented FP class before scale testing amplifies it.
5. **F2.2 → F2.4 → F2.3** — resolution layer first (F2.4 extractors feed it; aliases depend on canonical targets).
6. **F3.1(b)** — honesty patch immediately; **F3.2** after Wave 1–3 data.
7. **F4.x** interleaved as touched areas overlap.

## 📝 Change Log

- **2026-08-22:** Ledger created from L1–L10 field findings + N1–N3 source-audit findings; F1.1 implemented and verified (`deny_unknown_fields` on `RuleConfig` + `BoundaryRule`, enriched config errors, README schema section, regression tests).
- **2026-08-22 (cont.):** F1.2 shipped — `RuleConfig::validate_patterns()` rejects uncompilable globs at load with rule name + pattern in the error; runtime matching untouched. F1.3 shipped — all five lenient MCP handlers hardened to required-arg + type validation; unknown tool → `-32602`; `execute_sandboxed` echo-default side effect eliminated. Workspace: 24/24 suites green.
- **2026-08-22 (cont.):** F1.4 shipped — `mcp start --workspace/-w` + `$DAGR_WORKSPACE` override via `resolve_workspace()` (flag > env > CWD), stderr preset-fallback banner, rules-provenance fields (`workspace`/`rules_source`/`active_rules`) in guard responses. F2.1 shipped — `module_under_prefix()` segment-boundary matcher replaces raw `starts_with`, eliminating sibling-prefix false positives while preserving bare-prefix convenience. Workspace: 25/25 suites green, 90 tests passing.
- **2026-08-22 (cont.):** F2.2 shipped — relative import specifiers are lexically resolved to canonical workspace candidates before pattern matching, closing the L2 evasion (`../db/client` vs `packages/core/src/db/**`). Workspace: 25/25 suites, 93 tests passing.
- **2026-08-22 (cont.):** F2.4 shipped — importer rewritten as per-dialect probes: Rust `use` visibility (H-R1 confirmed & fixed), `require()` / dynamic `import()` / side-effect imports, Go block lines, comment-trap rejection (N3). Workspace: 25/25 suites, 98 tests passing.
- **2026-08-22 (cont.):** F2.3 shipped — tsconfig/jsconfig alias resolution (wildcard + exact keys, JSONC-tolerant, workspace-relative targets); alias evasions now caught when root path-mappings exist. Workspace: 25/25 suites, 103 tests passing.
- **2026-08-22 (cont.):** F3.1 shipped — `--depth` honesty patch (`Option<usize>`, stderr no-op warning naming F3.2, honest clap/MCP descriptions). F2.5 explicitly deferred pending the barrel-index architecture decision. Workspace: 25/25 suites, 103 tests passing.
- **2026-08-22 (cont.):** F2.5 shipped — one-hop barrel following via lazy cached re-export index (re-exports only, never private imports); `ArchitectureGuard` gains `workspace_root` + cache, expansion gated behind direct-miss so clean scans stay IO-free. Phase F2 (guard intelligence) complete. Workspace: 25/25 suites, 105 tests passing.
- **2026-08-22 (cont.):** F4.1 + F4.3 shipped — opencode MCP installer (own schema, merge-preserving, e2e-probed) and SIGPIPE exit-code hygiene (`libc` SIG_DFL + BrokenPipe branch; libc reused from in-tree dagr-sandbox, gate-flagged consciously). Verification extended beyond fixtures: repo E2E harness 10/10, clippy/fmt clean on all touched code. Workspace: 25/25 suites, 107 tests passing.
- **2026-08-22 (cont.):** F4.2 shipped — `dagr schema rules` emits draft-2020-12 JSON Schema for `.dagr/rules.yaml` (strict-contract mirror, anti-drift-tested). L9.2 fully closed: schema documented AND machine-readable. Workspace: 25/25 suites, 110 tests passing.
- **2026-08-22 (cont.):** F3.2 shipped (v1, single hop, relative imports) — cross-file type-contract hoisting via tree-sitter-native import extraction; `--depth` now truthful (one effective hop). Phase F3 complete except multi-hop/alias extensions pending Wave data. Workspace: 26/26 suites, 111 tests passing; E2E harness 10/10; live CLI probe hoists PaymentPayload across files.
- **2026-08-23:** HYPERPLAN adversarial scrutiny executed inline (5 hostile personas, cross-critique, full transcript persisted). Phase T-EDGE P0 batteries shipped — sandbox-escape (documented CoW-scope limitation P1-a), traversal-read proof, seeded fuzz that **hardened two real extractor quote-leaks**, infra-failure battery; walker ignore-list front-loaded (EC-F2); net-of-contracts metric definition (EC-F5); tools/list allowlist test (EC-P7); field findings copied in-repo (EC-P6); T0 harness skeleton with self-smoke PASS. Prior-session WIP preserved via `agent-os-wip` branch and merged to main — main restored to a self-contained build (`cargo install` verified end-to-end, catching a broken-packaging state). Workspace: 27/27 suites, 119 tests; E2E harness 10/10.
- **2026-08-23 (cont.):** Phase T-EDGE P1 batteries shipped — determinism goldens (E6), migration idempotency matrix (E7), concurrency pair-test (E8), hostile filesystem matrix (E9), hostile-YAML battery (E11). CI quality workflow added (fmt + clippy -D warnings + tests + T0 smoke + E2E). Post-merge clippy debt cleared to zero. Wave 1 opener on vitejs/vite + vercel/next.js (findings in docs/findings/). F3.2 v1.1 alias-aware hoisting shipped. Workspace: 29/29 suites, 130 tests; E2E harness 10/10; T0 smoke PASS on self AND vite AND next.js.
