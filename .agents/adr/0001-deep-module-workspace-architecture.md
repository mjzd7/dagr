# ADR 0001: Deep Module Workspace Architecture & Clean Seams

## Status
Accepted

## Context
DAGR is a native Rust hypervisor for AI coding agents. To prevent software entropy and maintain clean boundaries, we structure the codebase into deep crates with narrow public interfaces.

## Decision
1. `dagr-core`: Shared domain types (`CodeGraphNode`, `MinimalContextSlice`, `Language`, `SymbolKind`), token metrics (`tiktoken-rs`), and SQLite storage.
2. `dagr-slicer`: Deep AST parser and backward data-flow slicer with contract hoisting for TypeScript, JavaScript, Python, Go, and Rust.
3. `dagr-guard`: In-memory layer boundary import checker and zero-trust prompt injection sanitizer.
4. `dagr-sandbox`: Copy-on-Write (CoW) shadow filesystem sandbox with atomic commit and rollback.
5. `dagr-mcp`: Stdio JSON-RPC 2.0 protocol server exposing standard MCP tools and A2A swarm tools.
6. `dagr-cli`: Clap v4 terminal CLI entrypoint with formatted Unicode UI.

## Invariants & Seams
- All logging must use `tracing` routed to `stderr`; `stdout` is strictly reserved for JSON-RPC frames.
- Tests must target public crate interfaces (seams), not private AST traversal internals.
