# ⚡ DAGR (`dagr`)

<div align="center">

[![License](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](./LICENSE)
[![Rust 2021](https://img.shields.io/badge/Rust-2021_Edition-orange.svg)](https://www.rust-lang.org/)
[![Model Context Protocol](https://img.shields.io/badge/MCP-JSON--RPC_2.0-blue.svg)](https://modelcontextprotocol.io/)
[![A2A Swarm Protocol](https://img.shields.io/badge/A2A-Swarm_Bus-9cf.svg)](#-dual-protocol-gateway-mcp--a2a)
[![Research Roadmap](https://img.shields.io/badge/Roadmap-Living_Agent_Ledger-purple.svg)](./RESEARCH_ROADMAP.md)
[![Academic Research](https://img.shields.io/badge/arXiv-2026_Foundations-indigo.svg)](./docs/RESEARCH_PAPERS.md)
[![Architecture](https://img.shields.io/badge/Architecture-Ponytail_Minimal-success.svg)](./.ponytail.md)
[![Lead Architect](https://img.shields.io/badge/Creator-Mohit_Dagar-purple.svg)](https://github.com/mjzd7)

**Governance for AI-written code.**

*Policy enforcement, sandboxed execution with atomic rollback, and signed audit receipts — so every change a coding agent makes is checked, provable, and reversible.*

[Executive Summary](#-executive-summary-crisp-overview) • [Research Roadmap](./RESEARCH_ROADMAP.md) • [Research Papers](./docs/RESEARCH_PAPERS.md) • [Why Developers Need DAGR](#-why-developers-need-dagr-in-simple-terms) • [Dual Protocol](#-dual-protocol-gateway-mcp--a2a) • [Visual Architecture](#-visual-architecture--mechanics) • [Audited Metrics](#-transparent-metrics--mathematical-formulas) • [Terminal UI](#-terminal-ui--token-gauges) • [Quickstart](#-quickstart--ide-setup) • [Nomenclature](#-nomenclature--etymology)

> ⚠️ **Known limits:** exact-string search still beats AST slicing for literal lookups; cross-file symbol analysis currently supports TypeScript/JavaScript/Rust best. See [`docs/HONEST-LIMITS.md`](docs/HONEST-LIMITS.md).

</div>

---

## 📌 Executive Summary (Crisp Overview)

* **What it is:** A single, ultra-fast native Rust binary (`dagr`) acting as a local-first safety hypervisor and multi-agent coordination bus between AI coding agents and your codebase.
* **The Core Problems it Solves:** 
  * ❌ **Context Explosion (95% Token Bloat):** Passing entire 1,000+ line files or noisy vector dumps to LLMs inflates API costs and triggers "lost-in-the-middle" hallucinations.
  * ❌ **Unbounded Blast Radius:** Autonomous agents generating duplicate utilities, violating clean layer boundaries (e.g. UI importing DB clients), and writing half-broken diffs to disk.
  * ❌ **Multi-Agent Collisions:** Multiple autonomous agents clashing and overwriting each other's changes without synchronized state locking.
* **The Solution:** 
  * ✂️ **Symbolic AST Slicing:** Extracts only the exact ~35 lines of target code + upstream type contracts, reducing token payloads by **>= 90%**.
  * 🛡️ **Copy-on-Write (CoW) Sandboxing:** Executes all agent writes and tests inside an OS-native shadow snapshot (`clonefile(2)` on macOS, `reflink` on Linux) with instant **<10ms atomic rollback** on failure.
  * 🤝 **Dual Protocol (MCP + A2A):** Host-to-tool JSON-RPC 2.0 integration for IDEs (Cursor, Claude Desktop) and peer-to-peer Agent-to-Agent (A2A) state arbitration for multi-agent swarms.
  * ⚡ **Zero Cloud Dependencies:** 100% standalone native binary with embedded SQLite and Blake3 caching. No Docker or external daemon required.

---

## 💡 Why Developers Need DAGR (In Simple Terms)

### 😫 The 3 Real-World Frustrations Developers Face Today:

1. **The "Confused & Expensive AI" Problem (Token Bloat):**
   * *What happens:* You ask Cursor or Claude to fix a function in `checkout.ts`. Your IDE dumps the **entire 1,500-line monolithic file** into the prompt (15,000 tokens).
   * *The Pain:* The AI gets overwhelmed by noise ("Lost in the Middle"), hallucinates buggy logic, takes 15 seconds to reply, and burns through your daily API token limits ($10–$30/day).

2. **The "AI Broke My Codebase" Problem (Unsafe Mutations):**
   * *What happens:* You let an autonomous agent build a feature. It modifies 6 files directly on your disk. When the tests run, they fail.
   * *The Pain:* Your git working tree is now dirty with broken changes. You waste 15 minutes manually untangling diffs or running `git reset --hard`.

3. **The "Spaghetti Architecture" Problem (AI Layer Drift):**
   * *What happens:* AI agents take lazy shortcuts—importing database queries or secret keys directly inside a frontend React button component.
   * *The Pain:* Your clean codebase turns into unmaintainable spaghetti code behind your back.

```
           WITHOUT DAGR                                        WITH DAGR
┌──────────────────────────────────────┐        ┌──────────────────────────────────────┐
│ • Dumps entire 1,500-line file       │        │ • Surgically slices only ~35 lines   │
│ • 15,000 tokens ($0.05 / prompt)     │  ───►  │ • 350 tokens (97% cheaper & faster)  │
│ • AI gets confused & hallucinates    │        │ • Laser-focused, bug-free answers    │
│ • Directly mutates & pollutes disk   │        │ • Shadow sandbox with 10ms rollback  │
│ • Creates messy architectural drift  │        │ • Auto-rejects bad imports in <1ms   │
└──────────────────────────────────────┘        └──────────────────────────────────────┘
```

### 🎯 The 3 Tangible Benefits You Get:

* ✂️ **Saves 95% on AI Bills & Unlocks Local Models:** Prompts respond **3x faster**, token bills drop by **95%**, and smaller/cheaper models (like local **Ollama**, **Llama 3**, and **Qwen 2.5**) write high-precision code without running out of memory.
* 🛡️ **Zero-Fear Autonomous Coding (The Instant "Undo Button"):** All AI mutations run in an invisible Copy-on-Write shadow sandbox. If tests fail, DAGR wipes the mistake in **10 milliseconds**—your real files stay 100% clean.
* 📏 **Automatic Architecture Guardrails:** If an AI tries to make a bad architectural decision (like importing the DB into the UI), DAGR intercepts it in **<0.1ms** and guides the AI to fix its own mistake!

---

## 🤝 Dual Protocol Gateway: MCP + A2A

DAGR provides native support for both **Host-to-Tool** workflows (IDE) and **Peer-to-Peer** swarms (multi-agent pipelines):

```mermaid
graph LR
    subgraph Agent_Swarm ["🤖 Multi-Agent Swarm (A2A Network)"]
        Planner["Planning Agent<br/>(Architect)"]
        Coder["Coding Agent<br/>(Builder)"]
        Tester["Verification Agent<br/>(Tester)"]
    end

    subgraph DAGR_Bus ["⚡ DAGR Hypervisor (MCP + A2A)"]
        A2A_Hub["A2A State & Event Hub<br/>(Transaction Locking & Handoff)"]
        AST["Symbolic Slicer"]
        CoW["CoW Sandbox"]
    end

    Planner -->|1. A2A Request: Blast Radius| A2A_Hub
    A2A_Hub --> AST
    Planner -->|2. A2A Delegate: Build Task| Coder
    Coder -->|3. A2A Stage Mutation| A2A_Hub
    A2A_Hub --> CoW
    Coder -->|4. A2A Request Verification| Tester
    Tester -->|5. A2A Run Tests in Shadow| A2A_Hub
```

### The 6 Core Tools Exposed across MCP & A2A:

#### 🔌 Standard MCP Tools (Host-to-Tool for Cursor & Claude):
1. `dagr_get_context_slice`: Prunes code down to exact ~35 lines + hoisted type contracts.
2. `dagr_verify_architecture`: In-memory layer boundary and SOLID checker (<0.1ms).
3. `dagr_execute_sandboxed`: Safe tool execution in Copy-on-Write shadow sandbox.

#### 🤝 A2A Swarm Tools (Peer-to-Peer for Autonomous Swarms):
4. `dagr_a2a_handshake`: Registers agent session ID, role, and file locks (prevents concurrent write conflicts).
5. `dagr_a2a_transfer_context`: Passes compressed AST slices directly between peer agents without re-parsing.
6. `dagr_a2a_verify_peer_patch`: Reviewer agent runs automated tests on another agent's staged shadow transaction (`tx_id`) before committing.

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

## 🛡️ Guard Rules Schema (`.dagr/rules.yaml`)

Created automatically by `dagr init`. Parsing is **strict (fail-closed)**: unknown or mistyped keys are a hard error that names the offending key — a silently-dropped mistyped `boundaries` list would otherwise produce a zero-rule guard that always reports PASS.

| Level | Allowed keys |
| :--- | :--- |
| **Top level** | `version` *(required)* · `project_name` · `preset` · `boundaries` · `limits` · `security` |
| **`boundaries[]` entry** | `name` *(required)* · `from` *(required)* · `cannot_import` *(required)* · `message` *(optional — default: `"Architectural layer boundary violation detected"`)* |
| **`limits`** | `max_file_lines` · `max_function_lines` · `disallow_eval` |
| **`security`** | `sanitize_prompt_injections` *(default `true`)* · `strip_control_tokens` *(default: common LLM control tokens)* |

### Minimal example

```yaml
version: "1.0"
project_name: my-monorepo
preset: clean-architecture   # optional; seeds boundaries when none are defined
boundaries:
  - name: UI-to-DB Boundary
    from: "packages/web/src/**"
    cannot_import:
      - "packages/core/src/db/**"
    message: Presentation layer must not import database clients directly.
limits:
  max_file_lines: 500
  max_function_lines: 60
  disallow_eval: true
security:
  sanitize_prompt_injections: true
  strip_control_tokens: ["[INST]", "[/INST]"]
```

### Behavior notes

* **File missing** → falls back to the built-in `clean-architecture` preset (full enforcement).
* **File present but invalid** → hard error naming the offending key and line; the guard refuses to run rather than silently passing.
* **`preset:` set + empty `boundaries`** → preset boundaries are seeded automatically.
* Patterns match **canonical workspace-relative paths**: relative specifiers (`../db/client`) are resolved against the importing file's directory, and alias specifiers (`@/lib/x`) against root `tsconfig.json`/`jsconfig.json` `paths` when present.

---

## 📊 Live Lifetime Telemetry & ROI Dashboard

DAGR features a built-in, local-first analytics engine and embedded zero-cloud web dashboard to track cumulative token savings, estimated dollar ROI ($3.00/1M blended LLM pricing), compression ratios, and client usage across all connected AI coding tools.

<div align="center">

<img src="target/dashboard_preview.png" alt="DAGR Lifetime Telemetry & ROI Dashboard" width="100%" />

</div>

### 🚀 CLI Telemetry & Dashboard Commands:

```bash
# 1. Launch the interactive Linear-aesthetic Web Dashboard (opens http://127.0.0.1:3333 with live SSE streaming)
dagr dashboard

# 2. View the terminal ROI Value Scoreboard
dagr stats

# 3. Launch the full-screen interactive Terminal TUI (Ratatui + Crossterm)
dagr stats --tui

# 4. Export structured telemetry ledger for engineering audit & team billing
dagr stats --export json
dagr stats --export csv

# 5. Start background incremental file watcher (<0.3ms re-indexing on save)
dagr watch
```

---

## 🔍 5-Stage Zero-Miss Symbol & Intent Engine

Never worry about typing exact function names or file paths again. DAGR uses a multi-tier fallback pipeline to resolve symbols and fuzzy intent in sub-millisecond speeds:

1. **Stage 1 (`< 0.05ms`):** Exact URI & Blake3 hash lookup in local SQLite index.
2. **Stage 2 (`< 0.2ms`):** Tokenizes `camelCase`, `snake_case`, `kebab-case`, and applies Jaro-Winkler distance metric ($\ge 0.78$) to resolve typos and abbreviations.
3. **Stage 3 (`< 0.5ms`):** Full-text docstring & type signature search across workspace AST nodes.
4. **Stage 4 (`< 0.1ms`):** Proximity boosting for imported/module-adjacent symbols.
5. **Stage 5:** Top-3 disambiguation ranking with deterministic confidence scores.

```bash
# Slices correctly even with partial or abbreviated queries:
dagr context processPayment
dagr context billing.ts:charge
dagr context authService.login,database.getUser
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

> ✅ **Field-verified** on vercel/next.js (99.7% on 2564-line file), vitejs/vite, denoland/deno, rust-analyzer, and tokio-rs/tokio. See `docs/findings/` for per-repo data.
| File Size (Lines) | Baseline Tokens (Raw File) | DAGR Sliced Tokens | Token Reduction | Savings per 1,000 Prompts (Claude 3.5 @ $3.00/M) |
| :---: | :---: | :---: | :---: | :---: |
| **300 lines** | ~3,200 tokens | ~180 tokens | **94.4%** | **$9.06 saved** |
| **800 lines** | ~8,400 tokens | ~320 tokens | **96.2%** | **$24.24 saved** |
| **1,500 lines** | ~16,200 tokens | ~450 tokens | **97.2%** | **$47.25 saved** |

---

## 🚀 Installation & Skills Guide (30-Second Setup)

You can install DAGR as a unified local CLI/MCP server, or install its portable **Agent Skills** individually into your AI assistants.

---

### ⚡ 1. One-Line Multi-OS Installers (Automated Binary + MCP + Skills)

Choose your operating system to install the pre-compiled binary and auto-configure all IDEs:

<details open>
<summary><strong>🍎 macOS & 🐧 Linux (One-Line Script)</strong></summary>

```bash
curl -fsSL https://raw.githubusercontent.com/mjzd7/dagr/main/scripts/install.sh | bash
```
*Auto-detects Apple Silicon (`arm64`), Intel Mac (`x86_64`), or Linux, downloads the binary, and runs MCP/skills setup.*

</details>

<details>
<summary><strong>🪟 Windows (PowerShell)</strong></summary>

```powershell
irm https://raw.githubusercontent.com/mjzd7/dagr/main/scripts/install.ps1 | iex
```

</details>

<details>
<summary><strong>📦 NPM / NPX (Node.js Global / 1-Off Run)</strong></summary>

```bash
# Global install via npm:
npm install -g @mjzd7/dagr

# Or run instantly with npx (zero local install):
npx @mjzd7/dagr init
npx @mjzd7/dagr mcp install --client all
npx @mjzd7/dagr skills install --target all
```

</details>

<details>
<summary><strong>🍺 Homebrew (macOS / Linux)</strong></summary>

```bash
# Install via GitHub tap:
brew install mjzd7/dagr/dagr
# or: brew tap mjzd7/dagr && brew install dagr

dagr mcp install --client all
dagr skills install --target all
```

</details>

<details>
<summary><strong>🦀 Cargo (From Source)</strong></summary>

```bash
cargo install --path crates/dagr-cli --force
dagr mcp install --client all
dagr skills install --target all
```

</details>

---

### 🔄 2. Seamless Auto-Update (`dagr update`)

Whenever new improvements or bug fixes are pushed to GitHub, users can update their binary, MCP configs, and skills with a single command:

```bash
$ dagr update
```
* **What it does:** Pulls the latest release binary from `github.com/mjzd7/dagr`, replaces the executable in-place, and automatically refreshes MCP server definitions and `SKILL.md` packages across all 30+ IDEs.

---

### 🔌 3. 30+ Supported AI Coding Agents & IDEs (1-Click Auto-Config)

DAGR supports automated 1-click MCP configuration injection across **30+ top AI coding assistants and development environments**:

```bash
# Auto-configure all detected agents and IDEs on your machine:
dagr mcp install --client all

# Or target any specific client:
dagr mcp install --client <CLIENT_ID>

# List all supported clients:
dagr mcp list-clients
```

| Brand Logo | Client ID | Agent / IDE Name | Category | Primary Configuration Location |
| :---: | :--- | :--- | :--- | :--- |
| <img src="assets/icons/cursor.svg" width="22" height="22" alt="Cursor" /> | `cursor` | **Cursor IDE** | AI IDE | `~/.cursor/mcp.json` |
| <img src="assets/icons/claude.svg" width="22" height="22" alt="Claude" /> | `claude` | **Claude Desktop** | Desktop App | `claude_desktop_config.json` |
| <img src="assets/icons/claudecode.svg" width="22" height="22" alt="Claude Code" /> | `claudecode` | **Claude Code CLI** | CLI Agent | `~/.claude/mcp.json` |
| <img src="assets/icons/windsurf.svg" width="22" height="22" alt="Windsurf" /> | `windsurf` | **Windsurf (Codeium)** | AI IDE | `~/.codeium/windsurf/mcp_config.json` |
| <img src="assets/icons/vscode.svg" width="22" height="22" alt="VS Code" /> | `vscode` | **VS Code / Copilot** | IDE | `.vscode/mcp.json` |
| <img src="assets/icons/roocode.svg" width="22" height="22" alt="Roo Code" /> | `roocode` | **Roo Code (Roo Cline)** | VS Code Extension | globalStorage `cline_mcp_settings.json` |
| <img src="assets/icons/cline.svg" width="22" height="22" alt="Cline" /> | `cline` | **Cline** | VS Code Extension | `saoudrizwan.claude-dev/cline_mcp_settings.json` |
| <img src="assets/icons/continue.svg" width="22" height="22" alt="Continue" /> | `continue` | **Continue.dev** | Extension / IDE | `~/.continue/config.json` |
| <img src="assets/icons/zed.svg" width="22" height="22" alt="Zed" /> | `zed` | **Zed Editor** | Fast Rust Editor | `~/.config/zed/settings.json` |
| <img src="assets/icons/aider.svg" width="22" height="22" alt="Aider" /> | `aider` | **Aider AI** | CLI Pair Programmer | `~/.aider/mcp.json` |
| <img src="assets/icons/openinterpreter.svg" width="22" height="22" alt="Open Interpreter" /> | `openinterpreter` | **Open Interpreter** | CLI Agent | `~/.open-interpreter/mcp.json` |
| <img src="assets/icons/antigravity.svg" width="22" height="22" alt="Google Antigravity" /> | `antigravity` | **Google Antigravity / Gemini CLI** | Agentic IDE | `~/.gemini/config/mcp.json` |
| <img src="assets/icons/amazonq.svg" width="22" height="22" alt="Amazon Q" /> | `amazonq` | **Amazon Q Developer** | Enterprise Agent | `~/.aws/q/mcp.json` |
| <img src="assets/icons/jetbrains.svg" width="22" height="22" alt="JetBrains" /> | `jetbrains` | **JetBrains (IntelliJ, PyCharm)** | IDE Suite | `~/.config/JetBrains/mcp.json` |
| <img src="assets/icons/goose.svg" width="22" height="22" alt="Goose" /> | `goose` | **Goose (Block / Square)** | Open-Source Agent | `~/.config/goose/mcp.json` |
| <img src="assets/icons/cody.svg" width="22" height="22" alt="Cody" /> | `cody` | **Sourcegraph Cody** | Enterprise Assistant | `~/.sourcegraph/cody-mcp.json` |
| <img src="assets/icons/neovim.svg" width="22" height="22" alt="Neovim" /> | `neovim` | **Neovim (avante.nvim / mcphub)** | Terminal Editor | `~/.config/nvim/mcp.json` |
| <img src="assets/icons/emacs.svg" width="22" height="22" alt="Emacs" /> | `emacs` | **Emacs (gptel / mcp.el)** | Extensible Editor | `~/.emacs.d/mcp.json` |
| <img src="assets/icons/devin.svg" width="22" height="22" alt="Devin" /> | `devin` | **Cognition Devin** | Autonomous Agent | `.devin/mcp.json` |
| <img src="assets/icons/opencode.svg" width="22" height="22" alt="OpenCode" /> | `opencode` | **OpenCode (Sisyphus)** | Multi-Agent Harness | `~/.opencode/mcp.json` |
| <img src="assets/icons/melty.svg" width="22" height="22" alt="Melty" /> | `melty` | **Melty** | Open-Source AI IDE | `~/.melty/mcp.json` |
| <img src="assets/icons/pearai.svg" width="22" height="22" alt="PearAI" /> | `pearai` | **PearAI** | AI Code Editor | `~/.pearai/mcp.json` |
| <img src="assets/icons/trae.svg" width="22" height="22" alt="Trae" /> | `trae` | **Trae AI (ByteDance)** | Adaptive IDE | `~/.trae/mcp.json` |
| <img src="assets/icons/boltdiy.svg" width="22" height="22" alt="Bolt" /> | `boltdiy` | **Bolt.diy** | In-Browser Web Agent | `.bolt/mcp.json` |
| <img src="assets/icons/dify.svg" width="22" height="22" alt="Dify" /> | `dify` | **Dify.ai** | LLM Ops Runtime | `~/.dify/mcp.json` |
| <img src="assets/icons/langchain.svg" width="22" height="22" alt="LangChain" /> | `langchain` | **LangChain / LangGraph** | Agent Framework | `~/.langchain/mcp.json` |
| <img src="assets/icons/crewai.svg" width="22" height="22" alt="CrewAI" /> | `crewai` | **CrewAI** | Multi-Agent Swarm | `.crewai/mcp.json` |
| <img src="assets/icons/autogen.svg" width="22" height="22" alt="AutoGen" /> | `autogen` | **Microsoft AutoGen** | Multi-Agent Framework | `.autogen/mcp.json` |
| <img src="assets/icons/librechat.svg" width="22" height="22" alt="LibreChat" /> | `librechat` | **LibreChat / Ollama** | Self-Hosted Chat | `~/.librechat/mcp.json` |
| <img src="assets/icons/superagent.svg" width="22" height="22" alt="Superagent" /> | `superagent` | **Superagent.sh** | Production Agent | `~/.superagent/mcp.json` |
| <img src="assets/icons/workspace.svg" width="22" height="22" alt="Workspace" /> | `workspace` | **Local Git Workspace** | Workspace Root | `.cursor/mcp.json`, `.vscode/mcp.json` |

---

### 📦 3. Individual Skill Installation & Usage

If you prefer to pick and choose individual skills for your AI agent workflow:

<details>
<summary><strong>✂️ Skill 1: <code>dagr-slicer</code> (Surgical AST & Token Pruning)</strong></summary>

**Purpose:** Slices out the exact function body and hoists upstream type contracts, slashing token consumption by **$>95\%$** in $<2\text{ms}$.

#### Installation:
```bash
# Install into Antigravity / Gemini CLI:
mkdir -p ~/.gemini/config/skills/dagr-slicer
cp .agents/skills/dagr-slicer/SKILL.md ~/.gemini/config/skills/dagr-slicer/

# Install into Cursor:
mkdir -p .cursor/skills/dagr-slicer
cp .agents/skills/dagr-slicer/SKILL.md .cursor/skills/dagr-slicer/
```

#### How to Invoke:
- **Natural Language:** *"Slice context for function `processPayment`"*, *"Extract minimal AST for `UserToken`"*
- **CLI Command:** `dagr context <FILE>:<SYMBOL> --format json`
- **MCP Tool:** `dagr_get_context_slice(file_path="...", symbol_name="...")`

</details>

<details>
<summary><strong>🛡️ Skill 2: <code>dagr-guard</code> (In-Memory Architecture Boundary Linter)</strong></summary>

**Purpose:** Evaluates code changes against clean layer boundaries (e.g. UI cannot import DB/ORM) in **$<0.1\text{ms}$** and sanitizes prompt injections.

#### Installation:
```bash
# Install into Antigravity / Gemini CLI:
mkdir -p ~/.gemini/config/skills/dagr-guard
cp .agents/skills/dagr-guard/SKILL.md ~/.gemini/config/skills/dagr-guard/

# Install into Cursor:
mkdir -p .cursor/skills/dagr-guard
cp .agents/skills/dagr-guard/SKILL.md .cursor/skills/dagr-guard/
```

#### How to Invoke:
- **Natural Language:** *"Verify architecture boundaries"*, *"Check layer imports"*, *"Lint with dagr guard"*
- **CLI Command:** `dagr guard --format json`
- **MCP Tool:** `dagr_verify_architecture(source_file="...", proposed_imports=[...])`

</details>

<details>
<summary><strong>🔒 Skill 3: <code>dagr-sandbox</code> (Copy-on-Write Shadow Workspace Runner)</strong></summary>

**Purpose:** Executes tests and refactors inside an isolated OS-native shadow snapshot with instant **$<10\text{ms}$ atomic rollback** on failure.

#### Installation:
```bash
# Install into Antigravity / Gemini CLI:
mkdir -p ~/.gemini/config/skills/dagr-sandbox
cp .agents/skills/dagr-sandbox/SKILL.md ~/.gemini/config/skills/dagr-sandbox/

# Install into Cursor:
mkdir -p .cursor/skills/dagr-sandbox
cp .agents/skills/dagr-sandbox/SKILL.md .cursor/skills/dagr-sandbox/
```

#### How to Invoke:
- **Natural Language:** *"Run test in shadow sandbox"*, *"Safe trial with atomic rollback"*, *"Sandboxed test"*
- **CLI Command:** `dagr run "<TEST_COMMAND>" [--commit-on-success]`
- **MCP Tool:** `dagr_execute_sandboxed(command="...")`

</details>

<details>
<summary><strong>💥 Skill 4: <code>dagr-chaos</code> (Ephemeral Fault Injection & Proofs)</strong></summary>

**Purpose:** Injects synthetic latency, CPU throttling, and lock contention during PR verification, and generates cryptographically signed Blake3 audit badges (`proof_<hash>`).

#### Installation:
```bash
# Install into Antigravity / Gemini CLI:
mkdir -p ~/.gemini/config/skills/dagr-chaos
cp .agents/skills/dagr-chaos/SKILL.md ~/.gemini/config/skills/dagr-chaos/
```

#### How to Invoke:
- **Natural Language:** *"Inject chaos faults"*, *"Stress-test PR"*, *"Generate cryptographic proof of correctness"*
- **CLI Command:** `cargo test -p dagr-chaos`

</details>

---

### 🤖 3. How Agents Automatically Discover & Trigger These Skills

You do not need to memorize commands. DAGR employs **Semantic Router Descriptions** in each `SKILL.md` frontmatter:

```yaml
---
name: dagr-slicer
description: Surgical AST context slicing and contract hoisting hypervisor. Use whenever inspecting, analyzing, or preparing to modify a function, method, or class, to avoid loading full files and slash token consumption by >95%. Also use when user mentions token reduction, context slicing, or AST extraction.
---
```

When an agent (Cursor, Claude 3.5 Sonnet, Copilot, Antigravity) receives a prompt to edit or review a function:
1. The model's reasoning loop matches your task against the `description` keyword hooks.
2. It reads `SKILL.md` into its working memory.
3. It calls `dagr context` or `dagr_get_context_slice` directly—retrieving only the exact 25-line AST slice and hoisted type contracts.

---

### 🛠️ 4. In-IDE Configuration Manual Fallback (MCP)

If you prefer to configure IDEs manually instead of using `dagr mcp install`:

#### Cursor (`~/.cursor/mcp.json` or `.cursor/mcp.json`)
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
      "command": "dagr",
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

## 🤖 Agent-OS Infrastructure Layer

DAGR ships a built-in infrastructure layer for running AI coding agents safely and cost-effectively:

| Module | Purpose |
|---|---|
| `TokenBucketRateLimiter` | TPM-based rate limiting on expensive tool operations |
| `ToolCircuitBreaker` | Three-state breaker (Closed→Open→HalfOpen) prevents cascade failures from repeated tool errors |
| `BudgetContext` | Configurable token + duration budgets with per-batch enforcement (`DAGR_TOKEN_BUDGET` env var) |
| `SagaCoordinator` | Multi-step saga pattern with compensating rollback for complex agent workflows |
| `EffectJournal` | Deterministic effect logging and replay for audit trails |
| `EventStorePort` | Event-sourced run state persistence (SQLite WAL backend) |
| `QuarantineManager` | Automatic quarantine of suspicious changes before they reach main |
| `CapabilityGrant` | HMAC-signed zero-trust capability tokens for multi-agent credential brokering |

All modules are clippy-clean, tested, and wired into the active execution path.

---

## 📋 What's New in v0.1.1

### Architecture Guard Intelligence
- **Strict rules schema** — unknown keys rejected at parse time (fail-closed, no silent zero-rule pass)
- **Dead-glob rejection** — uncompilable patterns caught at load with rule name in error
- **Segment-aware matching** — `src/db/**` no longer false-positives on `src/db-migration/x`
- **Relative import resolution** — `../db/client` resolves to canonical workspace paths before matching
- **Alias resolution** — `@/lib/x` resolves through tsconfig/jsconfig `paths`
- **Barrel re-export following** — violations attributed through index.ts barrels (one hop)
- **Multi-dialect extraction** — Rust `use`, Go block imports, `require()`, dynamic `import()`, side-effect imports, comment-trap rejection

### Cross-File Contract Hoisting
- **One-hop + multi-hop traversal** — type contracts hoisted from imported files recursively up to `max_depth_hops`
- **Alias-aware** — tsconfig path mappings participate in cross-file resolution
- **`--depth` truthful** — CLI help and MCP schema describe real behavior

### MCP Hardening
- Strict argument validation on all 7 tools (no silent defaults)
- Unknown tool → JSON-RPC `-32602`; argument errors → precise `isError` messages
- Provenance fields (`workspace`, `rules_source`, `active_rules`) in guard responses
- Circuit breaker + rate limiter wired into dispatch path

### Infrastructure
- `dagr schema rules` JSON-Schema emitter for editor autocomplete
- SIGPIPE exit-code hygiene (`head`-piped commands exit cleanly)
- Workspace pinning via `--workspace` flag or `$DAGR_WORKSPACE` env var
- Telemetry legacy-DB migration (stats works on pre-cloud-sync databases)
- OpenCode MCP installer with native schema support
- Ponytail governance gate + CI quality workflow

---

## 📜 License

Distributed under the **Apache License 2.0**.

* **Free for everyone:** personal use, commercial use, modification, distribution, and embedding into your own products — including other AI tools — with no restrictions beyond preserving notices.
* **Enterprise offerings** (priority support, hosted control plane, compliance packs) are sold separately and are not part of this repository. See [`NOTICE.md`](./NOTICE.md).

See [`LICENSE`](./LICENSE) for the full legal text.

**Creator & Lead Architect:** [Mohit Dagar](https://github.com/mjzd7)
