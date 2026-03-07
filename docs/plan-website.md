# Plan: Bare Browser Website

> Goal: Drive broad adoption of the Bare browser by presenting it clearly, compellingly, and accessibly to developers and privacy-conscious users worldwide.

---

## Overview

A single, static website hosted on **GitHub Pages** (at `frankburmo.github.io/bare` or a custom domain) that serves as the browser's public face. It should load in under a second, require no JavaScript frameworks, and — appropriately — be viewable in Bare itself.

### Core Objectives

1. **Communicate the value proposition** in 5 seconds to a first-time visitor
2. **Lower the barrier to trying Bare** (one-click download or simple build instructions)
3. **Build community** (link to GitHub, Issues, Discussions)
4. **Be a living showcase** — the site itself should embody Bare's values: fast, minimal, no trackers

---

## GitHub Pages Setup

The website lives in the same repository, published from the `docs/` branch or a dedicated `/website` folder. Recommended approach:

| Option | Pros | Cons |
|--------|------|------|
| **`gh-pages` branch** | Keeps `main` clean | Slightly more complex CI |
| **`/website` folder on `main`** | Simple, no extra branch | Mixes source and site |
| **Separate `bare-site` repo** | Full isolation | Loses the "same repo" benefit |

**Recommendation:** Use a `gh-pages` branch with a GitHub Actions workflow that auto-deploys on pushes to `main`. The source lives in a `/website` folder on `main` and is deployed to `gh-pages`.

### GitHub Actions workflow (`.github/workflows/deploy-site.yml`)

```yaml
name: Deploy Website

on:
  push:
    branches: [main]
    paths:
      - 'website/**'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v4
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./website
```

---

## Website Structure

```
website/
├── index.html          # Main landing page (single page)
├── styles.css          # Minimal, fast CSS
├── assets/
│   ├── logo.svg        # Bare logo / animated "B"
│   ├── screenshot.png  # App screenshot (light mode)
│   ├── screenshot-dark.png  # App screenshot (dark mode)
│   └── favicon.ico
└── CNAME               # Optional: custom domain
```

A single-page site is sufficient at launch. Pages can be added later if needed (docs, changelog).

---

## Page Sections

### 1. Hero

- Large headline: **"The browser that gets out of your way."**
- One-sentence subheading: *"Bare renders Markdown directly — no JavaScript, no cookies, no tracking. Just content."*
- Two CTAs:
  - **Download** (links to latest GitHub Release)
  - **View on GitHub** (links to repo)
- Animated logo / app screenshot as a visual anchor

### 2. Why Bare?

Three-column feature highlights (icon + heading + 1-line description):

| Icon | Heading | Description |
|------|---------|-------------|
| ⚡ | Blazing Fast | Pages load in milliseconds, not seconds. No megabytes of JavaScript to parse. |
| 🔒 | Zero Tracking | Scripts and cookies are structurally impossible. Your browsing is yours alone. |
| 📖 | Pure Content | Markdown is rendered consistently regardless of source. No layout surprises. |

### 3. Protocol Support

A compact table or icon grid showing supported protocols:

- 🌐 HTTP / HTTPS
- 🪐 Gemini (`gemini://`)
- 🐝 Gopher (`gopher://`)
- 📁 Local files (`file://`)

Short paragraph: *"Bare speaks the original internet's languages. Navigate Gopher holes and Gemini capsules as first-class citizens alongside the modern web."*

### 4. How It Works

A simple two-step visual flow:

```
Enter a URL  →  Bare fetches the content  →  Renders clean Markdown
```

One short paragraph explaining the philosophy: HTML pages are converted to Markdown via a Readability-style extractor; native `.md` URLs are rendered directly.

### 5. Screenshot / Demo

- Side-by-side: traditional browser rendering a busy webpage vs. Bare rendering the same content as clean Markdown
- Or: an animated GIF / short looping video of the app in use
- Light / dark mode toggle to show both themes

### 6. Getting Started

Tabbed or toggle between **Download** and **Build from source**:

**Download (when releases are available):**
```
[Windows .exe]  [macOS .dmg]  [Linux .AppImage]
```

**Build from source:**
```bash
git clone https://github.com/FrankBurmo/bare.git
cd bare
cargo tauri build
```

Prerequisites list (Rust, Tauri) with links.

### 7. Open Source & Community

- GPL-3.0 badge
- GitHub star button (via shields.io badge or GitHub's native widget)
- Links: Issues · Discussions · Contribute
- Short "Bare is for everyone" paragraph inviting contributions

### 8. Footer

- License: GPL-3.0
- Links: GitHub · Issues · Discussions
- *"Built with Tauri and Rust. No trackers on this site either."*

---

## Design Principles

- **Vanilla HTML/CSS only** — no frameworks, no build tools, no bundlers
- **< 50 KB total page weight** (excluding screenshots)
- **No external dependencies** — no Google Fonts, no CDN scripts, no analytics
- **System fonts** — same as Bare itself: `-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif`
- **Light + dark mode** — via `@media (prefers-color-scheme: dark)`
- **Responsive** — works on mobile without a separate mobile site
- **Accessible** — semantic HTML, sufficient contrast, keyboard navigable
- **Self-hostable in Bare** — the site's own content should be viewable in Bare (provide a `README.md` version at `/index.md`)

### Color palette

| Role | Light | Dark |
|------|-------|------|
| Background | `#ffffff` | `#1a1a1a` |
| Surface | `#f5f5f5` | `#242424` |
| Text | `#1a1a1a` | `#e8e8e8` |
| Accent | `#3a7bd5` | `#6fa3ef` |
| Muted | `#666666` | `#999999` |

---

## SEO & Discoverability

- `<title>Bare — A minimal Markdown browser</title>`
- `<meta name="description">` with a concise value-prop sentence
- Open Graph tags (`og:title`, `og:description`, `og:image`) for link previews on GitHub, Reddit, Twitter/X, Hacker News
- `<link rel="canonical">` to avoid duplicate content if both `github.io` and a custom domain are used
- Screenshot image optimised as the OG image (1200×630 px)

**Target keywords:** markdown browser, privacy browser, minimal browser, gemini browser, gopher browser, Tauri app, Rust browser

---

## Content & Copy Strategy

The website copy should speak to two audiences:

| Audience | Message |
|----------|---------|
| **Developers** | "Built in Rust + Tauri. Open source. Hackable. Respects your machine." |
| **Privacy-conscious users** | "Structurally impossible to track you. Not a policy — an architecture." |

Key differentiators to emphasise:
- Architecture-level privacy (not a setting you can forget to enable)
- Multi-protocol (HTTP, Gemini, Gopher) — unique in the browser space
- Tiny binary size vs. Electron alternatives
- Keyboard-first, distraction-free reading

---

## Distribution & Promotion

Getting the site seen:

| Channel | Action |
|---------|--------|
| **Hacker News** | "Show HN: Bare — a Markdown-only browser with Gemini + Gopher support" |
| **Reddit** | r/rust, r/selfhosted, r/privacy, r/commandline |
| **Gemini/Gopher** | Host a capsule/hole pointing to the project |
| **GitHub Topics** | Tag repo: `markdown`, `browser`, `tauri`, `rust`, `gemini`, `gopher`, `privacy` |
| **Product Hunt** | Launch when v1.0 is ready |
| **Dev.to / Medium** | "Why I built a browser that refuses to run JavaScript" |

---

## Milestones

| Milestone | Description |
|-----------|-------------|
| **M1 — Launch page** | Hero + features + build instructions. Deployed to `gh-pages`. |
| **M2 — Release page** | Add download buttons when binaries are available on GitHub Releases. |
| **M3 — Docs page** | Keyboard shortcuts reference, protocol guide, settings documentation. |
| **M4 — Custom domain** | Point `bare-browser.io` (or similar) to GitHub Pages with HTTPS. |

---

## Implementation Notes

- **No `docs/` conflict:** The static site source lives in `/website/` on `main`, not in `/docs/`. This avoids collision with the documentation files already in `/docs/`.
- **GitHub Pages config:** In repo Settings → Pages, set source to the `gh-pages` branch, root `/`.
- **HTTPS:** GitHub Pages provides free HTTPS automatically (and via Let's Encrypt for custom domains).
- **`index.md` mirror:** Maintain a `website/index.md` that is a Markdown-formatted version of the landing page, so Bare users can actually browse the site in Bare itself — a neat dogfood story.
