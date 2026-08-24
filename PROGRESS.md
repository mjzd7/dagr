# DAGR Completion Plan — Progress Tracker

> Execution mode: sequential, single-chat. Zero new dependencies (ponytail).
> Gates per wave: `cargo test --workspace` exit 0 · `cargo clippy -D warnings` · `dagr guard` 0 violations · `scripts/ponytail_guard.sh` clean.
> Decisions ratified: wedge = **governance for AI-written code** · license = **Apache-2.0** · pillars parked, not deleted.

## Wave A — Foundation
| Task | Status | Evidence |
|---|---|---|
| A1 Relicense Apache-2.0 | ✅ | LICENSE+NOTICE.md+workspace Cargo.toml+package.json+README badge+CONTRIBUTING; cargo metadata all Apache-2.0 |
| A2 Park A2A behind feature flag | ✅ | `[features] a2a=[]` default-off; tools.rs+server.rs cfg-gated; default=15 tests (4 tools), `--features a2a`=16 tests (7 tools) |
| A3 Positioning lite (Cargo desc, BRAND, README header) | ✅ | Cargo desc + README header tagline + BRAND mindmap + mcp crate desc |
| Wave A gates + commit | ✅ | commit b6b92ff: 150 tests pass, ponytail clean, guard 0 violations |

## Wave B — Tier 0 Governance Surface
| Task | Status | Evidence |
|---|---|---|
| B1 ReverseIndex (TS/JS/Rust) | ✅ | reverse_index.rs; 4 tests: cross-file callers, dangling detection, import-suffix match, rust structs; import sites count as refs by design |
| B2 Secret scan (regex+entropy) | ✅ | secrets.rs: hand-rolled token shapes (AWS/GH/OpenAI/Anthropic/PK header/bearer) + Shannon entropy >=4.5; findings carry Blake3 hash prefix only; 7 tests |
| B3 License manifest scan | ✅ | licenses.rs: declared-license allowlist check (Cargo+package.json), workspace-inheritance aware, 4 tests |
| B4 `dagr prove` receipts | ✅ | governance.rs: Blake3 digest deterministic w.r.t. content; markdown+json+plain render; sandbox --test opt; smoke-tested VERIFIED receipt |
| B5 `dagr review-diff` verdicts | ✅ | dangling-import detection via ReverseIndex + named bindings; risk scores; PASS/BLOCKED exit codes; e2e git-fixture tests green; live smoke: BLOCKED exit=1 |
| B6 GitHub Action merge gate | ✅ | action.yml rewritten as DAGR Merge Gate: install + review-diff, GITHUB_STEP_SUMMARY markdown, YAML-validated |
| Wave B gates + commit | ⏳ | |

## Wave C — Trust & Evidence
| Task | Status | Evidence |
|---|---|---|
| C1 Pilot eval harness (zero-dep Node) | ✅ | evals/{run.mjs,lib/,tasks/×3}; mock 3/3 pass defects=0 both strategies; results/latest.json; slice-injection path exercises real `dagr context` |
| C2 Audit export (JSONL/OTLP/SOC2) | ✅ | dagr-core/src/audit_export.rs: 4 tests incl. OTLP conformance (traceId=32hex) + hash-chain verification |
| C3 Explainability v1 on resolver | ✅ | Resolution{via,confidence}; stderr provenance line + additive `resolution` key in context JSON; smoke-tested |
| C4 HONEST-LIMITS.md | ✅ | docs/HONEST-LIMITS.md covers slicing/reverse-index/scanning/audit/experimental limits; linked from README header + Why section |
| C5 README rewrite | ✅ | governance-led exec summary; strawman panel + \$-savings claims removed; FinOps matrix → outcome-metrics table citing artifacts; governance quickstart + Action snippet |
| Wave C gates + commit | ⏳ | |

## Wave D — Completeness
| Task | Status | Evidence |
|---|---|---|
| D1 Agent identity registry + `dagr revoke` | ✅ | dagr-core/registry.rs (.dagr/agents.json, atomic rename, expiry); 3 tests; CLI agent register/list + revoke smoke-tested live |
| D2 Per-agent cost attribution | ✅ | MCP tools accept `_agent`; validated against registry (revoked/expired rejected) then telemetry tagged mcp:<id> so stats client breakdown shows per-agent rows |
| D3 COMPLIANCE-MAPPING.md | ✅ | docs/COMPLIANCE-MAPPING.md: EU AI Act / NIST AI RMF / ISO 42001 / OWASP / SOC2 rows each naming artifact + verification path |
| D4 Docs split | ✅ | docs/getting-started.md + mcp-tools.md (+ _agent contract) + rules-schema.md (+ risk weight env vars); site/ untouched |
| D5 `dagr doctor` | ✅ | grammar/CoW-fs/SQLite-WAL(temp-file probe)/rules/IDE-config checks; pretty+json; exit 1 on hard failure; live run green |
| D6 Demo app (`evals/demo-app`) | ✅ | planted UI-to-DB violation caught by guard (1 violation); deletion flow BLOCKED with 2 dangling imports in scratch clone |
| FINAL verification sweep + commit | ✅ | 176/0 tests · mock eval 3/3 defects=0 · ponytail clean · dagr guard 0 violations · commits b6b92ff→eac52dd→c3a7a42→51c2844 |

## Deferred (disclosed)
- LSP bridge (tsserver/rust-analyzer shell-out) → docs/HONEST-LIMITS.md
- Benchmark expansion ≥6 repos / ≥100 tasks → run evals/ live when an API key is available; publish results page
- Live-provider eval runs (need ANTHROPIC_API_KEY / OPENAI_API_KEY in env)

**Status: ALL WAVES COMPLETE.** Wave A b6b92ff · B eac52dd · C c3a7a42 · D 51c2844

---
*Updated after every task. Learnings live in [LEARNINGS.md](LEARNINGS.md).*
