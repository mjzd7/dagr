# Guard Rules Schema (`.dagr/rules.yaml`)

Strict, fail-closed parsing: unknown keys are hard errors naming the key.

```yaml
version: "1.0"            # required
project_name: my-repo     # optional
preset: clean-architecture # optional; seeds boundaries when none defined

boundaries:
  - name: UI-to-DB Boundary        # required
    from: "packages/web/src/**"    # required, glob over canonical relative paths
    cannot_import:                 # required list
      - "packages/core/src/db/**"
    message: Presentation layer must not import DB clients directly.  # optional

limits:
  max_file_lines: 500
  max_function_lines: 60
  disallow_eval: true

security:
  sanitize_prompt_injections: true
  strip_control_tokens: ["[INST]", "[/INST]"]
```

Behavior notes:

- Missing file → built-in clean-architecture preset (full enforcement).
- Invalid file → hard error naming the offending key/line; guard refuses to run.
- `preset:` with empty `boundaries` → preset rules seeded automatically.
- Matching is segment-aware (`src/db/**` never matches `src/db-migration/x`);
  relative specifiers resolve against the importer's directory; tsconfig/jsconfig
  path aliases resolve via root config; barrel re-exports followed one hop.

## Review-diff risk weights

`review-diff` reads optional env overrides (defaults shown):

```
DAGR_RISK_W_DANGLING_IMPORT=40   # per dangling import in a changed file
DAGR_RISK_W_GUARD_VIOLATION=25   # per boundary violation in a changed file
```

Secrets force BLOCK regardless of weights. Weights rank files; they are not
calibrated probabilities (see [HONEST-LIMITS.md](HONEST-LIMITS.md)).
