# Changelog

All notable QRY changes are documented here. The format follows Keep a
Changelog and release tags use Semantic Versioning with a `v` prefix.

## [Unreleased]

### Planned

- complete onboarding, native CSV save flow and accessibility audit in Phase F;
- complete real TCC, overlay and multi-monitor validation before a stable macOS release.

### Added

- QRY compact daily panel opened by left-clicking the menu-bar Pulse, with live
  WPM, words, personal best, streak and last activity;
- dedicated Apple-style Settings and full Statistics windows with daily,
  weekly, monthly and yearly local aggregate views;
- detailed words, average WPM, peak WPM and active-time chart/table views;
- Settings control for menu-bar WPM synchronized with the native context menu;
- idle midnight rollover that closes the previous local day's active session.

- transparent click-through macOS overlay with live WPM and four visual bands;
- four screen positions, three size presets and three content modes;
- one-shot personal-record celebration and reduced-motion behavior;
- SQLite schema v2 for persisted visual preferences and safe v1 migration;
- automatic primary-display repositioning and multi-monitor fallback;
- QRY Pulse mark, application icon and dynamic menu-bar glyph;
- brand palette, system typography, product voice and Micro-Y text endorsement;
- identity-aligned Pip renderer with Walk, Run, Tired and one-shot Jump/Cheer.
- optional macOS Accessibility flow that places the PiP on the focused window's
  display without collecting app, title or content metadata;
- immediate focused-display placement with primary-display fallback and a
  dedicated two-monitor privacy checklist.
- optional persisted menu-bar WPM with a fixed three-digit slot independent from
  the PiP;
- adaptive live-WPM warm-up after a 250 ms observation span while retaining the
  10-second lookback and EMA smoothing;
- privacy-confined repetition protection for macOS auto-repeat and artificial
  same-key runs while preserving legitimate double letters;
- SQLite schema v3 and safe migration of the menu-bar WPM preference.

### Changed

- visible product, bundle, release archive and Homebrew cask name from TypePulse
  to QRY; the existing technical identifier and local database remain unchanged
  to preserve user data and macOS permission continuity.

### Fixed

- macOS menu-bar Pulse and flatline icons now preserve native template rendering
  after runtime state changes, keeping both the glyph and WPM title readable in
  Light and Dark appearances.
- changing between one-, two- and three-digit WPM values no longer shifts the
  menu-bar Pulse mark.
- the first live second now ramps progressively, and samples cannot update
  session statistics or the personal best until three seconds are observed;
- a transient focused-window Accessibility failure keeps the PiP on its current
  valid display instead of immediately moving it to the primary display.
- direct jumps from a Retina display to a farther 1× monitor now use the target
  scale, preventing the PiP from landing on an intermediate display.

## [0.1.0] - 2026-08-05

### Added

- privacy-preserving macOS Input Monitoring adapter;
- portable WPM, smoothing and session engine;
- aggregate-only SQLite persistence, daily rollover and CSV output;
- automatic login launch and monitoring preference;
- background macOS menu-bar shell without a Dock icon;
- GPL-3.0-only project licensing and contributor documentation;
- deterministic tests and manual macOS privacy checklists.

### Known limitations

- this version is a development release, not the stable V1;
- Input Monitoring, login, suspend and Gatekeeper flows still require manual
  verification on a release artifact;
- overlay, onboarding and final statistics/settings screens are incomplete.

[Unreleased]: https://github.com/TODO_OWNER/qry/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/TODO_OWNER/qry/releases/tag/v0.1.0
