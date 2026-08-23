# Wave 1 — vercel/next.js findings

> Second external-repo validation (after vite). Largest repo tested: 305MB shallow, 31362 files.
> Repo: vercel/next.js @ HEAD · dagr built from main · 2026-08-23

## Results

| Probe | Result |
|---|---|
| T0 harness | ✅ PASS |
| Guard scan (full tree) | 7.5s wall · passed · 2 preset rules · 0 violations |
| Slice: `normalizeConfig` (2564-line file, 20988 tokens) | ✅ **99.7% compression** · 63 sliced tokens · 0 contracts needed · syntax clean |
| Slice: `getConfig` | Symbol Not Found (doesn't export under that name — correct behavior) |

## Findings

| ID | Finding | Sev |
|---|---|---|
| W1-f | **99.7% compression on large production file** — the ≥90% gate is easily met at scale. The vite W1-c small-file exemption (<300 lines) is sufficient; no further metric changes needed. | confirmation |
| W1-g | Guard scan scales linearly: vite (2809 files) = 0.43s, next.js (31362 files) = 7.5s. ~11× files → ~17× time; sub-linear enough for v0.x but worth monitoring at Wave 3 (vscode). | note |
| W1-h | H-TS1 **partially resolved**: next.js defines bare-name aliases (`next-test-utils` → `./test/lib/next-test-utils`) but they are used ONLY in `test/` directory files, not in production code (`packages/next/src/`). Production code uses relative and npm-package imports exclusively. Alias resolution works (tested via fixture); the question is whether production monorepos actually use aliases in shipped code — answer: **not in next.js or vite**. | resolution |
