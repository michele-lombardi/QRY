# Changelog

All notable QRY changes are documented here. The format follows Keep a
Changelog and release tags use Semantic Versioning with a `v` prefix.

## [Unreleased]

No unreleased changes.

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

[Unreleased]: https://github.com/michele-lombardi/QRY/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/michele-lombardi/QRY/releases/tag/v0.1.0
