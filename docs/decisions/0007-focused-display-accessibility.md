# ADR 0007: Focused-display placement through macOS Accessibility

- Status: Accepted
- Date: 2026-08-05

## Context

Pip should appear on the display where the user is typing, not always on the
primary display. Pointer location is not a reliable signal, and Input Monitoring
does not identify a destination window in a privacy-safe way.

macOS Accessibility can expose the focused window and its geometry, but it uses
a separate TCC permission and can reveal much more information than QRY needs.

## Decision

The macOS adapter requests only:

1. `AXFocusedApplication` to reach the focused element;
2. `AXFocusedWindow`;
3. `AXPosition`;
4. `AXSize`.

The adapter immediately reduces position and size to the focused window's center
point. `src-tauri` maps that point to a monitor and positions Pip in the user's
configured corner of that monitor's work area.

Placement is evaluated before showing Pip, during accepted typing activity at
most once every 250 ms, and when display topology is reconsidered.

Accessibility remains optional. A failed reading after a valid placement keeps
the current display; a cold start uses the primary display and then the first
available display. A later valid reading takes precedence.

## Privacy constraints

- do not request `AXTitle`, `AXValue`, selected text, role, or URL;
- do not derive or expose application name, bundle identifier, or PID;
- do not send geometry to the frontend;
- do not store geometry in logs, SQLite, backups, or CSV;
- retain geometry only for the current placement operation;
- treat denial, revocation, and unsupported attributes as normal fallback states.

## Consequences

- focused-display routing works even when the pointer is elsewhere;
- the portable domain remains independent of macOS Accessibility;
- users see two separate permissions with different purposes;
- all typing metrics still work when Accessibility is skipped or unavailable;
- other platforms must implement an equivalent minimized signal or document
  their fallback behavior.
