# Bare - Ikon-instruksjoner

Ikonene for Bare-appen må genereres i flere formater og størrelser.

## Nødvendige filer

For Tauri 2.0 trenger du følgende ikonfiler:

- `32x32.png` - 32x32 piksler
- `128x128.png` - 128x128 piksler
- `128x128@2x.png` - 256x256 piksler (Retina)
- `icon.icns` - macOS app-ikon
- `icon.ico` - Windows app-ikon

## Generere ikoner

Du kan bruke Tauri CLI til å generere alle nødvendige formater fra en SVG eller høyoppløselig PNG:

```bash
# Fra prosjektets rot:
cargo tauri icon src-tauri/icons/icon.svg
```

Eller bruk et verktøy som:
- [Real Favicon Generator](https://realfavicongenerator.net/)
- [Icon Kitchen](https://icon.kitchen/)
- [ImageMagick](https://imagemagick.org/)

## Design-retningslinjer

Bare-ikonet skal:
- Være enkelt og minimalistisk
- Fungere i små størrelser
- Ha god kontrast i både lys og mørk modus
- Representere konseptet "bare innhold"
