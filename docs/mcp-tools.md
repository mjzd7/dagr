# MCP Tool Reference

Start the server: `dagr mcp start` (stdio, JSON-RPC 2.0). Auto-configure an
IDE with `dagr mcp install --client <cursor|claude|...>`.

## `dagr_get_context_slice`

Extracts the minimal backwards AST slice + hoisted type contracts for a
target symbol.

| Argument | Type | Required | Notes |
|---|---|---|---|
| `file_path` | string | ✓ | workspace-relative |
| `symbol_name` | string | ✓ | function/class/type name |
| `max_depth_hops` | number |  | v1 performs one effective cross-file hop |

**Response:** target symbol, sparse code lines, hoisted contracts, token
counts, compression ratio — plus a `resolution` object (`via`, `confidence`)
explaining *which resolver stage matched* when provenance is available.

## `dagr_verify_architecture`

Checks proposed imports against `.dagr/rules.yaml` boundaries (<1ms).

| Argument | Type | Required |
|---|---|---|
| `source_file` | string | ✓ |
| `proposed_imports` | string[] | ✓ |

**Response:** `valid`, per-violation details, `rules_source`
(`file` or `preset`), `active_rules` count.

## `dagr_execute_sandboxed`

Runs a shell command inside the Copy-on-Write shadow sandbox; rolls back on
failure.

| Argument | Type | Required |
|---|---|---|
| `command` | string | ✓ |

**Response:** `success`, stdout/stderr tails, `rolled_back`.

## `dagr_get_lifetime_stats`

Cumulative token/efficiency telemetry and per-client breakdown.

## Agent attribution (all tools)

Every tool accepts an optional top-level **`_agent`** argument:

```json
{ "file_path": "src/x.ts", "symbol_name": "f", "_agent": "cursor-alice" }
```

When `_agent` is present it must be an **active** id in
`.dagr/agents.json` (see `dagr agent register` / `dagr revoke`); revoked or
expired ids are rejected, and telemetry rows are tagged `mcp:<id>` so cost
attribution shows up per agent in `dagr stats`.

## Experimental: A2A swarm tools

Three peer-to-peer tools (`dagr_a2a_handshake`, `dagr_a2a_transfer_context`,
`dagr_a2a_verify_peer_patch`) exist behind the compile-time `a2a` cargo
feature and are **off by default**. See HONEST-LIMITS.md.
