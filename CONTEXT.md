# 📖 DAGR Domain Context & Ubiquitous Language (`CONTEXT.md`)

> **Standard:** [Matt Pocock Domain Modeling & Ubiquitous Language](file:///Users/mm/.gemini/config/skills/domain-modeling/SKILL.md)  
> **Repository:** [DAGR](file:///Users/mm/orca/projects/ME/DAGR)  
> **Last Updated:** 2026-08-18  

---

## 🎯 Core Domain Entities & Terms

| Term | Definition & Domain Invariant | Code Symbol / Seam |
| :--- | :--- | :--- |
| **Symbolic Slice** | A minimal, contiguous and non-contiguous AST subset extracted via backwards data-flow analysis, containing only lines affecting the target symbol plus its hoisted contracts. Reduces file tokens by >=90%. | [`MinimalContextSlice`](file:///Users/mm/orca/projects/ME/DAGR/crates/dagr-core/src/types.rs) |
| **Hoisted Contract** | The type definitions, interface declarations, and function signatures that a slice depends on, extracted and placed above the sliced implementation without bringing entire unused files. | [`ContractExtractor`](file:///Users/mm/orca/projects/ME/DAGR/crates/dagr-slicer/src/contracts.rs) |
| **Shadow Transaction** | An isolated Copy-on-Write (CoW) directory namespace (`.dagr/shadow/<tx_id>`) where staged file mutations and tests execute with atomic commit or <10ms rollback. | [`ShadowTransaction`](file:///Users/mm/orca/projects/ME/DAGR/crates/dagr-sandbox/src/tx.rs) |
| **Layer Boundary Invariant** | Architectural rules specifying prohibited import paths between modules (e.g. `UI` cannot import `@db`), evaluated in-memory before disk writes. | [`RuleChecker`](file:///Users/mm/orca/projects/ME/DAGR/crates/dagr-guard/src/checker.rs) |
| **Zero-Trust Prompt Sanitizer** | Lexical sanitizer stripping indirect LLM injection delimiters (`<|im_start|>`, `SYSTEM:`, `[INST]`) from ingested comments, docstrings, and diffs. | [`Sanitizer`](file:///Users/mm/orca/projects/ME/DAGR/crates/dagr-guard/src/sanitizer.rs) |
| **A2A Hub (Agent-to-Agent)** | Peer-to-peer coordination bus facilitating file lock reservation, AST slice transfer, and peer patch verification between autonomous subagents. | [`A2AProtocol`](file:///Users/mm/orca/projects/ME/DAGR/crates/dagr-mcp/src/protocol.rs) |
| **Blast Radius** | The graph distance (k-hops) and set of downstream files/symbols transitively impacted by a file mutation. | [`CodeGraphNode`](file:///Users/mm/orca/projects/ME/DAGR/crates/dagr-core/src/types.rs) |

---

## 📐 Architectural Seams & Invariants

1. **Stdout Isolation Invariant:** `stdout` is strictly reserved for JSON-RPC MCP frames. All logging (`tracing`, diagnostics, banners) MUST route to `stderr`.
2. **Deterministic Token Auditing:** All compression metrics must be audited against `tiktoken-rs` exact BPE counts (`cl100k_base` / `o200k_base`).
3. **Rollback Cleanliness Invariant:** A failed or canceled shadow transaction must leave `0` residual bytes in the working directory (`git status --porcelain` must be clean).
