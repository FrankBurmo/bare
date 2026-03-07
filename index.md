# Bare — A minimal Markdown browser

> "The browser that gets out of your way."

Bare renders Markdown directly — no JavaScript, no cookies, no tracking. Just the content you came to read.

- [Download (GitHub Releases)](https://github.com/FrankBurmo/bare/releases/latest)
- [View on GitHub](https://github.com/FrankBurmo/bare)

> ⚠️ Early development — build from source or watch releases.

---

## Why Bare?

### ⚡ Blazing Fast

Pages load in milliseconds, not seconds. No megabytes of JavaScript to parse — Bare fetches and renders only what matters.

### 🔒 Zero Tracking

Scripts and cookies are structurally impossible — not a setting you can forget to enable. Your browsing is yours alone, by architecture.

### 📖 Pure Content

Markdown is rendered consistently regardless of the source. No pop-ups, no ad carousels, no layout surprises. Just the words.

---

## Protocol Support

Bare speaks the original internet's languages. Navigate Gopher holes and Gemini capsules as first-class citizens alongside the modern web.

| Protocol | Scheme |
|----------|--------|
| 🌐 HTTP / HTTPS | `https://` |
| 🪐 Gemini | `gemini://` |
| 🐿️ Gopher | `gopher://` |
| 📁 Local files | `file://` |

---

## How It Works

1. Enter a URL in the address bar
2. Bare fetches the content
3. Clean Markdown is rendered

Native `.md` URLs are rendered directly. HTML pages are converted to Markdown via a Readability-style extractor — stripping away ads, navigation, and noise to surface the actual content.

---

## Security & Privacy by Design

Privacy in Bare is not a setting — it's the architecture.

| Feature | Status | Privacy benefit |
|---------|--------|----------------|
| JavaScript | ❌ Not supported | Zero tracking, no malware execution |
| Cookies | ❌ Not supported | No third-party tracking |
| CSS | ❌ Minimal / none | No CSS fingerprinting |
| Images | ⚠️ Optional | Prevents tracking pixels |
| Tracking | ❌ Impossible | Total protection |

---

## Getting Started

### Download

Pre-built binaries are coming soon. Check [GitHub Releases](https://github.com/FrankBurmo/bare/releases) for the latest builds for Windows, macOS, and Linux.

### Build from Source

Requires [Rust](https://rustup.rs/) and the [Tauri prerequisites](https://tauri.app/v2/guides/prerequisites/).

```bash
git clone https://github.com/FrankBurmo/bare.git
cd bare
cargo install tauri-cli
cargo tauri build
```

---

## Open Source & Community

Bare is free and open source under the **GPL-3.0** license. Contributions, bug reports, and ideas are all welcome.

- [Source Code](https://github.com/FrankBurmo/bare)
- [Issues](https://github.com/FrankBurmo/bare/issues)
- [Discussions](https://github.com/FrankBurmo/bare/discussions)
- [GPL-3.0 License](https://github.com/FrankBurmo/bare/blob/main/LICENSE)

Built with **Rust** + **Tauri**. Tiny binary, no Electron bloat.

---

*"For a world where content matters more than animations."*

No trackers on this site. Built with vanilla HTML & CSS.
