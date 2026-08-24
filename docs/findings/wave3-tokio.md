# Wave 3 — tokio-rs/tokio findings

> Rust workspace validation. Repo: tokio @ HEAD (shallow, 9.5MB).
> dagr built from main · 2026-08-23

## Results

| Probe | Result |
|---|---|
| T0 harness | ✅ PASS |
| Guard scan | 0.215s wall · passed · 2 preset rules · 0 violations |
| Slice: `Runtime` from `runtime.rs` | ✅ target resolved (`run`), 61.7% compression |

## Findings

| ID | Finding | Sev |
|---|---|---|
| W3-d | **Fastest guard scan of all repos tested** — 0.21s on 868 files. Confirms sub-linear scaling: tokio < rust-analyzer < deno < next.js in both size and scan time. | note |
| W3-e | Trait-heavy generics (tokio's `Runtime` wraps boxed trait objects) slice correctly but yield 0 contracts — generic type parameters aren't tracked by the contract hoister. This is a known F3.2 v1 scope boundary. | S2-note |
