# ADR 0008: Permission-gated lifecycle and clean relaunch

- Status: Accepted and implemented in code
- Date: 2026-08-06

## Context

Input Monitoring is required for QRY's primary function. The previous shell
could open without the permission, expose controls that could not work and only
start monitoring when the separate login preference was enabled. A user who
changed the permission in macOS also had no guided, bounded path back into a
working process.

Accessibility serves only focused-display placement and must remain optional.
It is not required to calculate WPM or store local aggregates.

## Decision

QRY evaluates Input Monitoring before creating its tray, overlay or monitor:

- permission granted and onboarding completed: create the normal background
  shell and start monitoring;
- permission missing or onboarding incomplete: show only a three-step
  permission gate;
- denial, gate close or a two-minute wait timeout: stop any live component and
  exit;
- grant: persist onboarding completion and perform one clean process restart;
- runtime revocation: stop the monitor and overlay, hide normal surfaces and
  return to the same gate;
- Accessibility: explain and offer it after Input Monitoring, with a supported
  skip path and primary-display fallback;
- launch at login: offer an unchecked explicit choice in the final onboarding
  step, register it only after required access, and reconcile the persisted
  preference with the real LaunchAgent on every bootstrap;
- invalid permission gate: remove any stale LaunchAgent and clear its preference.

The process remains alive only while the visible gate is active or while it is
performing the bounded permission wait. No permanent permission helper is
installed. The Tauri single-instance plugin is registered before other plugins
so relaunch, login and manual double-open converge on one process.

Monitoring now starts after every successful permission-gated app launch.
`Start automatically` controls whether macOS opens QRY at login; pausing the
current monitor remains a separate runtime action. This supersedes the part of
ADR 0006 that coupled monitor startup to the login preference.

## Consequences

- the normal UI cannot appear operational without required consent;
- existing beta users receive the new onboarding once through schema migration
  5, without losing preferences or aggregate statistics;
- shell and overlay state are absent during first-run denial, so commands must
  not assume those managed states exist;
- TCC, relaunch, logout/login and distributed bundles still require manual
  tests because automated tests cannot grant real macOS consent;
- Accessibility does not become a hidden prerequisite for core functionality.
- enabling or disabling launch at login never resumes or stops a manually
  controlled monitor during the current process.
