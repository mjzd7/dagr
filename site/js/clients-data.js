const CLIENTS_DATA = [
  { id: "cursor", name: "Cursor IDE", category: "AI IDE", icon: "assets/icons/cursor.svg", config: "~/.cursor/mcp.json", cmd: "dagr mcp install --client cursor", format: "json", rootKey: "mcpServers" },
  { id: "claude", name: "Claude Desktop", category: "Desktop App", icon: "assets/icons/claude.svg", config: "claude_desktop_config.json", cmd: "dagr mcp install --client claude", format: "json", rootKey: "mcpServers" },
  { id: "claudecode", name: "Claude Code CLI", category: "CLI Agent", icon: "assets/icons/claudecode.svg", config: "~/.claude/mcp.json", cmd: "dagr mcp install --client claudecode", format: "json", rootKey: "mcpServers" },
  { id: "windsurf", name: "Windsurf (Codeium)", category: "AI IDE", icon: "assets/icons/windsurf.svg", config: "~/.codeium/windsurf/mcp_config.json", cmd: "dagr mcp install --client windsurf", format: "json", rootKey: "mcpServers" },
  { id: "vscode", name: "VS Code / Copilot", category: "IDE", icon: "assets/icons/vscode.svg", config: ".vscode/mcp.json", cmd: "dagr mcp install --client vscode", format: "json", rootKey: "servers" },
  { id: "roocode", name: "Roo Code (Roo Cline)", category: "VS Code Extension", icon: "assets/icons/roocode.svg", config: "cline_mcp_settings.json", cmd: "dagr mcp install --client roocode", format: "json", rootKey: "mcpServers" },
  { id: "cline", name: "Cline", category: "VS Code Extension", icon: "assets/icons/cline.svg", config: "cline_mcp_settings.json", cmd: "dagr mcp install --client cline", format: "json", rootKey: "mcpServers" },
  { id: "continue", name: "Continue.dev", category: "Extension / IDE", icon: "assets/icons/continue.svg", config: "~/.continue/config.json", cmd: "dagr mcp install --client continue", format: "json", rootKey: "experimental.modelContextProtocolServers" },
  { id: "zed", name: "Zed Editor", category: "Fast Rust Editor", icon: "assets/icons/zed.svg", config: "~/.config/zed/settings.json", cmd: "dagr mcp install --client zed", format: "json", rootKey: "experimental.context_servers" },
  { id: "aider", name: "Aider AI", category: "CLI Pair Programmer", icon: "assets/icons/aider.svg", config: "~/.aider/mcp.json", cmd: "dagr mcp install --client aider", format: "json", rootKey: "mcpServers" },
  { id: "openinterpreter", name: "Open Interpreter", category: "CLI Agent", icon: "assets/icons/openinterpreter.svg", config: "~/.open-interpreter/mcp.json", cmd: "dagr mcp install --client openinterpreter", format: "json", rootKey: "mcpServers" },
  { id: "antigravity", name: "Google Antigravity", category: "Agentic IDE", icon: "assets/icons/antigravity.svg", config: "~/.gemini/config/mcp.json", cmd: "dagr mcp install --client antigravity", format: "json", rootKey: "mcpServers" },
  { id: "amazonq", name: "Amazon Q Developer", category: "Enterprise Agent", icon: "assets/icons/amazonq.svg", config: "~/.aws/q/mcp.json", cmd: "dagr mcp install --client amazonq", format: "json", rootKey: "mcpServers" },
  { id: "jetbrains", name: "JetBrains Suite", category: "IDE Suite", icon: "assets/icons/jetbrains.svg", config: "~/.config/JetBrains/mcp.json", cmd: "dagr mcp install --client jetbrains", format: "json", rootKey: "mcpServers" },
  { id: "goose", name: "Goose (Block)", category: "Open-Source Agent", icon: "assets/icons/goose.svg", config: "~/.config/goose/mcp.json", cmd: "dagr mcp install --client goose", format: "json", rootKey: "mcpServers" },
  { id: "cody", name: "Sourcegraph Cody", category: "Enterprise Assistant", icon: "assets/icons/cody.svg", config: "~/.sourcegraph/cody-mcp.json", cmd: "dagr mcp install --client cody", format: "json", rootKey: "mcpServers" },
  { id: "neovim", name: "Neovim (avante.nvim)", category: "Terminal Editor", icon: "assets/icons/neovim.svg", config: "~/.config/nvim/mcp.json", cmd: "dagr mcp install --client neovim", format: "json", rootKey: "mcpServers" },
  { id: "emacs", name: "Emacs (gptel / mcp.el)", category: "Extensible Editor", icon: "assets/icons/emacs.svg", config: "~/.emacs.d/mcp.json", cmd: "dagr mcp install --client emacs", format: "json", rootKey: "mcpServers" },
  { id: "devin", name: "Cognition Devin", category: "Autonomous Agent", icon: "assets/icons/devin.svg", config: ".devin/mcp.json", cmd: "dagr mcp install --client devin", format: "json", rootKey: "mcpServers" },
  { id: "opencode", name: "OpenCode (Sisyphus)", category: "Multi-Agent Harness", icon: "assets/icons/opencode.svg", config: "~/.opencode/mcp.json", cmd: "dagr mcp install --client opencode", format: "json", rootKey: "mcpServers" },
  { id: "melty", name: "Melty", category: "Open-Source AI IDE", icon: "assets/icons/melty.svg", config: "~/.melty/mcp.json", cmd: "dagr mcp install --client melty", format: "json", rootKey: "mcpServers" },
  { id: "pearai", name: "PearAI", category: "AI Code Editor", icon: "assets/icons/pearai.svg", config: "~/.pearai/mcp.json", cmd: "dagr mcp install --client pearai", format: "json", rootKey: "mcpServers" },
  { id: "trae", name: "Trae AI (ByteDance)", category: "Adaptive IDE", icon: "assets/icons/trae.svg", config: "~/.trae/mcp.json", cmd: "dagr mcp install --client trae", format: "json", rootKey: "mcpServers" },
  { id: "boltdiy", name: "Bolt.diy", category: "Web Agent", icon: "assets/icons/boltdiy.svg", config: ".bolt/mcp.json", cmd: "dagr mcp install --client boltdiy", format: "json", rootKey: "mcpServers" },
  { id: "dify", name: "Dify.ai", category: "LLM Ops", icon: "assets/icons/dify.svg", config: "~/.dify/mcp.json", cmd: "dagr mcp install --client dify", format: "json", rootKey: "mcpServers" },
  { id: "langchain", name: "LangChain / LangGraph", category: "Agent Framework", icon: "assets/icons/langchain.svg", config: "~/.langchain/mcp.json", cmd: "dagr mcp install --client langchain", format: "json", rootKey: "mcpServers" },
  { id: "crewai", name: "CrewAI", category: "Multi-Agent Swarm", icon: "assets/icons/crewai.svg", config: ".crewai/mcp.json", cmd: "dagr mcp install --client crewai", format: "json", rootKey: "mcpServers" },
  { id: "autogen", name: "Microsoft AutoGen", category: "Multi-Agent Framework", icon: "assets/icons/autogen.svg", config: ".autogen/mcp.json", cmd: "dagr mcp install --client autogen", format: "json", rootKey: "mcpServers" },
  { id: "librechat", name: "LibreChat", category: "Self-Hosted Chat", icon: "assets/icons/librechat.svg", config: "~/.librechat/mcp.json", cmd: "dagr mcp install --client librechat", format: "json", rootKey: "mcpServers" },
  { id: "superagent", name: "Superagent", category: "Production Agent", icon: "assets/icons/superagent.svg", config: "~/.superagent/mcp.json", cmd: "dagr mcp install --client superagent", format: "json", rootKey: "mcpServers" },
  { id: "workspace", name: "Workspace Root", category: "Local Git Workspace", icon: "assets/icons/workspace.svg", config: ".cursor/mcp.json", cmd: "dagr mcp install --client workspace", format: "json", rootKey: "mcpServers" }
];

function generateMcpJsonForClient(client) {
  if (!client) return "{}";
  
  if (client.id === "vscode") {
    return JSON.stringify({
      "servers": {
        "dagr": {
          "type": "stdio",
          "command": "dagr",
          "args": ["mcp", "serve"],
          "env": {}
        }
      }
    }, null, 2);
  }

  if (client.id === "zed") {
    return JSON.stringify({
      "context_servers": {
        "dagr": {
          "command": {
            "path": "dagr",
            "args": ["mcp", "serve"]
          }
        }
      }
    }, null, 2);
  }

  if (client.id === "continue") {
    return JSON.stringify({
      "experimental": {
        "modelContextProtocolServers": [
          {
            "transport": {
              "type": "stdio",
              "command": "dagr",
              "args": ["mcp", "serve"]
            }
          }
        ]
      }
    }, null, 2);
  }

  return JSON.stringify({
    "mcpServers": {
      "dagr": {
        "command": "dagr",
        "args": ["mcp", "serve"],
        "env": {}
      }
    }
  }, null, 2);
}
