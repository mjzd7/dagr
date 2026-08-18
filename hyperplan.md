# ⚡ DAGR: Exhaustive Master Execution Plan & Dependency DAG

> **Project:** **DAGR** (`dagr`)  
> **Repository:** [`github.com/mjzd7/dagr`](https://github.com/mjzd7/dagr)  
> **Lead Architect:** Mohit Dagar  
> **Harness Mode:** `/hyperplan` + `/ultrawork`  
> **Governance:** [Ponytail Minimal Architecture (Full)](./.ponytail.md)  
> **Status:** Execution-Ready Master Plan with Task Dependency Graph 🔒  

---

## 📑 Table of Contents
1. [Execution Dependency Topology (Sequential vs. Parallel Matrix)](#1-execution-dependency-topology)
2. [Master Phase 1 Execution Roadmap & Checkable ToDo Breakdown](#2-master-phase-1-execution-roadmap)
   - *Milestone 1: Cargo Workspace & Foundation Core (`crates/dagr-core`) [SEQUENTIAL]*
   - *Milestone 2: Symbolic Slicer Engine (`crates/dagr-slicer`) [PARALLEL TRACK A]*
   - *Milestone 3: Architecture Guardrails & Sanitizer (`crates/dagr-guard`) [PARALLEL TRACK B]*
   - *Milestone 4: Cross-Platform CoW Sandbox (`crates/dagr-sandbox`) [PARALLEL TRACK C]*
   - *Milestone 5: MCP JSON-RPC 2.0 Server (`crates/dagr-mcp`) [SEQUENTIAL INTEGRATION]*
   - *Milestone 6: Visual CLI & Terminal UI (`crates/dagr-cli`) [SEQUENTIAL INTEGRATION]*
   - *Milestone 7: Golden Benchmark & End-to-End Verification [FINAL VERIFICATION]*
3. [Parallel Execution Strategy for Subagents](#3-parallel-execution-strategy-for-subagents)
4. [Verification Harness & SLA Gateways](#4-verification-harness--sla-gateways)

---

## 1. Execution Dependency Topology

```mermaid
graph TD
    classDef seq fill:#f96,stroke:#333,stroke-width:2px;
    classDef par fill:#85C1E9,stroke:#333,stroke-width:2px;
    classDef fin fill:#82E0AA,stroke:#333,stroke-width:2px;

    M1["Milestone 1: Foundation Core<br/><code>crates/dagr-core</code><br/>(Domain Types, Storage, Errors, Tokenizer)"]:::seq

    subgraph Parallel_Execution_Cluster ["⚡ Parallel Execution Track (Independent Workers)"]
        M2["Milestone 2: Symbolic Slicer<br/><code>crates/dagr-slicer</code><br/>(Tree-sitter TS/Python/Go/Rust,<br/>Data-Flow Slicing)"]:::par
        M3["Milestone 3: Architecture Guard<br/><code>crates/dagr-guard</code><br/>(Rules YAML, Glob Boundary Engine,<br/>Prompt Sanitizer)"]:::par
        M4["Milestone 4: CoW Sandbox<br/><code>crates/dagr-sandbox</code><br/>(APFS clonefile, reflink,<br/>10ms Rollback Engine)"]:::par
    end

    M5["Milestone 5: Model Context Protocol<br/><code>crates/dagr-mcp</code><br/>(Stdio JSON-RPC Server & Tool Dispatcher)"]:::seq
    M6["Milestone 6: Unified Visual CLI<br/><code>crates/dagr-cli</code><br/>(Clap v4, Terminal UI Box-Drawing, TTY)"]:::seq
    M7["Milestone 7: Golden Benchmarks<br/><code>tests/fixtures/</code><br/>(>=90% Token Reduction & End-to-End Verification)"]:::fin

    M1 --> M2
    M1 --> M3
    M1 --> M4

    M2 --> M5
    M3 --> M5
    M4 --> M5

    M5 --> M6
    M6 --> M7
```

### Execution Flow Rules:
1. **Sequential Gate 1 (Foundation):** `Milestone 1 (dagr-core)` MUST be completed first because all other crates inherit its domain models, error types, SQLite storage, and `tiktoken-rs` tokenizer.
2. **Parallel Cluster (Milestones 2, 3, 4):** Once `dagr-core` compiles, **Milestone 2 (Slicer)**, **Milestone 3 (Guard)**, and **Milestone 4 (Sandbox)** have zero dependencies on each other and can be executed **100% concurrently**.
3. **Sequential Gate 2 (Integration):** `Milestone 5 (dagr-mcp)` integrates the Slicer, Guard, and Sandbox into the Model Context Protocol server.
4. **Sequential Gate 3 (User Interface):** `Milestone 6 (dagr-cli)` wraps the entire system into the beautiful terminal application.
5. **Final Gate (Validation):** `Milestone 7` validates the golden test fixtures and token compression benchmarks.

---

## 2. Master Phase 1 Execution Roadmap & Checkable ToDo Breakdown

---

### 📍 Milestone 1: Cargo Workspace & Foundation Core (`crates/dagr-core`)
* **Mode:** 🔒 **SEQUENTIAL (Must finish first)**
* **Objective:** Establish workspace root and fundamental domain types, storage, and utilities.

- [ ] **Task 1.1: Initialize Root `DAGR/Cargo.toml` Workspace**
  - Configure workspace resolver "2" with members: `crates/dagr-core`, `crates/dagr-slicer`, `crates/dagr-guard`, `crates/dagr-sandbox`, `crates/dagr-mcp`, `crates/dagr-cli`.
  - Pin workspace shared dependencies (`serde`, `tokio`, `thiserror`, `rusqlite`, `tiktoken-rs`, `blake3`, `uuid`, `glob`).
- [ ] **Task 1.2: Implement `crates/dagr-core/src/error.rs`**
  - Define `DagrError` using `thiserror` (`ParserError`, `StorageError`, `SandboxError`, `ConfigError`, `IoError`, `SerializationError`).
  - Define type alias `Result<T> = std::result::Result<T, DagrError>`.
- [ ] **Task 1.3: Implement `crates/dagr-core/src/types.rs`**
  - Implement `Language` enum (`TypeScript`, `JavaScript`, `Python`, `Go`, `Rust`, `Unknown`).
  - Implement `SymbolKind` enum (`Function`, `Method`, `Class`, `Struct`, `Interface`, `TypeAlias`, `Enum`, `Variable`, `Module`).
  - Implement `SymbolSpan` (`file_path`, `start_line`, `end_line`, `start_col`, `end_col`).
  - Implement `CodeGraphNode` (canonical URI, symbol name, kind, language, span, docstring, blake3 hash).
  - Implement `MinimalContextSlice` (`target_symbol`, `language`, `sparse_code_lines`, `type_contracts`, `estimated_tokens`, `original_file_tokens`, `compression_ratio`, `syntax_degraded`).
- [ ] **Task 1.4: Implement `crates/dagr-core/src/token.rs`**
  - Embed `tiktoken-rs` with `cl100k_base` and `o200k_base` tokenizers.
  - Implement `count_tokens(text: &str) -> usize` and `compute_compression_ratio(original: usize, sliced: usize) -> f32`.
- [ ] **Task 1.5: Implement `crates/dagr-core/src/storage.rs`**
  - Implement `LocalIndexStore` with embedded SQLite at `.dagr/index.db`.
  - Enable `PRAGMA journal_mode = WAL;` and in-memory temporary tables.
  - Implement `file_cache` table and `is_file_cached(path, hash) -> Result<bool>`.
  - Implement `symbol_index` table with fast compound lookup `(file_path, symbol_name)`.
- [ ] **Task 1.6: Milestone 1 Unit Tests**
  - Unit tests for token counting, error conversions, and SQLite CRUD caching in `crates/dagr-core/tests/`.

---

### 📍 Milestone 2: Tree-sitter Slicer Engine (`crates/dagr-slicer`)
* **Mode:** ⚡ **PARALLEL TRACK A (Can run concurrently with M3 & M4)**
* **Objective:** Parse ASTs across TS, Python, Go, and Rust; compute backwards data-flow slices.

- [ ] **Task 2.1: Implement `crates/dagr-slicer/src/parser.rs`**
  - Integrate native C grammars (`tree-sitter-typescript`, `tree-sitter-python`, `tree-sitter-go`, `tree-sitter-rust`, `tree-sitter-javascript`).
  - Implement `AstParser::parse(source: &str, old_tree: Option<&Tree>) -> Result<Tree>`.
  - Implement syntax error recovery: extract nearest valid enclosing parent node when `node.has_error()` is true.
  - Implement Lexical Indentation Fallback on complete parse failure with `syntax_degraded: true`.
- [ ] **Task 2.2: Implement `crates/dagr-slicer/src/extractor.rs`**
  - Build AST query walkers for each language extracting symbol definitions, function arguments, variable declarations, and import statements.
- [ ] **Task 2.3: Implement `crates/dagr-slicer/src/contracts.rs`**
  - Implement type-contract hoister: extracts interface signatures and type aliases while discarding implementation bodies.
- [ ] **Task 2.4: Implement `crates/dagr-slicer/src/slicer.rs`**
  - Implement `SymbolicSlicer::slice(source, lang, target_symbol)`:
    1. Locate target symbol AST node.
    2. Traverse internal identifier data-flow backwards using `HashSet<SymbolHash>` to prevent circular loops.
    3. Collect hoisted external type contracts.
    4. Assemble sparse ordered lines with line numbers.
    5. Calculate token savings via `dagr_core::token`.
- [ ] **Task 2.5: Milestone 2 Unit Tests**
  - Unit tests for TypeScript, Python, and Go slicing fixtures asserting `>= 90%` token reduction.

---

### 📍 Milestone 3: Architecture Guardrails & Sanitizer (`crates/dagr-guard`)
* **Mode:** ⚡ **PARALLEL TRACK B (Can run concurrently with M2 & M4)**
* **Objective:** Evaluate `.dagr/rules.yaml` layer boundaries and sanitize prompt injections.

- [ ] **Task 3.1: Implement `crates/dagr-guard/src/rules.rs`**
  - Define `RuleConfig`, `BoundaryRule`, and `Preset` structs with `serde`.
  - Implement `.dagr/rules.yaml` parser with default safe fallbacks.
  - Implement built-in presets: `clean-architecture`, `nextjs-app`, `fastapi-layered`.
- [ ] **Task 3.2: Implement `crates/dagr-guard/src/checker.rs`**
  - Implement glob-based import boundary matcher (`glob::Pattern`).
  - Implement `check_file_imports(file_path: &str, imports: &[String]) -> Vec<RuleViolation>`.
  - Optimize execution to complete in `<0.1ms` per file.
- [ ] **Task 3.3: Implement `crates/dagr-guard/src/sanitizer.rs`**
  - Implement `ZeroTrustSanitizer::sanitize(text: &str) -> String`.
  - Strip LLM prompt boundary delimiters (`<|im_start|>`, `<|im_end|>`, `SYSTEM:`, `[INST]`, `SYSTEM PROMPT:`).
- [ ] **Task 3.4: Implement `crates/dagr-guard/src/infer.rs`**
  - Implement smart invariant inference engine (`dagr rules suggest` / `dagr init` auto-detection).
- [ ] **Task 3.5: Milestone 3 Unit Tests**
  - Unit tests for rule violation detection, glob matching, and prompt sanitization.

---

### 📍 Milestone 4: Cross-Platform CoW Sandbox (`crates/dagr-sandbox`)
* **Mode:** ⚡ **PARALLEL TRACK C (Can run concurrently with M2 & M3)**
* **Objective:** Fast Copy-on-Write shadow filesystem staging with 10ms rollback.

- [ ] **Task 4.1: Implement `crates/dagr-sandbox/src/engine.rs`**
  - Implement OS-specific CoW engines:
    - `#[cfg(target_os = "macos")]`: `clonefile(2)` via Darwin syscalls.
    - `#[cfg(target_os = "linux")]`: `ioctl(FICLONE)` reflink / hard-link shadow tree.
    - `#[cfg(target_os = "windows")]`: ReFS duplicate extents / hard-link shadow staging.
- [ ] **Task 4.2: Implement `crates/dagr-sandbox/src/tx.rs`**
  - Implement `begin_transaction(root) -> Result<SandboxTx>`.
  - Implement `stage_write(tx, relative_path, content) -> Result<()>`.
  - Implement `verify(tx, command) -> Result<ExecutionResult>`.
  - Implement `commit(tx) -> Result<()>` (atomic swap back to workspace).
  - Implement `rollback(tx) -> Result<()>` (sub-10ms directory purge).
- [ ] **Task 4.3: Implement `crates/dagr-sandbox/src/journal.rs`**
  - Implement write-ahead transaction journal (`.dagr/journal.db`) to auto-clean orphaned locks on startup.
- [ ] **Task 4.4: Milestone 4 Unit Tests**
  - Unit tests for shadow creation, dry-run test execution, and rollback with 0 modified bytes remaining on disk.

---

### 📍 Milestone 5: Model Context Protocol (MCP) Server (`crates/dagr-mcp`)
* **Mode:** 🔒 **SEQUENTIAL (Requires M1, M2, M3, M4)**
* **Objective:** Stdio & SSE JSON-RPC 2.0 gateway for Cursor, Claude Desktop, and Windsurf.

- [ ] **Task 5.1: Implement `crates/dagr-mcp/src/protocol.rs`**
  - Implement JSON-RPC 2.0 request/response/error schemas.
  - Implement standard MCP message handlers: `initialize`, `tools/list`, `tools/call`.
- [ ] **Task 5.2: Implement `crates/dagr-mcp/src/server.rs`**
  - Implement Stdio event loop reading from `stdin` and writing formatted JSON-RPC to `stdout`.
  - Implement `tracing-subscriber` log isolation routing all internal logs to `stderr` or `~/.dagr/logs/daemon.log`.
- [ ] **Task 5.3: Implement Core MCP Tool Handlers**
  - `dagr_get_context_slice`: Dispatches to `dagr_slicer::SymbolicSlicer`.
  - `dagr_verify_architecture`: Dispatches to `dagr_guard::ArchitectureGuard`.
  - `dagr_execute_sandboxed`: Dispatches to `dagr_sandbox::CowSandbox`.
- [ ] **Task 5.4: Milestone 5 Integration Tests**
  - Automated JSON-RPC stdio mock tests validating tool call request/response cycles.

---

### 📍 Milestone 6: Unified Visual CLI & Terminal UI (`crates/dagr-cli`)
* **Mode:** 🔒 **SEQUENTIAL (Requires M5)**
* **Objective:** Modern, visually stunning CLI with Clap v4 and TTY auto-detection.

- [ ] **Task 6.1: Implement `crates/dagr-cli/src/main.rs`**
  - Define Clap v4 CLI parser with subcommands: `context`, `guard`, `run`, `mcp`, `init`.
  - Implement global `--json` and `--workspace` flags.
- [ ] **Task 6.2: Implement `crates/dagr-cli/src/ui.rs`**
  - Implement rich terminal formatting using Unicode box-drawing (`┌─`, `├─`, `└─`) and colors (`owo-colors` / `colored`).
  - Implement visual **Token Compression Scoreboard** showing before/after tokens, percentage saved, and latency.
  - Implement visual **Architectural Violation Card** highlighting forbidden imports and suggested fixes.
  - Implement **TTY Auto-Detection**: When piped (`| ollama`, `| pbcopy`), output raw code; when interactive, render full UI.
- [ ] **Task 6.3: Implement Command Handlers**
  - `dagr context <path>:<symbol>`
  - `dagr guard [--suggest]`
  - `dagr run "<command>" [--sandbox]`
  - `dagr mcp start [--port]`
  - `dagr init [--preset] [--install-hooks]`
- [ ] **Task 6.4: Milestone 6 CLI Tests**
  - CLI execution tests verifying command flags, pipe outputs, and exit codes.

---

### 📍 Milestone 7: Golden Benchmark Suite & End-to-End Verification
* **Mode:** 🏁 **FINAL GATE**
* **Objective:** Verify real-world performance, token compression, and safety.

- [ ] **Task 7.1: Build Golden Test Fixtures (`tests/fixtures/`)**
  - Complex TypeScript/React component tree fixture (1,200 lines).
  - Python FastAPI + SQLAlchemy service fixture (800 lines).
  - Go HTTP handler + interface fixture (600 lines).
- [ ] **Task 7.2: Automated Benchmark Assertions**
  - Assert `dagr context` achieves `>= 90%` token compression across all fixtures.
  - Assert `dagr guard` executes in `< 35ms` on 50 staged files.
  - Assert `dagr run --sandbox` leaves 0 modified bytes on working tree upon simulated test failure.
- [ ] **Task 7.3: Commit, Tag, and Push v0.1.0 to GitHub**

---

## 3. Parallel Execution Strategy for Subagents

When using autonomous subagents (`invoke_subagent`), execution should follow this exact parallel strategy:

```text
[Step 1]: Lead Agent implements Milestone 1 (`crates/dagr-core`) -> Compiles & Tests.
[Step 2]: Lead Agent spawns 3 Concurrent Subagents:
          ├── Subagent A (Role: "Slicer Specialist")   -> Implements Milestone 2 (`crates/dagr-slicer`)
          ├── Subagent B (Role: "Security & Guard")   -> Implements Milestone 3 (`crates/dagr-guard`)
          └── Subagent C (Role: "CoW Sandbox Lead")    -> Implements Milestone 4 (`crates/dagr-sandbox`)
[Step 3]: Subagents report back. Lead Agent validates all 3 crates compile concurrently.
[Step 4]: Lead Agent implements Milestone 5 (`crates/dagr-mcp`) & Milestone 6 (`crates/dagr-cli`).
[Step 5]: Run Milestone 7 Benchmark Suite -> Verify 100% Green -> Push to GitHub.
```

---

## 4. Verification Harness & SLA Gateways

| Metric | Target SLA | Verification Method |
| :--- | :---: | :--- |
| **Token Compression** | **>= 90%** (Target: 95%) | `tiktoken-rs` audited token comparison on golden fixtures |
| **AST Parse & Slice Latency** | **< 5ms** (Cold) / **< 0.5ms** (Cached) | In-memory timer benchmark on 1,000-line ASTs |
| **Pre-commit Guard Latency** | **< 35ms** | `dagr guard` execution across 50 staged files |
| **Rollback Execution Time** | **< 10ms** | Time to purge `.dagr/shadow/<tx_id>` snapshot |
| **Working Tree Cleanliness** | **100% (0 residual bytes)** | `git status --porcelain` assertion after failed sandbox runs |
