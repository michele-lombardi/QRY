# Development guide

## Current milestone

Phase E now implements the macOS menu-bar shell and a separate live overlay with
four positions, three sizes, configurable content and four animation bands. The
next milestone is completion of the Phase F screens and onboarding. The Phase B
TCC checklist and the real Phase E focus/click-through checks still require a
real user grant.

## Toolchain

The supported development environment for the first release is macOS.

Required:

- Rust stable;
- `rustfmt` and Clippy components;
- Node.js 24;
- npm;
- macOS SDK and command-line development tools.

Recommended VS Code extensions are committed in `.vscode/extensions.json`.

Check installed versions:

```bash
rustc --version
cargo --version
node --version
npm --version
```

## Install dependencies

```bash
cd TypePulse
npm install
```

The command installs the frontend dependencies and the project-local Tauri CLI.
Cargo downloads Rust dependencies on the first build. Application lockfiles are
committed to keep builds reproducible.

## Run the Phase B diagnostic app

```bash
cd TypePulse
npm run tauri dev
```

Tauri starts Vite on port 1420, compiles the Rust workspace and launches
TypePulse as a macOS menu-bar application. No window is shown automatically.
Select the TypePulse icon in the upper-right menu bar to open the diagnostic
window; right-click it for Open, Start monitoring, Pause monitoring and Quit.

Use the window to check/request Input Monitoring and the separate optional
Accessibility permission, inspect live/today metrics, change `Start automatically`
and configure the overlay. Accessibility lets the PiP follow the display of the
focused window; without it the primary-display fallback remains functional. The
startup checkbox
registers a macOS login item and starts monitoring whenever the app opens.
Closing the window only hides it; choose Quit from the tray menu to terminate
the background process cleanly.

The app cannot grant TCC permission itself. After changing Input Monitoring in
System Settings, macOS may require a quit/restart. Unsigned debug rebuilds can
also appear as a new identity. Follow `tests/manual/input-monitoring.md`; never
edit the TCC database.

## Useful commands

Run these inside `TypePulse/` unless otherwise indicated:

```bash
npm run dev            # frontend in a browser-like development server
npm run build          # TypeScript check and production frontend build
npm run lint           # ESLint
npm run format         # write frontend/config formatting
npm run format:check   # verify formatting
npm run tauri dev      # complete desktop development build

cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo check --workspace --all-targets
```

Run the complete local gate from the repository root:

```bash
./scripts/check.sh
```

The gate now ends with the Phase G privacy audit. Release-only helpers are
documented in `scripts/README.md` and `docs/release-process.md`; they create
generated outputs under the ignored `release/` directory.

## Workspace design

### `typepulse-core`

Owns portable domain logic. It may define activity timestamps, rolling WPM,
smoothing, sessions, summaries and repository traits. It cannot import Tauri,
SQLite or platform frameworks.

`TypingEngine` accepts an injected `Clock`. Use `ManualClock` in deterministic
tests and `SystemClock` in production. Do not add sleeps to core tests. Metric
semantics and default parameters are in ADR 0005.

### `typepulse-platform-macos`

Owns Input Monitoring permission, global macOS event integration and the
privacy-minimized focused-window geometry adapter. Its input boundary emits only
`TypingActivity` with a monotonic instant. Raw key information is discarded
inside the private adapter filter. The separate Accessibility boundary exposes
only a temporary global center point; it never exposes app, title or content.

### `typepulse-storage-sqlite`

Owns `rusqlite` queries, embedded migrations, pre-migration backup and local
persistence. It depends on domain models, never on the macOS adapter. Its schema
contains only completed-session aggregates, 60-second buckets and preferences.

On macOS the database is stored under the Tauri application-data directory as
`typepulse.sqlite3`. A sibling file named
`typepulse.sqlite3.pre-migration-vN-TIMESTAMP.bak` may be created before a
schema upgrade. Today is resolved from the local date on every query: midnight
starts a new empty summary automatically and keeps older dates intact.

### `src-tauri`

Composition root for the desktop runtime. It owns window/tray lifecycle, command
registration and routing between adapters and the frontend. Domain formulas do
not belong here.

`overlay.rs` creates a transparent, click-through webview window and controls
its position/visibility from aggregate snapshots. On macOS it maps the focused
window's temporary center to a display before presentation and during active
typing, then falls back to the primary display. It never observes input
identities or sends geometry to the frontend. Tauri's
`macOSPrivateApi` flag is enabled solely for transparent-window support; the
project does not target the Mac App Store.

### `src`

Vanilla TypeScript, HTML and CSS. The frontend receives prepared DTOs and emits
user intentions. It never sees key codes and never queries SQLite directly.

Brand SVG sources live under `assets/brand`; generated application/tray icons
live under `src-tauri/icons`. Follow `docs/brand-implementation.md` before
changing colors, the Pulse mark, Pip anatomy, thresholds, motion or product
copy. The original identity reference is retained in the repository-level
`brand identity/` directory.

## Dependency direction

```text
frontend
   ↓ Tauri commands/events
src-tauri ──→ platform-macos
   │               │
   ├──→ storage    │
   └───────────────┴──→ core
```

The core is at the bottom of the dependency graph. A dependency in the reverse
direction is an architectural regression.

## Tauri capabilities

The app grants the trusted main and overlay windows only `core:default`. No
opener, filesystem, shell, network or global-shortcut plugin permission is
enabled. New permissions require a concrete feature, the narrowest available
capability and documentation in the pull request.

## Tests

Rust unit tests live next to the implementation. Cross-crate fixtures and manual
macOS checklists live under `TypePulse/tests/`.

Automated tests must not require Input Monitoring permission. Clock, event source
and persistence boundaries will be injectable so CI remains deterministic.

Phase D storage and rollover checks are automated. Login-item behavior requires
a real macOS login session; follow `tests/manual/phase-d-persistence-startup.md`.

Manual Phase B checks live in `tests/manual/`. The release hot-path reference is:

```bash
cargo test -p typepulse-platform-macos --release \
  typing_callback_hot_path_reference -- --ignored --nocapture
```

The menu-bar lifecycle checklist is in
`tests/manual/menu-bar-shell.md`. It verifies macOS UI behavior that unit tests
cannot observe, including Dock and `Cmd + Tab` visibility.

The overlay focus, click-through, visual state and multi-monitor checklist is in
`tests/manual/phase-e-overlay.md`.

The focused-display permission, two-display routing, revocation and privacy
checklist is in `tests/manual/focused-display.md`.

## Logging rules

Allowed:

- lifecycle transitions without input detail;
- permission state;
- aggregate counts in explicit diagnostic builds;
- recoverable error categories.

Forbidden:

- characters or key codes;
- event dumps;
- active app and window information;
- text, passwords or clipboard data;
- per-key timestamps written to disk.

## Generated and committed files

Committed:

- `package-lock.json`;
- workspace `Cargo.lock`;
- Tauri configuration and capabilities;
- source, tests and migrations.

Ignored:

- `node_modules/`;
- `dist/`;
- Cargo `target/`;
- generated Tauri capability schemas;
- local environment and exported statistics.

## Troubleshooting

### `npm run tauri dev` cannot find Rust

Confirm that `cargo` is in the shell `PATH` and restart VS Code after installing
the toolchain.

### Port 1420 is already in use

Stop the older Vite/Tauri process. The port is fixed deliberately so the desktop
runtime does not silently connect to another development server.

### macOS build tools are missing

Install the macOS command-line development tools or the SDK required by the
Tauri build. A Linux or Windows machine cannot produce and exercise the macOS
platform adapter.

### Formatting differs in CI

Run `npm run format` and `cargo fmt --all`, review the resulting diff, then rerun
`./scripts/check.sh`.

### Today's values stay at zero while typing

The public daily summary contains completed sessions. Stop monitoring or wait
for the 30-second session timeout to flush the current session. Live WPM remains
visible while the session is active.

### Automatic startup is checked but monitoring does not start

Confirm TypePulse is present in macOS Login Items and has Input Monitoring
permission. The preference cannot grant TCC consent. Read the runtime error in
the diagnostic window, then use the Phase D manual checklist.

## TODO decisions

- fill the copyright-holder and public-contact placeholders in `NOTICE.md`;
- replace the provisional bundle identifier if the GitHub owner requires it;
- complete the Phase B TCC, revocation and Secure Input checklists;
- replace generated Tauri icons before the first release;
- add a private security contact before public contributions.
- configure the GitHub remote and Homebrew tap owner before any public release;
- complete Gate F and the Phase G manual quality checklist before tagging V1.
