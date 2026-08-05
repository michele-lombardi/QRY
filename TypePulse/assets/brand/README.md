# Canonical TypePulse brand assets

These SVG files implement the geometry and colors defined by
`../../../brand identity/TypePulse Brand Identity.dc.html` at repository level.

- `typepulse-mark.svg`: four-beat Pulse mark for normal-size use;
- `app-icon.svg`: dark squircle and cyan mark used to generate application icons;
- `tray-active.svg`: three-beat monochrome small-size exception;
- `tray-idle.svg`: flatline state for the menu bar.

The mark must not be skewed, outlined with a thin/flat stroke or filled with a gradient.
Pip is a product character and must not replace or enter the mark.

Regenerate platform icon files from the repository root with:

```bash
cd TypePulse
npm run tauri -- icon assets/brand/app-icon.svg
```

Tray PNGs are rendered from their SVG sources at 2× resolution and committed because
Tauri loads them at compile time.
