# Integrate QRY into Vorssaint

Upstream project: **[@vorssaint/vorssaint-utils](https://github.com/vorssaint/vorssaint-utils)**.
Maintain the notch adapter in that repository or a separate fork. This repository
contains QRY's producer and protocol documentation, with no Vorssaint source,
patch, installer or app bundle.

## Compatibility and scope

QRY `0.2.0-2` development source exports the macOS bridge described in
[protocol v1](bridge-protocol-v1.md). Record the exact tested QRY and Vorssaint
commits when implementing an adapter; this guide does not imply that an official
Vorssaint release already includes it.

Vorssaint's [module catalog](https://github.com/vorssaint/vorssaint-utils/blob/main/Sources/Vorssaint/Services/Notch/NotchSupport.swift)
and [notch view](https://github.com/vorssaint/vorssaint-utils/blob/main/Sources/Vorssaint/UI/Notch/NotchView.swift)
were checked on 2026-10-28. Destinations are registered in source and gated by
feature availability. Enabling sharing in QRY does not add a destination to the
installed Vorssaint app. Implement and build the module in the separate project.

A useful first version is an optional **QRY statistics** destination with:

- today's estimated words, active typing time and daily peak;
- all-time peak, 30-second and 60-second records;
- a selectable 7/30/90-day graph of daily average and peak WPM;
- explicit actions to open QRY Today, Statistics and Settings.

Treat compact Pip animation as a separate optional extension. Respect existing
activity priorities such as timers and alerts.

## 1. Prepare a separate checkout

Read the current [Vorssaint contribution guide](https://github.com/vorssaint/vorssaint-utils/blob/main/CONTRIBUTING.md)
and discuss a substantial feature with its maintainers. Follow their toolchain,
licensing, localization and test requirements.

For a local checkout beside QRY, run from the QRY repository root:

```bash
git clone https://github.com/vorssaint/vorssaint-utils.git ../vorssaint-utils
git -C ../vorssaint-utils switch -c feat/notch-qry
```

For a contribution, use your own fork as `origin` and retain the official project
as `upstream`. Keep all adapter commits there. Build the separate Developer app
while testing so the official installation retains its own preferences and identity.

## 2. Implement an aggregate reader

Read this file in the current user's account:

```text
~/Library/Caches/app.typepulse.desktop/integrations/state-v1.json
```

Use the [protocol](bridge-protocol-v1.md) as the source of truth. A reader should:

1. Read at most 65,537 bytes and reject files larger than 65,536 bytes.
2. Require schema `1`, a supported status and valid finite, nonnegative values.
3. Reject snapshots older than 30 seconds or more than 5 seconds in the future.
4. Display shared values only when `externalEnabled` is true.
5. Handle absent `statistics` as unavailable history, rather than zero records.
6. Validate ordered calendar dates, the 90-day limit and means no greater than peaks.
7. Clear stale live values and show an unavailable state when validation fails.

Poll no faster than the producer's approximately 500 ms interval. Stop background
work when the feature is disabled. Distinguish missing, stale, invalid and
unsupported data when possible; a missing file alone does not prove QRY is
uninstalled. Never access QRY's SQLite file, monitor keys, request input permission
or silently change sharing preferences.

The [synthetic fixture](fixtures/bridge-v1.json) has a fixed timestamp. Inject its
timestamp as the decoder's test clock; do not copy it into the live user cache.
QRY's [native reader](../../integrations/native/QRYBridge.swift) illustrates the
contract, but is QRY code under `GPL-3.0-only`. Resolve licensing before reusing
source in a project that accepts contributions under different GPL terms.

## 3. Register an optional destination

Follow the current upstream architecture rather than applying an old patch:

- register the feature in the feature catalog and keep it off by default;
- gate the destination, saved ordering, shortcuts and quick actions consistently;
- include user preferences in settings backup and validate restored range values;
- localize strings in every language supported by the host;
- keep decoding and availability decisions outside SwiftUI views;
- stop subscriptions and timers when the feature is disabled.

Display prepared metrics without recalculating live WPM. Daily means are
character weighted; they are not words divided by elapsed wall-clock time.
Keep days without activity as graph gaps. All-time records retain their scope
when the graph range changes.

If adding compact Pip, require both `externalEnabled` and `notchPipEnabled`.
Use `presentation`, `animationBand` and `celebrationSequence` from QRY, handle a
new `instanceId`, and honor Reduce Motion. Do not infer animation from raw input.

## 4. Open QRY through explicit actions

Resolve bundle identifier `app.typepulse.desktop` through `NSWorkspace`, then
open the installed QRY with one of these fixed links:

```text
qry://open/today
qry://open/statistics
qry://open/settings
```

Use links only after a user action. Do not execute a path, command or URL supplied
by the snapshot. QRY routes through its single instance and permission gate;
navigation cannot enable monitoring or bypass consent.

## 5. Validate and contribute upstream

In the separate checkout, follow upstream's current instructions. At the time
of writing, the development commands are:

```bash
./build.sh --list-tests
./build.sh --dev
./build/VorssaintDeveloper --selftest
./build.sh --test
git diff --check
```

Run the optimized build and self-test required by upstream before submitting.
Test sharing off, QRY absent/stopped/paused, malformed and stale files, unsupported
schema, older snapshots without history, empty days, local midnight, each graph
range, saved settings, multiple displays, themes and Reduce Motion.

Record actual hardware, macOS version, both commit IDs and observed results.
QRY checks do not validate a Vorssaint build. Open any issue or pull request in
the [upstream project](https://github.com/vorssaint/vorssaint-utils) or your fork;
keep its versioning, signing, updater and release decisions there.
