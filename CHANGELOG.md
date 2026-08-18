# 📜 DAGR Changelog & Release Notes

All notable changes to the **DAGR** project will be documented in this file in chronological order.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased] - 2026-08-18

### 🚀 Milestone 1: Core Domain, Storage & Exact Tokenizer (`crates/dagr-core`) - [`b6305a9`](https://github.com/mjzd7/dagr/commit/b6305a9)
- **Mathematical Token Counter (`token.rs`):** Integrated native `tiktoken-rs` (`cl100k_base` / `o200k_base`) calculating exact Byte-Pair Encoding token counts and compression ratios.
- **Embedded SQLite Index (`storage.rs`):** Embedded SQLite database at `.dagr/index.db` with Write-Ahead Logging (WAL mode) and 32-byte Blake3 content hash caching for `<0.05ms` lookup hits.
- **Domain Types (`types.rs`):** Implemented `CodeGraphNode`, `MinimalContextSlice`, `Language`, `SymbolKind`, and `SymbolSpan`.
- **Typed Error Hierarchy (`error.rs`):** Comprehensive `DagrError` enum using `thiserror`.

### ✂️ Milestone 2: Tree-sitter Parser & Symbolic Slicer (`crates/dagr-slicer`) - [`b6305a9`](https://github.com/mjzd7/dagr/commit/b6305a9)
- **Multi-Language AST Ingestion (`parser.rs`):** Static native C Tree-sitter parsers for TypeScript, JavaScript, Python, Go, and Rust.
- **Symbol AST Extractor (`extractor.rs`):** Query walker traversing ASTs to extract exact function/class boundaries and internal identifier references.
- **Contract Hoister (`contracts.rs`):** Surgically hoists referenced type definitions and interfaces while pruning unreferenced implementation bodies.
- **Symbolic Slicer (`slicer.rs`):** Combines parsing, data-flow traversal, contract hoisting, and sparse line assembly with fallback error recovery.

### 🛡️ Milestone 3: Architectural Guardrails & Prompt Sanitizer (`crates/dagr-guard`)
- **Declarative Rule Engine (`rules.rs`):** Implemented `.dagr/rules.yaml` schema parser with built-in presets (`clean-architecture`, `nextjs-app`, `fastapi-layered`).
- **In-Memory Boundary Checker (`checker.rs`):** Evaluates forbidden import pairs via `glob::Pattern` in `<0.05ms` per file.
- **Zero-Trust Prompt Sanitizer (`sanitizer.rs`):** Strips indirect prompt injection control tokens (`<|im_start|>`, `SYSTEM:`, `[INST]`, `system override:`) from comments and docstrings.
- **Smart Framework Inferrer (`infer.rs`):** Auto-detects repository structure for `dagr init` zero-configuration setup.

### 🛡️ Milestone 4: Cross-Platform Copy-on-Write (CoW) Sandbox (`crates/dagr-sandbox`)
- **Block-Level Cloning Engine (`engine.rs`):** Uses Darwin kernel `clonefile(2)` on macOS (sub-1ms) and cross-platform file fallback on Linux/Windows.
- **Shadow Transaction Lifecycle (`tx.rs`):** Implemented `begin`, `stage_file`, `verify`, `commit`, and `rollback` in `<10ms` leaving 0 dirty bytes on working tree upon failure.
- **Crash-Safe Transaction Journal (`journal.rs`):** Auto-cleans orphaned transaction shadow directories upon startup.

### 🔌 Milestone 5: Model Context Protocol (MCP) & A2A Swarm Bus (`crates/dagr-mcp`)
- **JSON-RPC 2.0 Engine (`protocol.rs`):** Standard request/response and error serialization for `initialize`, `tools/list`, `tools/call`.
- **Stdio Hypervisor Gateway (`server.rs`):** Strict stdout isolation for Cursor/Claude Desktop JSON-RPC streaming, routing all logs to stderr.
- **6-Tool Protocol Suite (`tools.rs`):**
  - `dagr_get_context_slice` (AST backwards data-flow & contract hoister).
  - `dagr_verify_architecture` (In-memory layer boundary checker).
  - `dagr_execute_sandboxed` (Zero-trust CoW test runner).
  - `dagr_a2a_handshake` (Agent registration & optimistic write locks).
  - `dagr_a2a_transfer_context` (Peer-to-peer AST slice transfer).
  - `dagr_a2a_verify_peer_patch` (Cross-agent shadow patch verification).
