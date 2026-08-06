# macOS release process

This guide is for maintainers publishing a QRY release. Public roadmap items
are not release approval: every artifact must pass automated checks and the
real-device release checklist.

## Versioning

QRY follows Semantic Versioning. Tags use `vMAJOR.MINOR.PATCH` or a SemVer
prerelease suffix such as `v1.0.0-rc.1`.

Before tagging, update the same version in:

- `QRY/package.json` and its lockfile;
- `QRY/src-tauri/tauri.conf.json`;
- `[workspace.package].version` in `QRY/Cargo.toml`;
- a dated section in `CHANGELOG.md`.

Validate the metadata:

```bash
./scripts/release-audit.sh v0.1.0
```

## Pre-release gate

1. Review the release diff and changelog.
2. Run `./scripts/check.sh` from the repository root.
3. Complete the relevant procedures in `QRY/tests/manual/` on a clean macOS
   account without recording raw input logs.
4. Confirm Input Monitoring, optional Accessibility, permission revocation,
   clean relaunch, login launch, sleep/wake, and multi-monitor behavior.
5. Confirm the unsigned installation flow in [installation.md](installation.md).

## Local packaging dry run

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

Pushing a `v*` tag starts `.github/workflows/release-macos.yml`:

1. validate version metadata, source quality, and privacy boundaries;
2. build `aarch64-apple-darwin` and `x86_64-apple-darwin` separately;
3. verify ad-hoc signing and bundle architecture;
4. generate immutable ZIPs and SHA-256 files;
5. create a draft GitHub prerelease containing all assets.

The workflow deliberately stops at a draft. A maintainer must inspect and test
the downloaded artifacts before publication.

## Homebrew cask

After the GitHub release is public, render the cask with its real checksums:

```bash
./scripts/render-homebrew-cask.sh \
  michele-lombardi \
  0.1.0 \
  ARM64_SHA256 \
  X86_64_SHA256
```

Copy the generated `release/qry.rb` to `Casks/qry.rb` in the separate
`homebrew-qry` tap, then validate it:

```bash
ruby -c release/qry.rb
brew style --cask michele-lombardi/qry/qry
brew audit --cask --strict michele-lombardi/qry/qry
```

Test the user flow on a clean account:

```bash
brew tap michele-lombardi/qry
brew install --cask qry
brew upgrade --cask qry
brew uninstall --cask qry
```

The tap is external to this repository and must be published before the README
installation command is described as available.

## Publication checklist

- the draft contains both architectures and matching checksums;
- downloaded ZIPs pass `shasum -a 256 -c` and open through the documented
  Gatekeeper flow;
- versions, tag, changelog, bundle metadata, and cask agree;
- installation, upgrade, and removal do not lose unexpected user data;
- the privacy audit and all automated tests pass;
- no critical privacy, data-loss, or resource-usage issue remains open;
- prerelease/stable status and known limitations are accurate.

Developer ID signing and Apple notarization can replace the manual Gatekeeper
flow later without changing the open-source license or privacy architecture.
