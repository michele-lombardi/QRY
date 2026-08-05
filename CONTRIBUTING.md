# Contributing to TypePulse

Thanks for helping build TypePulse. The project is in its foundation stage and
targets macOS first, with a portable Rust core for future Linux support.

## License

TypePulse is licensed under `GPL-3.0-only`. By contributing, you agree that your
contribution is distributed under the same license. Copyright-holder and
public-contact placeholders are tracked in `NOTICE.md`.

## Prerequisites

- macOS for the desktop application and platform adapter;
- Rust stable with `rustfmt` and `clippy`;
- Node.js 24 and npm;
- macOS development SDK/toolchain;
- VS Code is recommended but not required.

See [`docs/development.md`](docs/development.md) for setup and architecture.

## Local setup

```bash
cd TypePulse
npm install
npm run tauri dev
```

The Tauri CLI is a local npm development dependency. A global installation is
not required.

## Required checks

From the repository root:

```bash
./scripts/check.sh
```

The script checks frontend formatting, TypeScript, ESLint, the Vite build,
Rust formatting, Clippy and all workspace tests.

## Architecture rules

- `typepulse-core` must remain independent of Tauri and operating-system APIs.
- platform crates may depend on the core; the core must not depend on them.
- the SQLite adapter must never receive raw keyboard events.
- frontend code displays state and invokes commands; it does not calculate WPM.
- no layer may log individual keys, key codes, text, active apps or window titles.

Any pull request touching input or persistence must explain its privacy impact.

## Workflow

1. Open or choose an issue.
2. Create a short branch such as `feat/CORE-02-rolling-window`.
3. Keep the change focused on one acceptance criterion.
4. Add or update tests.
5. Run `./scripts/check.sh`.
6. Open a pull request using the provided template.

## Commit messages

Use a short imperative subject. Prefixes such as `feat:`, `fix:`, `docs:`,
`test:` and `chore:` are encouraged but not required.

Examples:

```text
feat: add portable typing activity type
test: cover session inactivity timeout
docs: record macOS event tap decision
```

## Definition of done

A contribution is complete when checks pass, acceptance criteria are verified,
privacy constraints remain true and the relevant documentation is updated.
The full definition is in [`docs/working-plan.md`](docs/working-plan.md).
