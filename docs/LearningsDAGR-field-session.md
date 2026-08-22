# LearningsDAGR — DAGR tool + Automate-Instagram-Posts codebase findings

> Session date: 2026-08-22 · Source: hands-on testing of `dagr` v0.1.0 against this monorepo.
> Companion to `docs/LEARNINGS.md` (project failure register). This file tracks what we learned while setting up & testing the DAGR hypervisor.

## Environment facts

| Fact | Value |
|---|---|
| Binary | `~/.cargo/bin/dagr`, v0.1.0 (Rust) |
| Upstream repo | `mjzd7/dagr` (public, main branch) |
| MCP registered in | Claude Desktop ✅, Cursor ✅, Windsurf ✅ — **opencode ❌** (opencode.json has `mcp: []`) |
| Workspace state | `.dagr/index.db` + `.dagr/rules.yaml` present |
| Cloud/sync | Local dashboard endpoint `http://127.0.0.1:3333`, org label "MICROSOFT Team", telemetry ledger local |

## Critical findings (would bite again)

### L1. Silent zero-rule parse on wrong schema (guard fail-open trap)

`.dagr/rules.yaml` shipped with field names that **don't exist** in the parser:

| File had | Parser expects (`crates/dagr-guard/src/rules.rs`) |
|---|---|
| `rules:` | `boundaries:` |
| `source:` | `from:` |
| `disallow:` | `cannot_import:` |
| `reason:` | `message:` (has default) |

Consequences observed:
- Unknown keys are silently ignored (serde defaults) → config parses OK with **`active_rules: 0`** → guard reports "✅ passed" forever. No error, no warning.
- If the file is *missing entirely*, guard instead falls back to a built-in clean-architecture **preset** — i.e., missing file = MORE enforcement than malformed file.
- **Action taken**: rewrote rules.yaml in correct schema (original preserved as `.dagr/rules.yaml.orig`).
- **Upstream bug worth filing**: `RuleConfig` should use `#[serde(deny_unknown_fields)]` or warn on unknown top-level keys; also README never documents the rules.yaml schema.

### L2. Guard matches raw import strings, NOT resolved paths

`check_import()` (checker.rs) compares `cannot_import` patterns against the **literal string inside the import statement**. Relative imports are never resolved to workspace paths.

- `import { openDb } from "../db/client"` will NEVER match rule pattern `packages/core/src/db/**`.
- Rules must be written in the codebase's own import dialect. In THIS repo that means:
  - `packages/core/src/*` uses **relative** cross-module imports: `../db/client.js`, `../images/constants.js`
  - `apps/web/app/api/*` uses **aliases**: `@/lib/repo-paths`, `@/auth`
- Correct pattern shape here: `../db/**` (matcher does `starts_with(pattern.trim_end_matches("/**"))`, so `../db/**` also catches `../db-migration/…` — known minor false-positive risk).

### L3. This codebase's import/convention map (for writing guard rules)

- Monorepo: pnpm workspaces, `apps/web` (Next.js 14 dashboard) + `packages/core`.
- Core modules: content-filter, crypto, db, images, quotes, instagram, hashtags, matching, pipeline, git, notify, threads, config.
- Content-filter files import only node builtins + `../images/constants.js` → currently clean vs its boundary.
- Crypto (`token-encryption.ts`) imports only `node:crypto`, `node:fs`, `node:url` → clean.
- Web API routes import `next/server`, node builtins, `@/auth`, `@/lib/*` → clean.

### L4. Verified-working CLI behaviors

- `dagr context path.ts:symbol --format json|pretty` works: sliced `textPassesFilter` (789→85 tokens, 89% compression), `wordCount` (251→37 tokens, 85%).
- `type_contracts` stayed **empty** even with `--depth 2` — contract hoisting appears not to trigger for intra-file deps; treat depth flag as currently inert for TS single-file slices.
- `dagr stats` / `dagr status` work; telemetry shows slices served + token savings.
- MCP binary exposes ≥7 tools (from strings): slice/context tool, `dagr_verify_architecture` (source_file + imports list), `dagr_execute_sandboxed` (command), `dagr_get_lifetime_stats`, `dagr_a2a_handshake`, `dagr_a2a_transfer_context`, `dagr_a2a_verify_peer_patch`.

### L5. Testing protocol that worked

1. Baseline run → expect pass.
2. Plant violation (append forbidden import), re-run → expect catch.
3. Revert immediately via `git checkout -- <file>`; verify `git status` clean.
4. Use `--format json` when pretty output hides internals (`active_rules` was only visible in JSON).

## Round 2 findings (MCP + sandbox + dashboard)

### L6. MCP tool argument names differ from CLI intuition

`dagr_verify_architecture` expects **`proposed_imports`** (not `imports`). Wrong/missing args **silently default to empty** → tool returns `valid: true` instead of erroring. Same fail-open pattern as L1: when in doubt, test the violating case, never just the clean case.

Verified-correct call shape:

```json
{"name": "dagr_verify_architecture",
 "arguments": {"source_file": "packages/core/src/content-filter/text-filter.ts",
               "proposed_imports": ["../db/client", "node:fs"]}}
```

### L7. MCP server resolves everything from process CWD

`McpServer::new(std::env::current_dir()?)` — rules.yaml, index.db, telemetry all resolve from wherever the server was launched. IDEs that spawn MCP servers from arbitrary CWDs will get **preset fallback rules** (not this repo's boundaries) with no warning. If guard results ever look generic/wrong from an IDE, check launch CWD first.

### L8. Verified-working surface (end-to-end)

| Surface | Status | Evidence |
|---|---|---|
| `dagr context` (CLI + MCP) | ✅ | 89%/85% compression, valid slices, both formats |
| `dagr guard` (after schema fix) | ✅ | planted violation caught w/ rule+advice; clean tree passes |
| `dagr run --sandbox` | ✅ | success→isolated, failure→rolled back, `-c`→committed |
| MCP initialize/tools/list | ✅ | 7 tools exposed |
| MCP verify_architecture | ✅ | violation + clean cases correct |
| MCP get_context_slice | ✅ | matches CLI output |
| MCP a2a_handshake | ✅ | agent registered, file locked |
| `dagr stats` / `status` | ✅ | telemetry ledger live |
| `dagr dashboard` | ✅ | HTTP 200, 45KB UI @ 127.0.0.1:3333 |
| MCP registered in opencode | ❌ not installed | add to opencode.json `mcp` if wanted |

### L9. Known quirks / upstream-issue candidates for mjzd7/dagr

1. Malformed rules.yaml parses as 0 rules (should be hard error — `deny_unknown_fields`).
2. README documents usage but not the rules.yaml schema.
3. MCP tools silently ignore unknown/missing arguments.
4. `--depth` flag appears inert for TS intra-file slices (`type_contracts` always empty).
5. Guard exit code masked when piped through `head` (cosmetic; real runs fine).

### L10. dagr MCP is NOT registered in opencode

`~/.config/opencode/opencode.json` has `"mcp": []` — despite `dagr mcp install` having registered dagr into Claude Desktop, Cursor, and Windsurf. So AI agents running inside opencode cannot call dagr tools natively; they must either shell out to the CLI or speak stdio JSON-RPC manually (as this session did).

To register natively, add to `opencode.json`:

```json
{
  "mcp": {
    "dagr": {
      "type": "local",
      "command": ["dagr", "mcp", "start"]
    }
  }
}
```

Note: takes effect on next opencode start/reload. Also remember L7 — opencode must launch the server with CWD inside the target repo, or guard/sandbox silently fall back to preset rules instead of this workspace's boundaries.
