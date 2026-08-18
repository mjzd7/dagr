---
name: dagr-sandbox
description: Copy-on-Write (CoW) shadow workspace runner. Use whenever executing tests, refactors, or potentially destructive commands with instant 10ms atomic rollback on failure. Also use when user mentions sandboxed run, safe trial, or atomic rollback.
---

# `dagr-sandbox` Agent Skill

## 🎯 When to Use
- When executing tests, builds, or scripts that might fail or leave dirty artifacts.
- When performing speculative refactors where you want guaranteed 0-side-effect rollback on failure.

## 🚀 How to Execute
Call the CLI command or MCP tool:
```bash
dagr run "<TEST_COMMAND>" [--commit-on-success]
```
Or MCP tool:
`dagr_execute_sandboxed(command="...")`
