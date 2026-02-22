# Gopher-protokoll i Bare

> ✅ **IMPLEMENTERT i v0.1.4** (Februar 2026) — Detaljert implementasjonsplan og dokumentasjon for gopher://-støtte i Bare-nettleseren

## Implementasjonsstatus

**Status:** Fullstendig implementert

| Komponent | Fil | Linjer | Tester | Status |
|-----------|-----|--------|--------|--------|
| Gopher-klient | `gopher.rs` | 809 | 19 | ✅ Fullført |
| Gophermap-konverterer | `gophermap.rs` | 324 | 11 | ✅ Fullført |
| Frontend-integrasjon | `navigation.js` | — | — | ✅ Fullført |
| Tauri-kommando | `commands.rs` | — | — | ✅ Fullført |

**Støttede funksjoner:**
- ✅ Gopher-menyer (gophermap) → Markdown konvertering
- ✅ Tekstfiler (type 0) → Markdown visning
- ✅ Alle standard elementtyper (0-9, i, h, s, +)
- ✅ Søk-dialog for interaktive søk (type 7)
- ✅ Relativ URL-oppløsning
- ✅ Timeout og størrelsesbegrensning (10s / 5 MB)
- ✅ Sømløs navigasjon mellom HTTP, Gemini og Gopher

---

## Innholdsfortegnelse

1. [Introduksjon](#introduksjon)
2. [Protokolloversikt](#protokolloversikt)
3. [Arkitektur og design](#arkitektur-og-design)
4. [Implementasjonsplan](#implementasjonsplan)
5. [Gopher-til-Markdown konvertering](#gopher-til-markdown-konvertering)
6. [Frontend-integrasjon](#frontend-integrasjon)
7. [Sikkerhet og personvern](#sikkerhet-og-personvern)
8. [Testing](#testing)
9. [Utfordringer og løsninger](#utfordringer-og-løsninger)
10. [Tanker om Gopher-klient i 2026](#tanker-om-gopher-klient-i-2026)

---

## Introduksjon

### Bakgrunn

Gopher er en kommunikasjonsprotokoll fra 1991 (RFC 1436) for distribusjon av dokumenter over internett. Protokollen ble utviklet ved University of Minnesota og var en av de dominerende måtene å navigere internett på før World Wide Web tok over.

Gopher passer perfekt inn i Bares filosofi:

| Egenskap | Gopher | Bare |
|----------|--------|------|
| **Enkelhet** | Minimalistisk protokoll | Minimalistisk nettleser |
| **Tekstfokus** | Primært tekstbasert innhold | Kun markdown-rendering |
| **Personvern** | Ingen cookies, ingen scripts | Null sporing |
| **Lav båndbredde** | Svært lette responser | Effektiv innholdslasting |

### Motivasjon

- Gopher deler Bares kjerneverdier om enkelhet og personvern
- Det finnes fortsatt et aktivt (om enn lite) Gopher-fellesskap med hundrevis av servere
- Implementasjonen er enklere enn Gemini (ingen TLS-krav i standard Gopher)
- Bare støtter allerede Gemini — Gopher kompletterer multiprotokoll-visjonen
- Gopher-innhold lar seg naturlig konvertere til markdown

### Mål

- Full støtte for Gopher-protokollen (RFC 1436) i Bare
- Sømløs konvertering av Gopher-menyer og tekstfiler til markdown
- Integrert navigasjon med eksisterende HTTP- og Gemini-støtte
- Valgfri Gopher-over-TLS (GoT) for sikker kommunikasjon

---

## Protokolloversikt

### Grunnleggende protokoll

Gopher er en enkel klient-server-protokoll over TCP:

```
1. Klient kobler til server på port 70 (standard)
2. Klient sender en selektor-streng + CRLF
3. Server sender respons
4. Server lukker tilkoblingen
```

### Forespørsel-format

```
<selektor>\r\n
```

- Tom selektor (`\r\n`) = rot-meny
- Selektor er en sti (f.eks. `/about` eller `/docs/readme.txt`)
- Søk: `<selektor>\t<søkestreng>\r\n`

### Respons-formater

Gopher har to hovedtyper av respons:

**1. Meny (directory listing):**
```
<type><visningsnavn>\t<selektor>\t<host>\t<port>\r\n
<type><visningsnavn>\t<selektor>\t<host>\t<port>\r\n
...
.\r\n
```

**2. Tekstfil:**
```
Rå tekst...
.\r\n
```

### Elementtyper (Item Types)

| Type | Beskrivelse | Relevans for Bare |
|------|-------------|-------------------|
| `0` | Tekstfil | ✅ Vis som markdown |
| `1` | Mappe/meny | ✅ Konverter til markdown-liste med lenker |
| `2` | CSO telefonbok | ⚠️ Vis info-melding |
| `3` | Feilmelding | ✅ Vis som feil |
| `4` | BinHex-fil (Mac) | ❌ Ignorer / vis lenke |
| `5` | DOS-binærfil | ❌ Ignorer / vis lenke |
| `6` | UUencodet fil | ❌ Ignorer / vis lenke |
| `7` | Søk | ✅ Vis søkefelt (som Gemini input) |
| `8` | Telnet-sesjon | ❌ Ikke støttet (sikkerhet) |
| `9` | Binærfil | ❌ Ignorer / vis lenke |
| `g` | GIF-bilde | ⚠️ Avhengig av bilde-innstillinger |
| `I` | Bilde (generelt) | ⚠️ Avhengig av bilde-innstillinger |
| `h` | HTML-fil | ✅ Konverter via html2md |
| `i` | Informasjonstekst | ✅ Vis som ren tekst |
| `T` | Telnet 3270 | ❌ Ikke støttet |

### URL-format

```
gopher://<host>[:<port>]/<type><selektor>
```

Eksempler:
- `gopher://gopher.floodgap.com/` — Rot-meny
- `gopher://gopher.floodgap.com/0/gopher/welcome` — Tekstfil
- `gopher://gopher.floodgap.com/1/world` — Undermeny
- `gopher://gopher.floodgap.com/7/v2/vs` — Søk

---

## Arkitektur og design

### Nye filer

```
src-tauri/src/
├── gopher.rs          # Gopher-klient (TCP-tilkobling, parsing)
└── gophermap.rs       # Gopher-meny til Markdown konvertering
```

### Dataflyt

```
1. Bruker skriver gopher://...
        ↓
2. URL-deteksjon i frontend (navigation.js)
        ↓
3. Tauri IPC → fetch_gopher command
        ↓
4. gopher.rs: TCP-tilkobling til server
        ↓
5. Send selektor + CRLF
        ↓
6. Motta respons
        ↓
   ┌────┴────┐
   ↓         ↓
 Meny    Tekstfil
   ↓         ↓
   │    Vis som markdown
   ↓
gophermap.rs: Konverter meny til markdown
        ↓
7. markdown.rs: Render til HTML
        ↓
8. Vis i viewport
```

### Integrasjon med eksisterende arkitektur

```
┌─────────────────────────────────────────────────────────────┐
│                        Bare Browser                          │
├─────────────────────────────────────────────────────────────┤
│                     Navigation Layer                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐     │
│  │  HTTP(S) │  │  Gemini  │  │  Gopher  │  │  file:// │     │
│  │ fetcher  │  │  client  │  │  client  │  │  loader  │     │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘     │
│       ↓             ↓             ↓             ↓            │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Markdown Conversion Layer                   │ │
│  │  html2md  │  gemtext.rs  │  gophermap.rs  │  passthrough│ │
│  └─────────────────────────────────────────────────────────┘ │
│       ↓             ↓             ↓             ↓            │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │          Unified Markdown Pipeline (pulldown-cmark)      │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Strukturer i Rust

```rust
/// Gopher-elementtype hentet fra meny-respons
#[derive(Debug, Clone, PartialEq)]
pub enum GopherItemType {
    TextFile,           // 0
    Directory,          // 1
    CsoPhonebook,       // 2
    Error,              // 3
    BinHex,             // 4
    DosBinary,          // 5
    UuEncoded,          // 6
    Search,             // 7
    Telnet,             // 8
    Binary,             // 9
    Gif,                // g
    Image,              // I
    Html,               // h
    Info,               // i
    Telnet3270,         // T
    Unknown(char),      // Ukjent type
}

/// Et enkelt element i en Gopher-meny
#[derive(Debug, Clone)]
pub struct GopherItem {
    pub item_type: GopherItemType,
    pub display: String,
    pub selector: String,
    pub host: String,
    pub port: u16,
}

/// Respons fra en Gopher-server
#[derive(Debug)]
pub struct GopherResponse {
    pub content_type: GopherContentType,
    pub body: String,
    pub items: Vec<GopherItem>,   // Kun for menyer
    pub final_url: String,
}

/// Type innhold i responsen
#[derive(Debug)]
pub enum GopherContentType {
    Menu,
    Text,
    Html,
    Binary,
    Search,
    Error,
}
```

---

## Implementasjonsplan

### Fase 1: Grunnleggende Gopher-klient (`gopher.rs`)

**Estimert tid:** 1-2 uker

#### Oppgaver

- [ ] Implementer `GopherClient` struct med TCP-tilkobling
- [ ] URL-parsing og validering for `gopher://`-URLer
- [ ] Send selektor og motta respons
- [ ] Parse meny-responser (tab-separerte felt)
- [ ] Håndter tekstfil-responser
- [ ] Håndter terminering (`.` på egen linje)
- [ ] Timeout-håndtering (10 sekunder, som Gemini)
- [ ] Maksimal respons-størrelse (5 MB)
- [ ] Feilhåndtering med custom error type

#### Kjernefunksjonalitet

```rust
/// Henter en Gopher-ressurs
///
/// # Arguments
/// * `url` - Gopher-URL (gopher://host[:port]/[type][selector])
///
/// # Returns
/// * `Ok(GopherResponse)` - Parsed respons
/// * `Err(GopherError)` - Ved feil
pub async fn fetch(url: &str) -> Result<GopherResponse, GopherError> {
    let parsed = parse_gopher_url(url)?;
    let stream = TcpStream::connect((parsed.host, parsed.port)).await?;

    // Send selektor
    stream.write_all(format!("{}\r\n", parsed.selector).as_bytes()).await?;

    // Les respons
    let response = read_response(&mut stream).await?;

    // Parse basert på type
    match parsed.item_type {
        GopherItemType::Directory => parse_menu(response),
        GopherItemType::TextFile => Ok(GopherResponse::text(response)),
        GopherItemType::Search => Ok(GopherResponse::search(response)),
        _ => Ok(GopherResponse::text(response)),
    }
}
```

#### URL-parsing

```rust
/// Parser en Gopher-URL til komponentene
///
/// Format: gopher://host[:port]/[type][selector]
///
/// Eksempler:
///   gopher://gopher.floodgap.com/          → host, port=70, type=1, selector=""
///   gopher://example.com/0/about.txt       → host, port=70, type=0, selector="/about.txt"
///   gopher://example.com:7070/1/docs       → host, port=7070, type=1, selector="/docs"
pub fn parse_gopher_url(url: &str) -> Result<GopherUrl, GopherError> {
    // ...
}
```

### Fase 2: Meny-til-Markdown konvertering (`gophermap.rs`)

**Estimert tid:** 1 uke

#### Oppgaver

- [ ] Konverter Gopher-menyer til markdown
- [ ] Håndter alle støttede elementtyper
- [ ] Generer klikkbare lenker for navigerbare elementer
- [ ] Vis informasjonslinjer (type `i`) som ren tekst
- [ ] Ekstraher tittel fra meny (første overskrift eller info-linje)
- [ ] Håndter relative og absolutte selektorer

#### Konverteringsregler

```
Gopher meny-element                    Markdown-resultat
─────────────────────────────────────────────────────────
i Velkomsttekst                    →   Velkomsttekst
i                                  →   (tom linje)
1Undermeny  /docs  host  70       →   📁 [Undermeny](gopher://host:70/1/docs)
0Tekstfil  /info.txt  host  70    →   📄 [Tekstfil](gopher://host:70/0/info.txt)
7Søk  /search  host  70           →   🔍 [Søk](gopher://host:70/7/search)
hWeb-lenke  URL:http://...        →   🌐 [Web-lenke](http://...)
IBilde  /pic.jpg  host  70        →   🖼️ [Bilde](gopher://host:70/I/pic.jpg)
3Feil  /  host  70                →   ⚠️ Feil
```

#### Eksempel på konvertering

**Gopher-meny (rå):**
```
iWelcome to Floodgap	fake	(NULL)	0
i	fake	(NULL)	0
1Floodgap Home	/	gopher.floodgap.com	70
0About Floodgap	/gopher/about	gopher.floodgap.com	70
7Search Veronica	/v2/vs	gopher.floodgap.com	70
iThis is information text	fake	(NULL)	0
```

**Resultat (Markdown):**
```markdown
Welcome to Floodgap

📁 [Floodgap Home](gopher://gopher.floodgap.com:70/1/)
📄 [About Floodgap](gopher://gopher.floodgap.com:70/0/gopher/about)
🔍 [Search Veronica](gopher://gopher.floodgap.com:70/7/v2/vs)

This is information text
```

### Fase 3: Tauri-kommandoer og frontend-integrasjon

**Estimert tid:** 1 uke

#### Backend (commands.rs)

- [ ] Ny Tauri-kommando: `fetch_gopher`
- [ ] Integrer med eksisterende feilhåndtering
- [ ] Søke-input håndtering (type 7)

```rust
#[tauri::command]
pub async fn fetch_gopher(url: String) -> Result<String, String> {
    let response = gopher::fetch(&url)
        .await
        .map_err(|e| e.to_string())?;

    match response.content_type {
        GopherContentType::Menu => {
            let markdown = gophermap::to_markdown(&response.items, &url);
            let html = crate::markdown::render_markdown(&markdown);
            Ok(html)
        }
        GopherContentType::Text => {
            let html = crate::markdown::render_markdown(&response.body);
            Ok(html)
        }
        GopherContentType::Search => {
            // Returner GOPHER_SEARCH_PROMPT for å trigge søkedialog
            Ok(format!("GOPHER_SEARCH_PROMPT:{}", url))
        }
        _ => Err("Ikke-støttet innholdstype".to_string()),
    }
}

#[tauri::command]
pub async fn gopher_search(url: String, query: String) -> Result<String, String> {
    let response = gopher::search(&url, &query)
        .await
        .map_err(|e| e.to_string())?;

    let markdown = gophermap::to_markdown(&response.items, &url);
    let html = crate::markdown::render_markdown(&markdown);
    Ok(html)
}
```

#### Frontend (navigation.js)

- [ ] Detekter `gopher://`-URLer i navigasjonsfunksjonen
- [ ] Kall `fetch_gopher` via Tauri IPC
- [ ] Håndter søke-prompts (gjenbruk Gemini input-dialog)
- [ ] Håndter kryssprotokoll-navigasjon (Gopher ↔ HTTP ↔ Gemini)

```javascript
async function navigateToUrl(url) {
    if (url.startsWith('gopher://')) {
        await loadGopherUrl(url);
    } else if (url.startsWith('gemini://')) {
        await loadGeminiUrl(url);
    } else {
        await loadHttpUrl(url);
    }
}

async function loadGopherUrl(url, addHistory = true) {
    showStatus('Laster Gopher-side...');

    try {
        const result = await invoke('fetch_gopher', { url });

        if (result.startsWith('GOPHER_SEARCH_PROMPT:')) {
            showGopherSearchDialog(url);
            return;
        }

        displayContent(result);
        if (addHistory) addToHistory(url);
        updateUrlBar(url);
    } catch (error) {
        showError(`Gopher-feil: ${error}`);
    }
}
```

### Fase 4: Søkefunksjonalitet (type 7)

**Estimert tid:** 3-5 dager

#### Oppgaver

- [ ] Gjenbruk Gemini input-dialog for Gopher-søk
- [ ] Send søkestreng som `<selektor>\t<søkestreng>\r\n`
- [ ] Parse søkeresultater (returneres som standard Gopher-meny)
- [ ] Vis resultater som markdown-liste

#### Søkedialog

Gopher-søk bruker samme UI-mønster som Gemini input-forespørsler:

```javascript
function showGopherSearchDialog(url) {
    // Gjenbruk eksisterende input-dialog fra Gemini
    const dialog = createInputDialog({
        title: 'Gopher-søk',
        prompt: 'Skriv inn søkeord:',
        sensitive: false,
        onSubmit: async (query) => {
            const result = await invoke('gopher_search', { url, query });
            displayContent(result);
        }
    });
}
```

### Fase 5: Polering og edge cases

**Estimert tid:** 3-5 dager

#### Oppgaver

- [ ] Håndter binære responser (vis feilmelding, ikke forsøk rendering)
- [ ] Håndter Gopher+-utvidelser (graceful degradation)
- [ ] Håndter ugyldige menylinjer (robust parsing)
- [ ] Encoding-deteksjon og konvertering (Latin-1, UTF-8)
- [ ] Håndter servere som ikke følger protokollen nøyaktig
- [ ] Støtte for `gopher://`-lenker i markdown-innhold fra andre protokoller

---

## Gopher-til-Markdown konvertering

### Detaljerte konverteringsregler

#### Informasjonslinjer (type `i`)

Informasjonslinjer er den primære teksttypen i Gopher-menyer. De har ingen selektor og brukes for å vise statisk tekst.

```rust
// Input:  "iDette er informasjonstekst\tfake\t(NULL)\t0"
// Output: "Dette er informasjonstekst"

fn convert_info_line(item: &GopherItem) -> String {
    item.display.clone()
}
```

#### Mapper/menyer (type `1`)

```rust
// Input:  "1Dokumenter\t/docs\texample.com\t70"
// Output: "📁 [Dokumenter](gopher://example.com:70/1/docs)"

fn convert_directory(item: &GopherItem) -> String {
    let url = build_gopher_url(item);
    format!("📁 [{}]({})", item.display, url)
}
```

#### Tekstfiler (type `0`)

```rust
// Input:  "0README\t/readme.txt\texample.com\t70"
// Output: "📄 [README](gopher://example.com:70/0/readme.txt)"

fn convert_text_file(item: &GopherItem) -> String {
    let url = build_gopher_url(item);
    format!("📄 [{}]({})", item.display, url)
}
```

#### Søk (type `7`)

```rust
// Input:  "7Søk i arkivet\t/search\texample.com\t70"
// Output: "🔍 [Søk i arkivet](gopher://example.com:70/7/search)"

fn convert_search(item: &GopherItem) -> String {
    let url = build_gopher_url(item);
    format!("🔍 [{}]({})", item.display, url)
}
```

#### HTML-lenker (type `h`)

```rust
// Input:  "hGoogle\tURL:https://google.com\texample.com\t70"
// Output: "🌐 [Google](https://google.com)"

fn convert_html_link(item: &GopherItem) -> String {
    let url = if item.selector.starts_with("URL:") {
        &item.selector[4..]
    } else {
        &item.selector
    };
    format!("🌐 [{}]({})", item.display, url)
}
```

### Komplett konverteringsfunksjon

```rust
pub fn to_markdown(items: &[GopherItem], base_url: &str) -> String {
    let mut output = String::new();
    let mut prev_was_info = false;

    for item in items {
        let line = match item.item_type {
            GopherItemType::Info => {
                prev_was_info = true;
                convert_info_line(item)
            }
            GopherItemType::Directory => {
                if prev_was_info { output.push('\n'); }
                prev_was_info = false;
                convert_directory(item)
            }
            GopherItemType::TextFile => {
                if prev_was_info { output.push('\n'); }
                prev_was_info = false;
                convert_text_file(item)
            }
            GopherItemType::Search => {
                if prev_was_info { output.push('\n'); }
                prev_was_info = false;
                convert_search(item)
            }
            GopherItemType::Html => {
                if prev_was_info { output.push('\n'); }
                prev_was_info = false;
                convert_html_link(item)
            }
            GopherItemType::Error => {
                prev_was_info = false;
                format!("⚠️ {}", item.display)
            }
            GopherItemType::Gif | GopherItemType::Image => {
                if prev_was_info { output.push('\n'); }
                prev_was_info = false;
                let url = build_gopher_url(item);
                format!("🖼️ [{}]({})", item.display, url)
            }
            _ => {
                // Binære filer, telnet, etc. — vis som ikke-klikkbar info
                prev_was_info = false;
                format!("  {} *({})*", item.display, item.item_type.description())
            }
        };

        output.push_str(&line);
        output.push('\n');
    }

    output
}
```

---

## Frontend-integrasjon

### URL-gjenkjenning

Utvid den eksisterende URL-deteksjonslogikken i `navigation.js`:

```javascript
function detectProtocol(url) {
    if (url.startsWith('gemini://')) return 'gemini';
    if (url.startsWith('gopher://')) return 'gopher';
    if (url.startsWith('file://'))   return 'file';
    return 'http';  // Standard fallback
}
```

### Navigasjonshistorikk

Gopher-URLer integreres i den eksisterende historikk-stacken uten endringer. Back/forward fungerer sømløst mellom protokoller.

### Bokmerker

Gopher-URLer kan bokmerkes med eksisterende funksjonalitet. Visning i bokmerkelisten markeres med protokoll-ikon:

```javascript
function getProtocolIcon(url) {
    if (url.startsWith('gemini://')) return '♊';
    if (url.startsWith('gopher://')) return '🐿️';
    return '🌐';
}
```

### Søkedialog for Gopher

Gjenbruk av den eksisterende Gemini input-dialogen med tilpasset tekst:

```javascript
function showGopherSearchDialog(url) {
    const overlay = document.getElementById('gemini-input-overlay');
    const promptText = document.getElementById('gemini-prompt-text');
    const input = document.getElementById('gemini-input-field');

    promptText.textContent = 'Skriv inn søkeord:';
    input.type = 'text';
    input.value = '';

    // Tilpass submit-handling for Gopher
    input.onkeydown = async (e) => {
        if (e.key === 'Enter') {
            const query = input.value;
            overlay.style.display = 'none';
            await loadGopherSearch(url, query);
        }
    };

    overlay.style.display = 'flex';
    input.focus();
}
```

---

## Sikkerhet og personvern

### Sikkerhetsvurderinger

| Aspekt | Tilnærming |
|--------|------------|
| **Klartekst** | Standard Gopher er ukryptert — vis advarsel som ved HTTP |
| **Gopher-over-TLS** | Valgfri støtte for GoT (fremtidig) |
| **Telnet-lenker** | ❌ Blokkeres helt (type 8 og T) — sikkerhetsrisiko |
| **HTML-innhold** | Sanitize via html2md før visning (eksisterende pipeline) |
| **Binærfiler** | Aldri kjør, kun vis informasjon |
| **Redirect-angrep** | Gopher har ingen innebygd redirect, men h-lenker valideres |
| **URL-validering** | Validerer host, port, og selektor før tilkobling |
| **Buffer overflow** | Maksimal respons-størrelse: 5 MB |
| **Port-scanning** | Begrens til port 70 og bruker-spesifiserte porter |

### URL-validering

```rust
fn validate_gopher_url(url: &str) -> Result<GopherUrl, GopherError> {
    let parsed = Url::parse(url)?;

    // Kun gopher://-skjema
    if parsed.scheme() != "gopher" {
        return Err(GopherError::InvalidUrl("Ikke en gopher-URL".into()));
    }

    // Host er påkrevd
    let host = parsed.host_str()
        .ok_or(GopherError::InvalidUrl("Mangler host".into()))?;

    // Blokker localhost/privat nettverk (valgfritt)
    if is_private_address(host) {
        return Err(GopherError::InvalidUrl("Privat adresse blokkert".into()));
    }

    // Port (standard: 70)
    let port = parsed.port().unwrap_or(70);

    // Valider URL-lengde
    if url.len() > 1024 {
        return Err(GopherError::InvalidUrl("URL for lang".into()));
    }

    Ok(GopherUrl { host, port, item_type, selector })
}
```

### Personvern

- **Ingen cookies**: Gopher har ikke cookies — perfekt for Bare
- **Minimal fingerprinting**: Ingen User-Agent eller headers sendes
- **Ingen JavaScript**: Gopher-innhold inneholder aldri scripts
- **Ingen sporing**: Protokollen har ingen mekanismer for sporing

---

## Testing

### Unit tests for `gopher.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gopher_url_basic() {
        let url = parse_gopher_url("gopher://example.com/").unwrap();
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, 70);
        assert_eq!(url.item_type, GopherItemType::Directory);
        assert_eq!(url.selector, "");
    }

    #[test]
    fn test_parse_gopher_url_with_type() {
        let url = parse_gopher_url("gopher://example.com/0/readme.txt").unwrap();
        assert_eq!(url.item_type, GopherItemType::TextFile);
        assert_eq!(url.selector, "/readme.txt");
    }

    #[test]
    fn test_parse_gopher_url_custom_port() {
        let url = parse_gopher_url("gopher://example.com:7070/1/docs").unwrap();
        assert_eq!(url.port, 7070);
    }

    #[test]
    fn test_parse_gopher_url_invalid_scheme() {
        assert!(parse_gopher_url("http://example.com/").is_err());
    }

    #[test]
    fn test_parse_menu_line() {
        let line = "1Documents\t/docs\texample.com\t70";
        let item = parse_menu_line(line).unwrap();
        assert_eq!(item.item_type, GopherItemType::Directory);
        assert_eq!(item.display, "Documents");
        assert_eq!(item.selector, "/docs");
        assert_eq!(item.host, "example.com");
        assert_eq!(item.port, 70);
    }

    #[test]
    fn test_parse_menu_line_info() {
        let line = "iWelcome text\tfake\t(NULL)\t0";
        let item = parse_menu_line(line).unwrap();
        assert_eq!(item.item_type, GopherItemType::Info);
        assert_eq!(item.display, "Welcome text");
    }

    #[test]
    fn test_parse_menu_line_html_link() {
        let line = "hGoogle\tURL:https://google.com\texample.com\t70";
        let item = parse_menu_line(line).unwrap();
        assert_eq!(item.item_type, GopherItemType::Html);
        assert_eq!(item.selector, "URL:https://google.com");
    }

    #[test]
    fn test_parse_menu_termination() {
        let response = "iHello\tfake\t(NULL)\t0\r\n.\r\n";
        let items = parse_menu(response).unwrap();
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn test_parse_menu_ignores_empty_lines() {
        let response = "iHello\tfake\t(NULL)\t0\r\n\r\niWorld\tfake\t(NULL)\t0\r\n.\r\n";
        let items = parse_menu(response).unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_validate_url_too_long() {
        let long_url = format!("gopher://example.com/{}", "a".repeat(2000));
        assert!(validate_gopher_url(&long_url).is_err());
    }

    #[test]
    fn test_validate_url_missing_host() {
        assert!(validate_gopher_url("gopher:///path").is_err());
    }

    #[test]
    fn test_item_type_from_char() {
        assert_eq!(GopherItemType::from_char('0'), GopherItemType::TextFile);
        assert_eq!(GopherItemType::from_char('1'), GopherItemType::Directory);
        assert_eq!(GopherItemType::from_char('7'), GopherItemType::Search);
        assert_eq!(GopherItemType::from_char('i'), GopherItemType::Info);
        assert_eq!(GopherItemType::from_char('h'), GopherItemType::Html);
    }
}
```

### Unit tests for `gophermap.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_info_line_to_markdown() {
        let item = GopherItem {
            item_type: GopherItemType::Info,
            display: "Welcome to Gopher".to_string(),
            selector: "fake".to_string(),
            host: "(NULL)".to_string(),
            port: 0,
        };
        let md = to_markdown(&[item], "gopher://example.com/");
        assert!(md.contains("Welcome to Gopher"));
        assert!(!md.contains("["));  // Ingen lenke
    }

    #[test]
    fn test_directory_to_markdown() {
        let item = GopherItem {
            item_type: GopherItemType::Directory,
            display: "Documents".to_string(),
            selector: "/docs".to_string(),
            host: "example.com".to_string(),
            port: 70,
        };
        let md = to_markdown(&[item], "gopher://example.com/");
        assert!(md.contains("📁"));
        assert!(md.contains("[Documents]"));
        assert!(md.contains("gopher://example.com:70/1/docs"));
    }

    #[test]
    fn test_text_file_to_markdown() {
        let item = GopherItem {
            item_type: GopherItemType::TextFile,
            display: "README".to_string(),
            selector: "/readme.txt".to_string(),
            host: "example.com".to_string(),
            port: 70,
        };
        let md = to_markdown(&[item], "gopher://example.com/");
        assert!(md.contains("📄"));
        assert!(md.contains("[README]"));
    }

    #[test]
    fn test_search_to_markdown() {
        let item = GopherItem {
            item_type: GopherItemType::Search,
            display: "Search".to_string(),
            selector: "/search".to_string(),
            host: "example.com".to_string(),
            port: 70,
        };
        let md = to_markdown(&[item], "gopher://example.com/");
        assert!(md.contains("🔍"));
        assert!(md.contains("[Search]"));
    }

    #[test]
    fn test_html_link_to_markdown() {
        let item = GopherItem {
            item_type: GopherItemType::Html,
            display: "Google".to_string(),
            selector: "URL:https://google.com".to_string(),
            host: "example.com".to_string(),
            port: 70,
        };
        let md = to_markdown(&[item], "gopher://example.com/");
        assert!(md.contains("🌐"));
        assert!(md.contains("[Google]"));
        assert!(md.contains("https://google.com"));
    }

    #[test]
    fn test_error_to_markdown() {
        let item = GopherItem {
            item_type: GopherItemType::Error,
            display: "Not found".to_string(),
            selector: "".to_string(),
            host: "".to_string(),
            port: 0,
        };
        let md = to_markdown(&[item], "gopher://example.com/");
        assert!(md.contains("⚠️"));
        assert!(md.contains("Not found"));
    }

    #[test]
    fn test_complete_menu_conversion() {
        let items = vec![
            GopherItem {
                item_type: GopherItemType::Info,
                display: "Welcome".to_string(),
                selector: "fake".to_string(),
                host: "(NULL)".to_string(),
                port: 0,
            },
            GopherItem {
                item_type: GopherItemType::Directory,
                display: "Docs".to_string(),
                selector: "/docs".to_string(),
                host: "example.com".to_string(),
                port: 70,
            },
        ];
        let md = to_markdown(&items, "gopher://example.com/");
        assert!(md.contains("Welcome"));
        assert!(md.contains("📁 [Docs]"));
    }
}
```

### Integrasjonstester

```rust
#[cfg(test)]
mod integration_tests {
    #[test]
    fn test_gopher_url_in_fetcher() {
        // Verifiser at fetcher.rs ruter gopher://-URLer riktig
        let url = "gopher://gopher.floodgap.com/";
        assert!(url.starts_with("gopher://"));
        // Faktisk nettverktest kun i manuelle tester
    }

    #[test]
    fn test_full_pipeline() {
        // Parse meny → konverter til markdown → render til HTML
        let raw_menu = "iHello\tfake\t(NULL)\t0\r\n\
                         1Docs\t/docs\texample.com\t70\r\n\
                         .\r\n";
        let items = parse_menu(raw_menu).unwrap();
        let markdown = to_markdown(&items, "gopher://example.com/");
        let html = crate::markdown::render_markdown(&markdown);
        assert!(html.contains("Hello"));
        assert!(html.contains("href="));
    }
}
```

### Manuelle test-scenarioer

| # | Scenario | Forventet resultat |
|---|----------|--------------------|
| 1 | Naviger til `gopher://gopher.floodgap.com/` | Viser rot-meny som markdown |
| 2 | Klikk på en mappe-lenke | Navigerer til undermeny |
| 3 | Klikk på en tekstfil-lenke | Viser tekstinnhold som markdown |
| 4 | Bruk søk (type 7) | Input-dialog vises, resultater rendres |
| 5 | Klikk tilbake/fremover | Historikk fungerer mellom protokoller |
| 6 | Naviger fra Gopher til HTTP-lenke (type h) | Bytter til HTTP-visning |
| 7 | Naviger fra HTTP til gopher://-lenke | Bytter til Gopher-visning |
| 8 | Gopher-side med bilder (type I/g) | Viser avhengig av bildeinnstillinger |
| 9 | Ugyldig Gopher-URL | Feilmelding vises |
| 10 | Server-timeout | Timeout-melding etter 10 sekunder |

---

## Utfordringer og løsninger

### Tekniske utfordringer

| Utfordring | Løsning |
|------------|---------|
| **Ingen Content-Type** | Bestem innholdstype fra URL-ens typetegn |
| **Variabel terminering** | Noen servere bruker `.` på egen linje, andre bare lukker tilkoblingen — håndter begge |
| **Encoding** | Gopher definerer ikke encoding — anta UTF-8 med Latin-1 fallback |
| **Binært innhold** | Detekter og avvis binærfiler, vis informativ melding |
| **Relative selektorer** | Bygg absolutte URLer basert på gjeldende server og sti |
| **Ugyldige menylinjer** | Robust parsing — ignorer linjer som ikke matcher formatet |
| **Portnummer i URL** | Håndter standard port 70 og custom porter korrekt |
| **Gopher+-servere** | Graceful degradation — ignorer utvidede metadata |

### UX-utfordringer

| Utfordring | Løsning |
|------------|---------|
| **Menyer vs. tekst** | Vis menyer med ikoner (📁, 📄, 🔍) for visuell differensiering |
| **Søk uten prompt** | Gopher type 7 gir ingen beskrivende tekst — bruk standard prompt |
| **Mangel på titler** | Ekstraher tittel fra første info-linje eller bruk host-navn |
| **Ukryptert trafikk** | Vis advarsel i URL-bar (lignende HTTP-advarsel) |
| **Binærfiler** | Vis tydelig melding om at Bare ikke håndterer binære filer |

---

## Tanker om Gopher-klient i 2026

### Relevansen av Gopher i dag

I 2026 er Gopher en protokoll som feirer sitt 35-årsjubileum. Til tross for at den aldri fikk den massive adopteringen som HTTP, har den overlevd og funnet ny relevans i en tid der internett-brukere i økende grad søker alternativer til det kommersielle, annonsetunge, og overvåkingsbaserte nettet.

#### Gopher-fellesskapets tilstand

- **~300-500 aktive servere** globalt, primært drevet av entusiaster
- **Floodgap** forblir det sentrale knutepunktet med kataloger og søkemotoren Veronica
- **SDF (Super Dimension Fortress)** tilbyr gratis Gopher-hosting
- **Tilde-fellesskap** og retro-computing-miljøer holder protokollen levende
- Nytt innhold publiseres jevnlig, spesielt personlige blogger og tekniske artikler

### Hvorfor bygge en Gopher-klient i 2026?

#### 1. Det minimalistiske internettets renessanse

Det er en voksende bevegelse mot det som kalles «det lille nettet» (*small web*). Prosjekter som Gemini (2019), Gopher-revivalen, og tekstbaserte sosiale nettverk viser at det finnes et reelt behov for enklere, mer menneskesentrerte internett-opplevelser.

Bare posisjonerer seg perfekt i denne bevegelsen. Ved å støtte Gopher, HTTP og Gemini blir Bare en av svært få klienter som gir tilgang til alle tre økosystemer i ett enkelt verktøy.

#### 2. Personvern som kjerneargument

I 2026 er personvern mer relevant enn noensinne:
- Europeiske reguleringer (GDPR, Digital Services Act) skjerpes
- AI-drevet sporing og profilering er allestedsnærværende
- Brukere søker aktivt etter verktøy som respekterer deres privatliv

Gopher er, ved sin natur, sporing-fri. Ingen cookies, ingen headers, ingen JavaScript — bare rå TCP med en selektor-streng. For personvern-bevisste brukere er dette en uvurderlig egenskap.

#### 3. Pedagogisk verdi

En Gopher-klient i 2026 har også pedagogisk verdi:
- Demonstrerer nettverksprogrammering på sitt mest grunnleggende
- Viser hvordan internett-protokoller fungerer uten abstraksjoner
- Inspirerer nye utviklere til å forstå hva som skjuler seg «under panseret»

### Utfordringer for en moderne Gopher-klient

#### Sikkerhet: Det uløste problemet

Den største utfordringen for Gopher i 2026 er mangelen på kryptering. All trafikk sendes i klartekst, noe som gjør den sårbar for:

- **Avlytting**: ISPer og mellommenn kan lese alt innhold
- **Manipulering**: Innhold kan endres i transit (MITM-angrep)
- **Sensur**: Klartekst-trafikk er enkel å blokkere

**Mulige løsninger for Bare:**

1. **Gopher-over-TLS (GoT)**: Pakke Gopher-protokollen i TLS, tilsvarende HTTPS. Noen servere støtter allerede dette på alternative porter. Bare kan implementere GoT med TOFU-modellen fra Gemini-implementasjonen.

2. **Hybrid-tilnærming**: Forsøk TLS først, fall tilbake til klartekst med en synlig advarsel til brukeren. Dette gir sikkerhet der det er tilgjengelig, uten å bryte kompatibilitet.

3. **Proxy-basert kryptering**: Bruk en lokal proxy (Alternativ 2 fra PLAN.md) som håndterer Gopher-tilkoblinger og pakker dem i en kryptert tunnel.

#### Innholdskvalitet og oppdagbarhet

Gopher-innhold er ofte:
- Skrevet i ren ASCII uten formatering
- Vanskelig å oppdage (ingen moderne søkemotorer indekserer Gopher)
- Ofte utdatert eller forlatt

**Bares rolle:** Ved å konvertere Gopher-innhold til velformatert markdown med typografi og visuell struktur, kan Bare gjøre Gopher-innhold mer tilgjengelig og behagelig å lese enn i tradisjonelle Gopher-klienter.

### Fremtidsrettede utvidelser

#### Gopher+-støtte

Gopher+ legger til metadata og alternative visninger. En fremtidig versjon av Bare kan:
- Vise MIME-type informasjon
- La brukeren velge mellom alternative representasjoner
- Vise admin-informasjon og abstracts

#### Cross-protokoll oppdagelse

Bare kan bygge en unik posisjon som en «universell lesbar-nettleser» ved å:
- Auto-detektere protokoll basert på URL
- Tilby en samlet bokmerkeliste på tvers av protokoller
- Vise innhold konsistent uavhengig av kilde (HTTP, Gemini, Gopher)
- Potensielt tilby en «oppdagelses-modus» som aggregerer innhold fra alle tre protokoller

#### Offline-støtte og caching

Gopher-innhold er typisk lite (rent tekst) og godt egnet for caching:
- LRU-cache av besøkte sider
- Offline-lesemodus for cached innhold
- Eksport av Gopher-menyer som markdown-filer for offline-arkivering

#### Tilgjengelighet

En moderne Gopher-klient bør tenke på tilgjengelighet:
- Skjermleser-vennlig rendering
- Tastaturnavigasjon (allerede støttet i Bare)
- Skalerbar tekst og høy kontrast

### Bares visjon: Et verktøy for det lesbare nettet

I 2026 er Bare mer enn en markdown-nettleser — det er et verktøy for det *lesbare nettet*. Ved å støtte HTTP, Gemini og Gopher, gir Bare tilgang til tre ulike visjoner for internett:

| Protokoll | Visjon | Æra |
|-----------|--------|-----|
| **Gopher** | Hierarkisk, menydrevet dokumentdistribusjon | 1991 |
| **HTTP** | Hypertext og multimedia | 1991-nå |
| **Gemini** | Minimalistisk, personvern-først innholdslevering | 2019 |

Alle tre deler en grunnleggende idé: internett handler om *innhold*, ikke om reklame, sporing, eller visuelt støy. Bare velger å hedre denne ideen ved å gi brukeren en konsistent, ren leseopplevelse — uansett hvilken protokoll innholdet kommer fra.

### Oppsummering

En Gopher-implementasjon i Bare i 2026 er ikke bare en nostalgisk øvelse — det er en bevisst designbeslutning som:

1. **Styrker Bares posisjon** som den ledende multiprotokoll markdown-leseren
2. **Gir tilgang** til et unikt innholds-økosystem som ellers er vanskelig tilgjengelig
3. **Demonstrerer verdien** av enkle, åpne protokoller i en kompleks digital verden
4. **Inspirerer** til refleksjon over hva vi egentlig trenger fra internett

Implementasjonen er teknisk overkommelig (enklere enn Gemini), filosofisk konsistent med Bares verdier, og tilfører en dimensjon av historisk bevissthet som gjør prosjektet unikt.

---

## Tidsestimat

| Fase | Oppgave | Estimat |
|------|---------|---------|
| 1 | Gopher-klient (`gopher.rs`) | 1-2 uker |
| 2 | Meny-konvertering (`gophermap.rs`) | 1 uke |
| 3 | Tauri-kommandoer og frontend | 1 uke |
| 4 | Søkefunksjonalitet | 3-5 dager |
| 5 | Polering og edge cases | 3-5 dager |
| | **Totalt** | **4-6 uker** |

## Avhengigheter

| Komponent | Crate/teknologi | Formål |
|-----------|----------------|--------|
| TCP-klient | `tokio::net::TcpStream` | TCP-tilkobling (allerede i prosjektet) |
| TLS (fremtidig) | `tokio-rustls` | Gopher-over-TLS (gjenbruk fra Gemini) |
| URL-parsing | `url` | URL-parsing (allerede i prosjektet) |
| Markdown | `pulldown-cmark` | Markdown-rendering (allerede i prosjektet) |

**Ingen nye avhengigheter** er nødvendige for grunnleggende Gopher-støtte. Alle nødvendige crates finnes allerede i prosjektet.

---

*Sist oppdatert: Februar 2026*
