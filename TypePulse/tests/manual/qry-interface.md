# QRY panel, Settings and Statistics manual test

Run this checklist on the exact debug or release bundle being evaluated. Grant Input
Monitoring for live values; grant Accessibility separately when also checking
focused-display Pip behavior.

## Naming and lifecycle

- [ ] The menu-bar tooltip, window titles and visible brand say **QRY**.
- [ ] QRY has no normal Dock icon and does not appear in `Cmd + Tab`.
- [ ] Left-clicking the Pulse opens the compact Today panel, not Settings.
- [ ] A second left click, `Escape` and clicking outside each close the panel.
- [ ] Right-clicking opens Today, Statistics, Settings, WPM, Start/Pause and Quit
      actions.
- [ ] Closing Settings or Statistics hides the window but leaves QRY running.

## Today panel

- [ ] WPM changes within the expected live warm-up without creating an early false
      personal record.
- [ ] Words today includes the current session before it ends.
- [ ] Personal best never decreases and does not accept unqualified warm-up.
- [ ] Quiet since shows the clock time of the most recent accepted activity.
- [ ] Settings and Full statistics open the correct complete windows.
- [ ] With three displays, clicking the menu-bar item places the panel on the display
      containing that status item; opening Today from a full window uses the primary
      display.

## Settings

- [ ] General, Appearance, Permissions and Privacy switch without opening new windows.
- [ ] Start/Pause updates monitor state and survives repeated use.
- [ ] Start automatically updates the macOS login item and starts monitoring.
- [ ] Show live WPM immediately adds/removes the menu-bar number and keeps the native
      right-click check state synchronized.
- [ ] Number changes do not move the Pulse icon because the slot stays fixed.
- [ ] Pip enabled, position, size and content apply live and persist after quit.
- [ ] Input Monitoring and Accessibility show independent accurate states.
- [ ] Request/Open System Settings actions target the correct macOS panes.
- [ ] Light and Dark system appearances keep text, controls and brand readable.

## Statistics

- [ ] Today, 7 days, 30 days and Year each load without error.
- [ ] Today headline words and active time update during the current session.
- [ ] The graph distinguishes cyan average WPM, green peaks and word bars.
- [ ] Table totals and period labels match the selected range.
- [ ] Copy CSV places aggregate-only rows in the clipboard.
- [ ] Reset today requires confirmation and removes completed today data only.
- [ ] Hiding and reopening Statistics refreshes stale values.

## Privacy regression

- [ ] UI, CSV and local SQLite inspection contain no key code, character, text,
      application name, window title or focused-window geometry.
- [ ] No network request is emitted while opening or refreshing these surfaces.

Record macOS version, architecture, display layout/scales, tested commit and any failed
item without attaching typed content or raw input logs.
