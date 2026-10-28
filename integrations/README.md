# QRY integrations

This directory contains QRY-owned native components. They consume prepared
aggregate metrics; they never observe keyboard input or read the SQLite database.

| Directory                | Contents                                                                                    |
| ------------------------ | ------------------------------------------------------------------------------------------- |
| `native/`                | QRY's bounded JSON reader and SwiftUI Pip renderer.                                         |
| `macos-capsule/Sources/` | Non-activating AppKit capsule, with one panel per display.                                  |
| `.local/`                | Ignored personal experiments, external checkouts and backups. Never package this directory. |

Build output goes to `build/macos-capsule/<architecture>/`, which is also ignored.
The release packager embeds the capsule inside `QRY.app`.

See the [integration guides](../docs/integrations/README.md) for the capsule,
the versioned JSON contract and integration with
[`vorssaint/vorssaint-utils`](https://github.com/vorssaint/vorssaint-utils).
Vorssaint source, patches, app bundles and installers are maintained separately
and are not distributed in this repository.
