# Utviklingsplan for Bare

> En åpen kildekode-nettleser som kun leser Markdown

## Innholdsfortegnelse

1. [Prosjektoversikt](#prosjektoversikt)
2. [Teknisk stack](#teknisk-stack)
3. [Arkitektur](#arkitektur)
4. [Utviklingsfaser](#utviklingsfaser)
5. [Server-komponent](#server-komponent)
6. [Utfordringer og løsninger](#utfordringer-og-løsninger)
7. [Inspirasjon fra lignende prosjekter](#inspirasjon-fra-lignende-prosjekter)
8. [Åpne spørsmål](#åpne-spørsmål)

---

## Prosjektoversikt

**Bare** er en eksperimentell nettleser designet for et tekst-basert internett. Den ignorerer tradisjonelle nettsider og rendrer kun `.md`-filer direkte fra HTTP-responser.

### Kjerneverdier

| Verdi | Beskrivelse |
|-------|-------------|
| **Lynrask** | Ingen tunge rammeverk å laste ned |
| **Personvern** | Ingen støtte for scripts = null sporing |
| **Fokus** | Innholdet presenteres konsistent uavhengig av kilden |

### Sammenligning med tradisjonelle nettlesere

| Funksjon | Tradisjonell nettleser | Bare |
|----------|------------------------|------|
| JavaScript | ✅ Full støtte | ❌ Ingen |
| Cookies | ✅ Full støtte | ❌ Ingen |
| CSS | ✅ Full støtte | ❌ Minimal/ingen |
| Tracking | ⚠️ Mulig | ❌ Umulig |
| Fingerprinting | ⚠️ Stor angrepsflate | ❌ Minimal |
| Typisk sidestørrelse | 2-5 MB | 5-50 KB |

---

## Teknisk stack

### Anbefalt stack

| Komponent | Teknologi | Begrunnelse |
|-----------|-----------|-------------|
| **App-rammeverk** | [Tauri 2.0](https://tauri.app/) | Minimal størrelse (~2-5 MB), Rust, sikkerhetsfokus |
| **Frontend** | Vanilla HTML/CSS | Enkel markdown-visning, ingen avhengigheter |
| **Markdown parser** | [pulldown-cmark](https://crates.io/crates/pulldown-cmark) | Rask, CommonMark-kompatibel, brukes av `cargo doc` |
| **HTTP-klient** | [reqwest](https://crates.io/crates/reqwest) | Async Rust HTTP med TLS-støtte |
| **HTML→MD** | [html2md](https://crates.io/crates/html2md) | Konvertering for vanlige nettsider |

### Hvorfor Tauri over Electron?

| Aspekt | Tauri | Electron |
|--------|-------|----------|
| Appstørrelse | ~2-5 MB | ~100-150 MB |
| Minnebruk | Lavt (OS WebView) | Høyt (Chromium) |
| Språk | Rust (sikkert) | JavaScript |
| Oppstartstid | Rask | Treg |
| Sikkerhet | Front-of-mind | Må konfigureres |

### Alternative teknologier vurdert

- **Electron**: Overkill for ren markdown-rendering
- **egui (Rust)**: Full kontroll, men krever mer arbeid for tekstlayout
- **Qt/GTK**: Større binærfiler, mer kompleks oppsett

---

## Arkitektur

### Høynivå-arkitektur

```
┌─────────────────────────────────────────────────────────────┐
│                        Bare Browser                          │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   URL Bar   │  │  Nav Stack  │  │     Bookmarks       │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                     Rendering Engine                         │
│  ┌─────────────────────────────────────────────────────────┐│
│  │                  Markdown Viewport                       ││
│  │  ┌─────────────────────────────────────────────────────┐││
│  │  │  # Heading                                          │││
│  │  │  Paragraph text with **bold** and *italic*          │││
│  │  │  - List item 1                                      │││
│  │  │  - List item 2                                      │││
│  │  └─────────────────────────────────────────────────────┘││
│  └─────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────┤
│                       Core Engine (Rust)                     │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐ │
│  │  Fetcher  │  │  Parser   │  │ Converter │  │   Cache   │ │
│  │ (reqwest) │  │(pulldown) │  │ (html2md) │  │  (sled?)  │ │
│  └───────────┘  └───────────┘  └───────────┘  └───────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Dataflyt

```
1. Bruker skriver URL
        ↓
2. Fetcher henter ressurs via HTTP/HTTPS
        ↓
3. Sjekk Content-Type
        ↓
   ┌────┴────┐
   ↓         ↓
.md fil   HTML/annet
   ↓         ↓
   │    Converter (html2md)
   │         ↓
   └────→ Markdown
              ↓
4. Parser (pulldown-cmark)
              ↓
5. Render til HTML for WebView
              ↓
6. Vis i viewport med Bare-styling
```

### Filstruktur

```
bare/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs           # Applikasjons-entrypoint
│   │   ├── commands.rs       # Tauri commands (IPC)
│   │   ├── markdown.rs       # pulldown-cmark rendering
│   │   ├── fetcher.rs        # HTTP requests
│   │   ├── converter.rs      # HTML → MD konvertering
│   │   ├── history.rs        # Navigasjonshistorikk
│   │   └── bookmarks.rs      # Bokmerker (JSON-lagring)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── index.html            # Minimal UI shell
│   ├── styles.css            # Markdown styling (lys/mørk)
│   └── app.js                # Minimal frontend-logikk
├── README.md
├── PLAN.md
└── LICENSE
```

---

## Utviklingsfaser

### Fase 1: Proof of Concept (2-3 uker) ✅ FULLFØRT

**Mål:** Vis at konseptet fungerer

- [x] Sett opp Tauri 2.0-prosjekt med Rust backend
- [x] Implementer lokal `.md`-fil visning
- [x] Grunnleggende URL-bar input
- [x] Enkel markdown-rendering med pulldown-cmark
- [x] Minimal CSS for lesbar visning
- [x] GitHub Actions for CI/CD (bygg og tester)

**Deliverables:**
- Kjørbar app som kan åpne lokale markdown-filer
- Grunnleggende brukergrensesnitt
- Automatisert bygg og test-pipeline

### Fase 2: Nettverksstøtte (2-3 uker) ✅ FULLFØRT

**Mål:** Hent markdown fra internett

- [x] HTTP/HTTPS-klient med reqwest
- [x] Håndter Content-Type headers
- [x] Parse og gjør lenker klikkbare
- [x] Relative URL-oppløsning
- [x] Back/forward navigasjonshistorikk
- [x] Feilhåndtering (404, timeout, etc.)

**Deliverables:**
- App kan navigere til markdown-URLer
- Lenker i dokumenter fungerer

### Fase 3: HTML-konvertering (2-3 uker)

**Mål:** Gjør Bare praktisk for vanlige nettsider

- [ ] Integrer html2md for HTML→Markdown konvertering
- [ ] Readability-modus for å ekstrahere hovedinnhold
- [ ] Fallback når `.md`-fil ikke finnes
- [ ] Håndter ulike encodings (UTF-8, etc.)
- [ ] Konfigurerbar: "Kun .md" vs "Konverter alt"

**Deliverables:**
- App kan vise innhold fra vanlige nettsider
- Bruker kan velge modus

### Fase 4: Brukeropplevelse (2-3 uker)

**Mål:** Gjør Bare behagelig å bruke daglig

- [ ] Themes (lys/mørk modus)
- [ ] Bokmerker med JSON-lagring
- [ ] Søk i side (Ctrl+F)
- [ ] Keyboard shortcuts (Vim-inspirert?)
- [ ] Zoom inn/ut
- [ ] Skrift-valg

**Deliverables:**
- Polert brukeropplevelse
- Personlige preferanser lagres

### Fase 5: Avanserte funksjoner (valgfritt)

**Mål:** Utvid funksjonalitet

- [ ] Gemini-protokoll støtte (gemini://)
- [ ] Gopher-protokoll støtte (gopher://)
- [ ] Eksporter sider som PDF
- [ ] Tab-støtte
- [ ] Synkronisering av bokmerker
- [ ] Utvidelser/plugins

---

## Server-komponent

### Trenger Bare en server?

**Kort svar:** Ikke nødvendigvis, men det kan være nyttig.

### Alternativ 1: Integrert konvertering (anbefalt for MVP)

All logikk kjører i Tauri-appen:

```
Bruker → Bare app → Internett
                ↓
         Mottar HTML
                ↓
         html2md konverterer
                ↓
         Viser markdown
```

**Fordeler:**
- Ingen ekstra komponenter
- Fungerer offline (for cached sider)
- Enkelt oppsett

**Ulemper:**
- All prosessering i klienten
- Kan være treg for tunge sider

### Alternativ 2: Lokal proxy-server

En separat prosess som kjører lokalt:

```
┌─────────────────┐     ┌──────────────────┐     ┌──────────────┐
│   Bare Browser  │────▶│  Bare Proxy      │────▶│  Internett   │
│   (localhost)   │     │  (localhost:8080)│     │              │
└─────────────────┘     └──────────────────┘     └──────────────┘
                               │
                               ▼
                        1. Hent HTML
                        2. Readability extract
                        3. HTML → Markdown
                        4. Cache resultat
                        5. Returner .md
```

**Fordeler:**
- Kan cache konverterte sider
- Avlaster nettleseren
- Kan deles mellom flere klienter

**Ulemper:**
- Ekstra prosess å kjøre
- Mer kompleks installasjon

### Alternativ 3: Skybasert proxy (fremtidig)

En sentral tjeneste som konverterer:

```
Bruker → Bare app → bare.io/proxy?url=... → Internett
```

**Fordeler:**
- Ingen lokal prosessering
- Kan optimaliseres sentralt
- Caching på tvers av brukere

**Ulemper:**
- Personvernsbekymringer (all trafikk via tredjepart)
- Driftskostnader
- Avhengighet av ekstern tjeneste

### Anbefaling

**Start med Alternativ 1** (integrert). Hvis ytelse blir et problem, migrer til **Alternativ 2** (lokal proxy) i en senere fase.

---

## Utfordringer og løsninger

### Tekniske utfordringer

| Utfordring | Løsning |
|------------|---------|
| **Relative lenker** | Parse base URL og resolve relative paths |
| **Bilder** | Standard: Vis inline. Fremtidig: Brukervalg per side eller globalt |
| **Tabeller** | pulldown-cmark støtter GFM tables |
| **Kode-blokker** | Syntax highlighting med syntect (Rust) |
| **Encoding** | Detekter og konverter til UTF-8 |
| **Redirects** | Følg HTTP 301/302/307 |
| **HTTPS-sertifikater** | Bruk native TLS (rustls eller native-tls) |

### Personvern-vurderinger

| Aspekt | Tilnærming |
|--------|------------|
| **Ingen cookies** | Ikke implementer cookie-jar |
| **Ingen JS** | WebView med JS disabled |
| **Ingen tracking-piksler** | Bilder er opt-in |
| **Minimal fingerprinting** | Enkel User-Agent, ingen canvas/WebGL |
| **DNS over HTTPS** | Vurder for fremtidig versjon |

### UX-utfordringer

| Utfordring | Løsning |
|------------|---------|
| **Sider uten .md** | Tydelig melding + tilbud om konvertering |
| **Dårlig konvertering** | "Vis original" fallback-knapp |
| **Manglende bilder** | Placeholder med alt-tekst og klikk-for-å-laste |
| **Komplekse layouts** | Aksepter at noe innhold ikke egner seg |

---

## Inspirasjon fra lignende prosjekter

### Gemini-protokollen

**Hva det er:** En applikasjonslag-protokoll (startet 2019) for distribusjon av dokumenter.

**Relevante konsepter:**
- Eget lett format ("gemtext")
- ~3900 aktive "capsules" (nettsteder) per 2024
- Klienter: Lagrange, Kristall, Amfora

**Hva Bare kan lære:**
- Enkelhet som designprinsipp fungerer
- Dedikert community kan vokse rundt minimalistiske verktøy
- Vurder gemini://-støtte i fremtiden

### Lynx

**Hva det er:** Den eldste nettleseren som fortsatt vedlikeholdes (siden 1992).

**Relevante konsepter:**
- Ren tekstgjengivelse
- Ingen JavaScript, bilder ignoreres
- Brukes for tilgjengelighet

**Hva Bare kan lære:**
- Tekstfokus gir naturlig personvern
- Lav båndbredde = rask lasting
- Viktig for tilgjengelighet

### Gopher

**Hva det er:** Kommunikasjonsprotokoll fra 1991 for dokumentdistribusjon.

**Relevante konsepter:**
- Menydrevet, hierarkisk navigasjon
- Ekstremt enkel protokoll
- ~325 aktive servere fortsatt

**Hva Bare kan lære:**
- Filsystem-metafor er intuitiv
- Protokollens enkelhet gjør implementasjon trivielt

---

## Åpne spørsmål

Disse spørsmålene bør avklares før/under utvikling:

### 1. Bildehåndtering ✅ BESLUTTET
> Skal bilder vises inline, som lenker, eller kun som alt-tekst?

**Beslutning:** 
- **Fase 1:** Vis bilder inline som standard (enklest implementasjon)
- **Fremtidig:** Brukervalg per side eller globalt (planlegges i arkitekturen)

**Implementasjonsplan for brukervalg:**
- Global innstilling i preferences: `images: "show" | "hide" | "placeholder"`
- Per-side override via toolbar-knapp
- Lagres i lokal konfigurasjon (JSON)

### 2. Konverteringsmodus
> Skal Bare automatisk konvertere HTML, eller kun vise native .md?

**Alternativer:**
- A) Kun .md-filer (puristisk tilnærming)
- B) Automatisk konvertering (praktisk)
- C) Spør brukeren hver gang
- D) Konfigurerbar innstilling

### 3. Protokoll-støtte
> Skal Bare utvides til Gemini og/eller Gopher?

**Vurdering:**
- Gemini deler filosofien om enkelhet
- Eksisterende community med ~3900 capsules
- Gopher er historisk interessant men mindre aktivt

### 4. Målplattformer
> Hvilke plattformer skal støttes først?

**Alternativer:**
- A) Desktop først (Windows, macOS, Linux)
- B) Inkluder mobil (iOS, Android) fra start
- C) Web-versjon (WASM)?

### 5. Lisensiering
> Hvilken åpen kildekode-lisens?

**Alternativer:**
- MIT (permissiv)
- Apache 2.0 (permissiv med patentbeskyttelse)
- GPL v3 (copyleft)
- AGPL v3 (sterk copyleft)

---

## CI/CD

### GitHub Actions

Prosjektet bruker GitHub Actions for automatisert bygg og testing:

| Workflow | Trigger | Beskrivelse |
|----------|---------|-------------|
| `ci.yml` | Push/PR til main | Kjører tester og linting |
| `build.yml` | Release tags | Bygger binærfiler for alle plattformer |

**CI Pipeline inkluderer:**
- `cargo fmt --check` - Kodeformatering
- `cargo clippy` - Linting
- `cargo test` - Unit tests
- `cargo tauri build` - Bygg-verifisering

**Støttede plattformer:**
- Windows (x64)
- macOS (x64, ARM64)
- Linux (x64)

---

## Ressurser

### Dokumentasjon
- [Tauri 2.0 Docs](https://tauri.app/v2/guides/)
- [pulldown-cmark](https://docs.rs/pulldown-cmark/)
- [reqwest](https://docs.rs/reqwest/)
- [html2md](https://docs.rs/html2md/)

### Inspirasjon
- [Gemini Protocol](https://geminiprotocol.net/)
- [Project Gemini FAQ](https://gemini.circumlunar.space/docs/faq.html)
- [Lynx Browser](https://lynx.invisible-island.net/)

### Lignende prosjekter
- [Lagrange](https://github.com/nickshanks/lagrange) - Gemini-klient
- [Amfora](https://github.com/makeworld-the-better-one/amfora) - Terminal Gemini-klient

---

*Sist oppdatert: Januar 2026*
