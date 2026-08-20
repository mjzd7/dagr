# 🗡️ DAGR Official Brand Trademark & Design System

**Official Brand Standards, Vector Geometry, Optical Scaling & Design System**  
*Version 1.0.0 • Maintained by DAGR Core Architecture Team*

---

## 🏛️ 1. Brand Philosophy & Etymology

```mermaid
mindmap
  root((DAGR Brand Identity))
    The Etymology
      Norse Mythology: Personification of Radiant Daylight & Clarity (Dagr)
      Computer Science: Directed Acyclic Graph (DAG) Slicing & AST Hypervisor
      Physical Precision: Surgical Dagger / Sub-Millisecond Blade
    Core Positioning
      Laser Focus for AI Coding Agents
      95% Token Bloat Pruning
      Zero-Trust Sandboxing & Deterministic Rollbacks
    Design Language
      Monochrome Titanium & Apple Space Black Palette
      Pure Euclidean Geometry & Graph Topology
      Effortless 16x16px Optical Sub-Pixel Clarity
```

---

## 💎 2. Official Trademark Logo: *The Tri-Node Euclidean DAG*

The official trademark mark of DAGR is **The Tri-Node Euclidean DAG** (Flagship 06 / 16px Option 03).

```
     1024px Brand Master Mark (Hero & Headers)                 16x16px Optical Favicon & Terminal CLI
     [Full detail with dashed contract bridge]                 [Distilled to 3 solid nodes & 2.5px rails]

               ● - - - - - - - - - ●                                       ●               ●
                \                 /                                         \             /
                 \               /          --- [Optical Scaling] --->       \           /
                  \             /                                             \         /
                   \           /                                                   ⦿
                        ⦿
```

### 📐 Geometry & Construction Rules
* **Angle of Convergence:** Exact **60° equilateral convergence** pointing downward.
* **Vector Rails:** Heavy Euclidean vector lines connecting the two upstream contract satellite nodes directly into the apex target ring (`stroke-linecap="round"`, `stroke-linejoin="round"`).
* **Upstream Satellite Nodes:** 2 solid circular contract nodes at the top left (`(22, 28)`) and top right (`(78, 28)`).
* **Target Execution Apex:** An illuminated double-stroke target ring centered on the bottom vertex junction (`(50, 78)`), symbolizing the isolated, sliced AST execution target.
* **Optical Scalability:** At $\le 24\text{px}$ (favicons, menu bars, CLI prompts), the dashed coordinate bridge intentionally drops out to ensure $100\%$ crisp, unblurred rendering on hardware pixels.

---

## 🎨 3. Official Color Scheme: *Monochrome Titanium*

DAGR uses the **Monochrome Titanium (Apple Space Black & Resend)** color system. Pure OLED black, stark white specular highlights, and liquid platinum hairlines. Timeless, confident, with zero color bias.

### Official Palette Tokens

| Semantic Role | Token Name | Hex / RGBA Code | Purpose & Usage |
| :--- | :--- | :--- | :--- |
| **Canvas Background** | `Pure OLED Black` | `#000000` | Deep page canvas, CLI terminal background |
| **Layer 1 Surface** | `Titanium Obsidian Glass` | `#0D0E12` (`rgba(18, 18, 22, 0.85)`) | Frosted glass cards, navbars, dialogs |
| **Layer 2 Elevation** | `Liquid Titanium` | `#16171D` (`rgba(255, 255, 255, 0.04)`) | Active states, hover lifts, code blocks |
| **Primary Accent** | `Specular Pure White` | `#FFFFFF` | Primary CTAs, active targets, brand logos |
| **Secondary Accent**| `Liquid Platinum` | `#E4E4E7` (`#D4D4D8`) | Hoisted contracts, status tags, pill badges |
| **Muted Metadata** | `Titanium Slate` | `#71717A` (`rgba(255, 255, 255, 0.45)`) | Descriptions, terminal output, comments |
| **Hairline Border** | `Platinum Specular` | `rgba(255, 255, 255, 0.10)` | 1px card outlines & specular rim highlights |

---

## 🔤 4. Typography Standards (Space Grotesk + Geist Stack)

DAGR standardizes on a specialized two-tier typography system: **Space Grotesk** for the authoritative brand wordmark, and **Geist & Geist Mono** for the high-contrast UI and telemetry interfaces.

```html
<!-- Load Official Google Fonts CDN -->
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Geist:wght@300;400;500;600;700;800;900&family=Geist+Mono:wght@400;500;600;700&family=Space+Grotesk:wght@600;700&display=swap" rel="stylesheet">
```

```css
/* 1. Official Brand Wordmark ('dagr') */
font-family: 'Space Grotesk', sans-serif;
font-weight: 700;           /* Bold */
letter-spacing: -0.03em;    /* Proportional deep-tech stance */
text-transform: lowercase;  /* dagr */

/* 2. Display Headings & UI Interface Stack */
font-family: 'Geist', -apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", "Inter", sans-serif;
letter-spacing: -0.035em;   /* Tight optical tracking on headings */

/* 3. Monospace, Code & Telemetry Stack */
font-family: 'Geist Mono', 'JetBrains Mono', 'SF Mono', monospace;
font-feature-settings: 'tnum' 1, 'zero' 1; /* Tabular Numerals & Slashed Zeros */
```

---

## 📦 5. Official Master SVG Assets

### Full Master Logo (`site/assets/logo.svg`)
```xml
<svg width="100" height="100" viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
  <path d="M 22 28 L 50 78 L 78 28" stroke="currentColor" stroke-width="6" stroke-linecap="round" stroke-linejoin="round" fill="none" />
  <line x1="26" y1="28" x2="74" y2="28" stroke="currentColor" stroke-width="2" stroke-dasharray="3 3" opacity="0.45" />
  <circle cx="22" cy="28" r="6" fill="currentColor" />
  <circle cx="78" cy="28" r="6" fill="currentColor" />
  <circle cx="50" cy="78" r="9" fill="#000000" stroke="currentColor" stroke-width="4" />
  <circle cx="50" cy="78" r="2.5" fill="currentColor" />
</svg>
```

### 16x16px Optical Favicon & CLI Glyph (`site/assets/favicon.svg`)
```xml
<svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
  <line x1="3" y1="3" x2="8" y2="12" stroke="#FFFFFF" stroke-width="2.5" stroke-linecap="round" />
  <line x1="13" y1="3" x2="8" y2="12" stroke="#FFFFFF" stroke-width="2.5" stroke-linecap="round" />
  <circle cx="3" cy="3" r="2" fill="#FFFFFF" />
  <circle cx="13" cy="3" r="2" fill="#FFFFFF" />
  <circle cx="8" cy="12" r="2.5" fill="#000000" stroke="#FFFFFF" stroke-width="1.5" />
</svg>
```

---

## 🗄️ 6. Prototype & Exploration Archive

All iterative concepts created during the design process are permanently preserved in the repository for historical reference:

1. **[16x16 Optical Favicon Lab](site/favicon-lab.html):** Testing 6 pixel-grid-aligned marks at native scales.
2. **[The Pure Vertex Master Suite](site/pure-vertex.html):** 6 Euclidean geometry refinements (The Floating Rail Vertex, Monolithic Slit, Continuous Ribbon, Stepped Funnel).
3. **[20 Evergreen Minimalist Logos](site/evergreen-logos.html):** 20 brand marks inspired by Apple, Linear, Git, Vercel, and Raycast.
4. **[Gurkha Kukri + DAG Suite](site/kukri-logos.html):** 10 monochrome vector explorations merging traditional Gurkha Kukri recurve blades with DAG graph topology.
5. **[Original Archetypes](site/logo-concepts.html):** 12 original concept explorations across 3 archetypes.

---

## 🔤 7. Lowercase Brand Identity & The Dot Convention

### Why Lowercase (`dagr`)?
Modern developer tool leaders (**bun, linear, resend, stripe, cursor, supabase, npm, git**) standardize on all-lowercase wordmarks for three strategic reasons:
1. **CLI Ergonomics & Symmetry:** Developers invoke the CLI using `$ dagr`. When the brand is written as `dagr`, the marketing brand name is 1:1 identical to the binary command you run.
2. **Anti-Corporate Confidence:** All-caps acronyms (`DAGR`, `IBM`, `ORACLE`) feel like 1990s legacy defense contractors. Lowercase `dagr` signals a lightweight, fast, modern developer-first tool.
3. **Typographic Rhythm:** In geometric fonts like Geist, `dagr` possesses ascenders and descenders (`d`, `g`, `r`) that form an instantly recognizable silhouette, unlike the homogeneous rectangle of `DAGR`.

### The Role of the Dot (`.dagr` vs `dagr.` vs `dagr`)
* **`dagr` (Canonical Product Mark):** Used across websites, titles, documentation, and npm/brew packages (`brew install dagr`).
* **`.dagr` (UNIX Daemon / Hidden Runtime State):** Used when referencing the local machine configuration and shadow workspace directory (`~/.dagr/`, `.dagrrc`, `.dagr/shadow/`).
* **`dagr.` / `dagr.dev` (Editorial & Domain Anchor):** Used for domain identity (`dagr.dev`) or punchy editorial statements ("*Sub-millisecond AST slicing. Period.*").
