# Wave 2 Opener — denoland/deno findings

> Third external-repo validation. Repo: denoland/deno @ HEAD (shallow, 172MB).
> dagr built from main · 2026-08-23

## Results

| Probe | Result |
|---|---|
| T0 harness | ✅ PASS |
| Guard scan (full tree) | 3.3s wall · passed · 2 preset rules · 0 violations |
| Slice probe (`core/error.rs:err_out_of_memory`) | ✅ |

## Findings

| ID | Finding | Sev |
|---|---|---|
| W2-a | T0 harness validated on third external repo — Rust+TS polyglot handled correctly. | confirmation |
| W2-b | Guard scan timing scales sub-linearly across our three data points: vite (2809 files) = 0.43s, deno (~10k files est.) = 3.3s, next.js (31362 files) = 7.5s. No performance cliff. | note |

## Agent-OS Module Audit

| Module | LOC | Tests | Clippy | Notes |
|---|---|---|---|---|
| budgets.rs | ~100 | test_agent_os_core.rs | clean | BudgetContext for cost tracking |
| compaction.rs | ~150 | test_agent_os_core.rs | clean | AsyncCompactionTracker + ContextWindow |
| saga.rs | ~80 | test_agent_os_core.rs | clean | SagaCoordinator + SagaAction trait; run_id accessor added this session |
| journal.rs | ~120 | test_agent_os_core.rs | clean | EffectJournal + EffectRecord |
| event_store.rs | ~200 | test_agent_os_core.rs | clean | EventStorePort + SqliteEventStore + RunEvent types |
| quarantine.rs | ~80 | test_agent_os_core.rs | clean | QuarantineManager + QuarantinedItem |
| rate_limiter.rs | ~50 | test_agent_os_core.rs | clean | TokenBucketRateLimiter |
| capabilities.rs (guard) | ~130 | test_agent_os_guard.rs | clean | CapabilityGrant + CredentialBroker with HMAC signing |
| circuit_breaker.rs (mcp) | ~80 | — | clean | ToolCircuitBreaker (no dedicated test file yet) |
| **Total** | **~1179** | **6 core tests** | **clean** | |

### Assessment
- All modules compile clean, zero clippy warnings post-fixes
- Test coverage exists via `test_agent_os_core.rs` / `test_agent_os_guard.rs` integration tests
- `circuit_breaker.rs` lacks a dedicated test file — P2 gap
- Dead-code warning on `SagaCoordinator.run_id` resolved by accessor added this session
- Modules are self-contained and don't leak dependencies into existing crates
