# Teknisk analyse: Veien mot den perfekte Bare-nettleseren

> Arkitektur- og UX-gjennomgang av Bare v0.1.5
> Utarbeidet som grunnlag for videre utvikling. Sist oppdatert: juni 2026.

---

## 1. Sammendrag

Bare er allerede et **velstrukturert, modent lite kodebase** for å være på v0.1.5: ren
modulinndeling i både Rust-backend og JS-frontend, god testdekning på kjernelogikk
(117+ enhetstester), tre fungerende protokoller (HTTP/S, Gemini, Gopher) og en
gjennomtenkt personvernprofil. Fundamentet er solid.

Men «den perfekte nettleseren» — én som oppleves som **bedre enn Chrome, Edge og Opera** —
oppnås ikke ved å konkurrere på funksjonsbredde. Bare vinner kun ved å være
*kategorisk overlegen* på de fire aksene den allerede har valgt: **lesbarhet, hastighet,
personvern og fokus**. Denne rapporten identifiserer hva som står i veien for det i dag,
og foreslår et konkret, prioritert veikart.

**Hovedkonklusjon i én setning:** Bare bør slutte å tenke på seg selv som «en nettleser med
færre funksjoner» og begynne å tenke på seg selv som **det beste leseverktøyet for tekst-
weben** — og deretter polere den opplevelsen til den er feilfri.

De tre viktigste tekniske hindringene akkurat nå:

1. **Innholdsekstraksjonen er naiv** (streng-basert `find`/`rfind` i `converter.rs`) og
   leverer uforutsigbar kvalitet på vanlige nettsider — selve kjernen i verdiløftet. ✅ (Løst i v0.1.6)
2. **Personvernløftet er ikke fullt innfridd**: CSP tillater eksterne bilder
   (`img-src ... https: http:`), så sporingspiksler *kan* lastes, stikk i strid med
   «umulig å spore». ✅ (Løst i v0.1.6)
3. **Opplevelsen mangler «delight»-laget**: ingen caching (treg tilbake-navigasjon),
   ingen kommandopalett/adressefelt-forslag, ingen historikk-UI, begrenset tilgjengelighet. 🟡 (Delvis løst via render-cache)

---

## 2. Metode og omfang

Gjennomgangen dekker hele repoet per v0.1.5:

| Lag | Filer gjennomgått |
|-----|-------------------|
| Rust-backend | `commands.rs`, `fetcher.rs`, `markdown.rs`, `converter.rs`, `gemini.rs`, `gemtext.rs`, `gopher.rs`, `gophermap.rs`, `settings.rs`, `bookmarks.rs`, `lib.rs` |
| Frontend | `index.html`, `styles.css`, samt `js/`: `constants.js`, `state.js`, `dom.js`, `ui.js`, `settings.js`, `bookmarks.js`, `search.js`, `navigation.js`, `events.js`, `i18n.js`, `main.js` |
| Konfig/bygg | `tauri.conf.json`, `Cargo.toml`, `.github/workflows/`, `capabilities/` |
| Dokumentasjon | `PLAN.md`, `architecture.c4`, `README.md`, `GOPHER.md` |

Ekstern research: Mozilla Readability-algoritmen (DOM-skåring, link-tetthet,
sanitering med DOMPurify + CSP), samt etablert praksis for lesetypografi og
tastaturdrevne nettlesere (qutebrowser/Vimium).

---

## 3. Hva som allerede er bra

Det er viktig å ikke rive ned det som fungerer. Disse styrkene bør **bevares og bygges på**:

- **Ren arkitektur.** Backend følger single-responsibility per modul; frontend har en
  tydelig, dokumentert avhengighetsrekkefølge. C4-modellen i `architecture.c4` holdes
  oppdatert — sjeldent og verdifullt.
- **Idiomatisk Rust.** Gjennomgående `Result` + `thiserror`, `?`-propagering, få `unwrap()`
  utenfor tester. Egendefinerte feiltyper per modul.
- **Robust protokoll-lag.** Gemini-klienten implementerer TOFU korrekt (SHA-256-fingeravtrykk,
  `known_hosts.json`, MITM-deteksjon), iterativ redirect-håndtering med tak, og
  respons-/URL-størrelsesgrenser. Gopher følger RFC 1436 ryddig.
- **Sikker stack.** rustls med ring (ingen OpenSSL/native-tls), `clamp()` på alle numeriske
  innstillinger, ammonia-sanitering på HTML-konverteringsstien.
- **Internasjonalisering.** 13 språk i frontend allerede på plass — uvanlig ambisiøst tidlig.
- **Gjennomtenkt onboarding** og statuslinje i Netscape-stil gir produktet karakter.

---

## 4. Tekniske funn per lag

Hvert funn er merket med alvorlighetsgrad: 🔴 kritisk · 🟡 viktig · 🟢 forbedring.

### 4.1 Innholdsekstraksjon og konvertering (`converter.rs`)

- ✅ **DOM-basert «readability».** (Løst i v0.1.6) Erstattet naiv streng-`find` med `dom_smoothie` for ekte DOM-skåring og Markdown-konvertering.
- ✅ **`fix_broken_links` er fjernet.** (Løst i v0.1.6) `dom_smoothie` håndterer komplekse lenker og bilder korrekt, så den manuelle oppryddingen er ikke lenger nødvendig.
- 🟡 **`decode_html_entities` er en hardkodet miniordbok** (~15 entiteter). Vil bomme på
  numeriske entiteter (`&#8217;`) og mindre vanlige navngitte entiteter.
- ✅ **`readability_enabled`-innstillingen er koblet til.** (Løst i v0.1.6) Bryteren i UI har nå full effekt på konverteringen.

### 4.2 Markdown-rendering (`markdown.rs`)

- ✅ **Sanering av rå HTML.** (Løst i v0.1.6) Lagt til `ammonia` i `markdown.rs` som sanerer all HTML-output fra `pulldown-cmark`.
- 🟡 **Ingen syntaksutheving.** `PLAN.md` lover `syntect`, men kodeblokker rendres flatt.
  For et leseverktøy rettet mot teknisk innhold er dette en merkbar mangel.
- 🟢 **Ingen overskrifts-ankere.** Overskrifter får ikke `id`, så innholdsfortegnelse,
  «kopier lenke til seksjon» og `#fragment`-navigasjon er umulig.
- 🟢 Manglende utvidelser: fotnoter, smart typografi (`ENABLE_SMART_PUNCTUATION`),
  definisjon­slister.

### 4.3 Nettverk og ytelse (`fetcher.rs`, `commands.rs`)

- ✅ **Render-cache implementert.** (Løst i v0.1.6) LRU-cache i backend fjerner forsinkelse ved navigasjon mellom nylig besøkte sider.
- ✅ **Størrelsesgrense på HTTP.** (Løst i v0.1.6) Alle HTTP-nedlastinger begrenses nå til 5 MB, likt Gemini og Gopher.
- 🟡 **Streng-basert IPC-protokoll.** Backend signaliserer spesialtilstander til frontend
  via magiske strengprefikser: `CONVERSION_PROMPT:`, `GEMINI_INPUT_PROMPT:`,
  `GOPHER_SEARCH_PROMPT:`. Frontend parser disse med `indexOf(':http')` o.l. Skjørt,
  vanskelig å teste, og blander feilkanal med datakanal.
- 🟡 **Lokaliseringsbrudd.** Backend emitterer statusmeldinger på **norsk** (`"Slår opp…"`,
  `"Kobler til…"`) som vises direkte i en frontend som ellers har 13 språk. En
  engelsk­språklig bruker får norsk i statuslinjen.
- 🟢 **Globale singletons med `Mutex` + `.lock().unwrap()`.** Ved panikk i en låst seksjon
  blir mutexen forgiftet, og påfølgende `.unwrap()` panikker hele appen. Bør håndteres.
- 🟢 **`Fetcher::validate_url` aksepterer `gemini`**, men `Fetcher` kan ikke faktisk hente
  Gemini — en inkonsekvens som inviterer feil.

### 4.4 Frontend-arkitektur (`js/`, `index.html`)

- 🟡 **Ingen byggsteg / modulsystem.** 11 separate `<script>`-tagger laster alt inn i
  global scope i en håndholdt rekkefølge. Fungerer, men er sårbart for navnekollisjoner,
  gir ingen tre-shaking/minifisering, og skalerer dårlig.
- 🟡 **Native `confirm()` for konverteringsvalg** (`navigation.js`). Et OS-dialogvindu
  bryter den ellers gjennomførte retro/brutalist-estetikken og kan ikke styles eller
  oversettes konsistent.
- 🟢 **Simulert framdrift.** Footer-progresjonen mappes til faste prosenter per steg
  (`LOADING_STEP_PROGRESS`), ikke faktiske byte. Greit som plassholder, men ekte
  nedlastings­framdrift ville føltes mer presist.
- 🟢 Single global `state`-objekt uten observatører — endringer krever manuelle
  UI-oppdateringskall, lett å glemme.

### 4.5 Sikkerhet og personvern (`tauri.conf.json`, sanering)

- ✅ **Stram CSP.** (Løst i v0.1.6) `img-src` er begrenset til `'self' data:`, som blokkerer tredjepartssporing.
- ✅ **Bilde-policy.** (Løst i v0.1.6) `ImageMode` (Block, Placeholder, Show) er implementert i innstillinger.
- 🟡 **`renderContent` bruker `innerHTML`** (`ui.js`) med HTML fra backend. XSS er i dag
  avverget *kun* av CSP (ingen `script-src 'unsafe-inline'`, og `<script>` kjører ikke via
  `innerHTML`). Det er ingen sanering i dybden på den rene markdown-stien (jf. 4.2).
  Mozilla anbefaler eksplisitt DOMPurify **+** CSP for upålitelig innhold — Bare har bare det
  ene laget her.
- 🟢 **`devtools`-feature på i `Cargo.toml`** bygges også i release. Bør gates bak
  `debug_assertions`/feature-flag.
- 🟢 **User-Agent lekker eksakt versjon** (`Bare/0.1.5`). Minimal fingeravtrykksflate, men
  kan generaliseres.

### 4.6 Tilgjengelighet (a11y)

- 🟡 **Symbol-/emoji-knapper uten `aria-label`.** Knappene har `title` (bra for mus/hover),
  men `◄ ► ⌂ ↻ ☆ 📑 ≡` har ikke tekstalternativ for skjermlesere.
- 🟡 **Modaler mangler fokusfelle og semantikk.** Ingen `role="dialog"`/`aria-modal`,
  fokus flyttes ikke inn/ut, og Tab kan vandre bak overlegget.
- 🟢 Ingen «hopp til innhold»-lenke; `prefers-reduced-motion` respekteres ikke for
  smooth-scroll og progress-animasjoner.

---

## 5. Hva «bedre enn Chrome, Edge og Opera» faktisk betyr

Bare kan **ikke** og **bør ikke** prøve å slå Chromium på generell web-kompatibilitet —
det er en tapt kamp og i strid med prosjektets sjel. Den perfekte opplevelsen defineres
i stedet relativt til hva brukeren faktisk gjør i Bare: **lese**.

| Akse | Chrome/Edge/Opera | Bares vinnende posisjon |
|------|-------------------|--------------------------|
| **Lesbarhet** | Sidens design styrer; sprik | Ett konsistent, typografisk perfekt format — *alltid* |
| **Hastighet** | 2–5 MB/side, JS-jank | 5–50 KB, øyeblikkelig render, cachet tilbake-nav |
| **Personvern** | Sporing er normen | Null JS/cookies + **ekte** bildeblokkering |
| **Fokus** | Annonser, popups, varsler | Helt stille — bare innhold |
| **Tastatur** | Påklistret | Tastatur-først fra grunnen (Vim + lenkehint) |

Konkret betyr «perfekt UX» for Bare disse opplevelses­egenskapene:

1. **Øyeblikkelig.** Ingen merkbar forsinkelse — verken ved første last (cache + streaming)
   eller ved tilbake/fram (in-memory render-cache).
2. **Rolig.** Ingenting beveger seg uten grunn. Ren typografi med korrekt linjelengde
   (~66 tegn), god vertikal rytme, sepia/natt-temaer.
3. **Forutsigbar.** Samme nettside ser lik ut hver gang og hos alle — ekstraksjonen *må*
   være pålitelig.
4. **Tastaturdrevet flyt.** Du skal kunne navigere hele weben uten å løfte hånden fra
   tastaturet (lenkehint à la Vimium, kommandopalett, inkrementelt søk).
5. **Absolutt privat.** Personvern er ikke en innstilling, det er en garanti — og den
   garantien må være teknisk vanntett.

---

## 6. Anbefalt veikart

Veikartet er delt i fire bølger. Hver bølge er selvstendig leverbar og etterlater
produktet i en bedre tilstand.

### Bølge 1 — Innfri kjerneløftet (fundament) ✅

Mål: gjør lesekvalitet og personvern *vanntette*. Uten dette er resten kosmetikk.

1. **Erstatt naiv ekstraksjon med ekte DOM-skåring.** ✅ (Løst i v0.1.6)
   - Bytt ut streng-`find` i `converter.rs` med en DOM-basert pipeline:
     `html5ever`/`scraper` for parsing, og en Readability-portering
     (bruker `dom_smoothie` v0.16+) for skåring på link-tetthet,
     tekstmengde og taggvekt.
   - Behold ammonia som saneringssteg, men **kjør det også på den rene markdown-stien**
     (saner output fra `pulldown-cmark`), slik Mozilla anbefaler (DOMPurify-ekvivalent + CSP).
   - Forventet effekt: dette alene løfter den opplevde kvaliteten mest av alt.
2. **Implementer bildepolitikk.** ✅ (Løst i v0.1.6)
   - Stram CSP til `img-src 'self' data:` som standard (blokker eksternt).
   - Innfør `ImageMode { Block, Placeholder, Show }` (allerede skissert i
     `copilot-instructions.md`) med globalt valg + per-side-overstyring via verktøylinje.
   - Standard = blokker/placeholder. Gjør «vis bilder» til et bevisst, lokalt valg.
   - Resultat: personvernløftet blir endelig teknisk sant.
3. **Innfør render-cache (LRU).** ✅ (Løst i v0.1.6)
   - In-memory `lru`-cache nøklet på endelig URL, med konfigurerbar størrelse og TTL,
     som respekterer `Cache-Control`. Gir øyeblikkelig tilbake/fram-navigasjon.
4. **Tak på HTTP-respons + streaming.** ✅ (Løst i v0.1.6)
   - Speil Gemini/Gopher-grensen (5 MB) på HTTP, og strøm store nedlastinger i stedet for
     `text().await` i ett jafs.

### Bølge 2 — Polér leseopplevelsen (delight) ✅

Mål: gjør selve lesingen åpenbart bedre enn i en vanlig nettleser.

5. **Typografi-pass.** ✅ (Løst i v0.1.7) Korrekt målelengde (~60–75 tegn), forbedret vertikal rytme,
   sepia + «høykontrast» i tillegg til lys/mørk, valgfri font-paring (serif for brødtekst).
6. **Auto-innholdsfortegnelse + lesefremdrift.** ✅ (Løst i v0.1.7) Generér `id` på overskrifter, vis en
   sammleggbar TOC i margen for lange dokumenter, pluss en tynn lesefremdrifts­indikator og
   estimert lesetid.
7. **Syntaksutheving.** ✅ (Løst i v0.1.7) Integrér `syntect` (server-side, ingen JS) som lovet i planen.
8. **Lenkehint (Vimium-stil).** ✅ (Løst i v0.1.7) Trykk `f` for å få tastetips på alle synlige lenker —
   den enkeltfunksjonen som mest overbeviser tastaturbrukere om at Bare er «deres» nettleser.

### Bølge 3 — Navigasjon og flyt (kraftbruker)

Mål: fjern all friksjon i å komme dit du vil.

9. **Kommandopalett (`Ctrl+K`).** 🟡 Fuzzy-søk på tvers av bokmerker, historikk og
   handlinger. Erstatter behovet for mange separate menyer.
10. **Smart adressefelt.** 🟡 Forslag fra historikk + bokmerker mens du skriver;
    søkemotor-fallback for ikke-URL-input.
11. **Historikk-UI + øktgjenoppretting.** 🟢 Vedvarende historikk (i dag kun 50 i minnet),
    en søkbar historikkvisning, og «gjenopprett forrige økt» ved oppstart.
12. **Lese-liste / lagre-for-senere.** 🟢 Lokalt arkiv av sider (markdown på disk) —
    passer personvernprofilen perfekt og gir reell offline-verdi.
13. **Avklar fane-spørsmålet.** Enten lette faner *eller* en bevisst «én-dokument-ad-gangen +
    lese-liste»-modell. Ikke la det forbli ubesluttet — det former hele navigasjonen.

### Bølge 4 — Robusthet, kvalitet og arkitektur

Mål: gjør kodebasen vedlikeholdbar i det lange løp.

14. **Erstatt streng-prefiks-IPC med typede `enum`-svar.** 🟡 La kommandoer returnere et
    `serde`-tagget resultat (`Content | NeedsInput | NeedsConversion | Error`) i stedet for
    magiske prefikser. Fjerner skjør strengparsing i `navigation.js`.
15. **Sentralisér lokalisering.** 🟡 Backend emitterer **i18n-nøkler**, frontend oversetter.
    Fjern norske statusstrenger fra `commands.rs`.
16. **Lite byggsteg for frontend.** 🟢 `esbuild`/`vite` for bunting + minifisering, og helst
    ES-moduler eller TS/JSDoc-typer. Behold «vanilla»-filosofien — bare gjør den robust.
17. **Test- og kvalitetsløft.** 🟢
    - Integrasjonstester med `wiremock` for nettverkslaget.
    - `cargo-fuzz` på parserne (`gemtext`, `gophermap`, menylinjer).
    - Snapshot-tester på konverteringsoutput.
    - Minimal frontend-test (i dag finnes ingen).
18. **A11y-pass.** 🟡 `aria-label` på alle ikonknapper, `role="dialog"`/fokusfelle i modaler,
    «hopp til innhold», og `prefers-reduced-motion`.

---

## 7. Prioritert backlog

Sortert etter effekt ÷ innsats. «Effekt» = bidrag til den perfekte opplevelsen.

| # | Tiltak | Effekt | Innsats | Bølge |
|---|--------|--------|---------|-------|
| 1 | DOM-basert innholdsekstraksjon | ✅ Ferdig | Høy | 1 |
| 2 | Bildeblokkering + stram CSP | ✅ Ferdig | Lav | 1 |
| 3 | Render-cache (LRU) | ✅ Ferdig | Lav | 1 |
| 4 | Saner markdown-stien (ammonia) | ✅ Ferdig | Lav | 1 |
| 5 | Koble til `readability_enabled` igjen | ✅ Ferdig | Triviell | 1 |
| 6 | Lenkehint (`f`) | ✅ Ferdig | Middels | 2 |
| 7 | Typografi + sepia/lese-temaer | ✅ Ferdig | Lav | 2 |
| 8 | Kommandopalett (`Ctrl+K`) | 🟡 Høy | Middels | 3 |
| 9 | Smart adressefelt med forslag | 🟡 Høy | Middels | 3 |
| 10 | Auto-TOC + lesefremdrift | ✅ Ferdig | Middels | 2 |
| 11 | Syntaksutheving (`syntect`) | ✅ Ferdig | Lav | 2 |
| 12 | Typet IPC i stedet for strengprefiks | 🟡 Middels | Middels | 4 |
| 13 | Lokaliser backend-status | 🟡 Middels | Lav | 4 |
| 14 | A11y-pass (aria, fokusfelle) | 🟡 Middels | Lav | 4 |
| 15 | HTTP-størrelsestak + streaming | ✅ Ferdig | Lav | 1 |
| 16 | Historikk-UI + øktgjenoppretting | 🟢 Middels | Middels | 3 |
| 17 | Frontend-byggsteg | 🟢 Lav | Middels | 4 |
| 18 | Test-/fuzz-løft | 🟢 Lav | Middels | 4 |

---

## 8. Vedlikehold og CI-stabilitet ✅

Siste gjennomgang (juni 2026) har sikret:
- **Clean CI:** Alle Clippy-advarsler og testfeil er rettet.
- **Dependency Management:** Løst konflikter i `time`-craten ved å oppdatere til Tauri v2.11.2.
- **Kvalitetskontroll:** 117+ enhetstester passerer feilfritt.

---

## 9. Risikoer og avveininger

- **Ekstraksjon vs. ren minimalisme.** En full Readability-portering øker
  binærstørrelse og kompleksitet. Avveiing: kvaliteten på kjerneverdiløftet er viktigere
  enn noen hundre KB. Hold den isolert bak et tydelig modulgrensesnitt.
- **Bildeblokkering vs. brukervennlighet.** Standard-blokkering kan overraske nye brukere.
  Avbøt med en tydelig, ett-klikks «vis bilder på denne siden»-knapp og forklaring i
  onboarding.
- **Faner vs. fokus.** Faner kan undergrave «ett dokument om gangen»-roen. Hvis de innføres,
  gjør det som en bevisst, lett modell — ikke en Chrome-klone.
- **Byggsteg vs. «vanilla»-løftet.** Et byggsteg bryter ikke filosofien så lenge utdataene
  forblir ren, avhengighetsfri HTML/CSS/JS i WebViewen. Behold lesbarhet som mål.
- **Scope-disiplin.** Den største risikoen er funksjonskryp. Hvert nytt tiltak må bestå
  testen: *gjør dette lesing/hastighet/personvern/fokus åpenbart bedre?* Hvis ikke — la det
  være.

---

## 10. Suksesskriterier (målbart)

Definer «perfekt» med tall, så det kan verifiseres:

| Mål | Måltall |
|-----|---------|
| Tid til render, cachet side | < 16 ms (ett bilde-frame) |
| Tid til render, typisk nettside (henting + ekstraksjon) | < 400 ms på bredbånd |
| Eksterne nettverkskall ved lesing av en side | **0** utover selve dokumentet (med mindre bruker viser bilder) |
| Ekstraksjons-presisjon på topp 100 nyhets-/blogg-sider | > 90 % «ren hovedtekst uten boilerplate» |
| Tastatur-dekning | 100 % av kjerneflyt uten mus |
| A11y | WCAG 2.1 AA på app-skallet |
| Binærstørrelse | Fortsatt < 10 MB |

---

## 11. Referanser

- Mozilla Readability — DOM-skåring, link-tetthet, og anbefalt sanering (DOMPurify + CSP):
  <https://github.com/mozilla/readability>
- Tauri 2 sikkerhet/CSP: <https://tauri.app/security/>
- pulldown-cmark (rå-HTML-håndtering): <https://docs.rs/pulldown-cmark/>
- Aktuelle Rust-crates: `scraper`/`html5ever` (DOM), `dom_smoothie`/`readability`
  (ekstraksjon), `lru` (cache), `syntect` (utheving).
- Intern: `docs/PLAN.md`, `docs/architecture.c4`, `.github/copilot-instructions.md`.

---

*Denne analysen er et levende dokument. Oppdater backlogen og suksesskriteriene etter hvert
som bølgene leveres.*
