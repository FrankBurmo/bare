# Technology

> *"Technology is best when it disappears."* — adapted from Mark Weiser

Bare is not a small web browser. It is a **reading instrument** that happens to speak HTTP, Gemini and Gopher. Every engineering decision serves a single goal: turn whatever a server sends into clean, consistent, private text — and then get out of your way.

This page explains how that is built, honestly, including the things Bare deliberately does *not* do.

---

## 🏗️ The shape of the program

Bare is a [Tauri 2](https://tauri.app/) application: a Rust core wrapped in the operating system's own WebView, with **no bundled browser engine**.

```
Frontend (vanilla JS)  →  Tauri IPC bridge  →  Rust core
   renders finished HTML        message passing      fetch · extract · sanitize · render
```

Why this split matters:

- All networking, parsing and sanitization happen in **Rust** — never inside the page.
- The WebView only ever receives finished, already-sanitized HTML. **It never talks to the network itself.**
- There is no Chromium to ship, so the entire application is a few megabytes instead of a few hundred.

**Why not Electron:**

| | Bare (Tauri) | Electron |
|---|---|---|
| App size | single-digit MB | ~100–200 MB |
| Memory | ~10–20 MB | hundreds of MB |
| Engine | OS-native WebView | bundled Chromium |
| Core language | Rust (memory-safe) | Node.js |

---

## 🔬 The reading pipeline

The heart of Bare is what happens between *"you click a link"* and *"you see text."* For an ordinary HTML page it is four stages, all in Rust:

1. **Fetch** — [reqwest](https://crates.io/crates/reqwest) over `rustls` + `ring` (no OpenSSL, no native-tls), capped at 5 MB, sending an `Accept` header that politely asks the server for Markdown first:
   ```
   Accept: text/markdown, text/plain;q=0.9, text/html;q=0.5
   ```

2. **Extract** — a real **DOM-based readability** pass scores the document tree, lifts out the article, and discards navigation, sidebars, ad slots and comment threads. This replaced an earlier naive string-search method in v0.1.6, because content extraction *is* the core promise — it had to be done properly, not approximated.

3. **Sanitize** — the result is run through [ammonia](https://crates.io/crates/ammonia), an allowlist-based Rust HTML sanitizer, stripping scripts, event handlers and anything that could carry a payload. This runs on the Markdown path too, so untrusted content is cleaned in depth, not only fenced off by policy.

4. **Render** — [pulldown-cmark](https://crates.io/crates/pulldown-cmark) turns the cleaned Markdown into HTML with CommonMark + GitHub extensions (tables, task lists, strikethrough), and `syntect` adds syntax highlighting to code blocks.

Gemtext and gophermaps skip the extraction stage entirely — they are already clean by design and go straight to dedicated converters.

---

## 🔒 Privacy the code actually enforces

Privacy in Bare is not a preference panel; it is a property of the architecture (see [Philosophy](./philosophy.md)). Three mechanisms make it real rather than aspirational:

- **Structural image blocking.** The WebView ships with a strict Content-Security-Policy of `img-src 'self' data:`. The page is *forbidden at the engine level* from loading a remote image, so a tracking pixel cannot fire even if one survives extraction. The `ImageMode` setting (**Block · Placeholder · Show**) puts that choice entirely in your hands.

- **One click, one request.** The WebView never originates network traffic. Only the Rust core fetches, and it fetches exactly the document you asked for — nothing else loads in the background. There are no fonts, analytics beacons, or third-party assets to leak your visit.

- **TOFU for Gemini.** Gemini connections are verified Trust-On-First-Use: a SHA-256 fingerprint of each server's certificate is stored in `known_hosts.json`, and a later mismatch is flagged as a possible machine-in-the-middle attack — the same model SSH uses.

No JavaScript, no cookies, no telemetry. The most private data is the data that is never collected.

---

## ⚡ Speed by subtraction

Bare is fast because it does less, not because it works harder.

- An **LRU render cache** in the Rust core means returning to a recently visited page is instant — the readable HTML is already built.
- Typical pages, stripped to their content, are **5–50 KB** instead of the multi-megabyte payloads of the modern web.
- Async I/O throughout (`tokio`) means fetching never blocks the interface.

**Design targets the project measures itself against:**

| Metric | Target |
|--------|--------|
| Cached re-render | ≈ one frame (~16 ms) |
| Typical page, fetch → readable | well under half a second |
| External network calls beyond the document | zero |
| Application binary | single-digit megabytes |
| Resident memory | ~10–20 MB |

---

## 🌐 Protocol support

| Protocol | What Bare does |
|----------|----------------|
| **HTTP/HTTPS** | GET/HEAD, redirects, TLS via rustls, content negotiation, readability extraction |
| **Gemini** | Full client with TOFU certificates, gemtext → Markdown, interactive input pages |
| **Gopher** | RFC 1436 client, gophermap → Markdown, type-aware icons, search (item type 7) |
| **file://** | Open local Markdown files directly from disk |

Gemtext, for example, converts cleanly because it is already a line-oriented text format:

```gemini
# Heading 1
## Heading 2

This is a paragraph.

=> https://example.com Link description
```

---

## ✅ Quality and trust

Bare is small, but it is not casual about correctness:

- **117+ unit tests** across the core logic — extraction, protocol clients, converters, settings.
- **Idiomatic Rust:** `Result` + `thiserror` everywhere, `?` propagation, almost no `unwrap()` outside tests, per-module error types.
- **A safe stack:** `rustls` with `ring` (no OpenSSL), clamped numeric settings, allowlist sanitization.
- **13 interface languages**, in place unusually early for a project this size.

---

## 🛠️ Build it yourself

Bare is GPL-3.0 and builds from source on Windows, macOS and Linux.

```bash
# Clone the repository
git clone https://github.com/FrankBurmo/bare.git
cd bare

# Install Rust if you don't have it
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install the Tauri CLI
cargo install tauri-cli

# Run in development with hot reload
cargo tauri dev

# Produce an optimized release build
cargo tauri build
```

Before sending a change, the project expects a clean `cargo fmt`, no `cargo clippy` warnings, and a green `cargo test`.

---

## 🚫 What Bare will deliberately never build

A roadmap is also a list of refusals. These are not missing features — they are guarantees:

- **No plugin or extension system.** Non-extensibility is the central promise; a tool that cannot be extended cannot be quietly corrupted.
- **No JavaScript engine.** Ever. It is the largest source of tracking and attack surface on the web.
- **No telemetry, no cloud sync, no accounts.** Your reading is yours.
- **No author-controlled styling.** Presentation belongs to the reader.

What *is* genuinely on the table stays in character — improvements to *reading*, never to *running code*: PDF export, heading anchors and a table of contents, and richer reading-typography controls.

---

## 🔗 Useful links

- [Tauri documentation](https://tauri.app/v2/guides/)
- [Rust documentation](https://doc.rust-lang.org/)
- [pulldown-cmark](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/)
- [reqwest](https://docs.rs/reqwest/latest/reqwest/)

## 📚 Related reading

- [About Bare](./about.md) — what the browser is and who it's for
- [Philosophy](./philosophy.md) — why these constraints exist
- [History](./history.md) — the lineage Bare belongs to

---

[Back to Home](../index.md)
