---
name: dagr-slicer
description: Surgical AST context slicing and contract hoisting hypervisor. Use whenever inspecting, analyzing, or preparing to modify a function, method, or class, to avoid loading full files and slash token consumption by >95%. Also use when user mentions token reduction, context slicing, or AST extraction.
---

# `dagr-slicer` Agent Skill

## 🎯 When to Use
- Whenever preparing to edit, review, or write a function, method, or class.
- When you need to understand a symbol's contract without blowing your token budget on a 2,000-line file.
- When the user asks you to inspect or refactor a specific function.

## 🚀 How to Execute
Call the CLI command or MCP tool:
```bash
dagr context <FILE_PATH>:<SYMBOL_NAME> --format json
```
Or MCP tool:
`dagr_get_context_slice(file_path="...", symbol_name="...")`

## 📋 Returned Context
1. **Hoisted Type Contracts**: Exact interfaces and classes referenced by the symbol.
2. **Minimal Implementation Slice**: Only the relevant body lines.
3. **Token Footprint**: Precise token count and compression ratio.
