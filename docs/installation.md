# Installing QRY on macOS

QRY supports macOS 10.15 Catalina or later. Current beta bundles are ad-hoc
signed and not Apple-notarized, so the first launch may require explicit
approval in macOS.

## Homebrew

Install QRY and add the official tap in one command:

```bash
brew install --cask michele-lombardi/qry/qry
```

Or add the tap explicitly:

```bash
brew tap michele-lombardi/qry
brew install --cask qry
```

The [`homebrew-qry`](https://github.com/michele-lombardi/homebrew-qry) tap checks
the latest public, non-prerelease GitHub Release every hour and automatically
updates the version and architecture-specific checksums. Drafts and prereleases
are intentionally ignored.

Once installed:

```bash
brew upgrade --cask qry
brew uninstall --cask qry
```

Homebrew cannot grant Input Monitoring or Accessibility permission. Those
choices always remain in macOS System Settings.

## Manual installation of the unsigned beta

1. Download the archive for your Mac from the official
   [GitHub Releases](https://github.com/michele-lombardi/QRY/releases) page:
   `aarch64` for Apple Silicon or `x86_64` for Intel.
2. Download the matching `.sha256` file into the same directory.
3. In Terminal, change to that directory and verify the archive. For example:

    ```bash
    shasum -a 256 -c QRY-0.1.1-aarch64.app.zip.sha256
    ```

    Continue only if the command reports `OK`.

4. Extract the ZIP and move `QRY.app` to `/Applications`.
5. Open QRY normally from Applications.

### If Gatekeeper blocks the first launch

1. Keep the QRY warning visible or close it.
2. Open **System Settings → Privacy & Security**.
3. Confirm that the blocked application is QRY, then select **Open Anyway**.
4. Confirm the launch once more when macOS asks.

Do not disable Gatekeeper globally. Do not run recursive quarantine-removal
commands against `/Applications`, your home directory, or another broad path.
Ad-hoc signing verifies bundle consistency but does not authenticate the author;
the official release URL and SHA-256 file are therefore part of the installation
check.

## Complete first-run setup

Gatekeeper approval and privacy permissions are separate:

1. QRY explains its local, content-free data model.
2. Grant **Input Monitoring** when macOS asks. It is required for global typing
   activity; QRY closes if it is not granted.
3. Choose whether to grant **Accessibility**. It is optional and is used only to
   place Pip on the display containing the focused window.
4. Choose whether QRY should start at login. The option is off by default.
5. QRY performs a clean restart after required permission becomes available.

If permission was revoked later, QRY stops monitoring, hides Pip, removes an
invalid login item, and returns to the permission flow.

## Build from source

For development builds and local compilation, follow the
[development guide](development.md). Rebuilding an unsigned executable can make
macOS treat it as a new identity and request Input Monitoring again.
