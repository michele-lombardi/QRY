# Changelog

All notable TypePulse changes are documented here. The format follows Keep a
Changelog and release tags use Semantic Versioning with a `v` prefix.

## [Unreleased]

### Planned

- complete statistics, settings, onboarding and accessibility in Phase F;
- complete real TCC, overlay and multi-monitor validation before a stable macOS release.

### Added

- transparent click-through macOS overlay with live WPM and four visual bands;
- four screen positions, three size presets and three content modes;
- one-shot personal-record celebration and reduced-motion behavior;
- SQLite schema v2 for persisted visual preferences and safe v1 migration;
- automatic primary-display repositioning and multi-monitor fallback;
- TypePulse Pulse mark, application icon and dynamic menu-bar glyph;
- brand palette, system typography, product voice and Micro-Y text endorsement;
- identity-aligned Pip renderer with Walk, Run, Tired and one-shot Jump/Cheer.
- optional macOS Accessibility flow that places the PiP on the focused window's
  display without collecting app, title or content metadata;
- immediate focused-display placement with primary-display fallback and a
  dedicated two-monitor privacy checklist.

### Fixed

- macOS menu-bar Pulse and flatline icons now preserve native template rendering
  after runtime state changes, keeping both the glyph and WPM title readable in
  Light and Dark appearances.

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

[Unreleased]: https://github.com/TODO_OWNER/typepulse/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/TODO_OWNER/typepulse/releases/tag/v0.1.0
