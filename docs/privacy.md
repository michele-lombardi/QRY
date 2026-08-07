# Privacy model

QRY measures when privacy-safe typing activity occurs, not which key was
pressed or what was written. Privacy is enforced by architecture, not only by a
promise in the interface.

## Data QRY may process

Transiently in memory:

- a monotonic timestamp for each accepted typing activity;
- temporary key identity inside the private native filter only, long enough to
  reject modifiers, auto-repeat, and abusive repetition;
- temporary focused-window position and size when optional Accessibility access
  is enabled, reduced immediately to a display point.

Persisted locally:

- completed-session duration, estimated words, average WPM, and peak WPM;
- fixed aggregate metric buckets;
- peak, 30-second, and 60-second record values;
- local-date summaries and streak inputs;
- onboarding and user preferences.

## Data QRY does not collect

- characters, key codes, scancodes, words, passwords, or written text;
- per-key history or a persistent event timeline;
- application names, bundle identifiers, process identifiers, or window titles;
- URLs, browser history, document names, selected text, or clipboard contents;
- account identity, advertising identifiers, analytics, or crash telemetry;
- focused-window geometry in the frontend, database, CSV, logs, or backups.

## Structural guarantees

The public platform API emits `TypingActivity { occurred_at: Instant }`. Raw
key information has no serializable domain type and cannot enter the engine,
Tauri DTOs, frontend, or storage repository.

The macOS event tap is passive (`ListenOnly`) and returns events unchanged. The
Windows Raw Input adapter observes background activity without suppressing or
injecting input and does not bypass UAC secure desktop. Native callbacks perform
no disk, network, UI, or synchronous database work. Protected-input gaps are
accepted rather than bypassed.

The SQLite schema contains aggregate rows only. Migrations are embedded and a
local backup may be created before upgrading an older non-empty database. That
backup contains the same aggregate data as the source database.

Tauri capabilities do not grant general filesystem, shell, opener, global
shortcut, or network plugin access to the application windows.

## Permissions

### Input Monitoring

Required for observing global typing activity on macOS. QRY cannot grant it.
Without permission, the normal shell and monitor do not start; closing or
denying the guided flow exits the app.

### Accessibility

Optional. It is used only to determine which display contains the focused
window. QRY requests `AXFocusedApplication`, `AXFocusedWindow`, `AXPosition`,
and `AXSize`, then discards the temporary geometry. Without it, Pip uses the
fallback display and all metrics continue to work.

Windows does not require an equivalent permission. It reduces the foreground
window rectangle to a center point without reading its title, class, process or
content.

### Start at login

Optional and off by default during onboarding. It creates the platform's native
login registration only after the setup gate is valid. It does not change what
QRY collects.

## Local storage and export

Application data is stored under the operating system's application-data directory in
`typepulse.sqlite3`. QRY does not automatically upload it. CSV export occurs
only after a direct user action and contains aggregate statistics.

Resetting a day removes its completed sessions and minute buckets. Removing the
app does not necessarily remove its application-data directory; users may
delete that directory separately if they also want to erase local history.

## Logging policy

Allowed logs are limited to lifecycle transitions, permission state, aggregate
counts in explicit diagnostics, and categorized errors.

Logs must never contain raw events, characters, key codes, per-key timestamps,
typed text, focused applications, windows, URLs, or clipboard data.

## Review requirements

Every change involving input, permissions, storage, logging, exports, Tauri
capabilities, or external communication must answer:

1. Does raw input identity remain inside the private platform filter?
2. Can a new field reconstruct text or identify an application/window?
3. Is persistence aggregate-only and migration-safe?
4. Is a new permission or capability strictly necessary and visible to users?
5. Does `./scripts/check.sh` and the privacy audit still pass?

Security issues should be reported privately as described in
[SECURITY.md](../SECURITY.md). The architectural rationale is documented in
[ADR 0002](decisions/0002-input-privacy-boundary.md) and
[ADR 0007](decisions/0007-focused-display-accessibility.md).
