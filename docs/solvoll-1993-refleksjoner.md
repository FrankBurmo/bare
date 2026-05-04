# Refleksjoner: Solvoll (1993) og Bare-nettleseren

> Basert på: Dag Solvoll, Geir Ivarsøy, Håkon W. Lie og Per E. Dybvik —  
> *«Information exchange in MultiTorg»*, Telektronikk 4/93 (Cyberspace-utgaven, desember 1993)  
> Tilgjengelig: https://www.wiumlie.no/1993/telektronikk-4-93/Solvoll_D.html

---

## Om artikkelen

Artikkelen ble publisert i den historiske Telektronikk-utgaven fra 1993 som er kreditert som en av de første tidsskriftene publisert på World Wide Web. Telektronikk 4/93 vant anerkjennelse ved «Best of Web '94». Solvoll og kollegene — inkludert Håkon W. Lie, som senere foreslo CSS — beskriver det de kaller **Babels tårn-problemet** i informasjonsutveksling: Mangelen på felles formater for dokumentutveksling mellom ulike datasystemer gjør meningsfull kommunikasjon vanskelig.

Artikkelen analyserer en rekke dokumentstandarder fra tidlig nittitall:

- **ODA** (Office Document Architecture) — ISO-standard for sammensatte dokumenter, ambisiøs men kompleks
- **SGML** (Standard Generalized Markup Language) — metaspråk for strukturert dokumentutveksling
- **HTML** — en enkel SGML-applikasjon som raskt ble internettets lingua franca

**MultiTorg-prosjektet** var et praktisk forsøk ved Norsk Telecom Forskning på å bygge et distribuert elektronisk informasjonsmarked — en forgjenger til det vi i dag kjenner som internett — basert på WWW og Internett-protokoller.

Artiklens overordnede konklusjon er at **standardisering er avgjørende for informasjonsutveksling**, og at **enkelhet slår kompleksitet**: HTML vant over ODA fordi det var enkelt å implementere og enkelt å bruke.

---

## Tematiske paralleller til Bare

### 1. Formatsimplisitetens triumf — fra ODA til Markdown

Solvoll og kollegene dokumenterer en klar utviklingslinje:

```
ODA (svært rik, kompleks) → SGML (fleksibel, middels kompleks) → HTML (enkel, interoperabel)
```

I 1993 var spenningen mellom ODA's funksjonelle rikhet og HTML's brukervennlige enkelhet. HTML vant.

**Bare representerer det neste steget i den samme retningen:**

```
HTML (nå svært rik og kompleks) → Markdown (enkel, lesbar, interoperabel)
```

Der 1993-artikkelen observerte at kompleksitet er et hinder for informasjonsutveksling, viser dagens web at HTML selv har blitt for komplekst — ikke bare teknisk, men kommersielt og overvåkningsmessig. Bare sin markdown-filosofi er en direkte forlengelse av den innsikten.

---

### 2. Babels tårn — fortsatt uløst, men flyttet

I 1993 handlet Babels tårn-problemet om **proprietære applikasjonsformater** (Word, WordPerfect, Lotus Notes) som ikke snakket med hverandre.

I dag har problemet skiftet form: Det er ikke lenger mangelen på et felles format (HTML er universelt), men **lagene bygget oppå HTML** — JavaScript, tracking-piksler, fingerprinting, cookies, reklame-payloads — som skaper støy mellom innhold og leser.

**Bare svarer på det nye Babels tårn**: i stedet for å navigere kompleksiteten, striper den den vekk helt. Konverteringen fra HTML til Markdown (via html2md) er i bunn og grunn det samme som ODA-til-HTML-konvertering var i MultiTorg: en formattransformasjon for å oppnå interoperabilitet på mottakersiden.

---

### 3. Innhold kontra presentasjon

SGML og ODA hadde begge som kjernephilosofi å **skille innhold fra presentasjon**. HTML arvet dette prinsippet, men CSS og JavaScript har gradvis visket det ut igjen — spesielt i kombinasjon med kommersielle interesser.

Bare håndhever dette skillet absolutt:

- Ingen egendefinert CSS fra nettsider
- Ingen JavaScript-kjøring
- All presentasjon styres av brukerens egne preferanser (tema, skrift, zoom)

Dette er mer i tråd med den opprinnelige SGML-visjonen enn det moderne nettet er.

---

### 4. MultiTorg og det distribuerte informasjonsmarkedet

MultiTorg-prosjektets visjon var et **distribuert elektronisk informasjonsmarked** der leverandører tilbyr elektroniske varer og tjenester via åpne protokoller. Det er presist det internett ble — men med en kommersiell overlay som var ikke-eksisterende i 1993.

Bare's multi-protokoll-støtte (HTTP/HTTPS, Gemini, Gopher, lokale filer) reflekterer MultiTorg's opprinnelige visjon om **protokoll-uavhengig tilgang** til informasjon:

| Protokoll | Alder | Filosofi |
|-----------|-------|---------|
| `gopher://` | 1991 | Enkel hierarkisk navigasjon |
| `http://` | 1991 | Åpen hypertext-distribusjon |
| `gemini://` | 2019 | Minimal og personvernfokusert |

Det er ironisk — og passende — at Bare i 2025 støtter Gopher, en protokoll fra samme era som Solvoll-artikkelen, og Gemini, en protokoll som gjenoppfinner 1993-tankegangen med moderne personvernhensyn.

---

## Hvor Bare kan styrkes, sett i lys av artikkelen

Solvoll-artikkelen peker indirekte på flere områder der Bare kan forbedres eller presisere sin rolle.

### 5.1 Eksplisitt innholdstypeforhandling

Artikkelen understreker viktigheten av **format-forhandling** mellom avsender og mottaker — at systemene er enige om hvilken form informasjonen skal ha. HTTP har Content-Type-headere for nettopp dette, men de brukes inkonsekvent.

**Forbedringspotensial:** Bare kan sende eksplisitte `Accept`-headere som signaliserer preferanse for `text/markdown` og `text/plain` over `text/html`. Noen servere og CDN-er respekterer dette og kan da servere renere innhold direkte.

```
Accept: text/markdown, text/plain;q=0.9, text/html;q=0.5
```

Dette er en liten endring i `fetcher.rs` som er i full overensstemmelse med Solvoll-artiklenes ånd.

### 5.2 Semantisk strukturbevaring

SGML's styrke var **semantisk markup** — å definere hva noe *er*, ikke bare hvordan det *ser ut*. Konverteringen fra HTML til Markdown (via html2md) er per i dag relativt mekanisk og kan miste semantisk kontekst.

**Forbedringspotensial:** Bedre bevaring av dokumentets semantiske struktur ved konvertering:
- `<article>`, `<main>`, `<section>` → Markdown-seksjonsstruktur
- `<aside>` → Markdown-sidenote eller fjernes
- `<figure>` + `<figcaption>` → Bildebeskrivelse med kontekst
- `<blockquote cite="...">` → Kilde-attributtering i Markdown

### 5.3 Dokumentopprinnelse og metadata

Artikkelen la vekt på at informasjonssystemer bør bevare **kontekst om informasjonens opprinnelse**. I dag er dette relevant som kildeangivelse og tillit.

**Forbedringspotensial:** Bare kan vise enkel metadata om siden som vises:
- Opprinnelig URL
- Innholdstype (native Markdown, konvertert HTML, Gemtext, Gophermap)
- Hentetidspunkt
- Protokoll brukt

Dette kan vises i en subtil infoslinje eller i status-baren.

### 5.4 Lesbarhet som primærmål

Et av Solvoll-artiklenes implisitte poenger er at **tilgjengelighet til informasjon er et mål i seg selv** — ikke bare at informasjonen eksisterer, men at den kan forstås av mottakeren.

Bare prioriterer allerede lesbarhet, men kan styrkes ytterligere:
- Bedre typografisk grunnlinje (linjehøyde, skriftvalg som standard)
- Robust håndtering av ikke-latinske skriftsystemer
- Forbedret rendering av matematikk og kode i markdown (LaTeX, syntaks-highlighting)

### 5.5 Fra konsum til bidrag — informasjonsutveksling, ikke bare -mottak

MultiTorg handlet om **utveksling** — toveis informasjonsflyt. Bare er per i dag et rent leseverktøy. Solvoll-artiklens ånd tilsier at informasjonssystemer bør støtte begge retninger.

**Fremtidige muligheter (i tråd med Bares filosofi):**
- Enkel kopiering/eksport av side til Markdown-fil
- Annotasjoner lagret lokalt knyttet til URL
- PDF-eksport av ren Markdown-rendering

Viktig: Eventuelle "bidrag"-funksjoner bør **ikke** innebære dataoverføring til eksterne tjenester uten eksplisitt brukersamtykke. Bare er ikke en kommunikasjonsplattform.

---

## Oppsummering

| Tema fra Solvoll (1993) | Status i Bare (2025) | Potensial |
|-------------------------|----------------------|-----------|
| Enkelhet slår kompleksitet | ✅ Markdown > HTML | Fortsett kursen |
| Innhold skilt fra presentasjon | ✅ Brukers preferanser styrer | Styrk semantisk bevaring |
| Babels tårn — proprietære formater | ✅ Løst via konvertering | Bedre format-forhandling |
| Distribuert informasjonstilgang | ✅ HTTP + Gemini + Gopher | Eksplisitte Accept-headere |
| Toveis informasjonsutveksling | ❌ Kun lesing | Lokal eksport/annotasjon |
| Kontekst og metadata | ⚠️ Delvis | Kildeinfo i UI |

Solvoll og kollegenes 1993-innsikt var at det enkleste standardiserte formatet som er «godt nok» alltid vinner over det funksjonelt rikeste. HTML vant over ODA. Tretti år senere observerer Bare at Markdown — enda enklere, enda mer menneskelig lesbart — er det naturlige neste steget for de som vil tilbake til internettets opprinnelige løfte: **tilgang til informasjon, uten støy**.

---

*Dokument opprettet: mai 2026*  
*Relaterte dokumenter: [PLAN.md](PLAN.md), [GOPHER.md](GOPHER.md)*
