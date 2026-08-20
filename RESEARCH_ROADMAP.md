# 🗺️ DAGR Research-Backed Architecture & Feature Roadmap

> **Living Agent Specification & Autonomous Task Ledger**  
> **Target Audience:** Autonomous AI Coding Agents & Systems Engineers  
> **Last Synchronized:** August 2026

---

## 📌 Mission & Architecture Philosophy

DAGR is the DAG-native symbolic AST slicing hypervisor, copy-on-write safety sandbox, and A2A swarm bus designed to slash AI token bloat by >95% and eliminate rogue agent workspace mutations.

This roadmap translates the latest 2025–2026 academic research breakthroughs into production-ready Rust primitives and web simulators. Any AI agent working in this repository must inspect this file, prioritize pending items, and update checkboxes upon task completion.

---

## 🔬 Research Foundations & Paper Mapping

| Research Initiative | Academic Source / Citation | Core Theory | Primary Rust Crate |
| :--- | :--- | :--- | :--- |
| **1. Multi-Rubric AST Slicing** | *LaMR* (`arXiv:2605.15315`, May 2026) | Decompose code into Core Logic vs. Structural Type Hoists with comment stripping | `crates/dagr-slicer` |
| **2. Parallel Branch Sandboxing** | *DeltaBox* (`arXiv:2605.22781`) & *BranchFS* (`arXiv:2602.08199`) | $O(1)$ CoW shadow workspace forking with first-commit-wins resolution | `crates/dagr-sandbox` |
| **3. Progressive Guard Gating** | *YoloFS* (`arXiv:2604.13536`, Aug 2026) | Real-time mutation staging, canonical path resolution, progressive intent | `crates/dagr-guard` |
| **4. Failure-Causal Test Slicing**| *CausalRepair* (`arXiv:2608.10613`, Aug 2026) | Slice minimal call-graph & failure trace on test failures | `crates/dagr-slicer` & `dagr-cli` |
| **5. AST Session Memory Cache** | *CodeMEM* (`arXiv:2601.02868`, Jan 2026) | In-memory AST dependency graph caching across multi-turn chat sessions | `crates/dagr-core` |

---

## 📋 Phased Execution Task Ledger

### 🟢 Phase 1: Research Showcase & Machine-Readable Agent Ledger
- [x] **Task 1.1:** Compile comprehensive literature review in [`docs/RESEARCH_PAPERS.md`](file:///Users/mm/orca/projects/ME/DAGR/docs/RESEARCH_PAPERS.md).
- [x] **Task 1.2:** Establish living [`RESEARCH_ROADMAP.md`](file:///Users/mm/orca/projects/ME/DAGR/RESEARCH_ROADMAP.md) in repository root.
- [x] **Task 1.3:** Cross-link roadmap in [`AGENTS.md`](file:///Users/mm/orca/projects/ME/DAGR/AGENTS.md) and [`README.md`](file:///Users/mm/orca/projects/ME/DAGR/README.md).

### 🟢 Phase 2: Web App Research Showcase & Interactive Simulators
- [x] **Task 2.1:** Add **Academic Research Citations & Trust Ribbon** to `site/index.html`.
- [x] **Task 2.2:** Add **Multi-Rubric AST Slicing Tier Switcher** (Raw vs Standard AST vs LaMR Tiered) to `site/index.html` and `site/js/slicer-engine.js`.
- [x] **Task 2.3:** Add **Parallel BranchFS Sandbox Visualizer** (`dagr branch fork`) to `site/index.html` and `site/js/app.js`.
- [x] **Task 2.4:** Add **Research Paper Library Explorer Modal** in `site/index.html`.

### 🟢 Phase 3: Rust Core Engine Enhancements
- [x] **Task 3.1:** In `crates/dagr-slicer`, implement `SliceTier` enum (`Core`, `StructuralHoists`, `DocstringsStripped`) and `slice_multi_rubric()`.
- [x] **Task 3.2:** In `crates/dagr-sandbox`, implement `BranchContext` and `fork_branch(count: usize)` with first-commit-wins atomic resolution.
- [x] **Task 3.3:** In `crates/dagr-guard`, implement `ProgressivePermissionGate` with canonical path resolution and `Safe/Warning/Restricted` staging.
- [x] **Task 3.4:** In `crates/dagr-cli`, expose `--tier`, `--from-test`, and `branch fork` subcommands.

### 🟢 Phase 4: Automated Verification & Test Coverage
- [x] **Task 4.1:** Add unit tests for Multi-Rubric slicing in `crates/dagr-slicer/tests` (6/6 tests passing).
- [x] **Task 4.2:** Add unit tests for parallel branch forking in `crates/dagr-sandbox/tests` (2/2 tests passing).
- [x] **Task 4.3:** Validate web application JS syntax (`node -c`) and verify on `http://localhost:8080`.

---

## 🛡️ Edge-Case Defense Standards

Every implementation must adhere to the **HyperPlan Edge-Case Defense Matrix**:
1. **Incomplete / Syntax-Broken Code:** Must use Tree-sitter fault-tolerant error nodes + regex structural fallback.
2. **Non-CoW Filesystems:** Must detect runtime capabilities and fall back to hardlink/tmpfs scratchpad without crashing.
3. **Path Traversal & Encoded Symlinks:** Must run `std::fs::canonicalize()` before evaluating security rules.
4. **Parallel Port Collisions:** Must inject dynamic port offset environment variables into spawned agent branches.
5. **Stale Lock Deadlocks:** Must use RAII handles with a 5-second automatic stale lock harvesting mechanism.
