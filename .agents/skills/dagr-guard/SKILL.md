---
name: dagr-guard
description: In-memory architectural boundary and layer import linter (<0.1ms). Use whenever validating code changes, writing imports, creating PRs, or verifying clean architecture boundaries (e.g. UI cannot import DB/ORM). Also use when user mentions boundary rules, layer violations, or architecture guard.
---

# `dagr-guard` Agent Skill

## 🎯 When to Use
- Before submitting code changes, creating a commit, or opening a PR.
- To verify that presentation layers (UI) do not directly import database/ORM layers.
- To sanitize user comments against indirect prompt injections.

## 🚀 How to Execute
Call the CLI command or MCP tool:
```bash
dagr guard --format json
```
Or MCP tool:
`dagr_verify_architecture(source_file="...", proposed_imports=[...])`
