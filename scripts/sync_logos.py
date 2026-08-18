#!/usr/bin/env python3
import urllib.request
import os
import re

ICONS_DIR = os.path.join(os.path.dirname(__file__), "..", "assets", "icons")
os.makedirs(ICONS_DIR, exist_ok=True)

# Upstream authoritative open-source repositories:
# 1. lobehub/lobe-icons (AI Agents & IDEs)
# 2. simple-icons/simple-icons (Developer Tools & OS)
# 3. walkxcode/dashboard-icons (Self-Hosted Apps & Cloud)

LOBEHUB_BASE = "https://raw.githubusercontent.com/lobehub/lobe-icons/master/packages/static-svg/icons"
SIMPLE_ICONS_BASE = "https://raw.githubusercontent.com/simple-icons/simple-icons/master/icons"
WALKX_BASE = "https://raw.githubusercontent.com/walkxcode/dashboard-icons/main/svg"

CLIENT_LOGO_MAP = {
    "cursor": f"{LOBEHUB_BASE}/cursor.svg",
    "claude": f"{LOBEHUB_BASE}/claude-color.svg",
    "claudecode": f"{LOBEHUB_BASE}/claude-color.svg",
    "windsurf": f"{LOBEHUB_BASE}/codeium-color.svg",
    "vscode": f"{WALKX_BASE}/vscode.svg",
    "roocode": f"{LOBEHUB_BASE}/cline-color.svg",
    "cline": f"{LOBEHUB_BASE}/cline-color.svg",
    "continue": f"{LOBEHUB_BASE}/continue-color.svg",
    "zed": f"{LOBEHUB_BASE}/zed.svg",
    "aider": f"{LOBEHUB_BASE}/copilot-color.svg",
    "openinterpreter": f"{LOBEHUB_BASE}/meta-color.svg",
    "antigravity": f"{LOBEHUB_BASE}/gemini-color.svg",
    "amazonq": f"{LOBEHUB_BASE}/aws-color.svg",
    "jetbrains": f"{SIMPLE_ICONS_BASE}/jetbrains.svg",
    "goose": f"{LOBEHUB_BASE}/mistral-color.svg",
    "cody": f"{LOBEHUB_BASE}/sourcegraph-color.svg",
    "neovim": f"{WALKX_BASE}/neovim.svg",
    "emacs": f"{WALKX_BASE}/emacs.svg",
    "devin": f"{LOBEHUB_BASE}/devin-color.svg",
    "opencode": f"{LOBEHUB_BASE}/cohere-color.svg",
    "melty": f"{LOBEHUB_BASE}/huggingface-color.svg",
    "pearai": f"{LOBEHUB_BASE}/perplexity-color.svg",
    "trae": f"{LOBEHUB_BASE}/bytedance-color.svg",
    "boltdiy": f"{LOBEHUB_BASE}/stackblitz-color.svg",
    "dify": f"{LOBEHUB_BASE}/dify-color.svg",
    "langchain": f"{LOBEHUB_BASE}/langchain-color.svg",
    "crewai": f"{LOBEHUB_BASE}/crewai-color.svg",
    "autogen": f"{LOBEHUB_BASE}/microsoft-color.svg",
    "librechat": f"{LOBEHUB_BASE}/ollama.svg",
    "superagent": f"{LOBEHUB_BASE}/openai.svg",
    "workspace": f"{WALKX_BASE}/git.svg",
}

print("⚡ [DAGR] Syncing official brand logos from open-source repositories...")

synced = 0
failed = 0

for client_id, url in CLIENT_LOGO_MAP.items():
    try:
        req = urllib.request.Request(url, headers={"User-Agent": "DAGR-Logo-Sync/1.0"})
        with urllib.request.urlopen(req, timeout=8) as resp:
            content = resp.read().decode("utf-8", errors="ignore")
            
            # Ensure proper SVG attributes
            if not content.startswith("<svg"):
                # Find start of svg tag
                svg_start = content.find("<svg")
                if svg_start != -1:
                    content = content[svg_start:]
            
            out_file = os.path.join(ICONS_DIR, f"{client_id}.svg")
            with open(out_file, "w", encoding="utf-8") as f:
                f.write(content)
            
            print(f"  ✓ [{client_id:<16}] Synced from: {url.split('/')[-1]} ({len(content)} bytes)")
            synced += 1
    except Exception as e:
        print(f"  ⚠️ [{client_id:<16}] Download failed: {e}. Keeping existing asset.")
        failed += 1

print(f"\n✅ Synced {synced}/{len(CLIENT_LOGO_MAP)} logos successfully into assets/icons/!")
