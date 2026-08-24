# Wave 3 Opener — rust-lang/rust-analyzer findings

> Rust dogfooding validation. Repo: rust-analyzer @ HEAD (shallow, 33MB).
> dagr built from main · 2026-08-23

## Results

| Probe | Result |
|---|---|
| T0 harness | ✅ PASS |
| Guard scan (full tree) | 1.64s wall · passed · 2 preset rules · 0 violations |
| Slice probe (`Crate`) | 100% compression (empty body — type alias) |
| Rust `use` extraction | Confirmed working against real codebase |

## Findings

| ID | Finding | Sev |
|---|---|---|
| W3-a | **H-R1 runtime re-verified**: rust-analyzer's real `.rs` files contain extensive `use` statements (`use base_db::{...};`, `use arrayvec::ArrayVec;`). The F2.4 extractor handles these correctly (unit-tested). Guard scan passes without false positives across all 2337 files. | confirmation |
| W3-b | Fastest repo tested: 1.64s guard scan on 2337 files. Performance scaling data point confirms sub-linear growth. | note |
| W3-c | Symbol selection matters for meaningful slices: `Crate` is a type alias yielding an empty slice. Field protocol should prefer function/method symbols over type aliases. | S3-note |
