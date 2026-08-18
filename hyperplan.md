# ⚡ DAGR: Master Deterministic Architectural Specification & Implementation Blueprint

> **System Name:** **DAGR** (`dagr`)  
> **Creator & Lead Architect:** Mohit Dagar  
> **Repository:** [`github.com/mjzd7/dagr`](https://github.com/mjzd7/dagr)  
> **Governance:** [Ponytail Minimal Architecture (Full)](./.ponytail.md)  
> **Document Status:** All 7 Architectural Decisions Finalized & Locked 🔒  
> **Last Audited:** 2026-08-18  

---

## 📑 Table of Contents
1. [Executive Summary & Core Philosophy](#executive-summary--core-philosophy)
2. [Nomenclature, Etymology & Brand Alignment](#nomenclature-etymology--brand-alignment)
3. [The 7 Finalized Deterministic Architectural Decisions](#the-7-finalized-deterministic-architectural-decisions)
   - *Decision 1: Static Native Tree-sitter Ingestion & Cross-Compilation*
   - *Decision 2: Backwards Data-Flow Slicing & Contract Hoisting*
   - *Decision 3: Cross-Platform Copy-on-Write (CoW) Shadow Sandbox*
   - *Decision 4: Embedded SQLite with WAL Mode & Blake3 Cache*
   - *Decision 5: Stdio/SSE Model Context Protocol (MCP) Server*
   - *Decision 6: 3-Pillar Architectural Guardrails (`.dagr/rules.yaml`)*
   - *Decision 7: Visually Stunning CLI & TTY Auto-Detection*
4. [Terminal Visual Design System & Aesthetics](#terminal-visual-design-system--aesthetics)
5. [Exhaustive 16-Point Edge-Case Register](#exhaustive-16-point-edge-case-register)
6. [Phased Product Roadmap: Community First ➔ Enterprise Next](#phased-product-roadmap)
7. [Step-by-Step Implementation Guide for Autonomous Agents](#step-by-step-implementation-guide-for-autonomous-agents)

---

## 🧭 Executive Summary & Core Philosophy

**DAGR** is an ultra-fast, local-first safety hypervisor and symbolic AST slicing engine built in native Rust. Its primary mission is to eliminate:
1. **Context Explosion (95% Token Bloat):** AI coding tools choking on noisy vector dumps and massive 1,000+ line files, inflating token bills and degrading model reasoning.
2. **Architectural Drift & Unbounded Blast Radius:** Autonomous AI agents generating duplicate utilities, violating clean layer boundaries, and executing destructive file mutations.

DAGR operates on a **Local-First Asymmetric Scaling Model**:
* 99.9% of all hot execution (AST slicing, CoW shadow sandboxing, pre-commit linting) runs locally in a single compiled Rust binary (`dagr`) with zero server dependencies and sub-5ms latency.

---

## ⚡ Nomenclature, Etymology & Brand Alignment

```
                      ┌────────────────────────────────────────────────────────┐
                      │                     MOHIT DAGAR                        │
                      └──────────────────────────┬─────────────────────────────┘
                                                 │
                  ┌──────────────────────────────┼──────────────────────────────┐
                  ▼                              ▼                              ▼
    ┌───────────────────────────┐  ┌───────────────────────────┐  ┌───────────────────────────┐
    │     1. CS FOUNDATION      │  │     2. RUST MINIMALISM    │  │    3. ILLUMINATION LORE   │
    │  Directed Acyclic Graph   │  │   4-Letter Binary: `dagr` │  │ Norse "Dagr" (Day/Light)  │
    │        [ D-A-G ]          │  │       [ DAG + Rust ]      │  │ Illumination of Context   │
    └─────────────┬─────────────┘  └─────────────┬─────────────┘  └─────────────┬─────────────┘
                  │                              │                              │
                  └──────────────────────────────┼──────────────────────────────┘
                                                 ▼
                                     ┌───────────────────────┐
                                     │        D A G R        │
                                     │     (CLI: `dagr`)     │
                                     └───────────────────────┘
```

---

## 🔒 The 7 Finalized Deterministic Architectural Decisions

### 🏛️ Decision 1: Static Native Tree-sitter Ingestion & Cross-Compilation
* **Decision:** Bake official C/C++ Tree-sitter grammars (`tree-sitter-typescript`, `tree-sitter-python`, `tree-sitter-go`, `tree-sitter-rust`, `tree-sitter-javascript`) directly into the Rust binary.
* **Drawback Mitigations:**
  1. *Build toolchain:* Distribute pre-compiled native binaries via Homebrew and GitHub releases; end-users never compile from source.
  2. *CI Cross-compilation:* Use `cargo-zigbuild` in GitHub Actions for single-runner multi-target builds.
  3. *Custom DSLs:* Fall back to Lexical Indentation Bounding + Regex symbol slicing on unknown file extensions.
  4. *Thread Safety:* Use `thread_local!` parser pools for multithreaded daemon execution.

---

### 🏛️ Decision 2: Backwards Data-Flow Slicing & Contract Hoisting
* **Decision:** Extract the exact target function body, traverse its internal data-flow backwards, and hoist **only the type signatures and interface contracts** of external dependencies, pruning 95% of implementation bodies.
* **Token Reduction Metric:** Reduces a 1,200-line file (~12,000 tokens) to ~35 lines (<400 tokens) with audited `tiktoken-rs` metrics.

---

### 🏛️ Decision 3: Cross-Platform Copy-on-Write (CoW) Shadow Sandbox Engine
* **Decision:** OS-native block-level cloning:
  * **macOS (Darwin):** `clonefile(2)` (APFS block-level copy, sub-1ms).
  * **Linux:** `ioctl(FICLONE)` reflink (Btrfs/XFS) or hardlink shadow staging.
  * **Windows:** ReFS block clone or hardlink shadow tree.
* **Safety SLA:** Executes tool mutations and test suites inside `.dagr/shadow/<tx_id>`; rolls back 100% of mutations in **<10ms** if tests fail.

---

### 🏛️ Decision 4: Embedded SQLite with WAL Mode & Blake3 Cache
* **Decision:** Embed pure SQLite at `.dagr/index.db` with Write-Ahead Logging (`PRAGMA journal_mode = WAL;`) and Blake3 32-byte content hash indexing.
* **Speed:** If `blake3(file_content)` matches cached state, skip AST parsing entirely (**<0.05ms lookup**).

---

### 🏛️ Decision 5: Stdio/SSE Model Context Protocol (MCP) Server
* **Decision:** Standard JSON-RPC 2.0 protocol over Stdio and SSE with **global stdout isolation** (`tracing-subscriber` routes 100% of logs to `stderr`).
* **3 Core MCP Tools:**
  1. `dagr_get_context_slice` (Prunes code for LLMs).
  2. `dagr_verify_architecture` (In-memory layer boundary check).
  3. `dagr_execute_sandboxed` (Safe tool execution with auto-rollback).

---

### 🏛️ Decision 6: 3-Pillar Architectural Guardrails (`.dagr/rules.yaml`)
* **Decision:** 
  1. **Auto-Inference:** `dagr init` auto-detects framework structures and generates rules.
  2. **Presets:** 1-line standard presets (`clean-architecture`, `nextjs`, `fastapi`).
  3. **Custom Rules & Invariant Learning:** Declarative `.dagr/rules.yaml` glob matching + `dagr rules suggest`.
  4. **Security:** Built-in zero-trust sanitizer stripping indirect prompt injection tokens.

---

### 🏛️ Decision 7: Visually Stunning CLI & TTY Auto-Detection
* **Decision:** Clap v4 subcommand architecture (`dagr context`, `dagr guard`, `dagr run`, `dagr mcp`, `dagr init`).
* **TTY Awareness:** Colored unicode visual scoreboards when connected to terminal; pure raw minimal code when piped (`| pbcopy`, `| ollama`).

---

## 🎨 Terminal Visual Design System & Aesthetics

When running interactively, DAGR presents a modern, polished visual interface:

### 1. `dagr context` Visual Output:
```
⚡ DAGR Symbolic AST Slicer v0.1.0
┌────────────────────────────────────────────────────────────────────────┐
│ Target:    src/billing/charge.ts:processPayment                         │
│ Sliced:    34 lines (down from 1,180 lines in file)                    │
│ Tokens:    342 tokens (down from 11,840 tokens)                        │
│ Savings:   🟢 97.1% Token Compression                                  │
│ Latency:   ⚡ 1.8ms (Blake3 Cache HIT)                                  │
└────────────────────────────────────────────────────────────────────────┘

// --- Hoisted Type Contracts (2) ---
interface PaymentPayload { userId: string; amountCents: number; currency: string; }
type PaymentResult = { success: boolean; transactionId: string; };

// --- Extracted Symbolic Implementation (src/billing/charge.ts:L45-L78) ---
45: export async function processPayment(payload: PaymentPayload): Promise<PaymentResult> {
46:     const validated = validatePaymentPayload(payload);
47:     const taxAmount = calculateTax(validated.amountCents);
48:     return await stripeClient.charges.create({ ...validated, tax: taxAmount });
78: }
```

### 2. `dagr guard` Visual Output:
```
🛡️ DAGR Architecture Guard (<24ms)
Checking 48 staged files against .dagr/rules.yaml...

❌ Violation Detected [UI-to-DB Isolation]:
   ├─ File:    src/components/CheckoutButton.tsx (Line 4)
   ├─ Import:  import { db } from "@/db/client";
   ├─ Rule:    UI components must not import database clients directly.
   └─ Fix:     Route database requests through @/services/billing.

1 architectural violation found. Working tree protected.
```

---

## ⚡ Exhaustive 16-Point Edge-Case Register

| ID | Edge Case | Severity | Concrete Mitigation |
| :--- | :--- | :---: | :--- |
| **E01** | Mid-edit Syntax Errors (missing brackets) | 🔴 High | Tree-sitter error recovery + Lexical Indentation Fallback (`syntax_degraded: true`). |
| **E02** | Circular AST Dependencies (`A -> B -> A`) | 🔴 High | `HashSet<u64>` visited register + iterative BFS queue + 3-hop depth limit. |
| **E03** | Indirect Prompt Injection in Comments | 🔴 High | Zero-trust regex sanitizer stripping `<|im_start|>`, `SYSTEM:`, `[INST]` delimiters. |
| **E04** | Path Traversal (`../../etc/passwd`) | 🔴 High | Path canonicalization (`std::fs::canonicalize`) enforcing workspace root prefix. |
| **E05** | Process Crash during Shadow Mutation | 🟡 Med | Write-ahead journal (`.dagr/journal.db`) auto-rolling back orphan locks on boot. |
| **E06** | Cross-Platform CoW Compatibility | 🔴 High | Dynamic selector: `clonefile` on Darwin, `reflink`/hardlinks on Linux/Windows. |
| **E07** | Dynamic Metaprogramming (`eval`, `getattr`) | 🟡 Med | Capture lexical enclosing boundary box + attach `unresolved_dynamic_call` warning. |
| **E08** | Massive Monorepos (10M+ LOC) | 🟡 Med | Lazy on-demand AST parsing + LRU mmap cache + git-diff scoped indexing. |
| **E09** | Git Pre-commit Latency Breaches (>100ms) | 🟡 Med | Blake3 hash caching skipping unchanged files (<35ms total hook run). |
| **E10** | MCP Stdio Stream Corruption | 🔴 High | Strict `tracing-subscriber` routing all logs to `stderr`; `stdout` reserved for JSON-RPC. |
| **E11** | BPE Token Count Distortion in Code | 🟢 Low | Native `tiktoken-rs` (`cl100k_base` / `o200k_base`) for exact audited metrics. |
| **E12** | Cross-Language Boundaries (TS -> Rust API) | 🟡 Med | OpenAPI/Protobuf contract registry mapping HTTP routes to server handler ASTs. |
| **E13** | Concurrent MCP Tool Invocations | 🟡 Med | UUID-keyed shadow namespaces (`.dagr/shadow/<tx_id>`) + atomic directory rename. |
| **E14** | Binary & Non-UTF8 Files (.png, .wasm) | 🟢 Low | MIME sniffing + UTF-8 validation gate rejecting non-text files instantly. |
| **E15** | Layer Boundary Violations | 🔴 High | In-memory AST import linter (`dagr guard`) evaluating `.dagr/rules.yaml`. |
| **E16** | Downstream LLM Outages & Drops | 🟡 Med | Circuit Breaker state machine (Closed -> Open -> Half-Open) with exponential backoff. |

---

## 📦 Phased Product Roadmap

```
                                 DAGR PRODUCT EVOLUTION
                                 
    ┌──────────────────────────────────────────────────┐      ┌──────────────────────────────────────────────────┐
    │     PHASE 1: DAGR COMMUNITY (INDIVIDUAL DEVS)    │      │     PHASE 2: DAGR ENTERPRISE (TEAMS & CLOUD)     │
    │            "The 10x Local AI Hypervisor"         │      │      "The Continuous AI Verification Cloud"      │
    ├──────────────────────────────────────────────────┤      ├──────────────────────────────────────────────────┤
    │ • 100% Standalone native binary (zero cloud deps)│      │ • Centralized 3D Knowledge Graph (Memgraph)      │
    │ • Sub-5ms in-IDE MCP Gateway (Cursor/Claude)     │ ───► │ • Multi-repo cross-service AST dependency graph  │
    │ • Tree-sitter symbolic backwards AST slicing     │      │ • CI/CD PR Verification with MicroVM Chaos tests │
    │ • Local Copy-on-Write (CoW) shadow sandbox       │      │ • Organization-wide SOLID boundary compliance    │
    │ • Local Git pre-commit guard (<35ms runtime)     │      │ • Team FinOps dashboard & token savings analytics│
    │ • Local embedded SQLite + Blake3 hash cache      │      │ • Webhook gateway with Redpanda/Kafka event bus  │
    └──────────────────────────────────────────────────┘      └──────────────────────────────────────────────────┘
```

---

## 🛠️ Step-by-Step Implementation Guide for Autonomous Agents

This step-by-step guide is designed so that any AI agent with a small context window can execute each milestone sequentially:

### Milestone 1: Workspace Scaffolding & Core Types (`crates/dagr-core`)
1. Create `DAGR/Cargo.toml` with workspace members (`dagr-core`, `dagr-slicer`, `dagr-guard`, `dagr-sandbox`, `dagr-mcp`, `dagr-cli`).
2. Implement `dagr-core/src/types.rs`: `Language`, `SymbolKind`, `SymbolSpan`, `CodeGraphNode`, `MinimalContextSlice`.
3. Implement `dagr-core/src/error.rs`: Typed `DagrError` using `thiserror`.
4. Implement `dagr-core/src/token.rs`: BPE token counter using `tiktoken-rs`.
5. Implement `dagr-core/src/storage.rs`: SQLite index store with WAL mode.

### Milestone 2: Tree-sitter Parser & Symbolic Slicer (`crates/dagr-slicer`)
1. Implement `dagr-slicer/src/parser.rs`: Multi-language Tree-sitter parser with error recovery.
2. Implement `dagr-slicer/src/extractor.rs`: AST query walker identifying function bodies, parameters, and variable usages.
3. Implement `dagr-slicer/src/contracts.rs`: Interface and type definition hoister.
4. Implement `dagr-slicer/src/slicer.rs`: Backwards data-flow slicer assembling sparse lines.

### Milestone 3: Copy-on-Write Sandbox & Guard (`crates/dagr-sandbox`, `crates/dagr-guard`)
1. Implement `dagr-sandbox/src/cow.rs`: Platform-specific CoW sandbox (`clonefile` on Darwin, hardlinks on Linux/Windows).
2. Implement `dagr-guard/src/rules.rs`: Declarative `.dagr/rules.yaml` parser and glob evaluator.
3. Implement `dagr-guard/src/sanitizer.rs`: Zero-trust indirect prompt injection comment sanitizer.

### Milestone 4: Model Context Protocol (MCP) Gateway (`crates/dagr-mcp`)
1. Implement `dagr-mcp/src/server.rs`: Stdio JSON-RPC 2.0 server with `tracing-subscriber` log isolation.
2. Implement handlers for `dagr_get_context_slice`, `dagr_verify_architecture`, and `dagr_execute_sandboxed`.

### Milestone 5: Unified CLI & Visual Formatter (`crates/dagr-cli`)
1. Implement `dagr-cli/src/main.rs`: Clap v4 subcommand parser.
2. Implement `dagr-cli/src/ui.rs`: Beautiful terminal box-drawing, color scoreboards, and TTY auto-detection.
3. Write golden benchmark integration tests verifying `>= 90%` token reduction.
