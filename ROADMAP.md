# QRY roadmap

This roadmap communicates product direction, not sprint commitments. Individual
tasks, priorities, and release dates are tracked separately and may change as
the project learns from real-world use.

## Public beta

The current beta focuses on the complete local-first desktop experience:

- privacy-safe global typing activity monitoring;
- live WPM and the click-through Pip companion;
- local daily, weekly, monthly, and yearly statistics;
- peak, 30-second, and 60-second personal records;
- guided macOS permissions and permission-free Windows capability onboarding;
- user-controlled launch at login;
- reproducible Apple Silicon, Intel, Windows NSIS and Windows MSI artifacts.

## Toward V1

The stable desktop release requires:

- validation of permissions, relaunch, logout/login, sleep/wake, and revocation
  flows on clean macOS accounts;
- final multi-monitor, reduced-motion, keyboard, VoiceOver, and contrast checks;
- verified installation, upgrade, and removal through GitHub Releases and the
  project Homebrew tap;
- a native destination picker for CSV exports;
- release-candidate testing on both Apple Silicon and Intel;
- signed-off native input, tray, autostart, sleep/wake, install and uninstall
  checks on a clean Windows 11 x64 VM;
- no open critical privacy, data-loss, or resource-usage defects.

## After V1

Potential follow-up work includes:

- Apple Developer ID signing and notarization;
- Windows Authenticode signing when an approved identity is available;
- explicit Light and Dark appearance overrides;
- coordinated reset of an active session;
- improved export and backup workflows;
- a Linux platform adapter and native packages, while preserving the same
  privacy boundary and portable Rust core.

## What is not planned

QRY does not plan to add typed-text capture, per-key history, application
tracking, cloud accounts, advertising analytics, or productivity scoring.

Feature proposals are welcome through
[GitHub Issues](https://github.com/michele-lombardi/QRY/issues). Please describe
the user problem and its privacy impact rather than proposing a hidden input or
data-collection shortcut.
