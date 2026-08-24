# DAGR Pilot Eval — Benchmark

**Status: LIVE NUMBERS EXIST** (2026-08-24, via OpenRouter free tier).
Scale is still pilot-grade — read [caveats](#caveats-read-before-citing)
before citing anything.

## What this measures

Outcome quality — **task pass-rate and defect counts** — for a coding agent
given identical tasks under two context strategies:

| Strategy | Context given to the model |
|---|---|
| `baseline` | every task source file pasted whole |
| `dagr` | `dagr context` slice output injected instead |

Grading is mechanical: the model's response is written into a scratch copy of
the repo and hidden tests must pass. No self-reported scores. Model output is
fence-stripped before grading (hosts do the same).

## Live results — 2026-08-24

Provider: OpenRouter (`openrouter/free` auto-routing tier), temperature 0.
Raw evidence: [`evals/results/latest.json`](../evals/results/latest.json).

| Task | Strategy | Pass | Defects | Prompt tokens in | Latency |
|---|---|---|---|---:|---:|
| task-001-fix-function | baseline | ✅ | 0 | 233 | 1.8s |
| task-001-fix-function | dagr | ✅ | 0 | 391 | 7.0s |
| task-002-add-validation | baseline | ✅ | 0 | 217 | 2.4s |
| task-002-add-validation | dagr | ✅ | 0 | 359 | 9.9s |
| task-003-refactor-import | baseline | ✅ | 0 | 224 | 3.4s |
| task-003-refactor-import | dagr | ✅ | 0 | 336 | 2.9s |

**Totals: baseline 3/3 · dagr 3/3 · defects 0/0**

### What this run shows

1. The harness works end-to-end against a real model: prompt → completion →
   mechanical grading, no human in the loop.
2. Slice-injected context **does not hurt task success** on these tasks.

### What this run does NOT show

- **No quality differentiation yet.** Both strategies pass because the
  fixtures are small and unambiguous. Differentiation requires tasks where
  context noise actually causes failures — long files, decoy helpers,
  misleading look-alike symbols.
- **Slicing cost more tokens here, not less.** The slice payload carries
  metadata + hoisted contracts, so on 5–15-line fixture files it exceeds
  whole-file size (e.g. 391 vs 233 tokens). Slicing's token advantage only
  materializes on large files — which the pilot set deliberately lacks.
  Adding large-file tasks is the top scaling priority.
- Free-tier latency variance (0.7s–10s) is provider rotation noise, not
  signal.

## Reproduce

```bash
node evals/run.mjs --provider mock                       # mechanics check, no key
OPENAI_BASE_URL=https://openrouter.ai/api/v1 \
OPENAI_API_KEY=sk-or-... \
node evals/run.mjs --provider openai --model openrouter/free
```

Errors surface loudly in `results/*.json` (`"error"` field per row); free
tiers 429 aggressively, requests retry with backoff.

<a name="caveats-read-before-citing"></a>
## Caveats (read before citing anything)

- **Model identity is non-reproducible on `openrouter/free`** — the router
  picks whichever free model is available per request. For reproducible runs
  pin a named free model when rate limits permit.
- Exact-string search still beats AST slicing for literal lookups.
- Compression percentages vary by file shape; small files compress less
  (this run literally demonstrates it — see token column above).
- Token figures are prompt *input* metrics, not dollar savings.
- Mock-mode results verify harness mechanics only — never cite them as model
  performance.

## Roadmap to a citable benchmark

| Step | Status |
|---|---|
| Mechanical grading harness | ✅ done |
| First live run published (this page) | ✅ done |
| Large-file + decoy-symbol tasks where context strategy matters | ⏳ next |
| ≥6 OSS repos / ≥100 hand-verified tasks, named-model reproducibility, grep & embedding baselines | ⏳ scale-up pending scope approval |
