# About Bare

> "The browser that gets out of your way, so the content can shine."

Bare is an experimental browser with a radical focus on what truly matters: **the content**. In an era where websites have become bloated, overflowing with scripts, ads, and tracking, Bare offers a future where you only see what you came to read.

---

## What Makes Bare Different?

### 📄 Pure Markdown Rendering

Bare is designed to display Markdown files directly. When you visit a URL pointing to a `.md` file, it is fetched and displayed immediately in a clean, readable format. No HTML chaos, no distracting CSS, no JavaScript loading.

### 🚫 Zero Tracking. Ever.

Bare does **not** support JavaScript, cookies, or any tracking technology. This isn't a setting you can forget to enable — it's built into the architecture. Your browsing remains private, by design.

### ⚡ Blazing Fast Performance

Without heavy frameworks, scripts, or ads, pages load in milliseconds rather than seconds. Bare fetches and displays only what matters: the content.

### 🌐 Multi-Protocol Support

In addition to standard HTTP/HTTPS, Bare supports:

- **Gemini** (`gemini://`) — A modern, text-based protocol with mandatory encryption
- **Gopher** (`gopher://`) — The classic, menu-driven protocol from 1991
- **Local files** (`file://`) — Open Markdown files directly from your computer

---

## Who is Bare For?

Bare is perfect for:

- **Writers and researchers** who want to read and write without distractions
- **Privacy-conscious users** who don't want to be tracked
- **Minimalists** who prefer simplicity over complexity
- **Technology enthusiasts** curious about alternative web experiences
- **Anyone who misses the original, text-based internet**

---

## What Bare is NOT

❌ **A full-featured browser** — We will never support JavaScript
❌ **An HTML renderer** — Only Markdown is first-class
❌ **A text editor** — View only, not editing
❌ **A social media tool** — No integration with social platforms

---

## Why Create Such a Browser?

The internet has changed dramatically since its inception. Where it was once an open, text-based platform for sharing information, it's now a commercial arena filled with tracking, ads, and distracting elements.

Bare is an attempt to **revive the original spirit** of the internet:

- **Content over presentation**
- **Privacy over surveillance**
- **Simplicity over complexity**
- **User control over platform control**

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

- [Philosophy](./philosophy.md) — What Bare stands for
- [Technology](./technology.md) — What Bare is built with
- [History](./history.md) — The background and inspiration

---

*"For a world where content matters more than animations."*

---

[Back to Home](../index.md) | [GitHub](https://github.com/FrankBurmo/bare) | [License (GPL-3.0)](../LICENSE)
