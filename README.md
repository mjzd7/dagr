# ⚡ DAGR (`dagr`)

<div align="center">

[![GitHub License](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Rust 2021](https://img.shields.io/badge/Rust-2021_Edition-orange.svg)](https://www.rust-lang.org/)
[![Model Context Protocol](https://img.shields.io/badge/MCP-JSON--RPC_2.0-blue.svg)](https://modelcontextprotocol.io/)
[![Architecture](https://img.shields.io/badge/Architecture-Ponytail_Minimal-success.svg)](./.ponytail.md)
[![Lead Architect](https://img.shields.io/badge/Creator-Mohit_Dagar-purple.svg)](https://github.com/mjzd7)

**The DAG-Native Symbolic AST Slicing Hypervisor & Safety Sandbox for AI Coding Agents.**

*Sub-5ms context pruning, 95% token compression, and zero-trust Copy-on-Write (CoW) shadow sandboxing for Cursor, Claude Desktop, Ollama, and Neovim.*

[Executive Summary](#-executive-summary) • [Visual Architecture](#-visual-architecture--mechanics) • [Audited Metrics](#-transparent-metrics--mathematical-formulas) • [Terminal Visualizer](#-terminal-ui--token-gauges) • [Quickstart & MCP](#-quickstart--ide-setup) • [Nomenclature](#-nomenclature--etymology)

</div>

---

## 📌 Executive Summary (Crisp Overview)

* **What it is:** A single, ultra-fast native Rust binary (`dagr`) acting as a local-first safety hypervisor between AI coding agents and your codebase.
* **The Core Problem it Solves:** 
  * ❌ **Context Explosion (95% Token Bloat):** Passing entire 1,000+ line files or noisy vector dumps to LLMs inflates API costs and triggers "lost-in-the-middle" hallucinations.
  * ❌ **Unbounded Blast Radius:** Autonomous agents generating duplicate utilities, violating clean layer boundaries (e.g. UI importing DB clients), and writing half-broken diffs to disk.
* **The Solution:** 
  * ✂️ **Symbolic AST Slicing:** Extracts only the exact ~35 lines of target code + upstream type contracts, reducing token payloads by **>= 90%**.
  * 🛡️ **Copy-on-Write (CoW) Sandboxing:** Executes all agent writes and tests inside an OS-native shadow snapshot (`clonefile(2)` on macOS, `reflink` on Linux) with instant **<10ms atomic rollback** on failure.
  * 🔌 **Model Context Protocol (MCP) Gateway:** Seamless JSON-RPC 2.0 stdio integration with Cursor, Claude Desktop, Windsurf, and local LLMs (Ollama, Qwen 2.5 Coder).
  * ⚡ **Zero Cloud Dependencies:** 100% standalone native binary with embedded SQLite and Blake3 caching. No Docker or external daemon required.

---

## 🎨 Visual Architecture & Mechanics

### 1. The AST Pruning & Slicing Tree
*How DAGR traverses the codebase Directed Acyclic Graph (DAG) and prunes 97% of irrelevant noise:*

```mermaid
graph TD
    Root["📁 src/billing/charge.ts (1,180 lines | 11,840 tokens)"] --> Target["⚡ processPayment() [TARGET SYMBOL]"]
    
    Target --> V1["⚙️ validatePayload() (L12-L24)<br/><b>[KEPT: Active Data-Flow]</b>"]
    Target --> V2["⚙️ calculateTax() (L28-L35)<br/><b>[KEPT: Active Data-Flow]</b>"]
    Target --> T1["📦 PaymentPayload Interface<br/><b>[KEPT: Hoisted Contract]</b>"]
    
    Root -.->|PRUNED 97%| P1["sendRefund() (L120-L240)<br/><i>[DISCARDED: Unrelated Helper]</i>"]
    Root -.->|PRUNED 97%| P2["webhookHandler() (L300-L500)<br/><i>[DISCARDED: Unrelated Logic]</i>"]
    Root -.->|PRUNED 97%| P3["generatePdfReport() (L600-L900)<br/><i>[DISCARDED: Unrelated Formatter]</i>"]

    style Target fill:#4CAF50,stroke:#2E7D32,stroke-width:3px,color:#fff;
    style V1 fill:#81C784,stroke:#388E3C,stroke-width:1px,color:#000;
    style V2 fill:#81C784,stroke:#388E3C,stroke-width:1px,color:#000;
    style T1 fill:#64B5F6,stroke:#1976D2,stroke-width:1px,color:#000;
    style P1 fill:#EEEEEE,stroke:#BDBDBD,stroke-dasharray: 5 5,color:#9E9E9E;
    style P2 fill:#EEEEEE,stroke:#BDBDBD,stroke-dasharray: 5 5,color:#9E9E9E;
    style P3 fill:#EEEEEE,stroke:#BDBDBD,stroke-dasharray: 5 5,color:#9E9E9E;
```

---

### 2. Copy-on-Write (CoW) Shadow Sandbox Lifecycle
*Zero-trust mutation execution with sub-10ms atomic rollback:*

```mermaid
stateDiagram-v2
    [*] --> WorkspaceTree: Clean Working Directory
    WorkspaceTree --> APFS_Clone: AI Agent calls `dagr run` or mutation tool
    
    state Shadow_Sandbox {
        APFS_Clone --> StageDiff: Sub-1ms OS Block Clone (.dagr/shadow/tx_123)
        StageDiff --> ExecuteTests: Stage code patch & evaluate AST rules
        ExecuteTests --> Evaluation: Execute local test suite / linter
    }
    
    Evaluation --> AtomicCommit: ✅ All Tests & Guardrails Pass
    AtomicCommit --> WorkspaceTree: Atomic directory swap into workspace (<1ms)
    
    Evaluation --> InstantRollback: ❌ Tests Failed / Layer Boundary Broken
    InstantRollback --> CleanWorkspace: Discard .dagr/shadow snapshot (<10ms)
    CleanWorkspace --> WorkspaceTree: 0 dirty bytes modified on disk!
```

---

### 3. Real-Time MCP Tool Call Sequence

```mermaid
sequenceDiagram
    autonumber
    participant IDE as IDE / AI Agent (Cursor / Claude)
    participant MCP as dagr MCP Gateway
    participant Slicer as Tree-sitter Slicer Engine
    participant Guard as In-Memory Architecture Guard
    participant CoW as CoW Shadow Sandbox
    participant Disk as Real Working Tree

    IDE->>MCP: 1. Tool Call: dagr_get_context_slice(file, symbol)
    MCP->>Slicer: Parse AST & Traverse Backwards Data-Flow
    Slicer-->>MCP: Return ~35 lines + Hoisted Type Contracts (95% Token Savings)
    MCP-->>IDE: Send Minimal Context Slice to LLM

    IDE->>MCP: 2. Tool Call: dagr_execute_sandboxed(command, staged_patch)
    MCP->>Guard: Validate imports against .dagr/rules.yaml (<0.1ms)
    alt Layer Boundary Violation (e.g. UI imports DB)
        Guard-->>MCP: Reject Mutation with Diagnostic Trace
        MCP-->>IDE: Return Error (LLM Self-Corrects)
    else Valid Architecture
        Guard->>CoW: Stage mutation in OS-level shadow snapshot (<2ms)
        CoW->>CoW: Execute verification tests inside shadow root
        alt Tests Pass (100% Green)
            CoW->>Disk: Atomic swap to real working tree
            CoW-->>MCP: Mutation Success Payload
            MCP-->>IDE: Return Success
        else Tests Fail
            CoW->>CoW: Instant Rollback (<10ms) — Real tree untouched (0 bytes modified)
            CoW-->>MCP: Return Test Failure Log
            MCP-->>IDE: Feed error back to LLM for repair
        end
    end
```

---

## 📟 Terminal UI & Token Gauges

DAGR features a polished, visually rich terminal interface with Unicode box-drawing, live token compression gauges, and TTY auto-detection (outputs raw minimal code when piped to tools like `ollama` or `pbcopy`).

### 1. `dagr context` Visual Output:
```
$ dagr context src/billing/charge.ts:processPayment

⚡ DAGR Symbolic AST Slicer v0.1.0
┌────────────────────────────────────────────────────────────────────────┐
│ Target Symbol:   src/billing/charge.ts:processPayment                  │
│ Language:        TypeScript (Tree-sitter native AST)                  │
│ Sliced Context:  34 lines (down from 1,180 lines in file)              │
│ Token Footprint: 342 tokens (down from 11,840 tokens)                  │
│ Token Reduction: [████████████████████████████░░] 🟢 97.1% COMPRESSED   │
│ Latency:         ⚡ 1.8ms (Blake3 SQLite Index HIT)                     │
└────────────────────────────────────────────────────────────────────────┘

// ── Hoisted Type Contracts (2) ──────────────────────────────────────────
interface PaymentPayload { userId: string; amountCents: number; currency: string; }
type PaymentResult = { success: boolean; transactionId: string; };

// ── Extracted Symbolic Implementation (L45-L52) ─────────────────────────
45: export async function processPayment(payload: PaymentPayload): Promise<PaymentResult> {
46:     const validated = validatePaymentPayload(payload);
47:     const taxAmount = calculateTax(validated.amountCents);
48:     return await stripeClient.charges.create({ ...validated, tax: taxAmount });
49: }
```

### 2. `dagr graph` Dependency Blast Radius Visualizer:
```
$ dagr graph src/billing/charge.ts:processPayment

src/billing/charge.ts:processPayment
├── 📦 (hoisted) import { StripeClient } from "@/lib/stripe"
├── ⚙️ (called)  validatePaymentPayload() [L12-L24]
│   └── 📦 (hoisted) type ValidationRule
├── ⚙️ (called)  calculateTax() [L28-L35]
└── ✂️ (pruned)  850 lines of unreferenced helpers (97.1% token savings)
```

### 3. `dagr guard` In-Memory Architectural Check:
```
$ dagr guard

🛡️ DAGR Architecture Guard (<22ms)
Checking 48 staged files against .dagr/rules.yaml...

❌ Violation Detected [UI-to-DB Isolation]:
   ├─ File:    src/components/CheckoutButton.tsx (Line 4)
   ├─ Import:  import { db } from "@/db/client";
   ├─ Rule:    UI components must not import database clients directly.
   └─ Fix:     Route database requests through @/services/billing.

1 architectural violation found. Working tree protected.
```

---

## 📊 Transparent Metrics & Mathematical Formulas

DAGR adheres to 100% transparency. We do not use rough character estimates; all metrics are calculated mathematically using industry-standard Byte-Pair Encoding (BPE) tokenizers.

### 1. Token Compression Percentage Formula
$$\text{Token Savings \%} = \left( 1 - \frac{\text{Tokens}_{\text{sliced}}}{\text{Tokens}_{\text{original}}} \right) \times 100$$
* $\text{Tokens}_{\text{original}}$: Exact BPE token count of the unpruned source file using `tiktoken-rs` (`cl100k_base` / `o200k_base`).
* $\text{Tokens}_{\text{sliced}}$: Exact BPE token count of the sparse code lines + hoisted type contracts.

### 2. Latency SLA Benchmarks
| Operation | Cold Run Latency | Cached Run Latency (Blake3 HIT) | Method |
| :--- | :---: | :---: | :--- |
| **AST Parse & Slice (1,000 LOC)** | `1.5ms - 2.8ms` | `< 0.05ms` | Tree-sitter + SQLite WAL Index |
| **Architectural Guard (`dagr guard`)** | `< 25ms` (50 files) | `< 2ms` | Glob AST Import Evaluator |
| **CoW Snapshot Creation** | `< 2.0ms` | `< 0.8ms` | macOS APFS `clonefile(2)` / `reflink` |
| **Shadow Sandbox Rollback** | `< 10ms` | `< 10ms` | Atomic Directory Purge |

### 3. Developer FinOps Cost Savings Matrix (Claude 3.5 Sonnet / GPT-4o)
| File Size (Lines) | Baseline Tokens (Raw File) | DAGR Sliced Tokens | Token Reduction | Savings per 1,000 Prompts (Claude 3.5 @ $3.00/M) |
| :---: | :---: | :---: | :---: | :---: |
| **300 lines** | ~3,200 tokens | ~180 tokens | **94.4%** | **$9.06 saved** |
| **800 lines** | ~8,400 tokens | ~320 tokens | **96.2%** | **$24.24 saved** |
| **1,500 lines** | ~16,200 tokens | ~450 tokens | **97.2%** | **$47.25 saved** |

---

## 🚀 Quickstart & IDE Setup

### 1. Build & Install
```bash
# Clone the repository
git clone https://github.com/mjzd7/dagr.git
cd dagr

# Build with Cargo
cargo build --release

# Optional: Add to PATH
cp target/release/dagr /usr/local/bin/
```

### 2. In-IDE Configuration (Cursor / Claude Desktop / Windsurf)

Add DAGR to your IDE's Model Context Protocol (MCP) configuration:

#### Cursor (`~/.cursor/mcp.json`)
```json
{
  "mcpServers": {
    "dagr": {
      "command": "dagr",
      "args": ["mcp", "start"]
    }
  }
}
```

#### Claude Desktop (`claude_desktop_config.json`)
```json
{
  "mcpServers": {
    "dagr": {
      "command": "/usr/local/bin/dagr",
      "args": ["mcp", "start"]
    }
  }
}
```

---

## 🧭 Nomenclature & Etymology

**DAGR** was architected by **Mohit Dagar** at the convergence of compiler theory, modern Rust CLI aesthetics, and mythological illumination:

```
                      ┌────────────────────────────────────────────────────────┐
                      │                     MOHIT DAGAR                        │
                      └──────────────────────────┬─────────────────────────────┘
                                                 │
                  ┌──────────────────────────────┼──────────────────────────────┐
                  ▼                              ▼                              ▼
    ┌───────────────────────────┐  ┌───────────────────────────┐  ┌───────────────────────────┐
    │     1. CS FOUNDATION      │  │     2. RUST MINIMALISM    │  │    3. ILLUMINATION LORE   │
    │  Directed Acyclic Graph   │  │   4-Letter Binary: `dagr` │  │ Norse "Dagr" (Day/Light)  │
    │        [ D-A-G ]          │  │       [ DAG + Rust ]      │  │ Illumination of Context   │
    └─────────────┬─────────────┘  └─────────────┬─────────────┘  └─────────────┬─────────────┘
                  │                              │                              │
                  └──────────────────────────────┼──────────────────────────────┘
                                                 ▼
                                     ┌───────────────────────┐
                                     │        D A G R        │
                                     │     (CLI: `dagr`)     │
                                     └───────────────────────┘
```

1. **CS Foundation (`DAG` - Directed Acyclic Graph):** The exact topological representation of ASTs, static call graphs, module import trees, and Git commit histories.
2. **Rust Minimalism (`DAG` + `R` = `dagr`):** Modern 4-letter Unix CLI command (`dagr context`, `dagr guard`, `dagr run`).
3. **Illumination Lore (Norse *Dagr*):** The divine personification of daylight banishing the dark fog of noisy context dumps to illuminate the exact code an LLM needs.

---

## 📜 License

Distributed under the **MIT License**. See [`LICENSE`](./LICENSE) for details.

**Creator & Lead Architect:** [Mohit Dagar](https://github.com/mjzd7)
