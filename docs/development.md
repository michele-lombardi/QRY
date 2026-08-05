# Development guide

## Current milestone

Phase A establishes a reproducible Tauri/Rust/TypeScript project. It does not
monitor keyboard activity or implement product features. The next milestone is
the limited macOS input-monitoring spike described as Phase B in the working
plan.

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

## Run the foundation app

```bash
cd TypePulse
npm run tauri dev
```

Tauri starts Vite on port 1420, compiles the Rust workspace and opens the
foundation window. That window is intentionally temporary: tray-only behavior
is implemented in Phase E after the monitor and core are proven.

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

## Workspace design

### `typepulse-core`

Owns portable domain logic. It may define activity timestamps, rolling WPM,
smoothing, sessions, summaries and repository traits. It cannot import Tauri,
SQLite or platform frameworks.

### `typepulse-platform-macos`

Owns Input Monitoring permission and global macOS event integration. Its public
boundary will emit only an activity timestamp. Raw key information must be
discarded inside the adapter.

### `typepulse-storage-sqlite`

Owns migrations and local persistence. It depends on domain models, never on the
macOS adapter. Phase A wires the crate only; SQLite dependencies arrive with the
database design in Phase D.

### `src-tauri`

Composition root for the desktop runtime. It owns window/tray lifecycle, command
registration and routing between adapters and the frontend. Domain formulas do
not belong here.

### `src`

Vanilla TypeScript, HTML and CSS. The frontend receives prepared DTOs and emits
user intentions. It never sees key codes and never queries SQLite directly.

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

Phase A grants the main window only `core:default`. No opener, filesystem, shell,
network or global-shortcut plugin permission is enabled. New permissions require
a concrete feature, the narrowest available capability and documentation in the
pull request.

## Tests

Rust unit tests live next to the implementation. Cross-crate fixtures and manual
macOS checklists live under `TypePulse/tests/`.

Automated tests must not require Input Monitoring permission. Clock, event source
and persistence boundaries will be injectable so CI remains deterministic.

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

## TODO decisions

- choose the actual open-source license;
- replace the provisional bundle identifier if the GitHub owner requires it;
- choose the minimum macOS version during the Phase B API spike;
- replace generated Tauri icons before the first release;
- add a private security contact before public contributions.
