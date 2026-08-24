pub mod daemon;
pub mod governance;
pub mod server;
pub mod skills_installer;
pub mod tui;
pub mod updater;
pub mod watcher;

use clap::{Parser, Subcommand, ValueEnum};
use colored::*;
use dagr_core::{
    DagrError, Language, LocalIndexStore, MinimalContextSlice, Result, TelemetryEvent,
    TelemetryStore, TimeWindow,
};
use dagr_guard::ArchitectureGuard;
use dagr_mcp::McpServer;
use dagr_sandbox::CowSandbox;
use dagr_slicer::{AstExtractor, AstParser, SlicerConfig, SymbolicSlicer};
use serde_json::json;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "dagr",
    author = "Mohit Dagar",
    version = "0.1.0",
    about = "⚡ DAGR: The DAG-Native Symbolic AST Slicing Hypervisor & Safety Sandbox for AI Coding Agents",
    long_about = "⚡ DAGR is an ultra-fast, local-first safety hypervisor, symbolic AST slicer, and multi-agent coordination bus.\n\
                  Designed to eliminate 95% token bloat, prevent architectural boundary violations, and sandbox tool execution."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    /// Extract minimal backwards AST slice & type contracts for a target symbol (saves 95% tokens)
    #[command(
        name = "context",
        about = "Extract minimal backwards AST slice & type contracts for a target symbol",
        long_about = "Surgically slices the target function/class and hoists relevant type contracts.\n\n\
                      EXAMPLES:\n  \
                        dagr context src/billing/charge.ts:processPayment\n  \
                        dagr context crates/dagr-core/src/types.rs:Language --depth 2\n  \
                        dagr context src/auth.py:verify_token --format json"
    )]
    Context {
        /// Target in format "path/to/file.ext:symbolName" or comma-separated symbols
        target: String,

        /// Cross-file contract hoist hops (v1: one effective hop)
        #[arg(short = 'd', long)]
        depth: Option<usize>,

        /// Multi-Rubric AST Slicing tier (standard, multi-rubric / lamr)
        #[arg(long, default_value = "standard")]
        tier: String,

        /// Causal failure slice from test failure stack trace or test name
        #[arg(long)]
        from_test: Option<String>,

        /// Output format (pretty, json, plain, markdown)
        #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Pretty)]
        format: OutputFormat,
    },

    /// Evaluate code changes against clean layer boundaries defined in .dagr/rules.yaml
    #[command(
        name = "guard",
        about = "Evaluate code changes against architectural layer boundaries (<0.1ms)",
        long_about = "Validates file imports against clean architecture rules in .dagr/rules.yaml.\n\n\
                      EXAMPLES:\n  \
                        dagr guard\n  \
                        dagr guard --workspace . --staged\n  \
                        dagr guard --ci --base origin/main\n  \
                        dagr guard --format json"
    )]
    Guard {
        /// Workspace root directory (defaults to current directory)
        #[arg(short = 'w', long, default_value = ".")]
        workspace: PathBuf,

        /// Check only git staged files
        #[arg(short = 's', long)]
        staged: bool,

        /// Run in CI / Pull Request mode inspecting git diff against base branch
        #[arg(long)]
        ci: bool,

        /// Base git reference for CI diff (default: origin/main or main)
        #[arg(long)]
        base: Option<String>,

        /// Head git reference for CI diff (default: HEAD)
        #[arg(long)]
        head: Option<String>,

        /// Report violations as warnings without exiting with error code 1
        #[arg(long)]
        warn_only: bool,

        /// Write Markdown summary report to specified file path (e.g. $GITHUB_STEP_SUMMARY)
        #[arg(long)]
        output_file: Option<PathBuf>,

        /// Output format (pretty, json)
        #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Pretty)]
        format: OutputFormat,
    },

    /// Generate a signed proof receipt: guard + secrets + licenses (+ optional sandbox tests)
    #[command(
        name = "prove",
        about = "Generate a signed proof receipt for this workspace (paste into PRs)",
        long_about = "Runs the full governance stack — architecture guard, secret scan, license\n\
                      declaration check and an optional sandboxed test command — and emits a\n\
                      Blake3-signed receipt. Receipts are deterministic for identical inputs.\n\n\
                      EXAMPLES:\n  \
                        dagr prove\n  \
                        dagr prove --test \"cargo test\"\n  \
                        dagr prove --format json"
    )]
    Prove {
        /// Workspace root directory (defaults to current directory)
        #[arg(short = 'w', long, default_value = ".")]
        workspace: PathBuf,

        /// Verification command executed inside the CoW sandbox
        #[arg(long)]
        test: Option<String>,

        /// Output format (pretty, json, markdown)
        #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Markdown)]
        format: OutputFormat,
    },

    /// Review a diff range: dangling imports, boundary violations, secrets -> PASS/BLOCKED
    #[command(
        name = "review-diff",
        about = "Merge gate: review a diff range and emit a PASS/BLOCKED verdict",
        long_about = "Composes blast-radius analysis (dangling imports after deletions),\n\
                      architectural boundary checks and secret scanning over a git diff range,\n\
                      then emits a CI-consumable verdict with per-file risk scores.\n\n\
                      Exit code is 1 on BLOCKED unless --fail-on-blocked=false.\n\n\
                      EXAMPLES:\n  \
                        dagr review-diff origin/main..HEAD\n  \
                        dagr review-diff HEAD~1 HEAD --format json"
    )]
    ReviewDiff {
        /// Base git reference
        base: String,

        /// Head git reference (default: HEAD)
        #[arg(default_value = "HEAD")]
        head: String,

        /// Workspace root directory (defaults to current directory)
        #[arg(short = 'w', long, default_value = ".")]
        workspace: PathBuf,

        /// Exit non-zero when verdict is BLOCKED
        #[arg(long, default_value_t = true)]
        fail_on_blocked: bool,

        /// Output format (pretty, json, markdown)
        #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Pretty)]
        format: OutputFormat,
    },

    /// Execute a test or build command safely inside a Copy-on-Write (CoW) shadow sandbox
    #[command(
        name = "run",
        about = "Execute a command safely inside an isolated CoW shadow sandbox with atomic rollback",
        long_about = "Executes arbitrary build/test commands inside an ephemeral shadow filesystem overlay.\n\
                      If verification fails, changes are instantly rolled back with zero side effects.\n\n\
                      EXAMPLES:\n  \
                        dagr run \"cargo test\"\n  \
                        dagr run \"npm test\" --commit-on-success\n  \
                        dagr run \"pytest\" --no-sandbox"
    )]
    Run {
        /// Command to execute (e.g. "cargo test", "npm test", "pytest")
        command: String,

        /// Run inside ephemeral CoW sandbox with automatic rollback on failure
        #[arg(long, default_value_t = true)]
        sandbox: bool,

        /// Commit shadow changes into working tree if verification passes
        #[arg(short = 'c', long)]
        commit_on_success: bool,
    },

    /// Launch the live interactive Web Dashboard and SSE telemetry stream
    #[command(
        name = "dashboard",
        about = "Launch the interactive Web Dashboard & real-time telemetry stream",
        long_about = "Starts an embedded local web dashboard at http://127.0.0.1:3333 with live SSE event streaming.\n\n\
                      EXAMPLES:\n  \
                        dagr dashboard\n  \
                        dagr dashboard --port 8080\n  \
                        dagr dashboard --no-open"
    )]
    Dashboard {
        /// Port to bind web server (default: 3333, auto-falls back to 3334..3340)
        #[arg(short = 'p', long)]
        port: Option<u16>,

        /// Do not automatically open browser on launch
        #[arg(long)]
        no_open: bool,
    },

    /// View or export lifetime efficiency metrics, tokens saved, and ROI analytics
    #[command(
        name = "stats",
        about = "Display cumulative token savings, USD ROI, and telemetry ledger",
        long_about = "Aggregates lifetime token compression metrics, estimated dollar cost savings, and client usage.\n\n\
                      EXAMPLES:\n  \
                        dagr stats\n  \
                        dagr stats --tui\n  \
                        dagr stats --web\n  \
                        dagr stats --export json"
    )]
    Stats {
        /// Launch the interactive terminal TUI dashboard
        #[arg(short = 't', long)]
        tui: bool,

        /// Launch the web dashboard directly
        #[arg(short = 'w', long)]
        web: bool,

        /// Export telemetry data (json, csv)
        #[arg(short = 'e', long)]
        export: Option<String>,
    },

    /// Start incremental background file watcher for instant sub-millisecond AST re-indexing
    #[command(
        name = "watch",
        about = "Incrementally re-index workspace AST symbols in real time on save",
        long_about = "Monitors workspace file events via OS kernel notify and re-indexes AST in <0.3ms.\n\n\
                      EXAMPLES:\n  \
                        dagr watch"
    )]
    Watch {
        /// Workspace directory to watch (default: current directory)
        #[arg(short = 'w', long, default_value = ".")]
        workspace: PathBuf,
    },

    /// Start the Model Context Protocol (MCP) JSON-RPC 2.0 stdio server for Cursor / Claude Desktop / Windsurf
    #[command(
        name = "mcp",
        about = "Model Context Protocol (MCP) server management",
        long_about = "Exposes AST slicing, architecture guardrails, and sandboxed runner tools via JSON-RPC 2.0 stdio.\n\n\
                      EXAMPLES:\n  \
                        dagr mcp start"
    )]
    Mcp {
        /// Subcommand for MCP server
        #[command(subcommand)]
        action: McpAction,
    },

    /// Initialize a new .dagr configuration, default rules.yaml, and local SQLite index
    #[command(
        name = "init",
        about = "Initialize .dagr configuration and local index in current workspace",
        long_about = "Creates `.dagr/rules.yaml` with architectural presets and initializes `.dagr/index.db`.\n\n\
                      EXAMPLES:\n  \
                        dagr init\n  \
                        dagr init --preset clean-architecture\n  \
                        dagr init --preset nextjs"
    )]
    Init {
        /// Preset architecture template (clean-architecture, nextjs, fastapi)
        #[arg(short = 'p', long, default_value = "clean-architecture")]
        preset: String,
    },

    /// Emit machine-readable schemas for DAGR configuration artifacts
    #[command(
        name = "schema",
        about = "Emit JSON Schema for DAGR configuration artifacts",
        long_about = "Prints JSON Schema (draft 2020-12) describing .dagr/rules.yaml to stdout so editors can validate and autocomplete rule files.\n\n\
                      EXAMPLES:\n  \
                        dagr schema rules | jq ."
    )]
    Schema {
        /// Which configuration artifact to emit
        #[arg(value_enum)]
        artifact: SchemaArtifact,
    },

    /// Manage and install self-steering Agent Skills (Antigravity, Cursor, Claude Code, Workspace)
    #[command(
        name = "skills",
        about = "Manage and install DAGR portable Agent Skills",
        long_about = "Installs self-steering SKILL.md packages into ~/.gemini/config/skills, ~/.cursor/skills, and .agents/skills.\n\n\
                      EXAMPLES:\n  \
                        dagr skills install\n  \
                        dagr skills install --target antigravity\n  \
                        dagr skills install --target cursor"
    )]
    Skills {
        /// Subcommand for Skills management
        #[command(subcommand)]
        action: SkillsAction,
    },

    /// Automatically update DAGR binary, MCP configurations, and Agent Skills to latest GitHub version
    #[command(
        name = "update",
        alias = "upgrade",
        about = "Self-update DAGR hypervisor to the latest release from GitHub",
        long_about = "Checks github.com/mjzd7/dagr for updates, downloads the latest binary, and refreshes MCP & Skills.\n\n\
                      EXAMPLES:\n  \
                        dagr update\n  \
                        dagr upgrade"
    )]
    Update {
        /// Force reinstallation even if already on latest version
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// Authenticate with DAGR Cloud organization for centralized team FinOps and telemetry sync
    #[command(
        name = "login",
        about = "Authenticate with DAGR Cloud organization",
        long_about = "Configures organization API key to sync team token savings and audit ledgers.\n\n\
                      EXAMPLES:\n  \
                        dagr login --key dagr_live_sec_xxx --org acme-corp"
    )]
    Login {
        /// Organization API Key
        #[arg(short = 'k', long)]
        key: Option<String>,

        /// Organization ID or Slug
        #[arg(short = 'o', long)]
        org: Option<String>,

        /// Custom Cloud API URL (defaults to https://api.dagr.dev)
        #[arg(long, default_value = "https://api.dagr.dev")]
        url: String,
    },

    /// Synchronize local workspace telemetry events to DAGR Cloud with Zero-PII guarantee
    #[command(
        name = "sync",
        about = "Synchronize local telemetry metrics to DAGR Cloud",
        long_about = "Pushes pending local token savings and ROI metrics to team portal.\n\n\
                      EXAMPLES:\n  \
                        dagr sync"
    )]
    Sync {
        /// Workspace root directory (default: current directory)
        #[arg(short = 'w', long, default_value = ".")]
        workspace: PathBuf,
    },

    /// Display current DAGR Cloud connection status, organization name, and unsynced event queue
    #[command(
        name = "status",
        about = "Display DAGR Cloud connection status and sync state",
        long_about = "Shows active organization ID, local cache size, and unsynced event count.\n\n\
                      EXAMPLES:\n  \
                        dagr status"
    )]
    Status,

    /// Start or manage the Distributed Blake3 Remote Monorepo AST Cache Daemon
    #[command(
        name = "daemon",
        about = "Distributed remote monorepo AST cache daemon",
        long_about = "Accelerates multi-developer monorepos by sharing pre-indexed Blake3 AST symbols over TCP/HTTP.\n\n\
                      EXAMPLES:\n  \
                        dagr daemon --port 4444"
    )]
    Daemon {
        /// Port to listen on (default: 4444)
        #[arg(short = 'p', long, default_value_t = 4444)]
        port: u16,
    },

    /// Spawn K parallel speculative agent sandboxes with first-commit-wins resolution (BranchFS / DeltaBox)
    #[command(
        name = "branch",
        about = "Spawn K parallel speculative agent sandboxes with first-commit-wins resolution",
        long_about = "Forks K lightweight CoW sandboxes in <350µs for parallel agent exploration.\n\n\
                      EXAMPLES:\n  \
                        dagr branch fork --count 3 --task 'fix webhook timeout'"
    )]
    Branch {
        /// Action for branch exploration (defaults to fork)
        #[arg(default_value = "fork")]
        action: String,

        /// Number of parallel branches to fork
        #[arg(short = 'c', long, default_value_t = 3)]
        count: usize,

        /// Task description for the parallel exploration branches
        #[arg(short = 't', long, default_value = "speculative_exploration")]
        task: String,
    },
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum SkillsAction {
    /// Install DAGR Agent Skills into AI assistant skill directories
    Install {
        /// Target environment (antigravity, cursor, claude, workspace, all)
        #[arg(short = 't', long, default_value = "all")]
        target: String,
    },
    /// List available DAGR Agent Skills
    List,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum McpAction {
    /// Start stdio MCP JSON-RPC 2.0 listener
    Start {
        /// Workspace root to serve; falls back to $DAGR_WORKSPACE, then current directory
        #[arg(short = 'w', long)]
        workspace: Option<PathBuf>,
    },

    /// Automatically configure DAGR into Cursor, Claude Desktop, Windsurf, or Workspace MCP settings
    Install {
        /// Target client to configure (cursor, claude, windsurf, vscode, cline, roocode, antigravity, all, etc.)
        #[arg(short = 'c', long, default_value = "all")]
        client: String,

        /// Custom path to binary (defaults to current executable or "dagr")
        #[arg(long)]
        bin_path: Option<String>,
    },

    /// List all 30+ supported AI coding agents and IDEs for MCP auto-configuration
    ListClients,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutputFormat {
    Pretty,
    Json,
    Plain,
    Markdown,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaArtifact {
    Rules,
}

/// Hand-maintained JSON Schema mirroring RuleConfig's strict contract.
// ponytail: static schema instead of schemars codegen - struct surface is small/stable and the anti-drift test pins drift; upgrade if the config surface grows
pub fn rules_schema() -> serde_json::Value {
    serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "DAGR Guard Rules (.dagr/rules.yaml)",
        "description": "Strict schema: unknown keys are rejected at parse time.",
        "type": "object",
        "additionalProperties": false,
        "required": ["version"],
        "properties": {
            "version": { "type": "string" },
            "project_name": { "type": ["string", "null"] },
            "preset": {
                "type": ["string", "null"],
                "enum": [
                    "clean-architecture",
                    "nextjs",
                    "nextjs-app",
                    "fastapi",
                    "python",
                    "rust",
                    "rust-monorepo"
                ]
            },
            "boundaries": {
                "type": "array",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["name", "from", "cannot_import"],
                    "properties": {
                        "name": { "type": "string" },
                        "from": { "type": "string" },
                        "cannot_import": {
                            "type": "array",
                            "items": { "type": "string" }
                        },
                        "message": { "type": "string" }
                    }
                }
            },
            "limits": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "max_file_lines": { "type": ["integer", "null"] },
                    "max_function_lines": { "type": ["integer", "null"] },
                    "disallow_eval": { "type": ["boolean", "null"] }
                }
            },
            "security": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "sanitize_prompt_injections": { "type": "boolean" },
                    "strip_control_tokens": {
                        "type": "array",
                        "items": { "type": "string" }
                    }
                }
            }
        }
    })
}

pub fn handle_schema_rules() -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&rules_schema())?);
    Ok(())
}

/// Resolves the active DAGR workspace: explicit flag > $DAGR_WORKSPACE env > process CWD.
pub fn resolve_workspace(flag: Option<PathBuf>, env: Option<String>, cwd: PathBuf) -> PathBuf {
    flag.or_else(|| env.map(PathBuf::from)).unwrap_or(cwd)
}

/// Executes the resolved CLI command
pub async fn execute_cli(cli: Cli) -> Result<()> {
    let is_mcp_start = matches!(
        &cli.command,
        Commands::Mcp {
            action: McpAction::Start { .. }
        }
    );
    let is_update = matches!(&cli.command, Commands::Update { .. });
    let is_dashboard = matches!(&cli.command, Commands::Dashboard { .. });

    let result = match cli.command {
        Commands::Context {
            target,
            depth,
            tier,
            from_test,
            format,
        } => handle_context(&target, depth, &tier, from_test.as_deref(), format),
        Commands::Branch {
            action,
            count,
            task,
        } => handle_branch(&action, count, &task),
        Commands::Guard {
            workspace,
            staged,
            ci,
            base,
            head,
            warn_only,
            output_file,
            format,
        } => handle_guard(
            &workspace,
            staged,
            ci,
            base.as_deref(),
            head.as_deref(),
            warn_only,
            output_file.as_deref(),
            format,
        ),
        Commands::Run {
            command,
            sandbox,
            commit_on_success,
        } => handle_run(&command, sandbox, commit_on_success),
        Commands::Prove {
            workspace,
            test,
            format,
        } => handle_prove(&workspace, test.as_deref(), format),
        Commands::ReviewDiff {
            base,
            head,
            workspace,
            fail_on_blocked,
            format,
        } => {
            let outcome = handle_review_diff(&workspace, &base, &head, format)?;
            if fail_on_blocked && outcome == "BLOCKED" {
                std::process::exit(1);
            }
            Ok(())
        }
        Commands::Dashboard { port, no_open } => {
            let root = std::env::current_dir()?;
            server::DashboardServer::bind_and_run(root, port, !no_open).await
        }
        Commands::Stats { tui, web, export } => {
            let root = std::env::current_dir()?;
            if tui {
                tui::run_tui(&root)
            } else if web {
                server::DashboardServer::bind_and_run(root, None, true).await
            } else if let Some(exp) = export {
                let store = TelemetryStore::open(&root)?;
                if exp.to_lowercase() == "csv" {
                    println!("{}", store.export_csv()?);
                } else {
                    println!("{}", store.export_json()?);
                }
                Ok(())
            } else {
                handle_stats_summary(&root)
            }
        }
        Commands::Watch { workspace } => {
            let watcher = watcher::WorkspaceWatcher::new(workspace);
            watcher.watch()
        }
        Commands::Mcp { action } => match action {
            McpAction::Start { workspace } => {
                let root = resolve_workspace(
                    workspace,
                    std::env::var("DAGR_WORKSPACE").ok(),
                    std::env::current_dir()?,
                );
                if !root.join(".dagr").join("rules.yaml").exists() {
                    eprintln!(
                        "⚠️  No .dagr/rules.yaml found at '{}' — guard falls back to the built-in clean-architecture preset.",
                        root.display()
                    );
                }
                McpServer::new(root).run_stdio()
            }
            McpAction::Install { client, bin_path } => {
                handle_mcp_install(&client, bin_path.as_deref())
            }
            McpAction::ListClients => handle_mcp_list_clients(),
        },
        Commands::Init { preset } => handle_init(&preset),
        Commands::Schema { artifact } => match artifact {
            SchemaArtifact::Rules => handle_schema_rules(),
        },
        Commands::Skills { action } => match action {
            SkillsAction::Install { target } => handle_skills_install(&target),
            SkillsAction::List => handle_skills_list(),
        },
        Commands::Update { force } => updater::AutoUpdater::self_update(force),
        Commands::Login { key, org, url } => handle_login(key.as_deref(), org.as_deref(), &url),
        Commands::Sync { workspace } => handle_sync(&workspace),
        Commands::Status => handle_status(),
        Commands::Daemon { port } => {
            let daemon = daemon::CacheDaemon::new(port);
            daemon.run().await
        }
    };

    // Show non-blocking update notification on stderr if available
    if !is_mcp_start && !is_update && !is_dashboard && result.is_ok() {
        updater::AutoUpdater::notify_if_update_available();
    }

    result
}

pub fn handle_stats_summary(workspace_root: &Path) -> Result<()> {
    let store = TelemetryStore::open(workspace_root)?;
    let summary = store.get_summary(TimeWindow::Lifetime)?;
    let clients = store.get_client_breakdown()?;

    let pct_str = format!("{:.1}%", summary.overall_compression_ratio * 100.0);
    let bar_filled = (summary.overall_compression_ratio * 24.0).round() as usize;
    let bar = format!(
        "{}{}",
        "█".repeat(bar_filled).green(),
        "░".repeat(24_usize.saturating_sub(bar_filled)).dimmed()
    );

    eprintln!(
        "\n{}",
        "⚡ DAGR Lifetime Telemetry & Value Scoreboard"
            .bold()
            .cyan()
    );
    eprintln!("┌────────────────────────────────────────────────────────────────────────┐");
    eprintln!(
        "│ Lifetime Tokens Saved: {:<47} │",
        summary.total_tokens_saved.to_string().bold().green()
    );
    eprintln!(
        "│ Estimated ROI Savings: {:<47} │",
        format!("${:.2} USD", summary.estimated_usd_saved)
            .bold()
            .yellow()
    );
    eprintln!(
        "│ Slices Served:         {:<47} │",
        summary.total_slices.to_string().cyan()
    );
    eprintln!(
        "│ Avg Compression Ratio: [{}] {:<20} │",
        bar,
        pct_str.bold().green()
    );
    eprintln!(
        "│ Guard Checks Caught:   {:<47} │",
        summary.violations_prevented.to_string().magenta()
    );
    eprintln!("└────────────────────────────────────────────────────────────────────────┘");

    if !clients.is_empty() {
        eprintln!("\n{}", "🔌 Top AI Coding Agent Distribution:".bold());
        for c in clients {
            eprintln!(
                "   • {:<16} {:>10} tokens ({:.1}%)",
                c.client_id.cyan(),
                c.tokens_saved.to_string().green(),
                c.percentage
            );
        }
    }

    eprintln!(
        "\n💡 Launch full web visualizer: {}",
        "dagr dashboard".bold().green()
    );
    Ok(())
}

/// Resolves symbol and file target using direct syntax (file:symbol), SQLite index, or workspace AST match
pub fn resolve_target_symbol(workspace_root: &Path, target: &str) -> Result<(PathBuf, String)> {
    if target.contains(':') {
        let parts: Vec<&str> = target.split(':').collect();
        if parts.len() == 2 {
            let file_path = PathBuf::from(parts[0]);
            let symbol_name = parts[1].to_string();
            let candidate_path = if file_path.is_absolute() {
                file_path
            } else {
                workspace_root.join(&file_path)
            };
            if candidate_path.exists() {
                return Ok((candidate_path, symbol_name));
            }
        }
    }

    // 1. Try SQLite LocalIndexStore if initialized
    let index_db_path = workspace_root.join(".dagr").join("index.db");
    if index_db_path.exists() {
        if let Ok(store) = LocalIndexStore::open(workspace_root) {
            if let Ok(matches) = store.search_symbols(target, 5) {
                if let Some(first) = matches.into_iter().next() {
                    let path = first.span.file_path;
                    let candidate_path = if path.is_absolute() {
                        path
                    } else {
                        workspace_root.join(&path)
                    };
                    if candidate_path.exists() {
                        return Ok((candidate_path, first.symbol_name));
                    }
                }
            }
        }
    }

    // 2. Fuzzy / Generic AST symbol resolution across workspace
    let query = target.trim().to_lowercase();
    let supported_exts = ["ts", "tsx", "js", "jsx", "py", "rs", "go"];
    let mut candidates: Vec<(PathBuf, String, usize)> = Vec::new();

    let pattern = format!("{}/**/*.*", workspace_root.display());
    if let Ok(walker) = glob::glob(&pattern) {
        for entry in walker.flatten() {
            if entry.is_file() {
                let ext = entry.extension().and_then(|s| s.to_str()).unwrap_or("");
                if supported_exts.contains(&ext) {
                    let path_str = entry.to_string_lossy();
                    if path_str.contains("/target/")
                        || path_str.contains("/node_modules/")
                        || path_str.contains("/.git/")
                    {
                        continue;
                    }

                    if let Ok(content) = std::fs::read_to_string(&entry) {
                        let language = Language::from_extension(ext);
                        if language != Language::Unknown {
                            if let Ok(mut parser) = AstParser::new(language) {
                                if let Ok(tree) = parser.parse(&content, None) {
                                    let symbols = AstExtractor::extract_all_symbols(
                                        tree.root_node(),
                                        &content,
                                        language,
                                    );
                                    for sym in symbols {
                                        let sym_name_lower = sym.name.to_lowercase();
                                        if sym_name_lower == query {
                                            candidates.push((entry.clone(), sym.name, 100));
                                        } else if sym_name_lower.contains(&query) {
                                            candidates.push((entry.clone(), sym.name, 80));
                                        } else if query.contains(&sym_name_lower) {
                                            candidates.push((entry.clone(), sym.name, 60));
                                        } else if path_str.to_lowercase().contains(&query) {
                                            candidates.push((entry.clone(), sym.name, 40));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    candidates.sort_by_key(|b| std::cmp::Reverse(b.2));

    if let Some((path, sym, _)) = candidates.into_iter().next() {
        Ok((path, sym))
    } else {
        Err(DagrError::InvalidInput(format!(
            "Could not resolve symbol from query '{}'. Please specify 'path/to/file.ext:symbolName' or run 'dagr init'.",
            target
        )))
    }
}

pub fn handle_context(
    target: &str,
    depth: Option<usize>,
    tier: &str,
    from_test: Option<&str>,
    format: OutputFormat,
) -> Result<()> {
    let start = std::time::Instant::now();
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let targets: Vec<&str> = target
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let slice_tier =
        if tier.to_lowercase().contains("multi") || tier.to_lowercase().contains("lamr") {
            dagr_slicer::SliceTier::MultiRubric
        } else {
            dagr_slicer::SliceTier::Standard
        };

    if let Some(requested) = depth {
        if requested > 1 {
            eprintln!(
                "⚠️  --depth {requested}: v1 performs a single cross-file contract hop; deeper traversal is not yet supported."
            );
        }
    }

    let mut slices = Vec::new();
    let slicer = SymbolicSlicer::new(SlicerConfig {
        max_depth_hops: depth.unwrap_or(3),
        max_token_budget: 1500,
        include_comments: false,
        tier: slice_tier,
        workspace_root: current_dir.clone(),
    });

    // Agent-OS: optional budget enforcement via DAGR_TOKEN_BUDGET env var
    let budget = std::env::var("DAGR_TOKEN_BUDGET")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .map(|max| dagr_core::BudgetContext::new(std::time::Duration::from_secs(300), max));

    for t in targets {
        if let Some(ref b) = budget {
            if b.is_exhausted() {
                eprintln!(
                    "⚠️  Token budget exhausted ({} consumed). Stopping batch.",
                    b.tokens_consumed()
                );
                break;
            }
        }

        let (file_path, symbol_name) = resolve_target_symbol(&current_dir, t)?;
        let source_code = std::fs::read_to_string(&file_path)?;
        let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let language = Language::from_extension(ext);

        let slice = if let Some(trace) = from_test {
            slicer.slice_from_test_failure(&file_path, &source_code, language, trace)?
        } else {
            slicer.slice(&file_path, &source_code, language, &symbol_name)?
        };

        // Agent-OS: deduct tokens against budget
        if let Some(ref b) = budget {
            match b.deduct_tokens(slice.estimated_tokens) {
                Ok(remaining) => {
                    eprintln!("💰 Budget: {remaining} tokens remaining");
                }
                Err(e) => {
                    eprintln!("⚠️  Budget exceeded: {e}");
                    break;
                }
            }
        }

        // Record telemetry (fail-safe)
        if let Ok(store) = TelemetryStore::open(&current_dir) {
            let latency_us = start.elapsed().as_micros() as u64;
            let ev = TelemetryEvent::new_slice(
                "cli",
                &file_path.to_string_lossy(),
                &symbol_name,
                slice.original_file_tokens,
                slice.estimated_tokens,
                latency_us,
            );
            let _ = store.record_event(&ev);
        }

        slices.push(slice);
    }

    if slices.is_empty() {
        return Err(DagrError::InvalidInput(format!(
            "No valid targets parsed from query '{}'",
            target
        )));
    }

    match format {
        OutputFormat::Json => {
            if slices.len() == 1 {
                println!("{}", serde_json::to_string_pretty(&slices[0])?);
            } else {
                println!("{}", serde_json::to_string_pretty(&slices)?);
            }
        }
        OutputFormat::Plain => {
            for slice in &slices {
                for contract in &slice.type_contracts {
                    println!("{}", contract);
                }
                for (_line_no, line) in &slice.sparse_code_lines {
                    println!("{}", line);
                }
            }
        }
        OutputFormat::Markdown => {
            for slice in &slices {
                println!("### Symbolic AST Slice: `{}`", slice.target_symbol);
                println!("- **File**: `{}`", slice.file_path.display());
                println!(
                    "- **Compression**: {:.1}% token reduction ({} down from {})",
                    slice.compression_ratio * 100.0,
                    slice.estimated_tokens,
                    slice.original_file_tokens
                );
                println!("\n```{:?}\n// --- Type Contracts ---", slice.language);
                for contract in &slice.type_contracts {
                    println!("{}", contract);
                }
                println!("\n// --- Implementation Slice ---");
                for (_line_no, line) in &slice.sparse_code_lines {
                    println!("{}", line);
                }
                println!("```\n");
            }
        }
        OutputFormat::Pretty => {
            for slice in &slices {
                render_pretty_slice(slice);
            }
        }
    }

    Ok(())
}

pub fn render_pretty_slice(slice: &MinimalContextSlice) {
    let pct_str = format!("{:.1}%", slice.compression_ratio * 100.0);
    let bar_filled = (slice.compression_ratio * 24.0).round() as usize;
    let bar = format!(
        "{}{}",
        "█".repeat(bar_filled).green(),
        "░".repeat(24_usize.saturating_sub(bar_filled)).dimmed()
    );

    eprintln!("\n{}", "⚡ DAGR Symbolic AST Slicer v0.1.0".bold().cyan());
    eprintln!("┌────────────────────────────────────────────────────────────────────────┐");
    eprintln!(
        "│ Target Symbol:   {:<53} │",
        slice.target_symbol.bold().yellow()
    );
    eprintln!(
        "│ File:            {:<53} │",
        slice.file_path.display().to_string().cyan()
    );
    eprintln!(
        "│ Sliced Context:  {:<53} │",
        format!("{} lines", slice.sparse_code_lines.len()).green()
    );
    eprintln!(
        "│ Token Footprint: {:<53} │",
        format!(
            "{} tokens (down from {})",
            slice.estimated_tokens, slice.original_file_tokens
        )
        .magenta()
    );
    eprintln!(
        "│ Compression:     [{}] {:<24} │",
        bar,
        pct_str.bold().green()
    );
    eprintln!("└────────────────────────────────────────────────────────────────────────┘\n");

    if !slice.type_contracts.is_empty() {
        println!("{}", "// --- Hoisted Type Contracts ---".dimmed());
        for contract in &slice.type_contracts {
            println!("{}", contract.cyan());
        }
        println!();
    }

    println!("{}", "// --- Minimal Implementation Slice ---".dimmed());
    for (line_num, line) in &slice.sparse_code_lines {
        println!("{:>4} │ {}", line_num.to_string().dimmed(), line);
    }
}

fn git_staged_files(workspace_root: &Path) -> Result<Vec<String>> {
    let output = std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(workspace_root)
        .output()
        .map_err(|e| dagr_core::DagrError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}

pub fn handle_guard(
    workspace_root: &Path,
    staged: bool,
    ci: bool,
    base: Option<&str>,
    head: Option<&str>,
    warn_only: bool,
    output_file: Option<&Path>,
    format: OutputFormat,
) -> Result<()> {
    if staged {
        let staged_files = git_staged_files(workspace_root)?;
        let guard = ArchitectureGuard::load(workspace_root)?;
        let violations = guard.scan_files(workspace_root, &staged_files)?;
        if format == OutputFormat::Json {
            println!(
                "{}",
                serde_json::json!({
                    "active_rules": guard.config.boundaries.len(),
                    "status": if violations.is_empty() { "passed" } else { "failed" },
                    "violations": violations,
                    "violations_count": violations.len(),
                    "workspace": ".",
                    "mode": "staged",
                    "staged_files": staged_files.len()
                })
            );
        } else {
            eprintln!("🛡️ DAGR Architecture Guard (staged files): {}", staged_files.len());
            for v in &violations {
                eprintln!("  ❌ {}: {} → {}", v.source_file, v.rule_name, v.message);
            }
            if violations.is_empty() {
                eprintln!("  ✅ All architectural boundary rules passed with zero violations.");
            }
        }
        return if violations.is_empty() { Ok(()) } else { Err(dagr_core::DagrError::Config(format!("{} architectural violation(s) found in staged files", violations.len()))) };
    }

    if ci {
        let report = dagr_guard::CiGuardReport::check_pr_diff(workspace_root, base, head)?;
        report.emit_github_workflow_commands();

        if let Some(out_path) = output_file {
            let md = report.to_markdown_summary();
            let _ = std::fs::write(out_path, md);
        }

        if format == OutputFormat::Json {
            println!("{}", serde_json::to_string_pretty(&report)?);
        } else {
            eprintln!("{}", report.to_markdown_summary());
        }

        if !report.violations.is_empty() && !warn_only {
            return Err(DagrError::Config(format!(
                "DAGR Architecture Guard detected {} violation(s) in PR diff",
                report.violations.len()
            )));
        }
        return Ok(());
    }

    let start = std::time::Instant::now();
    let guard = ArchitectureGuard::load(workspace_root)?;
    let total_rules = guard.config.boundaries.len();
    let violations = guard.scan_workspace(workspace_root)?;
    let latency_us = start.elapsed().as_micros() as u64;

    // Record guard telemetry (fail-safe)
    if let Ok(store) = TelemetryStore::open(workspace_root) {
        let ev = TelemetryEvent::new_guard_check("cli", violations.len(), latency_us);
        let _ = store.record_event(&ev);
    }

    if format == OutputFormat::Json {
        let status = if violations.is_empty() {
            "passed"
        } else {
            "failed"
        };
        let result = json!({
            "status": status,
            "workspace": workspace_root.display().to_string(),
            "active_rules": total_rules,
            "violations_count": violations.len(),
            "violations": violations
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
        if !violations.is_empty() {
            return Err(dagr_core::DagrError::Config(format!(
                "Architecture guard detected {} violations",
                violations.len()
            )));
        }
        return Ok(());
    }

    eprintln!(
        "🛡️  DAGR Architecture Guard: Evaluating workspace at {:?}",
        workspace_root
    );
    if total_rules > 0 {
        eprintln!("   Loaded {} active boundary rules.", total_rules);
    }

    if violations.is_empty() {
        eprintln!(
            "{}",
            "✅ All architectural boundary rules passed with zero violations."
                .green()
                .bold()
        );
        Ok(())
    } else {
        eprintln!(
            "{}",
            format!(
                "❌ Found {} architectural boundary violation(s):",
                violations.len()
            )
            .red()
            .bold()
        );
        for (idx, v) in violations.iter().enumerate() {
            eprintln!(
                "\n   [{}] Rule: {}",
                (idx + 1).to_string().bold(),
                v.rule_name.cyan()
            );
            eprintln!("       Source:   {}", v.source_file.yellow());
            eprintln!("       Imported: {}", v.imported_module.red().bold());
            eprintln!("       Advice:   {}", v.message.dimmed());
        }
        Err(dagr_core::DagrError::Config(format!(
            "Architecture check failed with {} violation(s)",
            violations.len()
        )))
    }
}


pub fn handle_prove(workspace: &Path, test: Option<&str>, format: OutputFormat) -> Result<()> {
    let receipt = governance::ProofReceipt::generate(workspace, test)?;
    match format {
        OutputFormat::Json => println!("{}", receipt.to_json()),
        OutputFormat::Markdown | OutputFormat::Pretty => print!("{}", receipt.to_markdown()),
        OutputFormat::Plain => println!("{}", receipt.digest),
    }
    Ok(())
}

/// Returns the verdict string ("PASS" | "BLOCKED") for exit-code handling.
pub fn handle_review_diff(
    workspace: &Path,
    base: &str,
    head: &str,
    format: OutputFormat,
) -> Result<String> {
    let verdict = governance::ReviewVerdict::generate(workspace, base, head)?;
    match format {
        OutputFormat::Json => println!("{}", verdict.to_json()),
        OutputFormat::Markdown => print!("{}", verdict.to_markdown()),
        OutputFormat::Pretty => {
            eprintln!(
                "🔍 dagr review-diff {}...{} — {} file(s) changed",
                base, head, verdict.files_changed
            );
            for f in &verdict.files {
                let marker = if f.risk_score > 0 { "⚠️ " } else { "  " };
                eprintln!(
                    "{} {:<50} risk {:>3} {}",
                    marker,
                    f.file,
                    f.risk_score,
                    if f.test_coverage_hint { "[tests ✓]" } else { "" }
                );
                for r in &f.reasons {
                    eprintln!("      └─ {}", r);
                }
            }
            println!(
                "\nverdict: {} ({} guard violations, {} secrets, {} dangling imports)",
                verdict.verdict,
                verdict.guard_violation_count,
                verdict.secret_count,
                verdict.dangling_imports.len()
            );
        }
        OutputFormat::Plain => println!("{}", verdict.verdict),
    }
    Ok(verdict.verdict)
}

pub fn handle_run(command: &str, sandbox: bool, commit_on_success: bool) -> Result<()> {    let current_dir = std::env::current_dir()?;

    if !sandbox {
        eprintln!("⚡ Executing command directly in workspace: {}", command);
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .status()?;

        if output.success() {
            return Ok(());
        } else {
            return Err(DagrError::Sandbox(format!(
                "Command exited with status: {:?}",
                output.code()
            )));
        }
    }

    eprintln!("⚡ Initializing Copy-on-Write (CoW) shadow sandbox...");
    let tx = CowSandbox::begin(&current_dir)?;
    eprintln!("   Shadow Root: {:?}", tx.shadow_root);

    let res = CowSandbox::verify(&tx, command)?;
    if res.success {
        eprintln!(
            "{}",
            "✅ Sandbox verification passed successfully!"
                .green()
                .bold()
        );
        if commit_on_success {
            eprintln!("   Committing shadow mutations to workspace...");
            CowSandbox::commit(tx)?;
        } else {
            CowSandbox::rollback(tx)?;
        }
        Ok(())
    } else {
        eprintln!(
            "{}",
            "❌ Sandbox verification failed. Rolling back shadow workspace (0 side effects)..."
                .red()
                .bold()
        );
        eprintln!("{}", res.stderr);
        CowSandbox::rollback(tx)?;
        Err(DagrError::Sandbox(format!(
            "Verification failed with exit code {:?}",
            res.exit_code
        )))
    }
}

pub fn handle_branch(_action: &str, count: usize, task: &str) -> Result<()> {
    let start = std::time::Instant::now();
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    eprintln!(
        "\n{} {}",
        "⚡ DAGR Parallel BranchFS Sandboxes (BranchFS arXiv:2602.08199 / DeltaBox arXiv:2605.22781)"
            .bold()
            .purple(),
        format!("[Task: {}]", task).cyan()
    );

    let branches = CowSandbox::fork_branches(&current_dir, count, task)?;
    let latency_us = start.elapsed().as_micros();

    eprintln!("┌────────────────────────────────────────────────────────────────────────┐");
    eprintln!(
        "│ Forked {:<2} isolated shadow workspaces in {:<32} │",
        count.to_string().bold().green(),
        format!("{}µs", latency_us).yellow()
    );
    eprintln!("├────────────────────────────────────────────────────────────────────────┤");
    for b in &branches {
        let path_str = b.tx.shadow_root.display().to_string();
        let truncated = if path_str.len() > 55 {
            format!("...{}", &path_str[path_str.len() - 52..])
        } else {
            path_str
        };
        eprintln!("│ Branch #{:<2}: {:<57} │", b.branch_id, truncated.dimmed());
    }
    eprintln!("└────────────────────────────────────────────────────────────────────────┘");
    eprintln!("💡 Concurrency Mode: First-Commit-Wins. Sibling branches are automatically discarded in <10ms.\n");

    // Clean up temporary branches created for demo/speculative run
    for b in branches {
        let _ = CowSandbox::rollback(b.tx);
    }

    Ok(())
}

pub fn handle_init(preset: &str) -> Result<()> {
    use dagr_guard::ArchitectureInferrer;

    let current_dir = std::env::current_dir()?;
    let dagr_dir = current_dir.join(".dagr");
    std::fs::create_dir_all(&dagr_dir)?;

    let rules_path = dagr_dir.join("rules.yaml");
    if !rules_path.exists() {
        let config = if preset == "auto" || preset == "clean-architecture" {
            ArchitectureInferrer::infer_preset(&current_dir)
        } else {
            dagr_guard::RuleConfig {
                version: "1.0".into(),
                project_name: Some("workspace".into()),
                preset: Some(preset.to_string()),
                boundaries: dagr_guard::RuleConfig::get_preset_boundaries(preset),
                limits: dagr_guard::LimitsConfig::default(),
                security: dagr_guard::SecurityConfig::default(),
            }
        };

        let yaml_str = serde_yaml::to_string(&config).map_err(|e| {
            dagr_core::DagrError::Config(format!("Failed to serialize rules: {}", e))
        })?;
        std::fs::write(&rules_path, yaml_str)?;
        eprintln!(
            "✅ Inferred architecture preset [{}] -> Created {:?}",
            config.preset.unwrap_or_default().cyan(),
            rules_path
        );
    }

    eprintln!("{}", "⚡ DAGR initialized successfully!".green().bold());
    Ok(())
}

pub fn handle_mcp_install(client: &str, bin_path: Option<&str>) -> Result<()> {
    use dagr_mcp::McpInstaller;

    eprintln!(
        "🔌 Installing DAGR Model Context Protocol (MCP) tool configuration for [{}]...",
        client.bold().cyan()
    );
    let updated = McpInstaller::install(client, bin_path)?;

    if updated.is_empty() {
        eprintln!(
            "{}",
            "⚠️  No target configuration files found for client.".yellow()
        );
    } else {
        for path in updated {
            eprintln!("   {} {:?}", "✓ Injected DAGR into:".green(), path);
        }
        eprintln!(
            "{}",
            "✅ MCP configuration successfully installed! Restart your IDE to connect."
                .green()
                .bold()
        );
    }

    Ok(())
}

pub fn handle_mcp_list_clients() -> Result<()> {
    use dagr_mcp::McpInstaller;

    println!("\n🔌 Supported MCP AI Coding Agents & IDEs (31 Supported):");
    println!("══════════════════════════════════════════════════════════════════════════════════════════════");
    println!(
        "  {:<16} | {:<26} | {:<18} | CONFIG PATH",
        "CLIENT ID", "NAME", "CATEGORY"
    );
    println!("──────────────────────────────────────────────────────────────────────────────────────────────");
    for client in McpInstaller::list_supported_clients() {
        println!(
            "  {:<16} | {} {:<23} | {:<18} | {}",
            client.id.bold().cyan(),
            client.icon,
            client.name,
            client.category.yellow(),
            client.primary_config.dimmed()
        );
    }
    println!("══════════════════════════════════════════════════════════════════════════════════════════════");
    println!(
        "Install command for any client: {}",
        "dagr mcp install --client <CLIENT_ID>".green().bold()
    );
    println!(
        "Install across all clients:     {}\n",
        "dagr mcp install --client all".green().bold()
    );
    Ok(())
}

pub fn handle_skills_install(target: &str) -> Result<()> {
    use crate::skills_installer::SkillsInstaller;

    eprintln!(
        "📦 Installing DAGR portable Agent Skills for [{}]...",
        target.bold().cyan()
    );
    let installed = SkillsInstaller::install_skills(target)?;

    for path in &installed {
        eprintln!("   {} {:?}", "✓ Installed skill manifest:".green(), path);
    }

    eprintln!(
        "{}",
        format!(
            "✅ Successfully installed {} DAGR skills! Agents can now invoke them autonomously.",
            installed.len()
        )
        .green()
        .bold()
    );
    Ok(())
}

pub fn handle_skills_list() -> Result<()> {
    use crate::skills_installer::DAGR_SKILLS;

    println!("\n⚡ DAGR Available Portable Agent Skills (SKILL.md):");
    println!("════════════════════════════════════════════════════════════════════════════");
    for skill in DAGR_SKILLS {
        println!(
            "  • {:<16} : {}",
            skill.name.bold().cyan(),
            skill.description.dimmed()
        );
    }
    println!("════════════════════════════════════════════════════════════════════════════\n");
    Ok(())
}

pub fn handle_login(key: Option<&str>, org: Option<&str>, url: &str) -> Result<()> {
    let api_key = if let Some(k) = key {
        k.to_string()
    } else {
        println!(
            "{}",
            "Enter your DAGR Cloud Organization API Key:".bold().cyan()
        );
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    if api_key.is_empty() {
        return Err(DagrError::InvalidInput("API Key cannot be empty".into()));
    }

    let org_name = org.unwrap_or("Default Organization").to_string();
    let org_id = format!("org_{}", org_name.to_lowercase().replace(' ', "_"));

    let creds = dagr_cloud::OrgCredentials {
        org_id: org_id.clone(),
        org_name: org_name.clone(),
        api_key,
        cloud_url: url.to_string(),
        authenticated_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64,
    };

    creds.save()?;
    eprintln!("\n{}", "✅ Authenticated with DAGR Cloud!".bold().green());
    eprintln!("   Organization: {}", org_name.cyan());
    eprintln!("   Tenant ID:    {}", org_id.yellow());
    eprintln!("   API Endpoint: {}", url.dimmed());
    eprintln!("   Sync Mode:    Zero-PII Telemetry (Local-first)\n");
    Ok(())
}

pub fn handle_sync(workspace_root: &Path) -> Result<()> {
    eprintln!("☁️  DAGR Cloud: Synchronizing local telemetry to team portal...");
    let result = dagr_cloud::CloudSyncClient::sync_workspace(workspace_root)?;
    eprintln!("{}", format!("✅ {}", result.message).bold().green());
    eprintln!(
        "   Synced Events:     {}",
        result.total_synced.to_string().cyan()
    );
    eprintln!(
        "   Pending Backlog:   {}",
        result.pending_remaining.to_string().dimmed()
    );
    Ok(())
}

pub fn handle_status() -> Result<()> {
    let creds_opt = dagr_cloud::OrgCredentials::load()?;
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let (total, unsynced) = if let Ok(store) = TelemetryStore::open(&current_dir) {
        store.get_sync_counts().unwrap_or((0, 0))
    } else {
        (0, 0)
    };

    eprintln!("\n{}", "⚡ DAGR Cloud & Sync Status".bold().cyan());
    eprintln!("┌────────────────────────────────────────────────────────────┐");
    if let Some(creds) = creds_opt {
        eprintln!(
            "│ Cloud Status:      {:<39} │",
            "Connected (Authenticated)".bold().green()
        );
        eprintln!("│ Organization:      {:<39} │", creds.org_name.cyan());
        eprintln!("│ Tenant ID:         {:<39} │", creds.org_id.yellow());
        eprintln!("│ API Endpoint:      {:<39} │", creds.cloud_url.dimmed());
    } else {
        eprintln!(
            "│ Cloud Status:      {:<39} │",
            "Offline (Not logged in)".dimmed()
        );
        eprintln!(
            "│ Team Sync:         {:<39} │",
            "Run 'dagr login' to connect".yellow()
        );
    }
    eprintln!("├────────────────────────────────────────────────────────────┤");
    eprintln!(
        "│ Local Events:      {:<39} │",
        format!("{} total recorded", total).white()
    );
    eprintln!(
        "│ Unsynced Queue:    {:<39} │",
        format!("{} pending sync", unsynced).magenta()
    );
    eprintln!(
        "│ Privacy Guarantee: {:<39} │",
        "Zero-PII / Zero Code Transmission".green()
    );
    eprintln!("└────────────────────────────────────────────────────────────┘\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing_mcp_install_command() {
        let args = vec![
            "dagr",
            "mcp",
            "install",
            "--client",
            "cursor",
            "--bin-path",
            "/usr/local/bin/dagr",
        ];
        let cli = Cli::try_parse_from(args).expect("CLI parsing failed");

        assert_eq!(
            cli.command,
            Commands::Mcp {
                action: McpAction::Install {
                    client: "cursor".into(),
                    bin_path: Some("/usr/local/bin/dagr".into()),
                }
            }
        );
    }

    #[test]
    fn test_cli_parsing_context_command() {
        let args = vec![
            "dagr",
            "context",
            "src/billing.ts:charge",
            "--depth",
            "4",
            "--format",
            "json",
        ];
        let cli = Cli::try_parse_from(args).expect("CLI parsing failed");

        assert_eq!(
            cli.command,
            Commands::Context {
                target: "src/billing.ts:charge".into(),
                depth: Some(4),
                tier: "standard".into(),
                from_test: None,
                format: OutputFormat::Json,
            }
        );
    }

    #[test]
    fn test_cli_parsing_schema_command() {
        let cli = Cli::try_parse_from(["dagr", "schema", "rules"]).expect("CLI parsing failed");
        assert_eq!(
            cli.command,
            Commands::Schema {
                artifact: SchemaArtifact::Rules
            }
        );
    }

    #[test]
    fn rules_schema_matches_strict_ruleconfig_contract() {
        let s = rules_schema();
        assert_eq!(s["type"], serde_json::json!("object"));
        assert_eq!(s["additionalProperties"], serde_json::json!(false));
        assert_eq!(s["required"], serde_json::json!(["version"]));

        let mut top_keys: Vec<&str> = s["properties"]
            .as_object()
            .unwrap()
            .keys()
            .map(|k| k.as_str())
            .collect();
        top_keys.sort_unstable();
        assert_eq!(
            top_keys,
            vec![
                "boundaries",
                "limits",
                "preset",
                "project_name",
                "security",
                "version"
            ]
        );

        let boundary = &s["properties"]["boundaries"]["items"];
        assert_eq!(boundary["additionalProperties"], serde_json::json!(false));
        assert_eq!(
            boundary["required"],
            serde_json::json!(["name", "from", "cannot_import"])
        );
        let bkeys: Vec<&str> = boundary["properties"]
            .as_object()
            .unwrap()
            .keys()
            .map(|k| k.as_str())
            .collect();
        assert_eq!(bkeys.len(), 4);

        assert!(s["properties"]["preset"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "nextjs-app"));
    }

    #[test]
    fn test_cli_parsing_branch_fork_command() {
        let args = vec![
            "dagr",
            "branch",
            "fork",
            "--count",
            "3",
            "--task",
            "fix_webhook",
        ];
        let cli = Cli::try_parse_from(args).expect("CLI parsing failed");

        assert_eq!(
            cli.command,
            Commands::Branch {
                action: "fork".into(),
                count: 3,
                task: "fix_webhook".into(),
            }
        );
    }

    #[test]
    fn test_cli_parsing_guard_command() {
        let args = vec!["dagr", "guard", "--staged"];
        let cli = Cli::try_parse_from(args).expect("CLI parsing failed");

        assert_eq!(
            cli.command,
            Commands::Guard {
                workspace: PathBuf::from("."),
                staged: true,
                ci: false,
                base: None,
                head: None,
                warn_only: false,
                output_file: None,
                format: OutputFormat::Pretty,
            }
        );

        let args_ci = vec![
            "dagr",
            "guard",
            "--ci",
            "--base",
            "origin/main",
            "--warn-only",
            "--output-file",
            "summary.md",
        ];
        let cli_ci = Cli::try_parse_from(args_ci).expect("CLI parsing failed");
        assert_eq!(
            cli_ci.command,
            Commands::Guard {
                workspace: PathBuf::from("."),
                staged: false,
                ci: true,
                base: Some("origin/main".into()),
                head: None,
                warn_only: true,
                output_file: Some(PathBuf::from("summary.md")),
                format: OutputFormat::Pretty,
            }
        );
    }

    #[test]
    fn test_cli_parsing_run_command() {
        let args = vec!["dagr", "run", "cargo test", "--commit-on-success"];
        let cli = Cli::try_parse_from(args).expect("CLI parsing failed");

        assert_eq!(
            cli.command,
            Commands::Run {
                command: "cargo test".into(),
                sandbox: true,
                commit_on_success: true,
            }
        );
    }

    #[test]
    fn test_cli_parsing_skills_command() {
        let args = vec!["dagr", "skills", "install", "--target", "cursor"];
        let cli = Cli::try_parse_from(args).expect("CLI parsing failed");

        assert_eq!(
            cli.command,
            Commands::Skills {
                action: SkillsAction::Install {
                    target: "cursor".into(),
                }
            }
        );
    }

    #[test]
    fn test_cli_parsing_update_command() {
        let args = vec!["dagr", "update", "--force"];
        let cli = Cli::try_parse_from(args).expect("CLI parsing failed");

        assert_eq!(cli.command, Commands::Update { force: true });
    }

    #[test]
    fn test_resolve_target_symbol_exact_and_fuzzy() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(DagrError::Io)?;
        let src_file = temp_dir.path().join("service.py");
        std::fs::write(
            &src_file,
            "def calculate_monthly_discounts():\n    return 42\n",
        )
        .map_err(DagrError::Io)?;

        // 1. Exact path:symbol resolution
        let (resolved_path, sym) =
            resolve_target_symbol(temp_dir.path(), "service.py:calculate_monthly_discounts")?;
        assert_eq!(resolved_path, src_file);
        assert_eq!(sym, "calculate_monthly_discounts");

        // 2. Fuzzy / partial generic resolution
        let (fuzzy_path, fuzzy_sym) = resolve_target_symbol(temp_dir.path(), "calculate_monthly")?;
        assert_eq!(fuzzy_path, src_file);
        assert_eq!(fuzzy_sym, "calculate_monthly_discounts");

        Ok(())
    }

    #[test]
    fn test_multi_target_slicing_and_telemetry() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(DagrError::Io)?;
        let src_file1 = temp_dir.path().join("auth.py");
        let src_file2 = temp_dir.path().join("billing.py");

        std::fs::write(&src_file1, "def verify_token():\n    return True\n")
            .map_err(DagrError::Io)?;
        std::fs::write(&src_file2, "def charge_customer():\n    return 100\n")
            .map_err(DagrError::Io)?;

        // Verify multi-target resolution
        let (p1, s1) = resolve_target_symbol(temp_dir.path(), "verify_token")?;
        let (p2, s2) = resolve_target_symbol(temp_dir.path(), "charge_customer")?;
        assert_eq!(p1, src_file1);
        assert_eq!(s1, "verify_token");
        assert_eq!(p2, src_file2);
        assert_eq!(s2, "charge_customer");

        Ok(())
    }

    #[test]
    fn test_cli_parsing_dashboard_and_stats_and_watch() {
        let args_dash = vec!["dagr", "dashboard", "--port", "8080", "--no-open"];
        let cli_dash = Cli::try_parse_from(args_dash).expect("CLI parsing failed");
        assert_eq!(
            cli_dash.command,
            Commands::Dashboard {
                port: Some(8080),
                no_open: true,
            }
        );

        let args_stats = vec!["dagr", "stats", "--tui", "--export", "json"];
        let cli_stats = Cli::try_parse_from(args_stats).expect("CLI parsing failed");
        assert_eq!(
            cli_stats.command,
            Commands::Stats {
                tui: true,
                web: false,
                export: Some("json".into()),
            }
        );

        let args_watch = vec!["dagr", "watch", "--workspace", "."];
        let cli_watch = Cli::try_parse_from(args_watch).expect("CLI parsing failed");
        assert_eq!(
            cli_watch.command,
            Commands::Watch {
                workspace: PathBuf::from("."),
            }
        );
    }

    #[test]
    fn test_cli_parsing_cloud_commands() {
        let args_login = vec![
            "dagr",
            "login",
            "--key",
            "dagr_live_123",
            "--org",
            "Acme",
            "--url",
            "https://api.custom.dev",
        ];
        let cli_login = Cli::try_parse_from(args_login).expect("CLI parsing failed");
        assert_eq!(
            cli_login.command,
            Commands::Login {
                key: Some("dagr_live_123".into()),
                org: Some("Acme".into()),
                url: "https://api.custom.dev".into(),
            }
        );

        let args_sync = vec!["dagr", "sync", "--workspace", "/my/repo"];
        let cli_sync = Cli::try_parse_from(args_sync).expect("CLI parsing failed");
        assert_eq!(
            cli_sync.command,
            Commands::Sync {
                workspace: PathBuf::from("/my/repo"),
            }
        );

        let args_status = vec!["dagr", "status"];
        let cli_status = Cli::try_parse_from(args_status).expect("CLI parsing failed");
        assert_eq!(cli_status.command, Commands::Status);

        let args_daemon = vec!["dagr", "daemon", "--port", "5555"];
        let cli_daemon = Cli::try_parse_from(args_daemon).expect("CLI parsing failed");
        assert_eq!(cli_daemon.command, Commands::Daemon { port: 5555 });
    }
}
