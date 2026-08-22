# 🔧 DAGR Improvement Roadmap — Findings → Fixes Ledger

> **Living Fix Specification**
> **Companion:** [`TESTING_ROADMAP.md`](./TESTING_ROADMAP.md) (discovery program feeding this ledger)
> **Findings origin:** `/Automate-Instagram-Posts/LearningsDAGR.md` (L-series, 2026-08-22) + source audit (N-series, 2026-08-22)
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

### ☐ F2.2 Import resolution layer (relative → canonical workspace paths) `[L2]`
- **Root cause:** matching compares the literal string inside the import statement against workspace-relative glob patterns. Relative imports (`../db/client`) can never match absolute patterns (`packages/core/src/db/**`) — the exact failure observed in the field.
- **Fix approach:** introduce a resolution step between extraction and matching: given importing-file workspace path + relative import specifier, compute the candidate workspace-relative target path (handling `./`, `../`, extension probing `.ts/.tsx/.js/index.*`) and match against THAT, keeping raw-specifier matching as a secondary pass.
- **Acceptance criteria:** field-session scenario (`text-filter.ts` importing `../db/client` vs pattern `packages/core/src/db/**`) now detected; unit tests per dialect.
- **Status:** ☐

### ☐ F2.3 Alias resolution (tsconfig/jsconfig paths)
- **Approach:** parse `tsconfig.json`/`jsconfig.json` `compilerOptions.paths` + `baseUrl`; map `@/lib/x` → workspace-relative candidates pre-match. Cache parsed maps per workspace.
- **Acceptance criteria:** alias-import violation scenario caught in a fixture repo with standard Next.js/Vite aliases.
- **Status:** ☐

### ☐ F2.4 Importer coverage expansion `[N3, H-R1, H-GO1]`
- **Root cause:** `extract_imported_module` (checker.rs:121) handles TS/JS via `"from "` substring and Python prefixes only. Misses: `require("x")`, dynamic `import("x")`, side-effect `import "x";`, Rust `use` statements (**possibly all Rust imports invisible — see H-R1**), Go import blocks (single-line works accidentally via the Python branch). The substring probe also fires inside comments/strings.
- **Fix approach:** dialect-specific extractors (regex set or reuse tree-sitter queries already available via dagr-slicer); strip comments before line-probing.
- **Acceptance criteria:** per-dialect fixtures (TS variants, Rust `use`, Go blocks, comment traps) all yield correct extracted modules; Wave 3 confirms H-R1 resolution.
- **Status:** ☐

### ☐ F2.5 Barrel/re-export following (one hop)
- **Approach:** when an import resolves to an `index.ts`/mod.rs barrel, follow its re-exports one hop to attribute violations to the underlying module.
- **Status:** ☐ (deferred until F2.2/F2.4 land)

---

## 🟡 Phase F3 — Slicer Honesty & Contracts (P1)

### ☐ F3.1 Make `--depth` truthful `[L4, L9.4]`
- **Root cause (confirmed harder than reported):** `SlicerConfig.max_depth_hops` (crates/dagr-slicer/src/slicer.rs:21) is **never read** anywhere in `slice()`; contract hoisting (`contracts.rs::extract_hoisted_contracts`) walks only the *same file's* AST. Cross-file hops do not exist — the flag is not merely weak intra-file, it is fully inert.
- **Decision point:** (a) implement multi-hop collection (parse imported files up to `max_depth_hops`, hoist referenced contracts) — larger feature, or (b) short-term honesty: CLI warns "`--depth` has no effect for single-file slices; cross-file contracts land with F3.2" and/or hides the flag. Recommend (b) now, (a) via F3.2.
- **Acceptance criteria:** no user-visible knob that does nothing without explanation; JSON output notes contract scope honestly.
- **Status:** ☐

### ☐ F3.2 Cross-file type-contract hoisting (depends on Testing Waves 1–3 data)
- **Approach:** resolve import map (reuses F2.2 resolver), fetch + parse referenced files (Blake3-cached), hoist contracts across hops bounded by `max_depth_hops` and token budget.
- **Acceptance criteria:** billing_service-style fixture with external interface shows populated `type_contracts`; compression stays ≥85%; latency budget respected (<5ms warm cache).
- **Status:** ☐

---

## ⚪ Phase F4 — Ecosystem & Polish (P2)

### ☐ F4.1 `dagr mcp install --client opencode` `[L10]`
- Merge-write `~/.config/opencode/opencode.json` mcp object (`{"mcp": {"dagr": {"type": "local", "command": ["dagr","mcp","start"]}}}`), merge-not-clobber, idempotent. Note: takes effect on next opencode start; docs must repeat the CWD caveat from F1.4.
- **Status:** ☐

### ◐ F4.2 Schema documentation + JSON-Schema emitter `[L9.2]`
- README schema section ships with F1.1 ✅. Remaining: `dagr schema rules` emitting JSON Schema for editor autocomplete/validation.
- **Status:** ◐ (docs done; emitter pending)

### ☐ F4.3 Exit-code hygiene under SIGPIPE `[L9.5]`
- Guard exit codes masked when stdout piped through `head`. Handle SIGPIPE explicitly / detect closed pipe and preserve meaningful exit semantics.
- **Status:** ☐

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
