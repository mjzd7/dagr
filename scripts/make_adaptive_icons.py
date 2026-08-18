import os

icons_dir = os.path.join(os.path.dirname(__file__), "..", "assets", "icons")
os.makedirs(icons_dir, exist_ok=True)

# High-precision vector icons with dynamic Light/Dark mode CSS media queries
adaptive_svgs = {
    "cursor": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #F4F4F5; stroke: #E4E4E7; }
    .fg-cursor { fill: #000000; }
    .accent { fill: #00D8FF; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #18181B; stroke: #27272A; }
      .fg-cursor { fill: #FFFFFF; }
      .accent { fill: #00E5FF; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M7 6l10 5.5-4.5 1.5-2 4.5L7 6z" class="accent"/>
  <path d="M12.5 13l3.5 3.5" class="fg-cursor" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
</svg>""",

    "claude": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #FFF7ED; stroke: #FFEDD5; }
    .claude-body { fill: #D97757; }
    .spark { fill: #C2410C; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #291811; stroke: #431E12; }
      .claude-body { fill: #D97757; }
      .spark { fill: #FDBA74; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <circle cx="12" cy="12" r="5" class="claude-body"/>
  <path d="M12 4v2.5m0 11V20M4 12h2.5m11 0H20" class="spark" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>
</svg>""",

    "claudecode": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #F3F4F6; stroke: #E5E7EB; }
    .prompt { stroke: #D97757; }
    .cursor-line { stroke: #111827; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #1F2937; stroke: #374151; }
      .prompt { stroke: #F97316; }
      .cursor-line { stroke: #F9FAFB; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M6.5 8.5l3.5 3.5-3.5 3.5" class="prompt" fill="none" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
  <line x1="12" y1="15.5" x2="17.5" y2="15.5" class="cursor-line" stroke-width="2" stroke-linecap="round"/>
</svg>""",

    "windsurf": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #ECFDF5; stroke: #D1FAE5; }
    .wave { fill: #09B6A2; }
    .core { fill: #047857; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #062E28; stroke: #0D4E44; }
      .wave { fill: #14B8A6; }
      .core { fill: #5EEAD4; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M4.5 13.5c4-5 11-5 15 0-4 5-11 5-15 0z" class="wave"/>
  <circle cx="12" cy="13.5" r="2.5" class="core"/>
</svg>""",

    "vscode": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #EFF6FF; stroke: #DBEAFE; }
    .ribbon { fill: #007ACC; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #0F172A; stroke: #1E293B; }
      .ribbon { fill: #38BDF8; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M17.5 4.5l-8.5 7.5 8.5 7.5 2.5-1.2V5.7zM9 12L5.5 9.5l-2 .8 2.8 1.7-2.8 1.7 2 .8L9 12z" class="ribbon"/>
</svg>""",

    "roocode": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #ECFDF5; stroke: #A7F3D0; }
    .roo { fill: #059669; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #064E3B; stroke: #047857; }
      .roo { fill: #34D399; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M7 16c2-4 5-7 9-7 0 3-2 6-5 8H7z" class="roo"/>
  <circle cx="15" cy="8" r="1.5" class="roo"/>
</svg>""",

    "cline": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #EFF6FF; stroke: #BFDBFE; }
    .robot { stroke: #2563EB; fill: none; }
    .eye { fill: #2563EB; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #1E3A8A; stroke: #1D4ED8; }
      .robot { stroke: #60A5FA; }
      .eye { fill: #60A5FA; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <circle cx="9" cy="11" r="1.5" class="eye"/>
  <circle cx="15" cy="11" r="1.5" class="eye"/>
  <rect x="6" y="7" width="12" height="10" rx="3" class="robot" stroke-width="1.8"/>
  <path d="M9 14h6" class="robot" stroke-width="1.8" stroke-linecap="round"/>
</svg>""",

    "continue": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #EEF2FF; stroke: #C7D2FE; }
    .arrow { stroke: #4F46E5; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #312E81; stroke: #4338CA; }
      .arrow { stroke: #818CF8; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M6 18l6-6-6-6m6 12l6-6-6-6" class="arrow" fill="none" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
</svg>""",

    "zed": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #F4F4F5; stroke: #E4E4E7; }
    .zed-char { stroke: #09090B; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #18181B; stroke: #27272A; }
      .zed-char { stroke: #FAFAFA; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M6.5 7h11l-8.5 10h8.5" class="zed-char" fill="none" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"/>
</svg>""",

    "aider": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #FEF3C7; stroke: #FDE68A; }
    .aider-icon { stroke: #D97706; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #451A03; stroke: #78350F; }
      .aider-icon { stroke: #FBBF24; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <circle cx="12" cy="12" r="7.5" class="aider-icon" stroke-width="1.8" fill="none"/>
  <path d="M7.5 12.5l3 3 6-6" class="aider-icon" fill="none" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"/>
</svg>""",

    "openinterpreter": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #F5F3FF; stroke: #DDD6FE; }
    .globe { stroke: #7C3AED; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #2E1065; stroke: #4C1D95; }
      .globe { stroke: #A78BFA; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <circle cx="12" cy="12" r="7" class="globe" stroke-width="1.8" fill="none"/>
  <path d="M12 5a7 7 0 0 1 0 14M5 12h14" class="globe" stroke-width="1.8"/>
</svg>""",

    "antigravity": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #EFF6FF; stroke: #BFDBFE; }
    .star { fill: #2563EB; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #1E293B; stroke: #334155; }
      .star { fill: #60A5FA; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M12 4l2.5 5.5L20 12l-5.5 2.5L12 20l-2.5-5.5L4 12l5.5-2.5L12 4z" class="star"/>
</svg>""",

    "amazonq": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #FFFBEB; stroke: #FDE68A; }
    .q-body { stroke: #D97706; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #381A05; stroke: #682E06; }
      .q-body { stroke: #F59E0B; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M6.5 10c0-3 2.5-5 5.5-5s5.5 2 5.5 5v3.5c0 3-2.5 5-5.5 5s-5.5-2-5.5-5V10z" class="q-body" stroke-width="2" fill="none"/>
  <path d="M13.5 15.5l3 3" class="q-body" stroke-width="2.2" stroke-linecap="round"/>
</svg>""",

    "jetbrains": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #F4F4F5; stroke: #E4E4E7; }
    .jb-line { stroke: #18181B; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #18181B; stroke: #27272A; }
      .jb-line { stroke: #FAFAFA; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <rect x="5" y="5" width="6.5" height="6.5" fill="#FC801D" rx="1"/>
  <rect x="12.5" y="12.5" width="6.5" height="6.5" fill="#087CFA" rx="1"/>
  <path d="M7 16h10" class="jb-line" stroke-width="2" stroke-linecap="round"/>
</svg>""",

    "goose": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #FFF1F2; stroke: #FECDD3; }
    .goose-body { fill: #E11D48; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #4C0519; stroke: #881337; }
      .goose-body { fill: #FB7185; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M8 15c2 2 6 2 8 0 0-4-3-8-6-8-2 0-3 2-3 4 0 2 1 3 1 4z" class="goose-body"/>
  <circle cx="11" cy="9" r="1.2" fill="#FFFFFF"/>
</svg>""",

    "cody": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #FFF1F2; stroke: #FFE4E6; }
    .cody-body { stroke: #FF5543; fill: none; }
    .eye { fill: #FF5543; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #450A0A; stroke: #7F1D1D; }
      .cody-body { stroke: #F87171; }
      .eye { fill: #F87171; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <circle cx="9" cy="12" r="2.2" class="eye"/>
  <circle cx="15" cy="12" r="2.2" class="eye"/>
  <rect x="5" y="8.5" width="14" height="7" rx="2.5" class="cody-body" stroke-width="1.8"/>
</svg>""",

    "neovim": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #F0FDF4; stroke: #BBF7D0; }
    .nvim { fill: #16A34A; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #052E16; stroke: #14532D; }
      .nvim { fill: #4ADE80; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M6 5v14l3.5-2.5V5zm8.5 0v14L18 16.5V5zM9.5 5.5l5 13" class="nvim" stroke="currentColor" stroke-width="1.5"/>
</svg>""",

    "emacs": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #FAF5FF; stroke: #E9D5FF; }
    .emacs-icon { stroke: #7C3AED; fill: none; }
    .emacs-core { fill: #7C3AED; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #3B0764; stroke: #581C87; }
      .emacs-icon { stroke: #C084FC; }
      .emacs-core { fill: #C084FC; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <circle cx="12" cy="12" r="7.5" class="emacs-icon" stroke-width="1.8"/>
  <path d="M8 12c2-3 6-3 8 0-2 3-6 3-8 0z" class="emacs-core"/>
</svg>""",

    "devin": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #EFF6FF; stroke: #BFDBFE; }
    .devin-d { stroke: #2563EB; fill: none; }
    .devin-dot { fill: #2563EB; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #172554; stroke: #1E3A8A; }
      .devin-d { stroke: #60A5FA; }
      .devin-dot { fill: #60A5FA; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M6 6h5.5c4 0 6.5 2.5 6.5 6s-2.5 6-6.5 6H6V6z" class="devin-d" stroke-width="2"/>
  <circle cx="11.5" cy="12" r="2" class="devin-dot"/>
</svg>""",

    "opencode": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #F8FAFC; stroke: #E2E8F0; }
    .gear { stroke: #475569; fill: none; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #0F172A; stroke: #1E293B; }
      .gear { stroke: #94A3B8; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <circle cx="12" cy="12" r="4.5" class="gear" stroke-width="2"/>
  <path d="M12 4v2.5m0 11V20M4 12h2.5m11 0H20M6.5 6.5l2 2m7 7l2 2M6.5 17.5l2-2m7-7l2-2" class="gear" stroke-width="1.8" stroke-linecap="round"/>
</svg>""",

    "melty": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #FFFBEB; stroke: #FDE68A; }
    .honey { stroke: #D97706; fill: none; }
    .drop { fill: #D97706; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #451A03; stroke: #78350F; }
      .honey { stroke: #F59E0B; }
      .drop { fill: #F59E0B; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M12 4.5l6.5 3.8v7.4L12 19.5l-6.5-3.8V8.3L12 4.5z" class="honey" stroke-width="2"/>
  <circle cx="12" cy="12" r="2.2" class="drop"/>
</svg>""",

    "pearai": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #F7FEE7; stroke: #D9F99D; }
    .pear { fill: #65A30D; }
    .stem { stroke: #4D7C0F; fill: none; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #14532D; stroke: #166534; }
      .pear { fill: #84CC16; }
      .stem { stroke: #A3E635; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M12 6c-2 0-3.5 2.5-3.5 5 0 3.5 2 6 3.5 6s3.5-2.5 3.5-6c0-2.5-1.5-5-3.5-5z" class="pear"/>
  <path d="M12 4c0 1.5.8 2 1.5 2" class="stem" stroke-width="1.8" stroke-linecap="round"/>
</svg>""",

    "trae": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #FDF2F8; stroke: #FBCFE8; }
    .trae-line { stroke: #DB2777; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #500724; stroke: #831843; }
      .trae-line { stroke: #F472B6; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M6 8h12M12 8v10m-4 0h8" class="trae-line" stroke-width="2.2" stroke-linecap="round"/>
</svg>""",

    "boltdiy": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #FEFCE8; stroke: #FEF08A; }
    .bolt { fill: #CA8A04; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #422006; stroke: #713F12; }
      .bolt { fill: #FACC15; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M13 3.5L6 13h5l-1 7.5 8-11h-5l1-6z" class="bolt"/>
</svg>""",

    "dify": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #EFF6FF; stroke: #BFDBFE; }
    .cube { fill: #1D4ED8; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #1E3A8A; stroke: #1D4ED8; }
      .cube { fill: #60A5FA; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <rect x="5.5" y="5.5" width="5.5" height="5.5" rx="1.2" class="cube"/>
  <rect x="13" y="5.5" width="5.5" height="5.5" rx="1.2" class="cube"/>
  <rect x="5.5" y="13" width="5.5" height="5.5" rx="1.2" class="cube"/>
  <rect x="13" y="13" width="5.5" height="5.5" rx="1.2" class="cube"/>
</svg>""",

    "langchain": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #ECFDF5; stroke: #A7F3D0; }
    .link-dot { fill: #059669; }
    .link-line { stroke: #059669; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #064E3B; stroke: #047857; }
      .link-dot { fill: #34D399; }
      .link-line { stroke: #34D399; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <circle cx="7.5" cy="7.5" r="3" class="link-dot"/>
  <circle cx="16.5" cy="16.5" r="3" class="link-dot"/>
  <line x1="9.5" y1="9.5" x2="14.5" y2="14.5" class="link-line" stroke-width="2.2" stroke-linecap="round"/>
</svg>""",

    "crewai": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #FEF2F2; stroke: #FECACA; }
    .crew { fill: #DC2626; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #450A0A; stroke: #7F1D1D; }
      .crew { fill: #F87171; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <circle cx="8" cy="9.5" r="2.5" class="crew"/>
  <circle cx="16" cy="9.5" r="2.5" class="crew"/>
  <circle cx="12" cy="15" r="2.5" class="crew"/>
</svg>""",

    "autogen": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #F0F9FF; stroke: #E0F2FE; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #0C4A6E; stroke: #075985; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <rect x="5.5" y="5.5" width="5.5" height="5.5" fill="#F25022" rx="1"/>
  <rect x="13" y="5.5" width="5.5" height="5.5" fill="#7FBA00" rx="1"/>
  <rect x="5.5" y="13" width="5.5" height="5.5" fill="#00A4EF" rx="1"/>
  <rect x="13" y="13" width="5.5" height="5.5" fill="#FFB900" rx="1"/>
</svg>""",

    "librechat": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #F1F5F9; stroke: #CBD5E1; }
    .bubble { fill: #334155; }
    .dot { fill: #F1F5F9; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #0F172A; stroke: #334155; }
      .bubble { fill: #E2E8F0; }
      .dot { fill: #0F172A; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M5 6.5h14v8H9l-4 3.5V6.5z" class="bubble"/>
  <circle cx="9" cy="10.5" r="1.1" class="dot"/>
  <circle cx="12" cy="10.5" r="1.1" class="dot"/>
  <circle cx="15" cy="10.5" r="1.1" class="dot"/>
</svg>""",

    "superagent": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #FDF4FF; stroke: #F5D0FE; }
    .portal { stroke: #C026D3; fill: none; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #4A044E; stroke: #701A75; }
      .portal { stroke: #E879F9; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <circle cx="12" cy="12" r="6.5" class="portal" stroke-width="1.8"/>
  <path d="M12 5.5v13M5.5 12h13" class="portal" stroke-width="1.6"/>
</svg>""",

    "workspace": """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24">
  <style>
    .bg { fill: #FFF1F2; stroke: #FFE4E6; }
    .git-node { fill: #F05032; }
    .git-branch { stroke: #F05032; }
    @media (prefers-color-scheme: dark) {
      .bg { fill: #450A0A; stroke: #7F1D1D; }
      .git-node { fill: #FB7185; }
      .git-branch { stroke: #FB7185; }
    }
  </style>
  <rect class="bg" width="24" height="24" rx="6" stroke-width="1"/>
  <path d="M18 13.5a2.5 2.5 0 1 1-3.5-2.3V7.8a2.5 2.5 0 1 1 2 0v3.4c.9.5 1.5 1.4 1.5 2.3z" class="git-node"/>
  <path d="M8.5 7a2.5 2.5 0 1 1 0 5 2.5 2.5 0 0 1 0-5z" class="git-node"/>
  <path d="M9.5 12l5 2.5" class="git-branch" stroke-width="1.6" fill="none"/>
</svg>"""
}

for name, content in adaptive_svgs.items():
    file_path = os.path.join(icons_dir, f"{name}.svg")
    with open(file_path, "w", encoding="utf-8") as f:
        f.write(content.strip() + "\n")
    print(f"Generated adaptive Light/Dark SVG: {name}.svg")

print(f"\n✅ Successfully generated all {len(adaptive_svgs)} adaptive Light/Dark SVGs!")
