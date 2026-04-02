# Implementasjonsbeskrivelse: Norsk startportal for Bare-nettleseren

## Kontekst og mål

Du skal implementere en statisk norsk startportal for nettleseren **Bare** — en minimal Markdown-nettleser bygget med Rust og Tauri. Bare renderer Markdown direkte uten JavaScript, cookies eller tracking. Startportalen er den siden brukerne ser når de åpner Bare.

Portalen er **én enkelt Markdown-fil** (`index.md`) hostet på **GitHub Pages**. Den skal ikke inneholde HTML, JavaScript eller CSS — kun ren, semantisk Markdown. Den skal oppleves som kuratert og nyttig, ikke som en teknisk demo.

Målgruppen er norske brukere som er interesserte i et privat, raskt og støyfritt nettlesingsalternativ.

---

## Tekniske krav

- **Format**: Ren Markdown (`.md`)
- **Hosting**: GitHub Pages i repoet `FrankBurmo/bare-start` (eller tilsvarende)
- **Filstruktur**: Én primærfil — `index.md` — er tilstrekkelig for MVP
- **Ingen byggesteg**: Filen skal fungere direkte uten Jekyll, Hugo eller andre statiske generatorer
- **Lenker**: Alle lenker skal være absolutte HTTPS-URLer
- **Språk**: Norsk bokmål gjennomgående
- **Kompatibilitet**: Skal primært fungere i Bare, men også leses greit i GitHub-forhåndsvisning og andre Markdown-renderere

---

## Filosofi og tone

Portalen skal speile Bares egne verdier:

- **Minimalistisk** — ingen unødvendig tekst, ingen fyllmasse
- **Tillitsvekkende** — ingenting skjult, ingen tracking, ingen agendaer
- **Nyttig fremfor imponerende** — prioriter lenker folk faktisk bruker
- **Norsk kontekst** — offentlige tjenester, norske nyhetskilder, norsk infrastruktur

Portalen skal *ikke* føles som en generisk lenkeside eller et hobbyprosjekt. Den skal føles som et gjennomtenkt verktøy.

---

## Innholdsstruktur

Portalen er organisert i følgende seksjoner, i denne rekkefølgen:

### 1. Header / intro (øverst)

Kort velkomsttekst — maks 2–3 linjer. Skal kommunisere:
- Hva dette er (startside for Bare i Norge)
- At siden er statisk, rask og uten tracking
- Eventuelt dato for siste manuell oppdatering (`Sist oppdatert: [dato]`)

Eksempel på tone:
> Bare-start er en kuratert startside for norske Bare-brukere. Ingen reklame, ingen sporing — bare nyttige lenker.

### 2. Dynamiske tjenester

En liten seksjon med lenker til **dynamiske Markdown-endepunkter** som kjører på en egen server (Cloudflare Workers, implementeres senere). Disse eksisterer ikke ennå — bruk placeholder-URLer på formatet `https://bare-start.no/[tjeneste]`.

Inkluder følgende planlagte tjenester:
- **Vær** — `https://bare-start.no/vaer` — Yr-data for Oslo, Bergen, Trondheim, Tromsø
- **Nyheter** — `https://bare-start.no/nyheter` — NRK og andre RSS-kilder
- **Trafikk** — `https://bare-start.no/trafikk` — Meldinger fra Statens vegvesen
- **Valuta** — `https://bare-start.no/valuta` — Kurser fra Norges Bank

Marker tydelig at disse er «kommer snart» eller ikke aktive ennå, slik at brukere ikke blir frustrerte.

### 3. Offentlige tjenester

Lenker til sentrale norske offentlige nettsteder som fungerer godt i Bare (primært innholdstunge sider). Grupper dem logisk:

**Stat og forvaltning**
- Altinn — `https://www.altinn.no`
- NAV — `https://www.nav.no`
- Skatteetaten — `https://www.skatteetaten.no`
- Kartverket — `https://www.kartverket.no`
- Norge.no — `https://www.norge.no`

**Helse**
- Helsenorge — `https://www.helsenorge.no`
- FHI — `https://www.fhi.no`

**Samferdsel og infrastruktur**
- Statens vegvesen — `https://www.vegvesen.no`
- Ruter (Oslo) — `https://ruter.no`
- Entur (nasjonale reiser) — `https://entur.no`

### 4. Nyheter og medier

Lenker til norske nyhetskilder. Prioriter kilder med god lesbarhet i Bare (rent HTML, lite JavaScript-avhengig innhold). Inkluder:

- NRK Nyheter — `https://www.nrk.no`
- Aftenposten — `https://www.aftenposten.no`
- Dagbladet — `https://www.dagbladet.no`
- VG — `https://www.vg.no`
- Digi.no (teknologi) — `https://www.digi.no`

Legg gjerne til en linje om at Bare sin Readability-ekstraktor fjerner reklame og støy fra disse sidene automatisk.

### 5. Søk

En seksjon med direktelenker til søkemotorer og norsk-relevante søkeverktøy:

- DuckDuckGo — `https://duckduckgo.com` (anbefalt for personvern)
- Kvasir — `https://www.kvasir.no` (norsk)
- Google — `https://www.google.no`
- Bing — `https://www.bing.com`

Legg til en kort note om at DuckDuckGo er spesielt godt egnet for Bare-brukere pga. personvernprofil.

### 6. Teknologi og utvikling

En seksjon for teknisk interesserte brukere (Bares kjernemålgruppe):

- GitHub — `https://github.com`
- Hacker News — `https://news.ycombinator.com`
- Lobsters — `https://lobste.rs`
- The Old Net — `https://theoldnet.com` (historisk internett)

**Gemini og Gopher** (egne protokoller som Bare støtter):
- Legg til 3–5 interessante Gemini-kapsler på `gemini://`-format
- Legg til 1–2 Gopher-servere på `gopher://`-format
- Marker tydelig at disse krever Bare eller tilsvarende nettleser

Eksempel på Gemini-lenker å inkludere:
- `gemini://gemini.circumlunar.space` — Gemini-prosjektets hjemmeside
- `gemini://kennedy.gemi.dev` — Kennedy Gemini-leser
- `gemini://tilde.pink` — Tilde-fellesskap

### 7. Om Bare

En kort seksjon som forklarer Bare for nye brukere, med lenker til:
- Bares nettside — `https://frankburmo.github.io/bare/`
- GitHub-repoet — `https://github.com/FrankBurmo/bare`
- Releases (nedlasting) — `https://github.com/FrankBurmo/bare/releases/latest`

Hold dette konsist — portalen er ikke primært en markedsføringsside for Bare.

### 8. Footer

Enkel avslutning med:
- Lenke til GitHub-repoet for portalen selv
- Lisens (f.eks. MIT eller CC0)
- «Ingen sporing på denne siden»
- Oppfordring til å sende inn forslag via Issues

---

## Markdown-konvensjoner å følge

- Bruk `##` for seksjonsoverskrifter, `###` for undergrupper
- Lenker skal ha beskrivende lenkitekst — ikke bare URL som tekst
- Bruk korte beskrivelser (maks én linje) for hver lenke der det gir verdi
- Unngå tabeller — de brytes lett i ulike renderere
- Unngå inline HTML
- Hold linjene under 120 tegn for lesbarhet i råformat
- Bruk horisontale skillelinjer (`---`) mellom seksjoner sparsomt men konsekvent

---

## Eksempel på god lenkesyntaks

```markdown
- [Altinn](https://www.altinn.no) — innlogging, skjemaer og meldinger fra det offentlige
- [NAV](https://www.nav.no) — dagpenger, sykepenger, jobbsøk
```

Ikke slik:
```markdown
- https://www.altinn.no
- Altinn: https://www.altinn.no (klikk her)
```

---

## Hva som bevisst er utelatt

- **Værvarsler direkte i filen** — dette hører hjemme i det dynamiske endepunktet
- **Sosiale medier** — ikke i tråd med Bare-filosofien
- **Nettbutikker** — ikke relevant for en nøytral startportal
- **Reklame eller sponsede lenker** — aldri
- **JavaScript-tunge sider** — Bare ekstraherer innhold, men opplevelsen er dårligere

---

## Vedlikehold og oppdateringer

Filen oppdateres manuelt ved behov. Det er ikke nødvendig med automatisering for den statiske filen. En commit-kommentar som `Oppdatert lenker - april 2026` er tilstrekkelig versjonering.

Når de dynamiske endepunktene (`bare-start.no/*`) er operative, skal placeholder-notatene i seksjon 2 fjernes og erstattes med aktive lenker.
