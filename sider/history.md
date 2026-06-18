# History

> "Those who cannot remember the past are condemned to repeat it." – George Santayana

The history of Bare is not just the history of a browser, but also the history of **the internet itself** – how it started as an open, text-based platform, became commercialized and complex, and how a new movement is now trying to revive the original spirit.

---

## 🌱 Roots: The Original Internet

### ARPANET and the Early Days (1960-1980)

The internet as we know it today began as **ARPANET** in 1969, a project funded by the U.S. Department of Defense. The goal was to create a decentralized communication network that could survive a nuclear attack.

**Key characteristics of the early internet:**
- **Text-based**: Everything was text – email, file transfers, discussions
- **Open**: Protocols were simple and publicly documented
- **Decentralized**: No central control
- **Academic**: Primarily used by researchers and students

### Invention of the World Wide Web (1989-1991)

In 1989, while working at **CERN** in Switzerland, [Tim Berners-Lee](https://en.wikipedia.org/wiki/Tim_Berners-Lee) proposed a system for sharing information between researchers. He invented:

- **HTML** (HyperText Markup Language) – For structuring documents
- **HTTP** (HyperText Transfer Protocol) – For transferring documents
- **URL** (Uniform Resource Locator) – For identifying resources
- **The first browser** – "WorldWideWeb.app" for NeXT computers

**The first website** ([info.cern.ch](http://info.cern.ch)) went online in **1991** and explained what the World Wide Web was.

### The First Browsers

The earliest browsers were simple, text-based tools:

- **ViolaWWW** (1991) – Written by Pei-Yuan Wei
- **Lynx** (1992) – The first popular text-based browser
- **Mosaic** (1993) – The first graphical browser

These early browsers were:
- **Simple**: No advanced features
- **Fast**: Loading pages in fractions of a second
- **Secure**: No way to run malicious code

---

## 📜 The Markdown Revolution (2004)

### Invention of Markdown

In **March 2004**, [John Gruber](https://daringfireball.net/) (creator of the Daring Fireball blog) launched **Markdown** – a simple, readable format for writing structured text.

**Co-creator: Aaron Swartz**

The young programmer and activist [Aaron Swartz](https://en.wikipedia.org/wiki/Aaron_Swartz) (1986-2013) played a crucial role in the development of Markdown. He:
- Contributed to syntax design
- Wrote the first Markdown to HTML converter
- Was an important "sounding board" for Gruber

Swartz had previously invented the **atx** markup language, which influenced Markdown's heading syntax.

### Markdown Philosophy

Gruber and Swartz designed Markdown with the following principles:

1. **Readable as plain text**: Markdown files should look good even without formatting
2. **Easy to write**: The syntax should be intuitive
3. **Machine-readable**: Easy to parse and convert to other formats
4. **Compatible**: Should work with existing tools

### Markdown's Influence

Markdown quickly became popular among:
- **Blogger** – Easier than HTML for writing posts
- **Developers** – Used in README files (GitHub adopted it in 2009)
- **Writers** – Simple format for documentation

Today, Markdown is used everywhere:
- GitHub, GitLab, Bitbucket
- Reddit, Stack Overflow
- Obsidian, Notion
- Over 1 billion Markdown files on GitHub

---

## 🌐 Text-Based Protocols

### The Gopher Protocol (1991)

While Tim Berners-Lee was inventing the World Wide Web, [Mark P. McCahill](https://en.wikipedia.org/wiki/Mark_P._McCahill) and his team at **University of Minnesota** created the **Gopher protocol**.

**What was Gopher?**

- A **menu-based** system for navigating and retrieving documents
- Designed for **text-based terminals**
- Supported **hierarchical structures** (menus and submenus)
- Was **simpler** than the Web in many ways

**Gopher's popularity:**
- In **1992**, there were **over 1000 Gopher servers** in operation
- Gopher was **more popular than the Web** in the early years
- University of Minnesota charged a **license fee** for commercial use (1993)

**Gopher's decline:**
- Web supported **hyperlinks** (Gopher only had menus)
- Web supported **images** (Gopher was text-based)
- Web was **open** (Gopher had license restrictions)

**Gopher today:**
- **Gopherspace**: The collective network of Gopher servers
- **~100-200 active servers** (2024)
- **Enthusiast community** keeping the protocol alive
- **Inspiration** for new, minimalist protocols

### The Gemini Protocol (2019)

In **June 2019**, a developer under the pseudonym [Solderpunk](https://geminiprotocol.net/) launched the **Gemini protocol**.

**Background:**

Solderpunk was part of the **Gopherspace community** and had become frustrated with:
- The complexity of the modern Web
- Privacy issues
- Lack of simple, text-based communication

**Gemini's design goals:**

1. Simpler than Web, but more powerful than Gopher
2. **Mandatory TLS encryption** (no insecure connections)
3. **Text-based** (but with support for binary files)
4. **Client-driven** (the client decides how content is displayed)

**Gemtext format:**
```gemini
# Heading
## Subheading

This is a paragraph.

=> https://example.com Link description
```

**Gemini today (2026):**
- **~3,900 known servers** ("capsules")
- **~600,000+ URIs** in "Geminispace"
- **Growing community** of developers and users
- **Many clients** (browsers) available

---

## 🖥️ Text-Based Browsers

The first browsers were text-based, and they continue to be important tools:

### Lynx (1992 – Present)

**Origin:**
- Developed at **University of Kansas** in 1992
- Name comes from the **lynx** (animal), known for its sharp vision
- Created as a **Gopher client**, but was adapted for the Web

**Features:**
- Full HTML support (displays text only)
- SSL/TLS support
- Bookmarks and history
- Form support (limited)

**Use cases:**
- **Accessibility**: Popular among visually impaired (works with screen readers)
- **Privacy**: No images, no JavaScript = no tracking
- **Server administration**: Used to check websites from the command line

**Status:** Still actively maintained (2024)

### Links (1999 – Present)

**Origin:**
- Written by **Mikulas Patocka** in 1999
- Created as a **text-based** browser

**Features:**
- **Graphical mode** (Links2) with image support
- CSS support (limited)
- JavaScript support (experimental)
- Tab support

**Status:** Still actively developed

### w3m (1995 – Present)

**Origin:**
- Developed in **Japan** in 1995
- Name stands for "WWW text-based browser"

**Features:**
- Support for **colors** in the terminal
- Support for **SSL**
- **Inline images** (on terminals that support it)
- **Tab support**
- **Bookmarks and history**

**Special features:**
- **Emacs integration**: Can be used inside Emacs
- **URL rewriting**: Can redirect links
- **Tor support**: Can be used with torsocks

**Status:** Still maintained

---

## 🔄 Web Evolution: From Simplicity to Complexity

The internet has undergone a dramatic transformation since the early days:

### 1990s: The Golden Decade

**Early 90s (1991-1995):**
- **Simple websites**: HTML files with minimal formatting
- **Static pages**: No databases, no server-side scripting
- **Personal publishing**: Anyone could create a website

**Mid 90s (1995-2000):**
- **JavaScript** (1995): Adds interactivity
- **CSS** (1996): Separates content from presentation
- **CGI** (1993): Server-side scripting
- **Databases**: Dynamic websites

### 2000s: Commercialization

**Early 2000s (2000-2005):**
- **.com bubble** (2000): Explosive growth in commercial websites
- **Google AdSense** (2003): Ads become a major industry
- **Social media**: MySpace (2003), Facebook (2004)
- **Web 2.0**: User-generated content

**Mid 2000s (2005-2010):**
- **AJAX** (2005): Asynchronous page updates
- **YouTube** (2005): Video becomes dominant
- **iPhone** (2007): Mobile browsers
- **Cloud computing**: Data moves to the cloud

### 2010s: Surveillance and Complexity

**Early 2010s (2010-2015):**
- **Smartphones**: Mobile browsers dominate
- **Social media**: Facebook, Twitter, Instagram
- **Tracking**: Cookies, fingerprints, tracking technologies
- **SPA** (Single Page Applications): JavaScript-rendered pages

**Mid 2010s (2015-2020):**
- **React, Angular, Vue**: JavaScript frameworks dominate
- **AMP**: Google's attempt to make the web faster
- **GDPR** (2018): Privacy regulation in the EU
- **Dark Patterns**: Design as manipulation

### 2020s: The Reaction

**Early 2020s (2020-2024):**
- **JAMstack**: Static websites with JavaScript
- **Serverless**: Backend as a service
- **WebAssembly**: Native performance in the browser
- **Privacy focus**: GDPR, CCPA, cookie banners

**Minimalist movements:**
- **IndieWeb**: Own your own content
- **Static Site Generators**: Hugo, Jekyll, Eleventy
- **Markdown-first**: Write in Markdown, publish anywhere
- **Alternative protocols**: Gemini, Gopher, IPFS

---

## 💡 Bare's Place in History

Bare represents a **return to the roots** while embracing **modern technology**:

| Aspect | Early Web (1990) | Modern Web (2020) | Bare (2025) |
|--------|-------------------|---------------------|-------------|
| **Format** | HTML | HTML+CSS+JS | Markdown |
| **Content** | Text | Multimedia | Text |
| **Size** | KB | MB | KB |
| **Complexity** | Low | High | Low |
| **Privacy** | Good | Poor | Excellent |
| **Performance** | Fast | Slow | Lightning fast |
| **Technology** | Simple | Complex | Modern, simple |

### Why Markdown?

1. **Historical continuity**: Markdown is a natural continuation of the text-based internet
2. **Simplicity**: As simple as HTML was in 1990
3. **Readability**: Can be read as plain text
4. **Convertibility**: Can be converted to HTML, PDF, etc.
5. **Popularity**: Already widespread among developers and writers

### Why Tauri?

1. **Lightweight**: Like the early browsers
2. **Secure**: Rust provides memory safety
3. **Modern**: Uses modern WebView technology
4. **Cross-platform**: Works on all major operating systems

### Why Text-Based Protocols?

1. **Historical heritage**: Gopher and Gemini represent the text-based tradition
2. **Privacy**: Mandatory encryption (Gemini)
3. **Simplicity**: No images, no JavaScript
4. **Curiosity**: Exploration of alternative web experiences

---

## 🎯 Bare as Part of the Minimalist Movement

Bare is not alone. It's part of a **growing movement** towards a simpler, more privacy-focused internet:

### Similar Projects

| Project | Description | Status |
|----------|--------------|--------|
| **[Lagrange](https://gmi.gplv2.be/lagrange/)** | Graphical Gemini client | Active |
| **[Amfora](https://github.com/makeworld-the-better-one/amfora)** | Terminal-based Gemini client | Active |
| **[Bombadillo](https://bombadillo.colorfield.space/)** | Text-based browser for Gemini and Gopher | Active |
| **[Gopherus](https://gopherus.com/)** | Modern Gopher client | Active |
| **[Elinks](http://elinks.or.cz/)** | Advanced text-based browser | Active |

### Minimalist Websites

- **[Motherfucking Website](https://motherfuckingwebsite.com/)** – A manifesto for simple websites
- **[The Minimalist Web](https://minimalistweb.dev/)** – Resources for minimalist web development

### Open Source Movement

Bare is part of the **open source movement**, which has its roots back to:
- **Free Software Foundation** (1985)
- **GNU Project** (1983)
- **Open Source Initiative** (1998)

---

## 🔮 The Future

### What's in Store for Bare?

**Short-term (0-2 years):**
- Stable, user-friendly browser
- Full support for Markdown, Gemini, Gopher
- Growing user community

**Medium-term (2-5 years):**
- A full ecosystem for text-based browsing
- Integration with other open source projects
- Inspiration for new, minimalist browsers

**Long-term (5+ years):**
- To contribute to a **renaissance of text-based internet**
- To inspire a new generation of developers
- To preserve the **original spirit** of the internet

### Challenges

1. **Adoption**: Convincing people of the benefits of text-based browsing
2. **Content**: Ensuring there's enough Markdown content available
3. **Ecosystem**: Building an ecosystem of tools and services
4. **Sustainability**: Ensuring long-term maintenance

### Opportunities

1. **Privacy awareness**: Growing focus on privacy
2. **Minimalism trend**: People seeking simplicity
3. **Text-based content**: Markdown is more popular than ever
4. **Alternative protocols**: Growing interest in Gemini and Gopher

---

## 📚 Related Reading

- [About Bare](./about.md) – What the browser is
- [Philosophy](./philosophy.md) – Why Bare was created
- [Technology](./technology.md) – How Bare is built

---

## 🔗 Historical Resources

### Gopher
- [Gopher Protocol Wikipedia](https://en.wikipedia.org/wiki/Gopher_(protocol))
- [History of Gopher](https://www.ils.unc.edu/callee/gopherpaper.htm)
- [Gopherus](https://gopherus.com/) – Modern Gopher client

### Gemini
- [Gemini Protocol Official Site](https://geminiprotocol.net/)
- [History of Project Gemini](https://geminiprotocol.net/history/)
- `gemi://geminiprotocol.net` – Explore Geminispace

### Markdown
- [Markdown Wikipedia](https://en.wikipedia.org/wiki/Markdown)
- [Daring Fireball: Markdown](https://daringfireball.net/projects/markdown/)
- [CommonMark](https://commonmark.org/) – Markdown specification

### Text-Based Browsers
- [Lynx Official Site](https://lynx.invisible-island.net/)
- [Links Browser](http://links.twibright.com/)
- [w3m Browser](https://w3m.sourceforge.net/)

---

*"The future belongs to those who remember the past."*

---

[Back to Home](../index.md)
