# 🔥 HYPERPLAN Scrutiny — TESTING_ROADMAP.md

> Adversarial multi-persona review executed inline (sequential, one-by-one; no sub-agents spawned).
> Subject: [`TESTING_ROADMAP.md`](../TESTING_ROADMAP.md) + current DAGR code state @ `c9bdcb2`+.
> Date: 2026-08-23 · Roster: Saboteur → Pedant → Performance Cynic → Pragmatist Skeptic → Visionary Contrarian → Lead synthesis.
> Each round was persisted to this file immediately upon completion; synthesis written only after all five rounds were saved.

---

## ROUND 1 — THE SABOTEUR (adversarial / security lens)

Verdict on the plan: *it tests that the guard catches honest mistakes; it never asks what a motivated adversary or a hostile input stream does.* Cross-critique note: none possible (first round).

| ID | Finding | Sev |
|---|---|---|
| S1 | **No sandbox-escape battery.** Plan drills CoW rollback with benign commands. Never tests: commands writing to absolute paths outside shadow, reading `~/.ssh`, symlink swaps inside `.dagr/shadow`, nested-git confusion during rollback. | S0 |
| S2 | **Hostile `rules.yaml` inputs untested**: duplicate mapping keys (serde = last-wins, silent!), 10MB+ files, deep-nesting recursion, BOM/CRLF, non-UTF8 bytes. Fail-open era ended (F1.1) but resource/hostile-input surface untested. | S1 |
| S3 | **Evasion corpus absent**: template-literal imports `` import(`${p}`) ``, computed specifiers, deliberate >1-hop re-export chains (F2.5 follows exactly one hop — adversarial chaining defeats it by design), import maps. Need a pinned adversarial fixture corpus with expected catch/miss labels. | S1 |
| S4 | **Path-traversal probes missing for readers**: barrel reader + cross-file hoister join workspace_root with resolved candidates. Lexical clamp exists, but zero tests assert that `../`-heavy specifiers read NOTHING outside the workspace root. | S0 |
| S5 | **Concurrency/TOCTOU untested**: file mutated between `exists()` and `read`; two dagr processes sharing one `.dagr/index.db` WAL while recording events; barrel_cache stampede on cold start. | S1 |

Proposed tests seeded: `EC-S1..S5` (see synthesis catalog).

---

## ROUND 2 — THE PEDANT (spec-correctness; cross-critiques Round 1)

Accepts S1–S4 as valid. Challenges **S5 scope**: TOCTOU on config files is inherent to non-atomic FS APIs everywhere in DAGR; demand one representative regression test (alias/tsconfig read race) rather than exhaustive coverage — otherwise the battery is unfalsifiable busywork. Accepted with that narrowing.

Plan-internal defects found:

| ID | Finding | Sev |
|---|---|---|
| P1 | Protocol step 4 promises violations "naming rule+**advice**" — `Violation` has no advice field (it has `message`). Spec↔code drift; harness asserting 'advice' would fail forever. | S1 |
| P2 | Metric contradiction: TESTING_ROADMAP targets ≥90% median compression; README markets 95%+. Neither defines denominator (does F3.2's cross-file contract payload count against sliced tokens?). Post-F3.2 this is now measurable and likely violated on some slices. | S1 |
| P3 | Regression-gate rows don't cite the exact test id that satisfies them (some do). Make citation mandatory — unauditable otherwise. | S2 |
| P4 | Wave repositories have no pinned ref/size/clone-profile → runs unreproducible; results not comparable across weeks. | S1 |
| P5 | Severity classes (S0–S3) exist but nothing binds severity → fix SLA/priority. Findings rot. | S2 |
| P6 | Dead link: plan points to `../Automate-Instagram-Posts/LearningsDAGR.md` — outside this repo, invisible to fresh clones/CI agents. Copy the canonical findings into `docs/`. | S2 |
| P7 | "`tools/list` expect ≥7" is brittle in both directions (addition passes silently, rename fails cryptically). Assert exact set equality. | S2 |

Cross-critique of Round 1 accepted-with-narrowing recorded above.

Proposed tests seeded: `EC-P1..P7`.

---

## ROUND 3 — THE PERFORMANCE CYNIC (cross-critiques Rounds 1–2)

Challenges Round 1: S1's sandbox battery is valuable but must be **hermetic** — tests that touch real `$HOME` or network will flake; demand tmpdir-jail design. Accepts with that constraint.
Challenges Round 2: P7's exact-set assertion makes every tool addition a test edit — accept friction deliberately (tool surface SHOULD be reviewed), but via an allowlist constant, not inline literals.

Plan-internal defects:

| ID | Finding | Sev |
|---|---|---|
| F1 | "<5ms P99 slice" is unmeasurable as specified: no hardware spec, warmup policy, percentile math, or outlier rule. Perf numbers without methodology are marketing. | S1 |
| F2 | Sequencing error: walker ignore-list audit (F4.4/H-W1) is deferred to Wave 1/4 data, yet scan cost explodes on the very first monorepo clone. Front-load ignore-list extension + `.gitignore` respect BEFORE Wave 1, or waves burn hours indexing junk. | S1 |
| F3 | Barrel cache (F2.5) and alias map are **unbounded memory** on huge repos. Need entry caps/LRU and a memory-ceiling test on synthetic 10k-import repos. | S1 |
| F4 | No overhead baseline: F2.5 barrel probing performs FS reads on direct-miss — clean-repo scans now pay probe cost. Plan never measures guard overhead vs pre-F2.5 baseline. | S1 |
| F5 | Compression metric is now **gameable/incorrect** post-F3.2: hoisted contracts ADD tokens. Metric must be net (contracts included), else "95% savings" claims are unfalsifiable. Add regression threshold per fixture. | S0-for-claims |

Proposed tests seeded: `EC-F1..F5`.

---

## ROUND 4 — THE PRAGMATIST SKEPTIC (ops/feasibility; cross-critiques Rounds 1–3)

Challenges Round 3: F1's methodology demand is right but full stats rigor is overkill for v0.x — adopt a minimal fixed protocol (N=50 warm iterations, p99, same machine class recorded) rather than blocking. Accepted-with-simplification.
Challenges Round 1: S2 hostile-YAML cases — verify serde_yaml actually safe against alias-expansion before writing tests nobody needs; one DoS-size test suffices.

Plan-internal defects:

| ID | Finding | Sev |
|---|---|---|
| R1 | Waves clone multi-GB repos (vscode ≈350MB+, kubernetes ≈GBs). No shallow/sparse strategy = CI-prohibitive cost. Specify `--depth 1` / sparse-checkout profiles per repo. | S1 |
| R2 | **Sequencing risk**: T0 (harness) is prerequisite for EVERY wave yet listed first-but-unbuilt. All wave checkboxes should be hard-blocked on T0 completion. | S0-process |
| R3 | Findings scatter across cloned repos' local files → lost. Centralize ledger in DAGR repo (`docs/findings/`) with index + template. | S1 |
| R4 | No offline-rule codification: python E2E suite is hermetic today *by luck*. Codify "no network in unit/e2e" or provider-dependent flakiness will rot the suite. | S2 |
| R5 | Version skew: waves testing moving `main` against moving repos gives unreproducible results. Pin wave runs to dagr release tags. | S1 |
| R6 | No wave exit-criteria: how many findings / what duration ends a wave? Timebox + finding-quota needed or waves never 'complete'. | S2 |

Proposed tests/processes seeded: `EC-R1..R6`.

---

## ROUND 5 — THE VISIONARY CONTRARIAN (coverage blindspots; cross-critiques Rounds 1–4)

Challenges Round 3: F3's LRU cap adds code before evidence — first MEASURE memory on a 10k-import synthetic; only cap if it exceeds a stated budget. Accepted-with-measurement-first.
Challenges Round 4: R5's release-tag pinning slows feedback on hotfixes — allow `main` runs labeled 'exploratory', releases for 'comparable' runs. Accepted-with-labeling.

Missing dimensions the plan never mentions:

| ID | Finding | Sev |
|---|---|---|
| V1 | **No property-based/fuzz testing** of the two most attack-exposed pure surfaces: `extract_imported_module` and `resolve_relative_candidates`. Invariants: never panic on arbitrary bytes; extracted module is always a quoted substring of input (when quoted form); resolver output never contains `..` segments. proptest or hand-rolled seeded fuzz loop (stdlib rand via hash of index — zero deps). | S0 |
| V2 | **Determinism/snapshot tests absent**: same slice twice → byte-identical JSON. Would have caught HashMap-iteration nondeterminism risks (alias exact map, barrel cache). Golden-file hashes. | S1 |
| V3 | Language waves are clone-and-run only; no **real-idiom corpora**: Python relative imports (`from . import x`), Rust `mod`/`pub use` trees, Go module replace directives. Extractor unit fixtures exist; corpus-level fixtures per language don't. | S1 |
| V4 | Infra failure-injection gaps: disk-full during CoW write, permission-denied on tsconfig read, **corrupted index.db** (observed LIVE this session!), WAL lock contention. Wave 0 matrix lists none of the DB cases. | S0 |
| V5 | **Migration matrix**: telemetry migration (this session) has exactly one test. Needs: column-exists-but-NULL rows, double-migration idempotency, future v-next partial columns. | S1 |
| V6 | Concurrency story (A2A selling point) has zero tests: two processes slicing/recording simultaneously on one workspace. | S1 |
| V7 | Hostile filesystem matrix: emoji/unicode filenames, spaces in paths, >260-char paths, CRLF sources, UTF-8-BOM sources — extractor/resolver/walker all untested against these. | S1 |

Proposed tests seeded: `EC-V1..V7`.

---

## LEAD SYNTHESIS (written only after all five rounds persisted)

### Edge-Case Test Catalog (consolidated, deduped, prioritized)

**P0 — ship before any Wave:**
- `EC-S1` sandbox-escape battery (tmpdir-jail, hermetic): absolute-path writes, `$HOME` reads, shadow-symlink swap, nested-git rollback.
- `EC-S4` traversal-read proof: `../`-heavy specifiers + barrel/hoister readers touch nothing outside workspace root (fs-walk assertion after probe).
- `EC-V1` property/fuzz harness for extractor+resolver: no-panic invariant + quoted-substring invariant + no-`..`-in-output invariant (seeded, deterministic).
- `EC-V4` infra-failure battery: corrupted index.db (garbage bytes, truncated WAL), permission-denied tsconfig, disk-full simulation on CoW commit path.
- `EC-R2` process gate: waves blocked on T0 harness existence (CI assert).

**P1:**
- `EC-S2` hostile-YAML: duplicate keys, 10MB, CRLF/BOM, non-UTF8 (serde_yaml alias-safety verified once, then one DoS-size case).
- `EC-S3` adversarial import corpus with expected catch/miss labels (incl. >1-hop chains documented as known-miss).
- `EC-V2` determinism goldens: slice twice → identical bytes (alias/barrel iteration order).
- `EC-V5` migration matrix (NULL-column rows, idempotent double-run, future-partial).
- `EC-V6` two-process concurrency pair-test on one workspace.
- `EC-V7` unicode/space/long-path/CRLF/BOM source matrix through extractor+walker.
- `EC-F5` net-token metric definition + per-fixture regression threshold (post-F3.2).
- `EC-F2` front-load walker ignore-list extension (build/out/.output/.turbo/.venv/__pycache__/vendor) BEFORE Wave 1.
- `EC-F1` minimal perf protocol (N=50 warm, p99, machine-class recorded).
- `EC-P1/P2/P6/P7` doc↔code consistency asserts: 'advice'→'message' wording fix; compression denominator definition; findings copied in-repo; tools/list exact-set allowlist.

**P2:** EC-F3 measure-then-cap memory; EC-R1 shallow/sparse profiles; EC-R3 centralized findings dir; EC-R4 offline codification; EC-R5 labeled main-vs-release runs; EC-R6 wave timeboxes; EC-P3/P5 gate-citations + severity-SLA mapping; EC-S5 narrowed race test; EC-V3 idiom corpora.

### Learnings & Feedback from this scrutiny exercise

1. **The plan inherited strong protocol DNA from real failures (P1–P5 principles) but its negative space was empty**: everything tests what the tool SHOULD do; nothing attacked what hostile inputs, broken infrastructure, or adversaries do. Five of eight P0 items are adversarial/infra classes.
2. **Spec↔code drift is already real**: 'advice' field, 95%-vs-90% compression, tool-count brittleness, dead external link. Docs rot faster than code; consistency asserts belong in CI, not in reviewer memory.
3. **Sequencing was inverted**: F4.4 (walker audit) was deferred for data while Wave 1 cannot run sanely without it; T0 harness gates everything but wasn't enforced as a blocker. Plans need dependency edges, not just checklists.
4. **Metrics lacked operational definitions** (percentiles, denominators, hardware) — post-F3.2 the compression metric actively became wrong. Every metric needs: formula + denominator + measurement protocol + regression threshold, or delete it.
5. **The enforcement culture proved itself mid-scrutiny**: the pre-commit cargo gate surfaced the shadowed opencode arm (a real path-loss bug from F4.1) while preparing this very review — scrutiny tooling pays for itself outside the review too.
6. **Inline sequential panel (user-mandated) worked well**: cross-critique fidelity was high because each persona saw prior rounds; zero spawn overhead; incremental file persistence enabled crash-safe resume. Trade-off: no true independence — a real subagent pass later should VERIFY these findings rather than regenerate them.
7. **Feedback loop into IMPROVEMENT_ROADMAP**: several scrutiny items (walker front-load, metric redefinition) are code changes, not just tests — they must be filed as F-tickets, not left as test wishes.

*End of scrutiny transcript. All five rounds persisted prior to this synthesis, per protocol.*
