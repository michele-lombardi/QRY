# Scripts

Repeatable automation used by local development and GitHub Actions.

## Available commands

- `check.sh`: runs the complete frontend, Rust, and privacy gate;
- `audit-privacy.sh`: rejects runtime input logging and verifies Tauri
  capabilities, the aggregate-only SQLite schema, and public DTO boundaries;
- `release-audit.sh vX.Y.Z`: verifies SemVer, Cargo/npm/Tauri versions,
  changelog state, and release-workflow placeholders;
- `package-macos.sh TARGET`: builds the ad-hoc-signed app, verifies its bundle
  and architecture, then creates a ZIP and SHA-256 file for `aarch64` or
  `x86_64`;
- `render-homebrew-cask.sh OWNER VERSION ARM_SHA INTEL_SHA`: renders the cask
  template with validated checksums under `release/`;
- `sample-resources.sh PID [SAMPLES] [INTERVAL]`: captures aggregate CPU and RSS
  for idle/typing tests without observing input events.

`release/` is generated and ignored by Git. Public artifacts must come from a
verified release tag and workflow, not from committed local output.
