# 🧪 DAGR Field-Testing Roadmap — Multi-Repository Validation Program

> **Living Test Specification & Discovery Ledger**
> **Companions:** [`IMPROVEMENT_ROADMAP.md`](./IMPROVEMENT_ROADMAP.md) (findings → fixes) · [`RESEARCH_ROADMAP.md`](./RESEARCH_ROADMAP.md) (research features)
> **Origin of findings:** `/Automate-Instagram-Posts/LearningsDAGR.md` (session 2026-08-22, dagr v0.1.0)
> **Last synchronized:** August 2026

---

## 📌 Mission

DAGR v0.1.0 has been validated end-to-end on exactly **one** external codebase (Automate-Instagram-Posts — TS monorepo). That single-repo pass surfaced **10 findings (L1–L10)** including an S0 fail-open guard trap, and the follow-up source audit surfaced **more issues (N-series)**. This program stress-tests DAGR across large, heterogeneous repositories to:

1. **Discover unknown shortcomings** before users do.
2. **Quantify real-world metrics** (compression, latency, detection rates).
3. **Feed a continuous finding→fix pipeline** into `IMPROVEMENT_ROADMAP.md`.

---

## 🧭 Guiding Principles (lessons already paid for)

| # | Principle | Origin |
|---|---|---|
| P1 | Never accept a clean-case pass alone. Always plant a violation and prove the catch. | L1/L6 fail-open traps |
| P2 | Assert on `--format json` internals (`active_rules`, `type_contracts`, `violations_count`), never on pretty-print prose. | L5 |
| P3 | Pin the workspace explicitly in every harness run (launch CWD == target repo); record CWD in results. | L7 |
| P4 | Revert planted violations immediately (`git checkout -- <file>`); assert clean tree after each probe. | L5 |
| P5 | Every finding becomes a numbered ledger entry within the session it was found, linked to an F-ticket. | process |

---

## 🚦 Severity & Finding Taxonomy

**Severity:** `S0` fail-open / security · `S1` incorrect results (FN/FP) · `S2` DX / ergonomics / coverage · `S3` cosmetic.

**ID convention:** `L<n>` = finding from LearningsDAGR.md field session · `N<n>` = finding from source audit · `H<n>` = hypothesis awaiting runtime confirmation · `W<n>.<m>` = wave task.

---

## 📋 Current Findings Registry

| ID | Summary | Severity | Fix ticket | Status |
|---|---|---|---|---|
| L1 | Malformed rules.yaml parses as **0 rules** (silent guard pass); missing file = *more* enforcement than malformed | S0 | F1.1 | fix shipped 2026-08-22 |
| L2 | Guard matches raw import strings, not resolved paths → relative imports invisible to absolute patterns | S1 | F2.2 | open |
| L4/L9.4 | `--depth` flag inert; `type_contracts` empty for TS intra-file slices | S2 | F3.1 | open |
| L6 | MCP tools silently default wrong/missing args to empty → `valid: true` | S0 | F1.3 | open |
| L7 | MCP server resolves rules/telemetry from launch CWD; IDEs get silent preset fallback | S1 | F1.4 | open |
| L9.2 | rules.yaml schema undocumented in README | S2 | F4.2 | partial (docs shipped with F1.1) |
| L9.5 | Guard exit code masked when piped through `head` | S3 | F4.3 | open |
| L10 | dagr MCP not registered in opencode by installer | S2 | F4.1 | open |
| N1 | Sibling-prefix false positives: `starts_with(trim_end_matches("/**"))` makes `src/db/**` match `src/db-migration/x` | S1 | F2.1 | open |
| N2 | Invalid glob patterns are silently swallowed (`if let Ok(Pattern::new(..))`) → dead rule, zero signal | S0 | F1.2 | open |
| N3 | Importer gaps: `require()`, dynamic `import()`, side-effect imports, Go import blocks, Rust `use` all missed; `"from "` substring can hit comments | S2 | F2.4 | open |

---

## 🔬 Standard Per-Repo Protocol (v1)

Every target repository runs this fixed battery. Results append to `LearningsDAGR-<repo>.md` (or a dated section) with severity-tagged IDs.

1. **Env snapshot:** dagr version, OS + filesystem (CoW capability), repo HEAD, LOC count, package manager, launch CWD (P3).
2. **Init & baseline:** `dagr init` → `dagr status`; confirm rules source is `file` vs `preset` via JSON; expect guard PASS on untouched tree.
3. **Slice fidelity battery:** pick 5 symbols — tiny fn · class w/ cross-file types · async handler · macro/metaprogramming-heavy · re-exported barrel API. Record tokens before/after, compression %, latency, whether `type_contracts` populated.
4. **Guard detection battery (P1):** for each boundary rule: plant forbidden import → expect FAIL naming rule+advice → revert → expect PASS. Repeat across dialects: relative, alias, dynamic `import()`, `require()`, barrel re-export.
5. **Sandbox drill:** failing command → rollback asserted (`git status` clean); passing command → isolated; `-c` commit path applied.
6. **MCP parity:** initialize → tools/list (expect ≥7) → repeat slice+verify calls over stdio JSON-RPC; compare vs CLI output where deterministic.
7. **Telemetry delta:** `dagr stats` before/after; `tokens_saved` increments must be sane and monotonic.
8. **Ledger write-up:** new findings get IDs (next free number) + linked F-tickets same session.

---

## 🌍 Test Matrix Dimensions

| Dimension | Variants to cover |
|---|---|
| Language | TypeScript/JS, Python, Rust, Go, Java/Kotlin, polyglot mixes |
| Import dialect | relative, tsconfig aliases (`@/*`), absolute package specifiers, re-export barrels, dynamic `import()`, `require()`, namespace imports, Rust `use` paths, Go import blocks |
| Build system | pnpm/yarn/nx turborepo, cargo workspace, go modules, maven/gradle |
| Repo shape | monorepo vs single-package, generated dirs (`dist`, `.next`, `target`, `__pycache__`, vendor trees) |
| Filesystem | APFS clonefile (CoW), ext4/XFS reflink, non-CoW fallback path, network volumes |
| Concurrency | parallel agent branches, port collision injection, stale-lock harvesting |

---

## 🎯 Target Repository Waves

### Wave 1 — TS/JS Monorepos (closest to validated profile)
| Repo | Size class | Primary stressors |
|---|---|---|
| vercel/next.js | ~1M LOC | tsconfig `paths` aliases everywhere; packages/* internal imports; examples tree noise |
| pnpm/pnpm | large | workspace-native resolution; generated code dirs |
| vitejs/vite | medium-large | TS+Rust mix (rolldown); barrel re-exports |

**Hypotheses to confirm:**
- **H-TS1:** alias imports (`@/lib/x`) produce guard blind spots identical to L2 (no alias resolver exists yet).
- **H-W1:** walker ignore-list (`.git/node_modules/target/.dagr/.next/dist`) misses `build`, `out`, `.output`, `.turbo`, storybook static builds → scan time blowup / FP noise.

### Wave 2 — Giants & Polyglot
| Repo | Stressors |
|---|---|
| microsoft/vscode | 1M+ LOC TS; AMD-style + relative mixed; huge generated typings |
| denoland/deno | Rust + TS polyglot; JSR-style specifiers |
| supabase/supabase | mixed-language monorepo; deep nesting |

**Hypotheses:**
- **H-N3a:** dynamic `import()` / `require()` imports are entirely invisible to `extract_imported_module` → planted violations escape detection.
- **H-N3b:** comment lines containing `from '` create phantom imports (substring matcher).

### Wave 3 — Rust Dogfooding (DAGR's home turf)
| Repo | Stressors |
|---|---|
| rust-analyzer/rust-analyzer | large cargo workspace; macros; trait-heavy generics |
| tokio-rs/tokio | workspace crates; cfg-gated modules |
| mjzd7/dagr (**self-host**) | eat own dog food on our own binary |

**Hypotheses:**
- **H-R1 (high priority):** Rust `use crate::foo::Bar;` statements contain no `"from "` and no Python prefix → **Rust imports may be entirely invisible to `scan_workspace` today**. Derived from source read of `extract_imported_module` (checker.rs); needs runtime confirmation. If true → new S1 finding + F-ticket (extends F2.4).
- **H-R2:** trait/struct contract hoisting behaves differently for generic bounds; measure `type_contracts` hit-rate.

### Wave 4 — Other Ecosystems (breadth proof)
| Repo | Language | Expected outcome |
|---|---|---|
| kubernetes/kubernetes (or golang/go stdlib subset) | Go | H-GO1: single-line `import "x"` accidentally works via the Python branch; **import blocks (`import ( ... )`) fully missed** |
| spring-projects/spring-boot | Java | Java `import com.x.y;` accidentally matches the Python branch — verify semicolon trimming holds |
| python/cpython or pydantic | C + Python | mixed-language parsing; `from . import x` relative forms |

### Wave 0 — Adversarial / Environment Matrix (runs continuously)
- Non-CoW filesystem fallback (tmpfs scratchpad) correctness.
- Parallel branch forks with injected port collisions.
- Stale lock deadlock harvesting (RAII 5s window).
- Symlink + encoded-path traversal against guard/sandbox (Edge-Case Defense Matrix §3).

---

## 🛡️ Regression Gate (must stay green forever)

Post-fix automated checks promoted into `scripts/comprehensive_test_suite.py`:

- [x] Legacy wrong-shape rules.yaml (top-level `rules:` + nested `source:/disallow:/reason:`) → hard parse error naming offending keys *(F1.1, shipped)*
- [ ] Valid minimal schema parses; nested unknown key rejected independently *(F1.1 tests)*
- [x] MCP verify_architecture rejects unknown/missing required args with `-32602` / precise `isError` content; unknown tool → `-32602` *(F1.3)*
- [x] Server honors `DAGR_WORKSPACE` env override; responses embed resolved root + rules source *(F1.4)*
- [x] `src/db/**` does not match `src/db-migration/x` *(F2.1)*
- [ ] Relative import `../db/client` caught by canonical pattern after resolution layer *(F2.2)*
- [ ] `--depth` either functional or emits explicit unsupported warning *(F3.1)*
- [ ] `dagr mcp install --client opencode` idempotent merge *(F4.1)*

---

## 📊 Metrics Targets (per repo, recorded every run)

| Metric | Target |
|---|---|
| Token compression (median slice) | ≥ 90% |
| Slice latency P99 | < 5 ms |
| Planted-violation detection rate | 100% (zero FN tolerated) |
| Clean-tree false-positive rate | < 1% |
| CoW rollback latency | < 10 ms |
| Contract-hoist hit-rate | reported per repo (baseline for F3.2) |

---

## 🧰 Harness Deliverables

- [ ] `scripts/fieldtest/run_protocol.sh|py` — parameterized protocol runner (repo path, phase selection, JSON results file).
- [ ] JSON results schema: `{ repo, head, dagr_version, cwd, phases: { baseline, slices[], guard[], sandbox, mcp }, metrics{}, findings[] }`.
- [ ] CI-able smoke mode (protocol steps 2+7 only) for nightly runs against pinned repos.
- [ ] Findings template: severity, repro command, JSON evidence snippet, proposed F-ticket.

## 📋 Phased Execution Ledger

- [ ] **T0:** Codify protocol as harness script; migrate existing probes from `comprehensive_test_suite.py`.
- [ ] **T1 (Wave 1):** next.js + pnpm + vite runs; confirm H-TS1, H-W1; file F-tickets for confirmed items.
- [ ] **T2 (Wave 3):** rust-analyzer/tokio/self-host; confirm/refute H-R1 immediately (highest information gain).
- [ ] **T3 (Wave 2):** vscode/deno/supabase scale runs; capture perf metrics at 1M LOC.
- [ ] **T4 (Wave 4):** Go/Java/Python breadth; document accidental-support matrix for extractor.
- [ ] **T5 (Wave 0):** adversarial matrix automation.
- [ ] **T6:** promote all regression-gate checks into CI once fixes land.

---

*Any agent working in this repository must treat newly discovered field findings as first-class ledger entries: add the row here, open the F-ticket in IMPROVEMENT_ROADMAP.md, then fix.*
