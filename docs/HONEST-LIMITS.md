# Honest Limits

DAGR's competitors publish their failure modes; so do we. Everything below is
a known, current limitation — not marketing.

## Context slicing (`dagr context`)

- **Exact-string search still wins.** If you need every literal occurrence of
  a string (a URL, an error message), ripgrep/grep is faster and more precise
  than AST slicing. DAGR is for *symbol-level* questions.
- **Compression percentages vary by file shape.** A 97% reduction on one
  1,500-line file does not predict results elsewhere; small files compress
  less, dense files with many hoisted contracts compress more.
- **Cross-file contract hoisting performs a single effective hop** for
  relative imports (`--depth > 1` warns and behaves as 1).
- **Languages**: best support is TypeScript / TSX / Rust. JavaScript, Go and
  Python parse but receive less contract hoisting.

## Reverse index & `review-diff`

- **Dynamic languages are approximated.** Reference detection is identifier-
  based, not flow-based: dynamic dispatch, string-built imports, re-export
  barrels beyond one hop, and reflection are not tracked.
- **Alias imports (`@/...`) are skipped** by dangling-import detection in v0.
- **Risk scores are heuristics**, not calibrated probabilities. Weights
  (`DAGR_RISK_W_*`) exist so teams can tune them; treat scores as ranking,
  not truth. Any secret finding blocks regardless of score by design.
- Deleted-file analysis assumes git history is present (falls back to empty
  diff on shallow clones).

## Secrets & license scanning

- Token shapes cover common credential formats plus an entropy fallback;
  novel encodings can slip through, and entropy hits can be false positives.
- License checking validates **declared** licenses of the repository's own
  manifests only — it does not resolve transitive dependency licenses (no
  SBOM). That requires cargo-metadata integration we have deliberately not
  added yet.

## Audit export

- OTLP output covers core span attributes only (service name/version,
  run id, step, input hash) — it is not a full semantic-conventions
  implementation.
- SOC2 evidence lines provide hash chaining over recorded effects; they are
  evidence *input*, not a compliance certification.

## Experimental / parked

- **A2A swarm bus**: compiled out by default (`--features a2a`). Untested at
  scale; do not build production swarms on it yet.
- **Token savings as ROI**: model prices fall and hosts increasingly retrieve
  context themselves. Measure outcomes (task pass-rate, defect counts) via
  [`evals/`](../evals/) rather than trusting token deltas alone.
