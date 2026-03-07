# bare
An open-source browser that only reads Markdown. No scripts, no cookies, no CSS chaos. Just the content you came to read, served in a clean and structured format directly from the source.

> "The internet doesn't have to be heavy."

**Bare** is an experimental browser built to explore a text-based internet.
It ignores traditional web pages and renders only `.md` files directly from HTTP responses.

### Why?
- **Blazing fast:** No heavy frameworks to download.
- **Privacy:** No script support means zero tracking.
- **Focus:** Content is presented consistently regardless of the source.

![Netscape_inspirert_B_animasjon_med_jord](https://github.com/user-attachments/assets/29ca95b2-d09b-4ba4-8293-752f4df8624c)

## Status

✅ **Version 0.1.4** — Core functionality is implemented! The application is fully functional for daily use.

**Completed phases:**
- ✅ Phase 1: Proof of Concept
- ✅ Phase 2: Network support
- ✅ Phase 3: HTML conversion
- ✅ Phase 4: User experience
- ✅ Phase 5 (part 1): Gemini protocol + Gopher protocol

See [PLAN.md](docs/PLAN.md) for the detailed development plan and future extensions.

## Technology

Bare is built with:

- **[Tauri 2.0](https://tauri.app/)** — Lightweight and secure app framework (~2–5 MB vs Electron's ~100 MB)
- **Rust** — Backend for safety and performance
- **[pulldown-cmark](https://crates.io/crates/pulldown-cmark)** — Fast CommonMark + GFM markdown parser
- **[reqwest](https://crates.io/crates/reqwest)** — Async HTTP client with TLS
- **Vanilla HTML/CSS/JS** — Minimal frontend without frameworks

## Features

### Implemented
- ✅ Rendering `.md` files over HTTP/HTTPS
- ✅ Local markdown files (Ctrl+O)
- ✅ HTML-to-Markdown conversion with Readability mode
- ✅ Back/forward navigation with history
- ✅ Bookmarks with persistent storage
- ✅ Light/dark mode with system sync
- ✅ In-page search (Ctrl+F)
- ✅ Zoom in/out (Ctrl+/Ctrl-)
- ✅ Keyboard shortcuts (Vim-inspired)
- ✅ Configurable settings (font, theme, zoom, content width)
- ✅ Three-dot menu for less-used functions
- ✅ About dialog with version information
- ✅ **Gemini protocol support (gemini://)** — Added in v0.1.3
  - Full Gemini protocol implementation
  - TOFU (Trust On First Use) certificate handling
  - Gemtext-to-Markdown conversion
  - Input dialog for interactive Gemini pages
- ✅ **Gopher protocol support (gopher://)** — New in v0.1.4!
  - Full RFC 1436 implementation
  - Gophermap-to-Markdown conversion with emoji icons
  - Support for text files, menus, and search
  - Search dialog for interactive Gopher queries

### Future possibilities
- ⚠️ PDF export
- ⚠️ Tab support
- ⚠️ Custom themes/plugins

## Security & Privacy

Bare is designed with privacy as the top priority:

| Feature | Status | Privacy benefit |
|---------|--------|----------------|
| JavaScript | ❌ Not supported | Zero tracking, no malware |
| Cookies | ❌ Not supported | No third-party tracking |
| CSS | ❌ Minimal/none | No CSS fingerprinting |
| Images | ⚠️ Optional | Prevents tracking pixels |
| Tracking | ❌ Impossible | Total protection |

## Installation

> ⚠️ The project is not yet ready for general use.

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (for Tauri CLI)
- [Tauri Prerequisites](https://tauri.app/v2/guides/prerequisites/)

### Building from source

```bash
# Clone the repository
git clone https://github.com/FrankBurmo/bare.git
cd bare

# Install Tauri CLI
cargo install tauri-cli

# Run in development mode
cargo tauri dev

# Build for production
cargo tauri build
```

## Usage

(Coming when the first release is ready)

## Roadmap

See [PLAN.md](docs/PLAN.md) for a detailed roadmap including:
- 5 development phases from PoC to polished product
- Technical decisions and architecture
- Evaluation of a server component
- Open questions and decisions

## Contributing

Contributions are welcome! Please read [.github/copilot-instructions.md](.github/copilot-instructions.md) for coding standards and project philosophy before submitting pull requests.

### Contribution principles
- **Simplicity first** — Don't add unnecessary complexity
- **Privacy always** — Never compromise on security or privacy
- **Test thoroughly** — Write tests for new functionality
- **Document** — Public APIs should have documentation

## Inspiration

Bare is inspired by:
- **[Gemini Protocol](https://geminiprotocol.net/)** — Minimalist document protocol
- **[Lynx](https://lynx.invisible-island.net/)** — Text-based browser since 1992
- **[Gopher](https://en.wikipedia.org/wiki/Gopher_(protocol))** — Simple document distribution since 1991

## Philosophy

### What Bare IS
- A minimal markdown reader for the modern internet
- A privacy tool
- An experiment in simplicity

### What Bare is NOT
- ❌ A full-featured browser (we will never support JavaScript)
- ❌ An HTML renderer (only markdown is first-class)
- ❌ A text editor (view only, not edit)
- ❌ A social media tool

## License

Bare is licensed under the GNU General Public License v3.0 (GPL-3.0).
See [LICENSE](LICENSE).

## Contact

- **Issues:** [GitHub Issues](https://github.com/FrankBurmo/bare/issues)
- **Discussions:** [GitHub Discussions](https://github.com/FrankBurmo/bare/discussions)

---

*"For a world where content matters more than animations."*
