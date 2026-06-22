# About Bare

> "The browser that gets out of your way, so the content can shine."

Bare is an experimental, open-source **reading instrument** for the text internet. It is not a smaller version of Chrome — it is a different kind of tool that does one thing deliberately: take whatever a server sends and turn it into clean, consistent, private text that *you* control.

---

## What reading in Bare is actually like

The difference is easiest to feel in a single click.

You type an address — or open a local `.md` file, or a `gemini://` capsule, or an old `gopher://` menu. Bare's Rust core fetches **that one document and nothing else**. If it's an ordinary web page, Bare reads the underlying HTML, scores it to find the actual article, and throws away the navigation bars, the cookie wall, the newsletter pop-up, the share buttons, the comment section, and the ad slots. What's left is converted to Markdown, sanitized, and rendered.

You then read it in **your** theme, **your** font, **your** line width, **your** zoom level — the same way every page looks, because presentation is your decision, not the author's. Click a link and the whole calm process repeats. No layout shift, no scripts spinning up, no fonts or beacons loading in the background. Just the next document.

That is the entire product: the web, minus everything that isn't what you came to read.

---

## The problem it solves

Reading something online has quietly become one of the most hostile experiences in computing. A short article can pull in megabytes of JavaScript, a dozen third-party trackers, autoplaying video, and several interruptions before you reach the first sentence — and the page often reflows under your eyes while it loads.

Bare assumes you opened the link to **read the thing**, and removes everything standing between you and that goal. The result is faster, quieter, and private not because you configured it to be, but because the parts that make pages slow and invasive are simply absent.

---

## Who Bare is for

- **Writers and researchers** who want to read long-form text without distractions
- **Privacy-conscious people** who would rather not be measured, profiled, and followed
- **Minimalists** who prefer one calm format to a thousand competing designs
- **Tinkerers and the curious** exploring Gemini, Gopher, and the text internet
- **Anyone who remembers** when the web was mostly documents, and misses it

---

## What Bare is NOT

Being clear about the boundaries is part of the design:

- ❌ **A full-featured web browser** — it will never run JavaScript
- ❌ **A faithful HTML renderer** — only the readable text is first-class; pixel-perfect layouts are not the goal
- ❌ **An editor or publishing tool** — Bare reads; it does not write
- ❌ **A social or "platform" app** — no accounts, no feeds, no sharing widgets

If you need any of those, a conventional browser is the right tool. Bare is intentionally not trying to be one.

---

## An honest status

Bare is **early and experimental**, released under the **GPL-3.0** license and developed in the open. What works today:

- Reading over **HTTP/HTTPS** with readability extraction
- The **Gemini** protocol (with TOFU certificate trust)
- The **Gopher** protocol (RFC 1436)
- **Local Markdown files** via `file://`
- Themes, typography, zoom, bookmarks, in-page search, and a command palette — in 13 languages

It is small, it is opinionated, and it is honest about what it does not do. If that resonates, you are exactly who it was built for.

---

## Getting Started

### 📥 Download

Pre-built binaries are under development. Check [GitHub Releases](https://github.com/FrankBurmo/bare/releases) for the latest builds for Windows, macOS, and Linux.

### 🛠️ Build from Source

Requirements:
- [Rust](https://rustup.rs/) (latest stable version)
- [Node.js](https://nodejs.org/) (for Tauri CLI)
- [Tauri prerequisites](https://tauri.app/v2/guides/prerequisites/)

```bash
git clone https://github.com/FrankBurmo/bare.git
cd bare
cargo install tauri-cli
cargo tauri build
```

---

## Learn More

- [Philosophy](./philosophy.md) — why the reader, not the author, is in charge
- [Technology](./technology.md) — how the reading pipeline is built
- [History](./history.md) — the lineage Bare belongs to, from MultiTorg to Gemini

---

*"For a world where content matters more than animations."*

---

[Back to Home](../index.md) | [GitHub](https://github.com/FrankBurmo/bare) | [License (GPL-3.0)](../LICENSE)

### 📥 Download

Pre-built binaries are under development. Check [GitHub Releases](https://github.com/FrankBurmo/bare/releases) for the latest builds for Windows, macOS, and Linux.

### 🛠️ Build from Source

Requirements:
- [Rust](https://rustup.rs/) (latest stable version)
- [Node.js](https://nodejs.org/) (for Tauri CLI)
- [Tauri prerequisites](https://tauri.app/v2/guides/prerequisites/)

```bash
git clone https://github.com/FrankBurmo/bare.git
cd bare
cargo install tauri-cli
cargo tauri build
```

---

## Learn More

- [Philosophy](./philosophy.md) — What Bare stands for
- [Technology](./technology.md) — What Bare is built with
- [History](./history.md) — The background and inspiration

---

*"For a world where content matters more than animations."*

---

[Back to Home](../index.md) | [GitHub](https://github.com/FrankBurmo/bare) | [License (GPL-3.0)](../LICENSE)
