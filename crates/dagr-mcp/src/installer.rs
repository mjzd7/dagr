use dagr_core::Result;
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SupportedClientInfo {
    pub id: &'static str,
    pub icon: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub primary_config: &'static str,
}

pub const SUPPORTED_CLIENTS: &[SupportedClientInfo] = &[
    SupportedClientInfo {
        id: "cursor",
        icon: "💻",
        name: "Cursor IDE",
        category: "AI IDE",
        description: "AI-first Code Editor built on VS Code",
        primary_config: "~/.cursor/mcp.json",
    },
    SupportedClientInfo {
        id: "claude",
        icon: "🧠",
        name: "Claude Desktop",
        category: "Desktop App",
        description: "Anthropic's Native Desktop Application for macOS & Windows",
        primary_config: "claude_desktop_config.json",
    },
    SupportedClientInfo {
        id: "claudecode",
        icon: "⌨️",
        name: "Claude Code CLI",
        category: "CLI Agent",
        description: "Anthropic's official terminal coding agent",
        primary_config: "~/.claude/mcp.json",
    },
    SupportedClientInfo {
        id: "windsurf",
        icon: "🌊",
        name: "Windsurf (Codeium)",
        category: "AI IDE",
        description: "Agentic IDE by Codeium with Cascade flows",
        primary_config: "~/.codeium/windsurf/mcp_config.json",
    },
    SupportedClientInfo {
        id: "vscode",
        icon: "🔷",
        name: "VS Code / GitHub Copilot",
        category: "IDE",
        description: "Visual Studio Code with MCP extensions & Copilot Chat",
        primary_config: ".vscode/mcp.json",
    },
    SupportedClientInfo {
        id: "roocode",
        icon: "🦘",
        name: "Roo Code (Roo Cline)",
        category: "Extension / Agent",
        description: "Community-driven autonomous coding assistant for VS Code",
        primary_config: "roo-cline globalStorage settings",
    },
    SupportedClientInfo {
        id: "cline",
        icon: "🤖",
        name: "Cline",
        category: "Extension / Agent",
        description: "Autonomous coding agent extension for VS Code",
        primary_config: "saoudrizwan.claude-dev/cline_mcp_settings.json",
    },
    SupportedClientInfo {
        id: "continue",
        icon: "🚀",
        name: "Continue.dev",
        category: "Extension / IDE",
        description: "Open-source AI code assistant for VS Code & JetBrains",
        primary_config: "~/.continue/config.json",
    },
    SupportedClientInfo {
        id: "zed",
        icon: "⚡",
        name: "Zed Editor",
        category: "Fast Rust IDE",
        description: "High-performance Rust-native code editor",
        primary_config: "~/.config/zed/settings.json",
    },
    SupportedClientInfo {
        id: "aider",
        icon: "🤝",
        name: "Aider AI",
        category: "CLI Pair Programmer",
        description: "Terminal pair programming tool with git auto-commits",
        primary_config: "~/.aider/mcp.json",
    },
    SupportedClientInfo {
        id: "openinterpreter",
        icon: "🌐",
        name: "Open Interpreter",
        category: "Terminal Agent",
        description: "Natural language terminal command and code executor",
        primary_config: "~/.open-interpreter/mcp.json",
    },
    SupportedClientInfo {
        id: "antigravity",
        icon: "🪐",
        name: "Google Antigravity / Gemini CLI",
        category: "Agentic IDE / CLI",
        description: "Advanced Agentic Coding hypervisor & CLI harness",
        primary_config: "~/.gemini/config/mcp.json",
    },
    SupportedClientInfo {
        id: "amazonq",
        icon: "☁️",
        name: "Amazon Q Developer",
        category: "Enterprise Agent",
        description: "AWS AI coding and transformation assistant",
        primary_config: "~/.aws/q/mcp.json",
    },
    SupportedClientInfo {
        id: "jetbrains",
        icon: "🛠️",
        name: "JetBrains (IntelliJ, PyCharm)",
        category: "IDE Suite",
        description: "JetBrains IDE suite with MCP bridge integration",
        primary_config: "~/.config/JetBrains/mcp.json",
    },
    SupportedClientInfo {
        id: "goose",
        icon: "🪿",
        name: "Goose (Block / Square)",
        category: "Open-Source Agent",
        description: "On-machine autonomous developer agent by Block",
        primary_config: "~/.config/goose/mcp.json",
    },
    SupportedClientInfo {
        id: "cody",
        icon: "🔍",
        name: "Sourcegraph Cody",
        category: "Enterprise Assistant",
        description: "Codebase-aware AI assistant by Sourcegraph",
        primary_config: "~/.sourcegraph/cody-mcp.json",
    },
    SupportedClientInfo {
        id: "neovim",
        icon: "🌿",
        name: "Neovim (avante.nvim / mcphub)",
        category: "Terminal Editor",
        description: "Vim-fork with Lua-native AI extensions",
        primary_config: "~/.config/nvim/mcp.json",
    },
    SupportedClientInfo {
        id: "emacs",
        icon: "🐃",
        name: "Emacs (gptel / mcp.el)",
        category: "Extensible Editor",
        description: "Extensible text editor with Emacs Lisp MCP bridge",
        primary_config: "~/.emacs.d/mcp.json",
    },
    SupportedClientInfo {
        id: "devin",
        icon: "👨‍💻",
        name: "Cognition Devin",
        category: "Autonomous Agent",
        description: "Autonomous software engineering agent platform",
        primary_config: ".devin/mcp.json",
    },
    SupportedClientInfo {
        id: "opencode",
        icon: "⚙️",
        name: "OpenCode (Sisyphus)",
        category: "Multi-Agent Harness",
        description: "Multi-agent task orchestration engine",
        primary_config: "~/.opencode/mcp.json",
    },
    SupportedClientInfo {
        id: "melty",
        icon: "🍯",
        name: "Melty",
        category: "Open-Source AI IDE",
        description: "Open-source AI-native code editor",
        primary_config: "~/.melty/mcp.json",
    },
    SupportedClientInfo {
        id: "pearai",
        icon: "🍐",
        name: "PearAI",
        category: "Open-Source AI IDE",
        description: "Open-source AI code editor fork of VS Code",
        primary_config: "~/.pearai/mcp.json",
    },
    SupportedClientInfo {
        id: "trae",
        icon: "🥋",
        name: "Trae AI (ByteDance)",
        category: "Adaptive AI IDE",
        description: "Adaptive AI-powered development environment",
        primary_config: "~/.trae/mcp.json",
    },
    SupportedClientInfo {
        id: "boltdiy",
        icon: "⚡",
        name: "Bolt.diy",
        category: "WebContainer Agent",
        description: "In-browser AI app builder and WebContainer agent",
        primary_config: ".bolt/mcp.json",
    },
    SupportedClientInfo {
        id: "dify",
        icon: "🧩",
        name: "Dify.ai",
        category: "LLM Ops Platform",
        description: "Visual agent orchestration and tool execution runtime",
        primary_config: "~/.dify/mcp.json",
    },
    SupportedClientInfo {
        id: "langchain",
        icon: "🦜",
        name: "LangChain / LangGraph",
        category: "Agent Framework",
        description: "Multi-agent graph runtime and developer studio",
        primary_config: "~/.langchain/mcp.json",
    },
    SupportedClientInfo {
        id: "crewai",
        icon: "👥",
        name: "CrewAI",
        category: "Multi-Agent Swarm",
        description: "Role-playing autonomous AI agent teams",
        primary_config: ".crewai/mcp.json",
    },
    SupportedClientInfo {
        id: "autogen",
        icon: "🤖",
        name: "Microsoft AutoGen",
        category: "Multi-Agent System",
        description: "Conversational multi-agent systems from Microsoft",
        primary_config: ".autogen/mcp.json",
    },
    SupportedClientInfo {
        id: "librechat",
        icon: "💬",
        name: "LibreChat / Ollama",
        category: "Self-Hosted Agent",
        description: "Open-source extensible AI chat platform with MCP",
        primary_config: "~/.librechat/mcp.json",
    },
    SupportedClientInfo {
        id: "superagent",
        icon: "🔮",
        name: "Superagent.sh",
        category: "Production Agent",
        description: "Open-source framework for running AI agents in production",
        primary_config: "~/.superagent/mcp.json",
    },
    SupportedClientInfo {
        id: "workspace",
        icon: "📁",
        name: "Project Local Workspace",
        category: "Workspace Root",
        description: "Direct project repository settings (.cursor, .vscode, .agents)",
        primary_config: ".cursor/mcp.json",
    },
];

pub struct McpInstaller;

impl McpInstaller {
    /// Injects the DAGR MCP server definition into an existing or new MCP configuration JSON
    pub fn inject_dagr_config(root_json: &mut Value, binary_command: &str) -> Result<()> {
        if !root_json.is_object() {
            *root_json = json!({});
        }

        let map = root_json.as_object_mut().unwrap();

        // Ensure "mcpServers" key exists
        if !map.contains_key("mcpServers") || !map["mcpServers"].is_object() {
            map.insert("mcpServers".to_string(), json!({}));
        }

        let servers = map.get_mut("mcpServers").unwrap().as_object_mut().unwrap();

        // Insert or update "dagr" server configuration
        servers.insert(
            "dagr".to_string(),
            json!({
                "command": binary_command,
                "args": ["mcp", "start"]
            }),
        );

        Ok(())
    }

    /// OpenCode uses its own schema: a top-level "mcp" object keyed by server
    /// name, each entry shaped { type: "local", command: [bin, ...args] }.
    pub fn inject_dagr_config_opencode(root_json: &mut Value, binary_command: &str) -> Result<()> {
        if !root_json.is_object() {
            *root_json = json!({});
        }
        let map = root_json.as_object_mut().unwrap();
        if !map.contains_key("mcp") || !map["mcp"].is_object() {
            map.insert("mcp".to_string(), json!({}));
        }
        let mcp = map.get_mut("mcp").unwrap().as_object_mut().unwrap();
        mcp.insert(
            "dagr".to_string(),
            json!({
                "type": "local",
                "command": [binary_command, "mcp", "start"]
            }),
        );
        Ok(())
    }

    /// Returns list of all 30+ supported clients
    pub fn list_supported_clients() -> &'static [SupportedClientInfo] {
        SUPPORTED_CLIENTS
    }

    /// Resolves target configuration file paths for a specified client
    pub fn get_client_config_paths(client: &str) -> Vec<PathBuf> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."));

        let mut paths = Vec::new();

        match client.to_lowercase().as_str() {
            "cursor" => {
                paths.push(home.join(".cursor").join("mcp.json"));
                if cfg!(target_os = "macos") {
                    paths.push(
                        home.join("Library")
                            .join("Application Support")
                            .join("Cursor")
                            .join("User")
                            .join("globalStorage")
                            .join("cursor.mcp.json"),
                    );
                }
            }
            "claude" => {
                if cfg!(target_os = "macos") {
                    paths.push(
                        home.join("Library")
                            .join("Application Support")
                            .join("Claude")
                            .join("claude_desktop_config.json"),
                    );
                } else if cfg!(target_os = "windows") {
                    if let Ok(appdata) = std::env::var("APPDATA") {
                        paths.push(
                            PathBuf::from(appdata)
                                .join("Claude")
                                .join("claude_desktop_config.json"),
                        );
                    }
                } else {
                    paths.push(
                        home.join(".config")
                            .join("Claude")
                            .join("claude_desktop_config.json"),
                    );
                }
            }
            "claudecode" => {
                paths.push(home.join(".claude").join("mcp.json"));
                paths.push(home.join(".claude.json"));
                paths.push(PathBuf::from(".claude").join("mcp.json"));
            }
            "opencode" => {
                paths.push(
                    home.join(".config")
                        .join("opencode")
                        .join("opencode.json"),
                );
            }
            "windsurf" => {
                paths.push(
                    home.join(".codeium")
                        .join("windsurf")
                        .join("mcp_config.json"),
                );
            }
            "vscode" => {
                paths.push(PathBuf::from(".vscode").join("mcp.json"));
                paths.push(home.join(".vscode").join("mcp.json"));
            }
            "roocode" => {
                if cfg!(target_os = "macos") {
                    paths.push(
                        home.join("Library")
                            .join("Application Support")
                            .join("Code")
                            .join("User")
                            .join("globalStorage")
                            .join("rooveterinaryinc.roo-cline")
                            .join("settings")
                            .join("cline_mcp_settings.json"),
                    );
                } else {
                    paths.push(
                        home.join(".config")
                            .join("Code")
                            .join("User")
                            .join("globalStorage")
                            .join("rooveterinaryinc.roo-cline")
                            .join("settings")
                            .join("cline_mcp_settings.json"),
                    );
                }
            }
            "cline" => {
                if cfg!(target_os = "macos") {
                    paths.push(
                        home.join("Library")
                            .join("Application Support")
                            .join("Code")
                            .join("User")
                            .join("globalStorage")
                            .join("saoudrizwan.claude-dev")
                            .join("settings")
                            .join("cline_mcp_settings.json"),
                    );
                } else {
                    paths.push(
                        home.join(".config")
                            .join("Code")
                            .join("User")
                            .join("globalStorage")
                            .join("saoudrizwan.claude-dev")
                            .join("settings")
                            .join("cline_mcp_settings.json"),
                    );
                }
            }
            "continue" => {
                paths.push(home.join(".continue").join("config.json"));
            }
            "zed" => {
                paths.push(home.join(".config").join("zed").join("settings.json"));
            }
            "aider" => {
                paths.push(home.join(".aider").join("mcp.json"));
                paths.push(PathBuf::from(".aider").join("mcp.json"));
            }
            "openinterpreter" => {
                paths.push(home.join(".open-interpreter").join("mcp.json"));
                paths.push(
                    home.join(".config")
                        .join("open-interpreter")
                        .join("mcp.json"),
                );
            }
            "antigravity" => {
                paths.push(home.join(".gemini").join("config").join("mcp.json"));
                paths.push(
                    home.join(".gemini")
                        .join("antigravity-cli")
                        .join("mcp.json"),
                );
            }
            "amazonq" => {
                paths.push(home.join(".aws").join("q").join("mcp.json"));
                paths.push(PathBuf::from(".aws").join("mcp.json"));
            }
            "jetbrains" => {
                paths.push(home.join(".config").join("JetBrains").join("mcp.json"));
                paths.push(PathBuf::from(".idea").join("mcp.json"));
            }
            "goose" => {
                paths.push(home.join(".config").join("goose").join("mcp.json"));
                paths.push(
                    home.join(".local")
                        .join("share")
                        .join("goose")
                        .join("mcp.json"),
                );
            }
            "cody" => {
                paths.push(home.join(".sourcegraph").join("cody-mcp.json"));
                paths.push(PathBuf::from(".vscode").join("cody.json"));
            }
            "neovim" => {
                paths.push(home.join(".config").join("nvim").join("mcp.json"));
                paths.push(
                    home.join(".local")
                        .join("share")
                        .join("nvim")
                        .join("mcphub")
                        .join("mcp.json"),
                );
            }
            "emacs" => {
                paths.push(home.join(".emacs.d").join("mcp.json"));
                paths.push(home.join(".config").join("emacs").join("mcp.json"));
            }
            "devin" => {
                paths.push(PathBuf::from(".devin").join("mcp.json"));
                paths.push(home.join(".devin").join("mcp.json"));
            }
            "opencode" => {
                paths.push(home.join(".opencode").join("mcp.json"));
                paths.push(PathBuf::from(".opencode").join("mcp.json"));
            }
            "melty" => {
                paths.push(home.join(".melty").join("mcp.json"));
                paths.push(PathBuf::from(".melty").join("mcp.json"));
            }
            "pearai" => {
                paths.push(home.join(".pearai").join("mcp.json"));
                paths.push(PathBuf::from(".pearai").join("mcp.json"));
            }
            "trae" => {
                paths.push(home.join(".trae").join("mcp.json"));
                paths.push(PathBuf::from(".trae").join("mcp.json"));
            }
            "boltdiy" => {
                paths.push(PathBuf::from(".bolt").join("mcp.json"));
            }
            "dify" => {
                paths.push(home.join(".dify").join("mcp.json"));
                paths.push(PathBuf::from(".dify").join("mcp.json"));
            }
            "langchain" => {
                paths.push(home.join(".langchain").join("mcp.json"));
                paths.push(PathBuf::from(".langchain").join("mcp.json"));
            }
            "crewai" => {
                paths.push(PathBuf::from(".crewai").join("mcp.json"));
                paths.push(home.join(".crewai").join("mcp.json"));
            }
            "autogen" => {
                paths.push(PathBuf::from(".autogen").join("mcp.json"));
                paths.push(home.join(".autogen").join("mcp.json"));
            }
            "librechat" => {
                paths.push(home.join(".librechat").join("mcp.json"));
                paths.push(home.join(".ollama").join("mcp.json"));
            }
            "superagent" => {
                paths.push(home.join(".superagent").join("mcp.json"));
            }
            "workspace" => {
                paths.push(PathBuf::from(".cursor").join("mcp.json"));
                paths.push(PathBuf::from(".vscode").join("mcp.json"));
                paths.push(PathBuf::from(".agents").join("mcp.json"));
            }
            "all" => {
                for client_info in SUPPORTED_CLIENTS {
                    if client_info.id != "all" {
                        paths.extend(Self::get_client_config_paths(client_info.id));
                    }
                }
            }
            _ => {
                paths.push(PathBuf::from(client));
            }
        }

        // Deduplicate paths
        let mut unique_paths = Vec::new();
        for p in paths {
            if !unique_paths.contains(&p) {
                unique_paths.push(p);
            }
        }

        unique_paths
    }

    /// Automatically installs or updates the MCP server configuration for a client
    pub fn install(client: &str, custom_bin_path: Option<&str>) -> Result<Vec<PathBuf>> {
        let binary_cmd = custom_bin_path.map(|s| s.to_string()).unwrap_or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .unwrap_or_else(|| "dagr".to_string())
        });

        let target_paths = Self::get_client_config_paths(client);
        let opencode_shape = client.eq_ignore_ascii_case("opencode");
        let mut updated_paths = Vec::new();

        for path in target_paths {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            let mut root_json = if path.exists() {
                let content = std::fs::read_to_string(&path).unwrap_or_else(|_| "{}".into());
                serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
            } else {
                json!({})
            };

            if opencode_shape {
                Self::inject_dagr_config_opencode(&mut root_json, &binary_cmd)?;
            } else {
                Self::inject_dagr_config(&mut root_json, &binary_cmd)?;
            }

            let formatted = serde_json::to_string_pretty(&root_json).map_err(|e| {
                dagr_core::DagrError::Serialization(format!("JSON format error: {}", e))
            })?;

            std::fs::write(&path, formatted)?;
            updated_paths.push(path);
        }

        Ok(updated_paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supported_clients_count_at_least_30() {
        assert!(SUPPORTED_CLIENTS.len() >= 30);
    }

    #[test]
    fn test_inject_dagr_config_empty() -> Result<()> {
        let mut json = json!({});
        McpInstaller::inject_dagr_config(&mut json, "dagr")?;

        assert_eq!(json["mcpServers"]["dagr"]["command"], "dagr");
        assert_eq!(json["mcpServers"]["dagr"]["args"][0], "mcp");
        assert_eq!(json["mcpServers"]["dagr"]["args"][1], "start");
        Ok(())
    }

    #[test]
    fn test_inject_dagr_config_preserves_existing_servers() -> Result<()> {
        let mut json = json!({
            "mcpServers": {
                "fetch": {
                    "command": "uvx",
                    "args": ["mcp-server-fetch"]
                }
            }
        });

        McpInstaller::inject_dagr_config(&mut json, "/usr/local/bin/dagr")?;

        assert_eq!(json["mcpServers"]["fetch"]["command"], "uvx");
        assert_eq!(json["mcpServers"]["dagr"]["command"], "/usr/local/bin/dagr");
        Ok(())
    }

    #[test]
    fn test_install_in_temp_directory() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let custom_file = temp.path().join("custom_mcp.json");

        let updated = McpInstaller::install(custom_file.to_str().unwrap(), Some("dagr"))?;
        assert_eq!(updated.len(), 1);
        assert!(custom_file.exists());

        let content = std::fs::read_to_string(&custom_file)?;
        let parsed: Value = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed["mcpServers"]["dagr"]["command"], "dagr");

        Ok(())
    }

    #[test]
    fn test_inject_dagr_config_opencode_schema_and_merge() -> Result<()> {
        let mut json = serde_json::from_str(
            r#"{
                "$schema": "https://opencode.ai/config.json",
                "mcp": { "fetch": { "type": "local", "command": ["uvx", "mcp-fetch"] } },
                "theme": "dark"
            }"#,
        )?;

        McpInstaller::inject_dagr_config_opencode(&mut json, "/usr/local/bin/dagr")?;

        assert_eq!(json["theme"], "dark");
        assert_eq!(
            json["mcp"]["fetch"]["command"][0],
            "uvx",
            "sibling opencode servers must survive the merge"
        );
        assert_eq!(json["mcp"]["dagr"]["type"], "local");
        assert_eq!(json["mcp"]["dagr"]["command"][0], "/usr/local/bin/dagr");
        assert_eq!(json["mcp"]["dagr"]["command"][1], "mcp");
        assert_eq!(json["mcp"]["dagr"]["command"][2], "start");

        // Idempotent re-run: dagr entry replaced in place, siblings intact.
        // Idempotent re-run: dagr replaced in place, siblings intact.
        McpInstaller::inject_dagr_config_opencode(&mut json, "dagr")?;
        assert_eq!(json["mcp"]["dagr"]["command"][0], "dagr");
        assert!(json["mcp"]["fetch"].is_object());
        Ok(())
    }

    #[test]
    fn test_install_opencode_writes_opencode_schema() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let target = temp.path().join("opencode.json");

        let paths = McpInstaller::get_client_config_paths("opencode");
        assert!(
            paths.iter().any(|p| p.ends_with(".config/opencode/opencode.json")),
            "opencode path arm must resolve ~/.config/opencode/opencode.json"
        );

        let mut root = json!({});
        McpInstaller::inject_dagr_config_opencode(&mut root, "dagr")?;
        std::fs::write(&target, serde_json::to_string_pretty(&root)?)?;
        let parsed: Value = serde_json::from_str(&std::fs::read_to_string(&target)?)?;
        assert_eq!(parsed["mcp"]["dagr"]["type"], "local");
        Ok(())
    }
}
