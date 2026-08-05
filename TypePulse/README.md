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

The current Phase A build displays a temporary foundation window. It does not monitor
input or persist statistics yet.

See [`../docs/development.md`](../docs/development.md) for the complete guide.
