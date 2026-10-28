# Integrations

QRY calculates typing metrics once and publishes an optional local JSON
snapshot. Companion surfaces display those prepared values without collecting
input or opening QRY's database.

These guides describe the `0.2.0-2` development source on `release/next`.
Check the installed version before expecting bridge or capsule support.

| Integration           | Status                                                                                       | Guide                                           |
| --------------------- | -------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| QRY Capsule           | QRY-owned macOS 14+ helper, included by the macOS release packager.                          | [Build, enable and use the capsule](capsule.md) |
| Vorssaint             | Integration design for a separately maintained adapter; no upstream adapter is shipped here. | [Integrate QRY into Vorssaint](vorssaint.md)    |
| Other local consumers | Read the same versioned aggregate snapshot.                                                  | [Bridge protocol v1](bridge-protocol-v1.md)     |

## Enable the producer

In **QRY Settings → Appearance → Desktop integrations**:

- **Right-hand capsule** enables QRY's bundled capsule.
- **Share local aggregate status** permits external adapters to display local
  WPM, all-time speed records and up to 90 days of daily aggregates.
- **Pip in the notch** permits an external adapter to display Pip; it also
  requires external sharing. It does not install a Vorssaint module.

All three preferences default to off. The desktop Pip overlay and menu-bar WPM
have independent controls. Integrations cannot grant Input Monitoring permission.

The macOS snapshot is stored at:

```text
~/Library/Caches/app.typepulse.desktop/integrations/state-v1.json
```

There is no server, account, telemetry or network transfer. The same local
account can read the file while enabled; it is not an authentication boundary
against other processes running as that account. See the [privacy model](../privacy.md).

## Source ownership

This repository ships QRY's producer, native capsule and synthetic protocol
fixture. Keep external application code in a separate checkout or fork. Link
the tested QRY commit from the adapter's documentation and honor each project's
license when contributing code.
