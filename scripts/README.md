# Scripts

Run these commands from the repository root. Public output is generated under
`release/`; native helper output is under `build/macos-capsule/`. Both are ignored.

## Quality and privacy

| Command                                                   | Purpose                                                                                                                        |
| --------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `./scripts/check.sh`                                      | Frontend formatting/lint/build, Rust formatting/Clippy/tests/check, privacy gate, and native capsule build/self-test on macOS. |
| `./scripts/audit-privacy.sh`                              | Runtime logging, Tauri capabilities, aggregate schema and DTO checks.                                                          |
| `node scripts/audit-privacy.mjs`                          | Cross-platform static privacy audit.                                                                                           |
| `node scripts/audit-release-content.mjs PATH...`          | Reject databases, exports, environment files and logs in selected package content.                                             |
| `node scripts/audit-release-assets.mjs DIRECTORY VERSION` | Verify the exact release asset set and every SHA-256 file.                                                                     |
| `./scripts/release-audit.sh vX.Y.Z`                       | Validate tag/version/changelog consistency and installer policy. Numeric prereleases such as `v0.2.0-2` are supported.         |
| `./scripts/sample-resources.sh PID [SAMPLES] [INTERVAL]`  | Capture aggregate CPU/RSS without observing input.                                                                             |

## Build and package

| Command                                                             | Purpose                                                                                                                                                         |
| ------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `./scripts/build-capsule.sh [arm64\|x86_64]`                        | Compile and ad-hoc sign QRY Capsule; run decoder self-tests on matching hardware.                                                                               |
| `./scripts/embed-capsule.sh QRY_APP_BUNDLE [ARCH]`                  | Build and embed the helper. Re-sign the parent app afterward.                                                                                                   |
| `./scripts/package-macos.sh TARGET`                                 | Build QRY and its embedded capsule, re-sign, verify architecture/content, and create ZIP/SHA-256 files. Targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`. |
| `./scripts/package-windows.ps1 -Target x86_64-pc-windows-msvc`      | Verify existing NSIS/MSI outputs, normalize public names and create SHA-256 files.                                                                              |
| `./scripts/render-homebrew-cask.sh OWNER VERSION ARM_SHA INTEL_SHA` | Render the public cask template with validated checksums.                                                                                                       |

Use the [release process](../docs/release-process.md) for publication and the
[capsule guide](../docs/integrations/capsule.md) for debug builds. Public artifacts
must come from a verified release tag/workflow. A local build or ordinary branch
push does not publish a release. These scripts do not install or modify Vorssaint.
