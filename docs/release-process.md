# Desktop release process

This guide is for maintainers publishing a QRY release. Public roadmap items
are not release approval: every artifact must pass automated checks and the
real-device release checklist.

## Versioning

QRY follows Semantic Versioning. Tags use `vMAJOR.MINOR.PATCH`. Because the
Windows MSI version field accepts only one numeric prerelease identifier, beta
and release-candidate tags use a number such as `v1.0.0-1`. Use the GitHub
Release title and notes to identify the build as Beta 1 or RC 1.

Before tagging, update the same version in:

- `QRY/package.json` and its lockfile;
- `QRY/src-tauri/tauri.conf.json`;
- `[workspace.package].version` in `QRY/Cargo.toml`;
- a dated section in `CHANGELOG.md`.

Validate the metadata:

```bash
./scripts/release-audit.sh v0.1.1
```

## Pre-release gate

1. Review the release diff and changelog.
2. Run `./scripts/check.sh` from the repository root.
3. Complete the relevant procedures in `QRY/tests/manual/` on clean macOS and
   Windows accounts without recording raw input logs.
4. Confirm permissions/capabilities, native input, clean relaunch, login launch,
   sleep/wake, tray behavior and multi-monitor placement on both platforms.
5. Confirm the unsigned installation flow in [installation.md](installation.md).

## Local macOS packaging dry run

On compatible macOS hardware:

```bash
./scripts/package-macos.sh aarch64-apple-darwin
./scripts/package-macos.sh x86_64-apple-darwin
```

Each command builds an ad-hoc-signed `.app`, verifies the bundle, creates an
architecture-specific ZIP, and writes a matching SHA-256 file under the ignored
`release/` directory. Cross-architecture builds still need compatible SDK and
toolchain support.

## GitHub release pipeline

Pushing a `v*` tag starts `.github/workflows/release.yml`:

1. validate version metadata, source quality, and privacy boundaries;
2. build `aarch64-apple-darwin` and `x86_64-apple-darwin` separately;
3. build Windows x64 NSIS and MSI installers on `windows-latest`;
4. verify bundle metadata, architecture, installer policy and forbidden files;
5. normalize all public names and generate SHA-256 files;
6. independently audit the exact eight-file asset set;
7. create one draft GitHub Release containing every platform artifact.

The workflow deliberately stops at a draft. A maintainer must inspect and test
the downloaded artifacts on clean systems before publication. A normal SemVer
tag creates a stable draft; only a tag with a prerelease suffix marks the draft
as a prerelease.

## Windows installers

Windows configuration is isolated in
`QRY/src-tauri/tauri.windows.conf.json`. It builds both:

- `QRY_<version>_x64-setup.exe` using NSIS, current-user installation and the
  WebView2 download bootstrapper;
- `QRY_<version>_x64_en-US.msi` using WiX on the Windows runner.

The current-user NSIS mode avoids an unnecessary administrator prompt. The
download bootstrapper keeps the artifact small and is appropriate for the
supported Windows 10/11 baseline.

For a Windows packaging dry run, first build in `QRY/` on Windows, then package
from the repository root:

```powershell
npm run tauri -- build --target x86_64-pc-windows-msvc --bundles "nsis,msi"
./scripts/package-windows.ps1 -Target x86_64-pc-windows-msvc
```

Set `QRY_REQUIRE_WINDOWS_SIGNATURE=1` only when an approved Authenticode
certificate has been installed securely on the build machine. The packaging
step will then reject unsigned or invalid installers. PFX files and passwords
must never enter the repository. Until signing is available, the release stays
draft and the unsigned-build warning is documented for testers.

## Homebrew cask

The public [`homebrew-qry`](https://github.com/michele-lombardi/homebrew-qry)
tap checks GitHub's latest stable release endpoint every hour. Once a release is
public and is neither a draft nor a prerelease, its workflow validates the two
architecture assets and checksum files, updates `Casks/qry.rb`, runs Homebrew
style/audit/livecheck, and commits the change automatically.

The repository template remains a manual fallback. Render it with the real
checksums when diagnosing or recovering the tap automation:

```bash
./scripts/render-homebrew-cask.sh \
  michele-lombardi \
  0.1.1 \
  ARM64_SHA256 \
  X86_64_SHA256
```

Copy the generated `release/qry.rb` to `Casks/qry.rb` in the tap, then validate
it:

```bash
ruby -c release/qry.rb
brew style --cask michele-lombardi/qry/qry
brew audit --cask --strict michele-lombardi/qry/qry
```

Test the user flow on a clean account:

```bash
brew install --cask michele-lombardi/qry/qry
brew upgrade --cask qry
brew uninstall --cask qry
```

The tap intentionally ignores prereleases. A maintainer can run its
`update-cask.yml` workflow manually to avoid waiting for the hourly schedule.

## Publication checklist

- the draft contains both macOS architectures, both Windows installers and all
  four matching checksums;
- downloaded ZIPs pass `shasum -a 256 -c` and open through the documented
  Gatekeeper flow;
- versions, tag, changelog, bundle metadata, and cask agree;
- installation, upgrade, and removal do not lose unexpected user data;
- the Windows NSIS/MSI install and uninstall checklist is signed off on a clean
  Windows 11 x64 VM;
- the privacy audit and all automated tests pass;
- no critical privacy, data-loss, or resource-usage issue remains open;
- prerelease/stable status and known limitations are accurate.

Developer ID signing and Apple notarization can replace the manual Gatekeeper
flow later without changing the open-source license or privacy architecture.
