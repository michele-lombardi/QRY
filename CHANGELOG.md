# Changelog

All notable QRY changes are documented here. The format follows Keep a
Changelog and release tags use Semantic Versioning with a `v` prefix.

## [Unreleased]

## [0.2.0-2] - 2026-10-28

Development prerelease source; publication remains subject to the release gate.

### Added

- opt-in local aggregate JSON bridge with live presentation values, all-time
  speed records and up to 90 days of daily summaries;
- QRY-owned native macOS 14+ capsule with Pip, live WPM, per-display placement,
  single-instance protection and embedded release packaging;
- allowlisted Today, Statistics and Settings navigation through QRY's single
  instance and permission gate;
- independent capsule, external-sharing and notch-Pip preferences, disabled by
  default, with SQLite migration support;
- English integration guides for the capsule and a separate Vorssaint adapter,
  including the versioned bridge contract and synthetic fixture.

### Fixed

- fit the quick QRY panel into its 360×470 window with all six indicators and
  actions visible, transparent rounded corners and no rectangular window shadow;
- merge the active session into the bridge's daily totals using the same
  character-weighted mean as stored statistics, without double counting;
- keep menu-bar activity independent of the desktop overlay preference.

### Maintenance

- run CI on `release/next` and compile the capsule for Apple Silicon and Intel;
- group public integration guides and brand references under `docs/`;
- exclude personal installers, upstream patches and local experiment records
  from the public source tree.

## [0.1.2-2] - 2026-08-07

### Added

- official `michele-lombardi/qry` Homebrew tap with automatic tracking of the
  latest public, non-prerelease GitHub Release.
- Windows 10/11 x64 parity through a privacy-minimized Raw Input adapter,
  permission-free onboarding, focused-display placement, tray and autostart;
- complete Windows CI plus NSIS and MSI packaging with normalized names,
  SHA-256 files and release-content audits;
- one cross-platform draft release workflow for Apple Silicon, Intel and
  Windows artifacts.

### Changed

- desktop documentation, privacy guarantees and contributor checks now cover
  both macOS and Windows without duplicating product logic.

## [0.1.1] - 2026-08-06

### Added

- three-step privacy and permission onboarding with required Input Monitoring
  and optional Accessibility;
- permission-gated bootstrap, bounded System Settings wait and clean relaunch;
- runtime permission-revocation gate and single-instance protection;
- persisted onboarding completion with a backward-compatible SQLite migration;
- private all-time WPM records over complete 30-second and 60-second windows,
  shown in Today and Statistics with the existing Pip record celebration;
- configurable 1–15 second Pip disappearance delay;
- explicit launch-at-login choice in onboarding with idempotent LaunchAgent
  reconciliation at startup and automatic cleanup when required access is
  unavailable.

### Changed

- monitoring starts after every successful permission check; launch at login is
  now an independent startup preference;
- Pip waits for the third accepted typing activity before appearing, then
  breathes during the configured quiet interval before fading out;
- launch at login no longer changes a manually paused monitor during the current
  run;
- public documentation is now English-first and organized around installation,
  vision, architecture, privacy, UI, development, release, and decision records;
- detailed internal plans and phase reports are no longer part of the public
  repository.

## [0.1.0] - 2026-08-05

First public macOS beta.

### Added

- privacy-preserving macOS Input Monitoring adapter;
- responsive live WPM with warm-up qualification, smoothing and repetition
  protection;
- aggregate-only SQLite persistence, automatic daily rollover and CSV export;
- compact daily panel opened from the menu-bar Pulse;
- Apple-style Settings and complete Statistics windows;
- separate WPM-speed and estimated-word charts with numeric Y axes and time on
  the X axes;
- transparent, click-through Pip overlay with four visual bands;
- configurable overlay position, size, content and background visibility;
- optional Accessibility flow that follows the focused window across displays
  without collecting application names, titles or content;
- automatic monitoring, login launch and menu-bar WPM preferences;
- personal best, streak, active time and daily/weekly/monthly/yearly summaries;
- adaptive Light/Dark menu-bar assets and stable three-digit WPM slot;
- GNU GPLv3-only licensing, contributor guidance and privacy documentation;
- automated frontend, Rust, packaging, privacy and release checks.

### Security and privacy

- individual keys, key codes, words, passwords, application names and window
  titles are never persisted or logged;
- only aggregate metric buckets and daily summaries cross the storage boundary;
- Tauri capabilities are restricted to the commands needed by the local app.

### Known beta limitations

- the app uses ad-hoc signing and is not notarized, so first launch requires the
  documented Gatekeeper procedure;
- Input Monitoring, Accessibility, login, suspend/resume and multi-monitor
  behavior require final manual validation on the distributed build;
- onboarding, a native CSV destination dialog and the full accessibility audit
  remain planned before the stable V1;
- Linux packaging is planned after the macOS beta and is not included here.

[Unreleased]: https://github.com/michele-lombardi/QRY/compare/v0.1.2-2...HEAD
[0.1.2-2]: https://github.com/michele-lombardi/QRY/compare/v0.1.1...v0.1.2-2
[0.1.1]: https://github.com/michele-lombardi/QRY/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/michele-lombardi/QRY/releases/tag/v0.1.0
