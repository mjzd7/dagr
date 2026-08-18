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
