# DAGR Completion Plan — Progress Tracker

> Execution mode: sequential, single-chat. Zero new dependencies (ponytail).
> Gates per wave: `cargo test --workspace` exit 0 · `cargo clippy -D warnings` · `dagr guard` 0 violations · `scripts/ponytail_guard.sh` clean.
> Decisions ratified: wedge = **governance for AI-written code** · license = **Apache-2.0** · pillars parked, not deleted.

## Wave A — Foundation
| Task | Status | Evidence |
|---|---|---|
| A1 Relicense Apache-2.0 | ✅ | LICENSE+NOTICE.md+workspace Cargo.toml+package.json+README badge/§License+CONTRIBUTING; `cargo metadata` → all pkgs Apache-2.0 |
| A2 Park A2A behind feature flag | ✅ | `[features] a2a=[]` default-off; tools.rs+server.rs cfg-gated; default=15 tests (4 tools), `--features a2a`=16 tests (7 tools) |
| A3 Positioning lite (Cargo desc, BRAND, README header) | ⏳ | |
| Wave A gates + commit | ⏳ | |

## Wave B — Tier 0 Governance Surface
| Task | Status | Evidence |
|---|---|---|
| B1 ReverseIndex (TS/JS/Rust) | ⏳ | |
| B2 Secret scan (regex+entropy) | ⏳ | |
| B3 License manifest scan | ⏳ | |
| B4 `dagr prove` receipts | ⏳ | |
| B5 `dagr review-diff` verdicts | ⏳ | |
| B6 GitHub Action merge gate | ⏳ | |
| Wave B gates + commit | ⏳ | |

## Wave C — Trust & Evidence
| Task | Status | Evidence |
|---|---|---|
| C1 Pilot eval harness (zero-dep Node) | ⏳ | |
| C2 Audit export (JSONL/OTLP/SOC2) | ⏳ | |
| C3 Explainability v1 on resolver | ⏳ | |
| C4 HONEST-LIMITS.md | ⏳ | |
| C5 README rewrite | ⏳ | |
| Wave C gates + commit | ⏳ | |

## Wave D — Completeness
| Task | Status | Evidence |
|---|---|---|
| D1 Agent identity registry + `dagr revoke` | ⏳ | |
| D2 Per-agent cost attribution | ⏳ | |
| D3 COMPLIANCE-MAPPING.md | ⏳ | |
| D4 Docs split | ⏳ | |
| D5 `dagr doctor` | ⏳ | |
| D6 Demo app (`evals/demo-app`) | ⏳ | |
| FINAL verification sweep + commit | ⏳ | |

## Deferred (disclosed)
- LSP bridge (tsserver/rust-analyzer shell-out) → see docs/HONEST-LIMITS.md
- Benchmark expansion ≥6 repos / ≥100 tasks → docs/benchmark.md

---
*Updated after every task. Learnings live in [LEARNINGS.md](LEARNINGS.md).*
