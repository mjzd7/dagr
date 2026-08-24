# DAGR Pilot Eval — Benchmark

**Status: infrastructure-blocked.** The harness is complete and
mock-verified; live numbers are pending a working provider key (see
[Blocked](#blocked)).

## What this measures

Outcome quality — **task pass-rate and defect counts** — for a coding agent
given identical tasks under two context strategies:

| Strategy | Context given to the model |
|---|---|
| `baseline` | every task source file pasted whole |
| `dagr` | `dagr context` slice output injected instead |

Grading is mechanical: the model's file is written into a scratch copy of the
repo and hidden tests must pass. No self-reported scores.

## Run it

```bash
# deterministic mechanics check (no key needed)
node evals/run.mjs --provider mock

# live run — OpenAI or any OpenAI-compatible gateway via OPENAI_BASE_URL
OPENAI_API_KEY=sk-... node evals/run.mjs --provider openai
ANTHROPIC_API_KEY=sk-ant-... node evals/run.mjs --provider anthropic
```

Results land in `evals/results/latest.json` (per-task rows include token
counts, latency, pass/fail, defect estimates).

<a name="blocked"></a>
## Blocked: live runs

Attempted 2026-08-24 with `--provider openai`: the environment's
`OPENAI_API_KEY` is a gateway key (`freellmapi-…`) whose endpoint rejects it
with `401 Invalid API key` on `/chat/completions` and `/models`, across
Bearer/x-api-key/api-key auth styles. Provider errors are surfaced loudly in
`results/*.json` (`"error": "openai 401: …"`).

**To unblock:** provide a valid key (env `OPENAI_API_KEY` + optional
`OPENAI_BASE_URL`), re-run, and this page gets real tables — failures
included.

## Current task set (pilot scale)

| Task | Skill tested |
|---|---|
| task-001-fix-function | locate & fix an off-by-one discount bug |
| task-002-add-validation | add input validation with exact throw semantics |
| task-003-refactor-import | inline a cross-module dependency safely |

Scaling to ≥6 OSS repos / ≥100 tasks is tracked as a roadmap item; the task
import format lands with that work.

## Honest caveats (read before citing anything)

- Exact-string search still beats AST slicing for literal lookups.
- Compression percentages vary by file shape; small files compress less.
- Mock-mode results verify harness mechanics only — never cite them as model
  performance.
- Token figures here are prompt *input* metrics, not dollar savings.
