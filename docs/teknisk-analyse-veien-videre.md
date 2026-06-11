# Teknisk analyse: Veien mot den perfekte Bare-nettleseren

> Arkitektur- og UX-gjennomgang av Bare v0.1.5
> Utarbeidet som grunnlag for videre utvikling. Sist oppdatert: juni 2026.

---

## 1. Sammendrag

Bare er allerede et solid verktøy for å lese Gemini og markdown-filer. Analysen identifiserer
fire hovedområder for forbedring: **Innholdskvalitet**, **Ytelse**, **Sikkerhet** og
**Internasjonalisering**. Ved å adressere disse i planlagte bølger, kan vi løfte Bare fra en
fungerende prototype til en moden, sikker tekst-nettleser.

---

## 2. Status Quo (v0.1.5)

### Styrker
- **Protokoll-bredde:** Støtter HTTPS, Gemini og Gopher ut av boksen.
- **Enkelhet:** Rent og fokusert grensesnitt uten unødvendig støy.
- **Konvertering:** Har allerede mekanismer for å gjøre HTML om til markdown.

### Kritiske svakheter (Wave 1 fokus)
- **Sikkerhetslekkasje:** CSP tillater alle eksterne bilder, noe som muliggjør sporing via tracking-pixels.
- **Ytelse:** Mangler cache for rendrede sider; navigasjon føles "seig" på grunn av re-parsing.
- **Ressurskontroll:** Ingen begrensning på HTTP-responsstørrelse (Gemini/Gopher har 5MB-tak).

---

## 5. Implementasjonsplan

### Bølge 1: Fundamentet & Sikkerhet ✅
1. **Rens opp i `converter.rs`**: Integrer `dom_smoothie` fullt ut og fjern `html2md`-artefakter (hvor mulig) for renere markdown. ✅
2. **Bildepolicy (Backend/Frontend)**: ✅
   - Implementer streng CSP (`img-src 'self' data:`) i `tauri.conf.json`.
   - Legg til `ImageMode` enum (Block, Placeholder, Show) i `Settings`.
3. **Render-cache (LRU)**: ✅
   - Implementer en enkel in-memory LRU-cache i backend for rendrede sider (maks 50 sider). Bruker `lru` crate.
4. **Grenser for HTTP-responser**: ✅
   - Speil Gemini/Gopher-grensen på 5MB for alle HTTP-nedlastinger i `fetcher.rs` ved bruk av streaming-bytes.
5. **Gjenopprett Readability**: ✅
   - Gjenopprett `readability_enabled`-bryteren i backend (`converter.rs`) ved bruk av `dom_smoothie::Readability`.

### Bølge 2: Multimedia & UX
6. **Bildevisning (Lazy-loading)**: Implementer kontrollert innlasting av bilder som respekterer `ImageMode`.
7. **Native Dialoger**: Erstatt `confirm()` med en innebygd Brutalist-style modal.
8. **Bedre statuslinje**: Flytt statusmeldinger til i18n-systemet slik at de følger valgt språk.

---

## 7. Konklusjon

Bare v0.1.5 er et solid fundament. Ved å fokusere på Wave 1 har vi nå sikret:
- En mer robust HTML-til-markdown konvertering.
- Bedre ytelse via LRU-cache.
- Strengere sikkerhet og ressurskontroll (5MB grense, bilde-policy).
- Re-aktivert readability-modus.

---
*Oppdatert: juni 2026.*
