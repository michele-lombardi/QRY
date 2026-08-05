# QRY interface and window model

## Product naming and compatibility

The visible product name is **QRY**. macOS builds, window titles, release ZIPs
and the Homebrew cask use `QRY` or `qry` as appropriate.

The following legacy technical values remain intentionally unchanged in this
release:

- workspace directory: `TypePulse/`;
- bundle identifier: `app.typepulse.desktop`;
- SQLite filename: `typepulse.sqlite3`;
- internal Rust crate names and event identifiers.

Keeping these values avoids creating a second local database and reduces the
risk of losing preferences, login-item state or existing macOS privacy grants.
A future identifier migration requires an explicit data and TCC migration plan.

## macOS surfaces

QRY is an accessory application: it has no normal Dock or `Cmd + Tab` presence.
It exposes four separate Tauri pages.

| Surface | Window label | Entry point | Purpose |
| --- | --- | --- | --- |
| Today panel | `dashboard` | `dashboard.html` | compact live daily glance opened by left click |
| Settings | `main` | `index.html` | complete editable preferences and permissions |
| Statistics | `statistics` | `statistics.html` | aggregate history, graph, table and export |
| Pip overlay | `overlay` | `overlay.html` | click-through live rhythm on the focused display |

The Today panel is borderless, always on top and anchored to the top-right work
area. It closes on a second Pulse click, `Escape`, loss of focus, or when a full
window opens. Settings and Statistics are ordinary closable windows; closing
them hides the window without stopping the background monitor.

## Menu-bar interaction

- left click: toggle the Today panel;
- right click: native quick-action menu;
- **Today**: open the compact panel on the primary display;
- **Statistics…**: open the detailed Statistics window;
- **Settings…**: open the complete Settings window;
- **Show WPM in menu bar**: persistently show or hide the fixed-width number;
- **Start/Pause monitoring**: control collection without quitting;
- **Quit QRY**: flush aggregate state and terminate.

The native WPM title reserves a stable three-character slot. It can be disabled
without changing Pip. The Settings switch and native checked item share the
same persisted preference and update one another during the same run.

## Today panel data

The panel refreshes once per second and shows:

- live displayed WPM from the portable engine;
- words today as completed-session words plus current-session characters / 5;
- personal best as the maximum of stored history and the qualified live record;
- consecutive local days with estimated words;
- the local clock time of the last accepted privacy-safe typing activity.

No keystroke identity, text, application name or window title enters a UI DTO.

## Settings

The Apple-style sidebar separates four concerns:

- **General**: start/pause, launch and monitor automatically, menu-bar WPM and
  Today-panel access;
- **Appearance**: Pip enabled, corner, size, content and a live visual preview;
- **Permissions**: separate Input Monitoring and optional Accessibility states,
  request actions and System Settings links;
- **Privacy**: local-storage, no-network and license explanation.

The interface follows the macOS system appearance automatically. Explicit
Light/Dark overrides remain a later task; System appearance is the supported
mode in this increment.

## Detailed statistics

The period selector supports Today, 7 days, 30 days and the last year. Each
view displays:

- estimated words;
- character-weighted average WPM;
- qualified peak WPM;
- active typing time and completed/current session count;
- words as activity bars, average WPM as a cyan line and peaks as green points;
- a period breakdown table and a short neutral rhythm insight.

Today uses persisted one-minute aggregate buckets for the chart and combines
the completed daily summary with the current in-memory session for the top
cards. Longer ranges use daily summaries and merge the current session into
the last day. This keeps headline data live while the chart remains
privacy-preserving and bounded. Statistics refresh on focus and every five
seconds while visible.

**Copy CSV** copies the selected daily aggregate range. **Reset today…** removes
completed sessions and persisted minute buckets for the current local date; an
already active in-memory session can still complete afterwards. A native file
destination and a coordinated active-session reset remain explicit follow-up
tasks.

## Open follow-up tasks

- first-run onboarding for the two distinct permissions;
- native CSV save dialog in addition to clipboard copy;
- explicit Light and Dark overrides;
- coordinated reset of an active session;
- VoiceOver, keyboard, contrast and reduced-motion validation on a release
  bundle;
- final owner/contact placeholders and QRY name/domain/trademark verification.
