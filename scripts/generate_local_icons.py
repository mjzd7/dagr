import os

icons_dir = "/Users/mm/orca/projects/ME/DAGR/assets/icons"
os.makedirs(icons_dir, exist_ok=True)

# Clean, professional, high-precision SVG vector icons (24x24 viewBox)
svg_defs = {
    "cursor": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#000000"/>
  <path d="M7 6l10 5.5-4.5 1.5-2 4.5L7 6z" fill="#00E5FF"/>
  <path d="M12.5 13l3.5 3.5" stroke="#FFFFFF" stroke-width="1.5" stroke-linecap="round"/>
</svg>""",

    "claude": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#D97757"/>
  <circle cx="12" cy="12" r="5" fill="#FFFFFF"/>
  <path d="M12 3v3m0 12v3M3 12h3m12 0h3" stroke="#FFFFFF" stroke-width="2" stroke-linecap="round"/>
</svg>""",

    "claudecode": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#1F2937"/>
  <path d="M6 8l4 4-4 4m6 0h6" stroke="#D97757" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
</svg>""",

    "windsurf": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#09B6A2"/>
  <path d="M4 14c4-6 12-6 16 0-4 6-12 6-16 0z" fill="#FFFFFF"/>
  <circle cx="12" cy="14" r="2.5" fill="#09B6A2"/>
</svg>""",

    "vscode": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#007ACC"/>
  <path d="M17.5 3.5l-9 8 9 8 3-1.5V5zM8.5 11.5L5 9l-2 1 3 2-3 2 2 1 3.5-2.5z" fill="#FFFFFF"/>
</svg>""",

    "roocode": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#10B981"/>
  <path d="M7 16c2-4 5-7 9-7 0 3-2 6-5 8H7z" fill="#FFFFFF"/>
  <circle cx="15" cy="8" r="1.5" fill="#FFFFFF"/>
</svg>""",

    "cline": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#2563EB"/>
  <circle cx="9" cy="11" r="1.5" fill="#FFFFFF"/>
  <circle cx="15" cy="11" r="1.5" fill="#FFFFFF"/>
  <rect x="6" y="7" width="12" height="10" rx="3" fill="none" stroke="#FFFFFF" stroke-width="1.5"/>
  <path d="M9 14h6" stroke="#FFFFFF" stroke-width="1.5" stroke-linecap="round"/>
</svg>""",

    "continue": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#6366F1"/>
  <path d="M6 18l6-6-6-6m6 12l6-6-6-6" stroke="#FFFFFF" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
</svg>""",

    "zed": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#18181B"/>
  <path d="M6 7h12l-9 10h9" stroke="#E4E4E7" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
</svg>""",

    "aider": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#F59E0B"/>
  <path d="M7 12l3 3 7-7" stroke="#FFFFFF" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
  <circle cx="12" cy="12" r="8" stroke="#FFFFFF" stroke-width="1.5" fill="none"/>
</svg>""",

    "openinterpreter": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#8B5CF6"/>
  <circle cx="12" cy="12" r="7" stroke="#FFFFFF" stroke-width="1.5" fill="none"/>
  <path d="M12 5a7 7 0 0 1 0 14M5 12h14" stroke="#FFFFFF" stroke-width="1.5"/>
</svg>""",

    "antigravity": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#4285F4"/>
  <path d="M12 4l2.5 5.5L20 12l-5.5 2.5L12 20l-2.5-5.5L4 12l5.5-2.5L12 4z" fill="#FFFFFF"/>
</svg>""",

    "amazonq": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#FF9900"/>
  <path d="M6 10c0-3 3-5 6-5s6 2 6 5v4c0 3-3 5-6 5s-6-2-6-5v-4z" stroke="#FFFFFF" stroke-width="2" fill="none"/>
  <path d="M14 16l3 3" stroke="#FFFFFF" stroke-width="2.5" stroke-linecap="round"/>
</svg>""",

    "jetbrains": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#000000"/>
  <rect x="4" y="4" width="7" height="7" fill="#FC801D"/>
  <rect x="13" y="13" width="7" height="7" fill="#087CFA"/>
  <path d="M6 17h12" stroke="#FFFFFF" stroke-width="2"/>
</svg>""",

    "goose": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#E11D48"/>
  <path d="M8 15c2 2 6 2 8 0 0-4-3-8-6-8-2 0-3 2-3 4 0 2 1 3 1 4z" fill="#FFFFFF"/>
  <circle cx="11" cy="9" r="1" fill="#E11D48"/>
</svg>""",

    "cody": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#FF5543"/>
  <circle cx="9" cy="12" r="2.5" fill="#FFFFFF"/>
  <circle cx="15" cy="12" r="2.5" fill="#FFFFFF"/>
  <path d="M5 9h14v6H5z" stroke="#FFFFFF" stroke-width="1.5" fill="none" rx="2"/>
</svg>""",

    "neovim": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#57A143"/>
  <path d="M6 5v14l4-3V5zm8 0v14l4-3V5zM10 5l8 14" stroke="#FFFFFF" stroke-width="1.5" fill="none"/>
</svg>""",

    "emacs": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#7F5AB6"/>
  <circle cx="12" cy="12" r="7" fill="none" stroke="#FFFFFF" stroke-width="2"/>
  <path d="M8 12c2-3 6-3 8 0-2 3-6 3-8 0z" fill="#FFFFFF"/>
</svg>""",

    "devin": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#3B82F6"/>
  <path d="M6 6h6c4 0 7 3 7 6s-3 6-7 6H6V6z" stroke="#FFFFFF" stroke-width="2" fill="none"/>
  <circle cx="12" cy="12" r="2" fill="#FFFFFF"/>
</svg>""",

    "opencode": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#475569"/>
  <circle cx="12" cy="12" r="5" fill="none" stroke="#FFFFFF" stroke-width="2"/>
  <path d="M12 4v3m0 10v3M4 12h3m10 0h3M6.3 6.3l2.1 2.1m7.2 7.2l2.1 2.1M6.3 17.7l2.1-2.1m7.2-7.2l2.1-2.1" stroke="#FFFFFF" stroke-width="1.5"/>
</svg>""",

    "melty": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#D97706"/>
  <path d="M12 4l7 4v8l-7 4-7-4V8l7-4z" fill="none" stroke="#FFFFFF" stroke-width="2"/>
  <circle cx="12" cy="12" r="2.5" fill="#FFFFFF"/>
</svg>""",

    "pearai": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#84CC16"/>
  <path d="M12 6c-2 0-4 3-4 6 0 4 2 7 4 7s4-3 4-7c0-3-2-6-4-6z" fill="#FFFFFF"/>
  <path d="M12 4c0 2 1 2 2 2" stroke="#FFFFFF" stroke-width="1.5"/>
</svg>""",

    "trae": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#EC4899"/>
  <path d="M6 8h12M12 8v10m-4 0h8" stroke="#FFFFFF" stroke-width="2.5" stroke-linecap="round"/>
</svg>""",

    "boltdiy": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#EAB308"/>
  <path d="M13 3L6 13h5l-1 8 8-11h-5l1-7z" fill="#FFFFFF"/>
</svg>""",

    "dify": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#155EEF"/>
  <rect x="6" y="6" width="5" height="5" rx="1" fill="#FFFFFF"/>
  <rect x="13" y="6" width="5" height="5" rx="1" fill="#FFFFFF"/>
  <rect x="6" y="13" width="5" height="5" rx="1" fill="#FFFFFF"/>
  <rect x="13" y="13" width="5" height="5" rx="1" fill="#FFFFFF"/>
</svg>""",

    "langchain": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#059669"/>
  <circle cx="8" cy="8" r="3" fill="#FFFFFF"/>
  <circle cx="16" cy="16" r="3" fill="#FFFFFF"/>
  <path d="M10 10l4 4" stroke="#FFFFFF" stroke-width="2"/>
</svg>""",

    "crewai": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#DC2626"/>
  <circle cx="8" cy="10" r="2.5" fill="#FFFFFF"/>
  <circle cx="16" cy="10" r="2.5" fill="#FFFFFF"/>
  <circle cx="12" cy="15" r="2.5" fill="#FFFFFF"/>
</svg>""",

    "autogen": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#008AD7"/>
  <rect x="5" y="5" width="6" height="6" fill="#F25022"/>
  <rect x="13" y="5" width="6" height="6" fill="#7FBA00"/>
  <rect x="5" y="13" width="6" height="6" fill="#00A4EF"/>
  <rect x="13" y="13" width="6" height="6" fill="#FFB900"/>
</svg>""",

    "librechat": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#334155"/>
  <path d="M5 6h14v9H9l-4 4V6z" fill="#FFFFFF"/>
  <circle cx="9" cy="10.5" r="1" fill="#334155"/>
  <circle cx="12" cy="10.5" r="1" fill="#334155"/>
  <circle cx="15" cy="10.5" r="1" fill="#334155"/>
</svg>""",

    "superagent": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#C026D3"/>
  <circle cx="12" cy="12" r="6" fill="none" stroke="#FFFFFF" stroke-width="2"/>
  <path d="M12 6v12M6 12h12" stroke="#FFFFFF" stroke-width="1.5"/>
</svg>""",

    "workspace": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <rect width="24" height="24" rx="5" fill="#F05032"/>
  <path d="M18 13.5a2.5 2.5 0 1 1-3.5-2.3V7.8a2.5 2.5 0 1 1 2 0v3.4c.9.5 1.5 1.4 1.5 2.3z" fill="#FFFFFF"/>
  <path d="M8.5 7a2.5 2.5 0 1 1 0 5 2.5 2.5 0 0 1 0-5z" fill="#FFFFFF"/>
  <path d="M9.5 12l5 2.5" stroke="#FFFFFF" stroke-width="1.5"/>
</svg>"""
}

for name, content in svg_defs.items():
    file_path = os.path.join(icons_dir, f"{name}.svg")
    with open(file_path, "w") as f:
        f.write(content.strip() + "\n")
    print(f"Generated clean local asset: {name}.svg")

print(f"\n✅ Successfully created all {len(svg_defs)} local vector SVGs in assets/icons/!")
