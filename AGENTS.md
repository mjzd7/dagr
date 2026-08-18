# ⚡ DAGR — Autonomous Agent Operating Manual & Protocol

> **Purpose:** Canonical operational guide for autonomous AI coding agents interacting with the DAGR repository and CLI.
> **Standard:** [Matt Pocock Writing for Agents](file:///Users/mm/.gemini/config/skills/writing-for-agents/SKILL.md) & [Domain Modeling](file:///Users/mm/orca/projects/ME/DAGR/CONTEXT.md)

---

## 🧭 Repository Context & Ubiquitous Language

Before editing or analyzing code, consult [CONTEXT.md](file:///Users/mm/orca/projects/ME/DAGR/CONTEXT.md) and respect [ADR 0001](file:///Users/mm/orca/projects/ME/DAGR/.agents/adr/0001-deep-module-workspace-architecture.md).

- **`MinimalContextSlice`**: The minimal subset of lines + hoisted type contracts computed via backwards data-flow slicing.
- **`ShadowTransaction`**: An isolated Copy-on-Write overlay (`.dagr/shadow/<tx_id>`) for dry-running tool mutations.
- **`stdout` Isolation Invariant**: `stdout` is strictly reserved for MCP JSON-RPC 2.0 frames; all diagnostics must route to `stderr`.

---

## 🛠️ CLI Quickstart Reference for Agents

The `dagr` binary is self-documenting. Use `dagr --help` or `dagr <subcommand> --help` to discover options dynamically.

```bash
# 1. Surgical AST Slicing (saves 95% tokens over reading full files)
dagr context <FILE_PATH>:<SYMBOL_NAME> [--depth <N>] [--format pretty|json|plain|markdown]
# Example:
dagr context crates/dagr-core/src/types.rs:Language --format json

# 2. In-Memory Architectural Boundary & SOLID Evaluation (<0.1ms)
dagr guard [--workspace <PATH>] [--staged] [--format pretty|json]

# 3. Isolated Copy-on-Write (CoW) Sandboxed Execution (Zero dirty state on failure)
dagr run "<SHELL_COMMAND>" [--sandbox] [--commit-on-success]
# Example:
dagr run "cargo test --workspace"

# 4. Start Model Context Protocol (MCP) JSON-RPC 2.0 Server
dagr mcp start

# 5. Initialize Rules & SQLite Index in a Workspace
dagr init [--preset clean-architecture|nextjs|fastapi]
```

---

## 🔄 Autonomous Engineering Lifecycle Protocol

Every coding agent working on DAGR MUST adhere to this 5-stage lifecycle:

```
[Stage 1 - Alignment & Modeling]:
    1. Check `CONTEXT.md` for ubiquitous language.
    2. Run `grill-with-docs` to resolve underspecified edge cases before modifying architecture.

[Stage 2 - Deep Module Design]:
    1. Consult `codebase-design`: Keep crate interfaces narrow, implementations deep.
    2. Reject bloat with `ponytail`: Favor Rust standard library before adding dependencies.

[Stage 3 - Tracer-Bullet Tickets]:
    1. Break changes into atomic vertical slices using `to-tickets`.

[Stage 4 - Test-Driven Development & Diagnosis]:
    1. Write a failing unit/integration test first (Red phase).
    2. Implement minimal code to pass (Green phase).
    3. If tests fail, enter `diagnosing-bugs` (Reproduce -> Locate -> Explain -> Verify).
    4. Run tests safely inside the CoW sandbox: `dagr run "cargo test"`.

[Stage 5 - Dual-Axis Review & Pre-Commit]:
    1. Run `code-review` along two independent axes:
       - Standards Axis: Conformance to Rust idioms & deep module design.
       - Spec Axis: Conformance to milestone requirements without scope creep.
    2. Verify with `dagr guard` and ensure working tree is clean (`git status --porcelain`).
```

---

## 🧪 Verification & Acceptance Criteria

Before concluding any task:
1. `cargo test --workspace` MUST return exit code `0` with 100% tests passing.
2. `dagr guard` MUST report 0 architectural boundary violations.
3. No lingering temporary files in `.dagr/shadow/`.
