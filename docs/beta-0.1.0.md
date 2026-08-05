# QRY 0.1.0 beta

QRY 0.1.0 is the first public macOS beta. It is an invisible menu-bar app that
shows typing rhythm and estimated WPM without recording what the user writes.

## Included

- live WPM in the menu bar and optional desktop Pip;
- focused-display Pip placement with optional Accessibility permission;
- daily panel, configurable Settings and detailed Statistics;
- separate WPM and estimated-word charts with numeric Y axes and time on X;
- local aggregate history, daily rollover, streaks, records and CSV export;
- automatic monitoring and optional launch at login;
- Light/Dark adaptive assets and transparent-background Pip mode.

## Privacy

QRY processes keyboard activity transiently. It does not persist or log keys,
key codes, words, passwords, application names, window titles or visited sites.
SQLite stores only aggregate metric buckets, session summaries and preferences.
All data remains on the local Mac unless the user explicitly exports a CSV.

## Installation status

The beta is open source and uses ad-hoc signing, not Apple notarization. Download
the ZIP for the correct architecture from GitHub Releases, extract `QRY.app`,
move it to Applications and follow [`gatekeeper.md`](gatekeeper.md) if macOS
blocks the first launch. Input Monitoring is required for global counting;
Accessibility is optional and is used only to place Pip on the focused display.

## Before promoting the draft

- verify both Apple Silicon and Intel archives and their SHA-256 checksums;
- complete the Input Monitoring, Accessibility and Gatekeeper smoke tests;
- verify login launch, sleep/wake and multi-monitor behavior;
- keep the GitHub Release marked as a prerelease;
- do not attach logs containing input data.

## Known limitations

- no Developer ID signature or notarization;
- onboarding and native CSV destination dialog are not complete;
- VoiceOver and final accessibility validation remain open;
- Linux and Windows builds are not included.
