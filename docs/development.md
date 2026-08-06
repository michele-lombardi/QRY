# Development guide

QRY is a Tauri 2 application with a Rust workspace and a small TypeScript
frontend. macOS is required to run the complete desktop application and its
platform adapter; most core and storage tests remain portable.

## Requirements

- macOS with Xcode Command Line Tools or a compatible Apple SDK;
- Rust stable with `rustfmt` and Clippy;
- Node.js 24;
- npm.

Check the toolchain:

```bash
xcode-select -p
rustc --version
cargo --version
node --version
npm --version
```

## Set up the repository

```bash
git clone https://github.com/michele-lombardi/QRY.git
cd QRY/QRY
npm ci
```

The Tauri CLI is installed as a project dependency; no global installation is
required. Cargo downloads Rust dependencies during the first build.

## Run the app

From the application directory:

```bash
npm run tauri dev
```

Tauri starts Vite on port 1420, builds the Rust workspace, and launches QRY.
Development builds still need explicit Input Monitoring permission. Because an
unsigned executable's identity may change after rebuilding, macOS can request
permission again.

QRY is a menu-bar accessory after onboarding. Left-click the Pulse for Today;
right-click it for Statistics, Settings, monitoring controls, menu-bar WPM, and
Quit. Closing a window hides it without terminating the background process.

## Quality checks

Run the complete gate from the repository root:

```bash
./scripts/check.sh
```

It verifies frontend formatting, ESLint, TypeScript, the production Vite build,
Rustfmt, Clippy, Rust tests, Tauri capabilities, SQLite schema boundaries, and
privacy-sensitive DTOs.

Individual commands, run from `QRY/` unless noted:

```bash
npm run format:check
npm run lint
npm run build
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo check --workspace --all-targets
```

Use `npm run format` and `cargo fmt --all` to apply formatting, then review the
diff before committing.

## Repository layout

```text
QRY/
├── crates/
│   ├── typepulse-core/
│   ├── typepulse-platform-macos/
│   └── typepulse-storage-sqlite/
├── src/                    # TypeScript, HTML, and CSS presentation
├── src-tauri/              # Tauri composition and desktop lifecycle
├── tests/manual/           # real-macOS checks that CI cannot grant
└── package.json
```

Read [architecture.md](architecture.md) before changing dependency boundaries
and [decisions/README.md](decisions/README.md) before revisiting an accepted
system choice.

## Engineering rules

- `typepulse-core` stays independent of Tauri, SQLite, and operating-system APIs.
- Platform adapters may emit only privacy-safe activity timestamps.
- The storage adapter receives aggregate domain models, never keyboard events.
- The frontend displays prepared DTOs and does not calculate authoritative WPM.
- Core tests use an injected manual clock instead of real sleeps.
- Event-tap callbacks must remain bounded, non-blocking, and free of I/O.
- No log may contain keys, text, active applications, window titles, or URLs.

The trusted application windows currently receive only `core:default` Tauri
capabilities. New permissions require a concrete feature, the narrowest
available scope, and a documented privacy review.

## Tests requiring a real Mac

Automated checks cannot grant TCC consent or exercise logout/login reliably.
Reproducible manual procedures live in [`QRY/tests/manual/`](../QRY/tests/manual/)
and cover Input Monitoring, Secure Input, callback performance, menu-bar
behavior, startup, overlay focus/click-through, multi-monitor placement, and
release installation.

Never attach raw keyboard-event logs to a test result. The callback reference
benchmark intentionally reports aggregate timing only:

```bash
cargo test -p typepulse-platform-macos --release \
  typing_callback_hot_path_reference -- --ignored --nocapture
```

## Data and generated files

Committed lockfiles, migrations, Tauri configuration, and capability files are
part of reproducible builds. Generated dependencies, build outputs, local
databases, exports, and release artifacts are ignored by Git.

The development database is `typepulse.sqlite3` under the Tauri application-data
directory. An older non-empty schema may produce a sibling
`typepulse.sqlite3.pre-migration-vN-TIMESTAMP.bak` before migration.

## Troubleshooting

### Rust is not found

Confirm `cargo` is on `PATH` and restart the terminal or editor after installing
Rust.

### Port 1420 is already in use

Stop the older Vite or Tauri process. The fixed port prevents the desktop runtime
from connecting silently to an unrelated server.

### macOS build tools are missing

Install Xcode Command Line Tools and confirm `xcode-select -p` returns an active
developer directory.

### macOS keeps asking for Input Monitoring

Local unsigned rebuilds can acquire a new identity. Remove only the stale QRY
entry in **System Settings → Privacy & Security → Input Monitoring**, launch the
exact build being tested, grant access again, and let QRY perform its clean
restart. Never edit the TCC database.

### Today's totals remain unchanged while typing

Daily persisted totals include completed sessions. Stop monitoring or wait for
the session timeout; live WPM and the current-session estimate remain visible.

### Start at login is enabled but QRY does not open

Confirm QRY is present in macOS Login Items and still has Input Monitoring.
Startup cannot bypass consent; invalid permission causes QRY to remove its stale
LaunchAgent and return to onboarding.

## Preparing a release

Release helpers and generated artifacts are documented in
[release-process.md](release-process.md) and [`scripts/README.md`](../scripts/README.md).
Outputs are written under the ignored `release/` directory.
