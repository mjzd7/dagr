# Getting Started

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/mjzd7/dagr/main/scripts/install.sh | bash   # macOS/Linux
# or: brew install mjzd7/dagr/dagr      # or: npm i -g @mjzd7/dagr
dagr --version
```

## 60-second tour

```bash
cd your-repo

dagr init                    # seeds .dagr/rules.yaml with clean-architecture boundaries
dagr guard                   # check the whole workspace (preset fallback if no rules file)
dagr guard --staged          # check only what you're about to commit
```

## Governance loop for agent work

```bash
# 1. Register the agent and its human owner
dagr agent register cursor-alice --owner alice --role builder --ttl-secs 86400

# 2. Let the agent run; MCP tools accept {"_agent": "cursor-alice"} so every
#    telemetry row is attributable, and revoked ids are rejected immediately.

# 3. Gate the merge
dagr review-diff origin/main HEAD        # PASS / BLOCKED (+ exit code)

# 4. Produce an audit receipt
dagr prove --test "cargo test"
```

## Environment sanity

```bash
dagr doctor                  # grammars, sandbox FS, SQLite WAL, rules, IDE configs
```

## Next steps

- MCP tool reference: [mcp-tools.md](mcp-tools.md)
- Boundary rules schema: [rules-schema.md](rules-schema.md)
- Known limitations: [HONEST-LIMITS.md](HONEST-LIMITS.md)
- Compliance mapping: [COMPLIANCE-MAPPING.md](COMPLIANCE-MAPPING.md)
- Outcome benchmark: [`evals/`](../evals/)
