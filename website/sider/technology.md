# Technology

> "Technology is best when it disappears." – Adapted from Alan Kay

Bare is built with modern, lightweight technology to deliver a fast, secure, and efficient browsing experience. Unlike heavy, resource-intensive browsers like Chrome or Electron-based applications, Bare is designed to be **minimal, secure, and performance-optimized**.

---

## 🏗️ Architecture

Bare uses a **client-server architecture** where:
- **Frontend** (User Interface): Vanilla HTML/CSS/JavaScript
- **Backend** (Logic and fetching): Rust
- **Bridge**: Tauri for communication between frontend and backend

This separation provides:
- **Security**: Rust backend handles all external communication
- **Performance**: Native Rust code for network operations
- **Flexibility**: Web technologies for user interface

---

## 🛠️ Main Components

### Tauri 2.0

**[Tauri](https://tauri.app/)** is the foundational framework that makes Bare possible.

**Why Tauri?**

| Aspect | Tauri | Electron | Tauri Advantage |
|--------|-------|----------|------------------|
| App size | ~2-5 MB | ~100-200 MB | 20-100x smaller |
| Memory usage | Low | High | 5x less RAM |
| Security | High (Rust) | Medium (JS) | Memory safety |
| Performance | High | Medium | Native speed |
| Platform support | Windows, macOS, Linux | Windows, macOS, Linux | Same, but lighter |

**Key features of Tauri used by Bare:**

- **System WebView**: Uses the OS's native WebView (WebKit on macOS/Linux, WebView2 on Windows)
- **Rust Backend**: Secure, fast system access
- **Minimal bundling**: No Chromium content
- **Platform-specific builds**: Optimal performance on each platform

### Rust

**[Rust](https://www.rust-lang.org/)** is the programming language powering Bare's backend.

**Why Rust?**

1. **Memory safety**: Rust's ownership rules prevent memory errors like buffer overflows
2. **Performance**: Compiled to native code, as fast as C/C++
3. **Security**: No garbage collection, no runtime overhead
4. **Concurrency**: Excellent support for asynchronous programming
5. **Ecosystem**: Rich package ecosystem (crates.io)

**Rust libraries used in Bare:**

- **[pulldown-cmark](https://crates.io/crates/pulldown-cmark)**: Fast CommonMark + GFM Markdown parser
  - Fast: Written in Rust, optimized for performance
  - Accurate: Full support for CommonMark specification
  - Flexible: Supports extensions like tables, footnotes, etc.

- **[reqwest](https://crates.io/crates/reqwest)**: Async HTTP client
  - Supports HTTP/1.1 and HTTP/2
  - Integrated TLS support
  - Async/await-based API

- **[tauri](https://crates.io/crates/tauri)**: Tauri Rust library
  - Communication with frontend
  - System API access
  - Window handling

### Vanilla Web Technologies

Bare's frontend is built with **pure web technologies**:

- **HTML5**: Semantic markup for structure
- **CSS3**: Minimal styling, focused on readability
- **JavaScript (ES6+)**: Clean, efficient code without frameworks

**Advantages of this approach:**

1. **No dependencies**: No npm packages, no build steps
2. **Fast loading**: No frameworks to load
3. **Easy maintenance**: No version conflicts
4. **Long lifespan**: Standards that don't change radically

---

## 🌐 Protocol Support

### HTTP/HTTPS

Standard web protocols with full support for:
- GET and HEAD requests
- Redirect handling
- SSL/TLS encryption
- Content-Type negotiation

**Special for Markdown:**
Bare sends an `Accept` header that signals preference for Markdown:
```
Accept: text/markdown, text/plain;q=0.9, text/html;q=0.5
```

This allows servers that support content negotiation to deliver cleaner content directly.

### Gemini Protocol

**[Gemini](https://geminiprotocol.net/)** is a modern, text-based protocol with mandatory TLS encryption.

**Bare's Gemini support includes:**

- Full protocol implementation (RFC)
- TOFU (Trust On First Use) certificate handling
- Gemtext to Markdown conversion
- Interactive pages (input dialog)

**Gemtext format:**
```gemini
# Heading 1
## Heading 2

This is a paragraph.

=> https://example.com Link description
```

### Gopher Protocol

**[Gopher](https://en.wikipedia.org/wiki/Gopher_(protocol))** is the classic protocol from 1991.

**Bare's Gopher support includes:**

- Full RFC 1436 implementation
- Gophermap to Markdown conversion
- Support for text files, menus, and search
- Emoji icons for different content types
- Search dialog for interactive Gopher queries

---

## 🔧 Features

### Markdown Rendering

Bare's Markdown rendering engine handles:

- **Basic formatting**: Bold, italic, headings, lists
- **Links**: Inline and reference-style links
- **Images**: Optional display (can be disabled for privacy)
- **Code blocks**: Syntax highlighting (optional)
- **Tables**: Full support
- **Footnotes**: Supported
- **HTML**: HTML in Markdown is escaped (security)

### HTML to Markdown Conversion

For HTML pages, Bare offers **Readability mode**:

- Extracts main content from article pages
- Removes ads, navigation, sidebars
- Converts to clean Markdown
- Preserves structure and formatting

**Example:**
A busy news site with ads and complex layout is converted to:
```markdown
# Article Title

This is the main content of the article...
```

### Navigation

- **History**: Full back/forward navigation
- **Bookmarks**: Save and organize favorite pages
- **URL autocomplete**: Smart address bar with suggestions
- **Protocol detection**: Automatic recognition of gemini://, gopher://, etc.

### User Experience

- **Theme**: Light, dark, sepia, high-contrast
- **Font family**: System, serif, sans-serif, monospace
- **Font size**: Adjustable (70%-150%)
- **Content width**: Adjustable (400-1200px)
- **Zoom**: Ctrl+/Ctrl- for zoom in/out

### Search

- **In-page search**: Ctrl+F to search current page
- **Regular expressions**: Advanced search with regex
- **Match highlighting**: Visual highlighting of matches

### Keyboard Shortcuts

Bare has comprehensive keyboard support:

| Shortcut | Action |
|----------|--------|
| Ctrl+K | Open command palette |
| Ctrl+F | Search in page |
| Ctrl+D | Bookmark page |
| Ctrl+B | Show bookmarks |
| Ctrl+O | Open local file |
| Ctrl+Plus | Zoom in |
| Ctrl+Minus | Zoom out |
| Alt+← | Back |
| Alt+→ | Forward |
| Alt+Home | Home |
| F5 | Reload |

---

## 📊 Performance Optimizations

### Memory Usage

- **Small footprint**: ~10-20 MB RAM for entire application
- **Efficient caching**: Cache of recently visited pages
- **Minimal state**: Only necessary data stored

### Load Time

- **Instant startup**: <1 second
- **Fast page switching**: Markdown loads immediately
- **Async loading**: No blocking operations

### Network

- **HTTP/2 support**: Multiplexed requests
- **Keep-alive**: Reuses connections
- **Compression**: Supports gzip, deflate, brotli

---

## 🔒 Security

### Architectural Security

1. **No JavaScript**: No way to run malicious code
2. **No plugins**: No extension API that can be abused
3. **Sandboxed WebView**: WebView runs in a restricted context
4. **Rust Backend**: Memory safety prevents many vulnerabilities

### Network Security

1. **TLS verification**: All HTTPS and Gemini connections verified
2. **TOFU for Gemini**: Trust On First Use for Gemini certificates
3. **No mixed content**: Blocks insecure resources on secure pages
4. **CSP**: Content Security Policy protection against XSS

### Data Protection

1. **No telemetry**: No data sent to developers
2. **Local storage**: All data stored locally
3. **No cloud sync**: No external storage
4. **Encrypted storage**: Sensitive data can be encrypted

---

## 🛠️ Build Process

### Development Environment

```bash
# Clone repository
git clone https://github.com/FrankBurmo/bare.git
cd bare

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Tauri CLI
cargo install tauri-cli

# Start development server
cargo tauri dev
```

### Production Build

```bash
# Build for current platform
cargo tauri build

# Build for specific platform
cargo tauri build --target x86_64-pc-windows-msvc
cargo tauri build --target x86_64-unknown-linux-gnu
cargo tauri build --target universal2-apple-darwin
```

### Build for Distribution

```bash
# Build all platforms (requires cross-compilation setup)
cargo tauri build --all-targets

# Build with update check
cargo tauri build --features updater
```

---

## 📦 Dependencies

### Rust Crates (backend)

```toml
[dependencies]
# Tauri
tauri = "2.0"
tauri-utils = "2.0"

# HTTP
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }

# Markdown parsing
pulldown-cmark = { version = "0.10", features = ["html", "tick_token"] }

# URL handling
url = "2.5"
percent-encoding = "2.3"

# Async
tokio = { version = "1.0", features = ["full"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# File system
walkdir = "2.4"
```

---

## 🚀 Technology Roadmap

### Short-term (0-6 months)
- ✅ Basic Markdown rendering
- ✅ HTTP/HTTPS support
- ✅ Gemini protocol support
- ✅ Gopher protocol support
- ⬜ PDF export
- ⬜ Tabs support

### Medium-term (6-18 months)
- ⬜ Plugin system
- ⬜ Custom CSS themes
- ⬜ Cross-device synchronization
- ⬜ Mobile platforms (Android, iOS)

### Experimental (18+ months)
- ⬜ P2P protocol support (IPFS, DAT)
- ⬜ AI-based content filtering
- ⬜ Voice control

---

## 📚 Related Reading

- [About Bare](./about.md) – What the browser is
- [Philosophy](./philosophy.md) – Why Bare was created
- [History](./history.md) – The background of text-based protocols

---

## 🔗 Useful Links

- [Tauri Documentation](https://tauri.app/v2/guides/)
- [Rust Documentation](https://doc.rust-lang.org/)
- [pulldown-cmark](https://docs.rs/pulldown-cmark/latest/pulldown_cmark/)
- [reqwest](https://docs.rs/reqwest/latest/reqwest/)

---

[Back to Home](../index.md)
