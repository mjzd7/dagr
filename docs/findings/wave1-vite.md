# Wave 1 Opener — vitejs/vite findings

> First external-repo validation of the T0 harness and F3.x features.
> Repo: vitejs/vite @ HEAD (shallow clone, 39MB) · dagr built from main · 2026-08-23

## Results

| Probe | Result |
|---|---|
| T0 harness (guard + slice probe + MCP parity) | ✅ PASS on external repo |
| Slice fidelity (`getNodeAssetAttributes`, real vite node code) | ✅ sliced; **3 same-file contracts hoisted** (`HtmlAssetAttribute`, `HtmlAssetSourceFilterData`, `HtmlAssetSource`) |
| Compression | **46.5%** — below the 85% gate |
| Guard scan (full tree, 2809 files) | 0.43s wall · 0 violations · 2 preset rules |

## Findings

| ID | Finding | Sev |
|---|---|---|
| W1-a | T0 harness validated on first external monorepo — protocol works end-to-end outside DAGR's own fixtures. | confirmation |
| W1-b | Contract hoisting produces correct interfaces from production TypeScript (not synthetic fixtures). | confirmation |
| W1-c | **Small-file compression falls below programmatic gates** (46.5% vs ≥85%): hoisted contracts dominate when the target file is only 189 lines. This is the EC-F5 net-of-contracts nuance materializing — gates need a minimum-file-size qualifier, or a contracts-excluded secondary metric. | S2 |
| W1-d | Guard scan performance fine at vite scale (0.43s); H-W1 remains open for larger trees (next.js). | note |
| W1-e | Vite has **no tsconfig `paths` aliases** — H-TS1 unconfirmable here. next.js (definitive alias repo) moves to front of Wave 1 queue. | note |

## Actions flowing to IMPROVEMENT_ROADMAP

- Amend compression gates with file-size qualifier (from W1-c).
- Re-order Wave 1: next.js before vite for alias coverage.
