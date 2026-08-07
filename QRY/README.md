# QRY application workspace

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
- `typepulse-platform-desktop`: target-gated permissions and input adapter boundary;
- `typepulse-storage-sqlite`: local persistence boundary;
- `src-tauri`: desktop composition root.

The Phase D workspace includes the portable WPM/session engine, SQLite aggregate
persistence, local-day rollover, CSV output and the macOS automatic-start preference.
The macOS shell launches as a menu-bar accessory without a Dock icon. A left click opens
the compact QRY daily panel; a right click exposes today, complete statistics, settings,
start, pause, WPM-number visibility and quit actions. The optional number uses a stable
three-digit slot, so the icon does not shift as WPM changes. A separate transparent,
click-through overlay renders a responsive live estimate and identity-aligned Pip
behaviors with persisted visual options. With optional macOS Accessibility consent it
follows the display containing the focused window while exposing only temporary
geometry. Transient read failures preserve the current valid display; a cold start falls
back to the primary display. Pip appears from the third accepted activity and breathes
for the configured quiet interval before disappearing. Live WPM ramps during its first
second, while peak statistics accept samples only after three seconds; separate all-time
records use complete 30-second and 60-second windows. The Pulse mark, app icon, dynamic
tray glyph, palette and product voice follow the supplied brand identity. Phase F now
includes Apple-style settings and daily, weekly, monthly and yearly aggregate
statistics. Permission onboarding is implemented; the native save destination for CSV
remains an open task.

See [`../docs/development.md`](../docs/development.md) for the complete guide.

Release packaging and quality automation are documented in
[`../docs/release-process.md`](../docs/release-process.md). No public release is created
from an ordinary branch build; only a version tag can start the draft release workflow.

## License

QRY is licensed under `GPL-3.0-only`. Copyright © 2026 Michele Lombardi; details are
available in [`../NOTICE.md`](../NOTICE.md).
