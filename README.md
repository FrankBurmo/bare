# bare
En åpen kildekode-nettleser som kun leser Markdown. Ingen scripts, ingen cookies, ingen CSS-kaos. Bare innholdet du kom for å lese, servert i et rent og strukturert format direkte fra kilden.

> "The internet doesn't have to be heavy."

**Bare** er en eksperimentell nettleser bygget for å utforske et tekst-basert internett. 
Den ignorerer tradisjonelle nettsider og renderer kun `.md`-filer direkte fra HTTP-responser.

### Hvorfor?
- **Lynrask:** Ingen tunge rammeverk å laste ned.
- **Personvern:** Ingen støtte for scripts betyr null sporing.
- **Fokus:** Innholdet presenteres konsistent uavhengig av kilden.

![Netscape_inspirert_B_animasjon_med_jord](https://github.com/user-attachments/assets/29ca95b2-d09b-4ba4-8293-752f4df8624c)

## Status

🚧 **Under utvikling** - Dette prosjektet er i tidlig fase. Se [PLAN.md](PLAN.md) for detaljert utviklingsplan.

## Teknologi

Bare er bygget med:

- **[Tauri 2.0](https://tauri.app/)** - Lett og sikker app-rammeverk (~2-5 MB vs Electron's ~100 MB)
- **Rust** - Backend for sikkerhet og ytelse
- **[pulldown-cmark](https://crates.io/crates/pulldown-cmark)** - Rask CommonMark + GFM markdown parser
- **[reqwest](https://crates.io/crates/reqwest)** - Async HTTP-klient med TLS
- **Vanilla HTML/CSS/JS** - Minimal frontend uten frameworks

## Funksjoner

### Nåværende (planlagt)
- ✅ Visning av `.md`-filer fra HTTP/HTTPS
- ✅ Lokale markdown-filer
- ✅ Back/forward navigasjon
- ✅ Bokmerker
- ✅ Lys/mørk modus
- ✅ Keyboard shortcuts

### Fremtidige muligheter
- ⚠️ HTML-til-Markdown konvertering for vanlige nettsider
- ⚠️ Gemini-protokoll støtte (gemini://)
- ⚠️ Gopher-protokoll støtte (gopher://)
- ⚠️ PDF-eksport
- ⚠️ Custom themes

## Sikkerhet og Personvern

Bare er designet med personvern som første prioritet:

| Funksjon | Status | Personvern-gevinst |
|----------|--------|-------------------|
| JavaScript | ❌ Ingen støtte | Null sporing, ingen malware |
| Cookies | ❌ Ingen støtte | Ingen tredjeparts sporing |
| CSS | ❌ Minimal/ingen | Ingen fingerprinting via CSS |
| Bilder | ⚠️ Valgfritt | Forhindrer tracking pixels |
| Tracking | ❌ Umulig | Total beskyttelse |

## Installasjon

> ⚠️ Prosjektet er ikke klart for bruk ennå.

### Forutsetninger

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (for Tauri CLI)
- [Tauri Prerequisites](https://tauri.app/v2/guides/prerequisites/)

### Bygge fra kildekode

```bash
# Klon repository
git clone https://github.com/FrankBurmo/bare.git
cd bare

# Installer Tauri CLI
cargo install tauri-cli

# Kjør i utviklingsmodus
cargo tauri dev

# Bygg for produksjon
cargo tauri build
```

## Bruk

(Kommer når første versjon er klar)

## Utviklingsplan

Se [PLAN.md](PLAN.md) for detaljert roadmap med:
- 5 utviklingsfaser fra PoC til polert produkt
- Tekniske valg og arkitektur
- Vurdering av server-komponent
- Åpne spørsmål og beslutninger

## Bidrag

Bidrag er velkomne! Vennligst les [.github/copilot-instructions.md](.github/copilot-instructions.md) for kodestandard og prosjektfilosofi før du sender inn pull requests.

### Prinsipper for bidrag
- **Enkelhet først** - Ikke legg til unødvendig kompleksitet
- **Personvern alltid** - Aldri kompromiss på sikkerhet eller personvern
- **Test grundig** - Skriv tester for ny funksjonalitet
- **Dokumenter** - Public APIs skal ha dokumentasjon

## Inspirasjon

Bare er inspirert av:
- **[Gemini Protocol](https://geminiprotocol.net/)** - Minimalistisk dokumentprotokoll
- **[Lynx](https://lynx.invisible-island.net/)** - Tekstbasert nettleser siden 1992
- **[Gopher](https://en.wikipedia.org/wiki/Gopher_(protocol))** - Enkel dokumentdistribusjon fra 1991

## Filosofi

### Hva Bare ER
- En minimal markdown-leser for det moderne internett
- Et personvern-verktøy
- Et eksperiment i enkelhet

### Hva Bare IKKE er
- ❌ En fullverdig nettleser (vi vil aldri støtte JavaScript)
- ❌ En HTML-renderer (kun markdown er førsteklasses)
- ❌ En tekstbehandler (kun visning, ikke redigering)
- ❌ Et sosialt medie-verktøy

## Lisens

[Kommer - avventer beslutning på MIT/Apache 2.0/GPL]

## Kontakt

- **Issues:** [GitHub Issues](https://github.com/FrankBurmo/bare/issues)
- **Diskusjoner:** [GitHub Discussions](https://github.com/FrankBurmo/bare/discussions)

---

*"For en verden hvor innhold er viktigere enn animasjoner."*
