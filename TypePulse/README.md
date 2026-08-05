# TypePulse application workspace

This directory contains the Tauri 2 desktop application, vanilla TypeScript frontend and
Rust workspace.

## Start locally

```bash
npm install
npm run tauri dev
```

## Validate

From the repository root:

```bash
./scripts/check.sh
```

## Crates

- `typepulse-core`: portable domain boundary;
- `typepulse-platform-macos`: permissions and input adapter boundary;
- `typepulse-storage-sqlite`: local persistence boundary;
- `src-tauri`: desktop composition root.

The Phase D workspace includes the portable WPM/session engine, SQLite aggregate
persistence, local-day rollover, CSV output and the macOS automatic-start preference.
The Phase E shell launches as a macOS menu-bar accessory without a Dock icon. Its tray
opens the temporary diagnostic window and exposes start, pause, WPM-number visibility
and quit actions. The optional number uses a stable three-digit slot, so the icon does
not shift as WPM changes. A separate transparent, click-through overlay renders a
responsive live estimate and identity-aligned Pip behaviors with persisted visual
options. With optional macOS Accessibility consent it follows the display containing the
focused window while exposing only temporary geometry; otherwise it falls back to the
primary display. The Pulse mark, app icon, dynamic tray glyph, palette and product voice
follow the supplied brand identity. Final Phase F statistics/settings screens remain in
progress.

See [`../docs/development.md`](../docs/development.md) for the complete guide.

Release packaging and quality automation are documented in
[`../docs/release-process.md`](../docs/release-process.md). No public release is created
from an ordinary branch build; only a version tag can start the draft release workflow.

## License

TypePulse is licensed under `GPL-3.0-only`. Copyright-holder and public-contact
placeholders are kept in [`../NOTICE.md`](../NOTICE.md).
