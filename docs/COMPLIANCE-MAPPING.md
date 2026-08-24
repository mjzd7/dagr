# Compliance Mapping

How DAGR artifacts map onto the frameworks security teams are audited
against. This page is written for evidence *gatherers*: each row names the
framework expectation, the concrete DAGR mechanism, and where an auditor can
verify it in this repository.

> DAGR does not certify compliance for you — it produces artifacts your
> compliance process can consume.

## EU AI Act (risk-tiered obligations for agentic systems)

| Expectation | DAGR mechanism | Verifiable at |
|---|---|---|
| Logging of agent decisions/actions | `EffectJournal` records every non-deterministic effect with input hash + timestamp | `crates/dagr-core/src/journal.rs` |
| Audit-trail export for authorities | `export_audit(Otlp \| Jsonl \| Soc2Evidence)` | `crates/dagr-core/src/audit_export.rs` |
| Human oversight of autonomous action | Merge gate verdicts require human-approved CI policy; agent registrations bind to human owners | `dagr review-diff`, `.dagr/agents.json` |

## NIST AI RMF (Govern / Map / Measure / Manage)

| Function | DAGR contribution | Verifiable at |
|---|---|---|
| GOVERN — accountability structures | Per-agent identity → owner mapping; revocation | `AgentRegistry`, `dagr revoke` |
| MEASURE — measurable behavior | Pilot eval harness: pass-rate/defect counts per strategy | [`evals/`](../evals/) |
| MANAGE — bounded risk | Risk-ranked diffs with configurable weights; secrets force-block | `ReviewVerdict`, `DAGR_RISK_W_*` |

## ISO/IEC 42001 (AI management systems)

| Control theme | DAGR artifact |
|---|---|
| Documented responsibility assignment | `agent register <id> --owner <human>` records |
| Operational logging & traceability | Effect journal + audit exports (hash-chained SOC2 lines) |
| Change management | `review-diff` verdicts attached to PR checks (`action.yml`) |
| Transparency of limitations | [`HONEST-LIMITS.md`](HONEST-LIMITS.md) |

## OWASP Agentic / LLM top risks

| Risk | Mitigation surface |
|---|---|
| Excessive agency | Capability-scoped execution inside CoW sandbox; atomic rollback on failure (`dagr-sandbox`) |
| Prompt-injection-driven boundary escape | Import-boundary enforcement on every proposed import (`dagr-guard` sanitizer + checker) |
| Credential leakage | Secret token-shape + entropy scanning on diffs (`dagr-guard/secrets.rs`); findings carry hashes, never raw secrets |
| Untraceable multi-agent action | Agent identity registry + per-agent telemetry tagging (`_agent` argument → `mcp:<id>` client rows in stats) |

## SOC 2 (auditor evidence requests)

Auditors increasingly ask for "what did the agent do" logs as change-management
evidence. Produce them with:

```bash
# hash-chained per-action evidence lines (JSONL)
sqlite3 .dagr/journal.db ".dump" >/dev/null   # existence check
dagr prove --test "cargo test"                # point-in-time control snapshot
```

The `Soc2Evidence` export emits one line per recorded action:
`actor`, `action`, `object_integrity` (input Blake3), `result`,
`timestamp_utc_iso`, plus a `prior_entry_hash → entry_hash` chain so tampering
is detectable.

## Gartner's cancellation driver: "inadequate risk controls"

The four controls above (identity, scoped execution, immutable trail,
merge gating) map 1:1 to the governance primitives cited across enterprise
deployment guides. DAGR implements them locally-first; no cloud dependency.
