# QRY application workspace

This directory contains the Tauri 2 desktop application, vanilla TypeScript frontend and
Rust workspace. See the [project README](../README.md) for product features and
installation.

## Run locally

```bash
npm ci
npm run tauri dev
```

Use macOS with Xcode Command Line Tools, or Windows with the MSVC toolchain and Windows
SDK. Both require Rust stable, Node.js 24 and npm. macOS input monitoring requires
explicit system consent.

## Workspace

| Path                                 | Responsibility                                                |
| ------------------------------------ | ------------------------------------------------------------- |
| `crates/typepulse-core/`             | Portable WPM, sessions, records and persistence contracts.    |
| `crates/typepulse-platform-desktop/` | Target-gated input, permissions and focused-display geometry. |
| `crates/typepulse-storage-sqlite/`   | Aggregate-only storage and ordered migrations.                |
| `src-tauri/`                         | Desktop lifecycle, commands, tray, windows and local bridge.  |
| `src/`                               | TypeScript and CSS presentation.                              |
| `tests/manual/`                      | Native desktop and release procedures.                        |
| `assets/brand/`                      | Canonical QRY brand assets.                                   |

Run the complete quality gate from the repository root:

```bash
./scripts/check.sh
```

The native capsule lives outside this workspace in
[`integrations/`](../integrations/README.md); a plain Tauri development launch will not
embed it. Follow the [capsule guide](../docs/integrations/capsule.md) for native
previews and bundled builds.

Read the [development guide](../docs/development.md),
[architecture](../docs/architecture.md) and
[release process](../docs/release-process.md) before changing system boundaries or
producing artifacts.

QRY is licensed under [GPL-3.0-only](../LICENSE). Copyright © 2026 Michele Lombardi; see
[NOTICE.md](../NOTICE.md).
