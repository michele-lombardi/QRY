# Phase E — overlay end-to-end checklist

Run this checklist on macOS with a real Input Monitoring grant. Record only pass/fail
notes and aggregate WPM values; never attach keyboard-event logs.

## Preparation

1. From `QRY/`, run `npm run tauri dev`.
2. Open QRY Settings from its menu-bar context menu.
3. Confirm Input Monitoring is `granted` and select **Start**.
4. Enable **Show overlay**, then choose `Top right`, `Medium`, and `WPM and animation`.
5. Open a disposable document in TextEdit or another normal editor.

## Focus, lifecycle and click-through

1. Press two accepted typing keys and confirm the overlay stays hidden. Press a third
   key and confirm it appears without QRY becoming the active application.
2. Set **Disappear after** to 5 seconds. Stop typing and confirm Pip changes to Breathe,
   remains present for the configured interval, then fades and disappears.
3. Type again before the 30-second session timeout: the same session resumes and the
   card reappears.
4. Place the overlay above a safe clickable control in another app, then click the
   control through the card. The underlying control must receive the click.
5. Confirm QRY remains absent from the Dock and `Cmd + Tab`.

Expected: the overlay never accepts focus, typing or pointer input.

## Position, size and content

While the monitor is running, exercise every option from QRY Settings:

- [ ] Disable **Show background card** and confirm only Pip/WPM remain visible, with no
      material rectangle, border or blur.

- four positions: top-left, top-right, bottom-left, bottom-right;
- three sizes: small, medium, large;
- content: WPM, animation, both;
- disappearance delay: 2, 3, 5, 8, 10 and 15 seconds;
- overlay disabled and enabled again.

Expected: each change persists after Quit/reopen and applies without restarting the
monitor. The card stays inside the useful screen area and does not overlap the macOS
menu bar or Dock.

## Visual states and record

1. Confirm Pip has one circular body, two eyes and two capsule feet, with no mouth,
   outline, character shadow or accessories.
2. Type below and above 70 WPM and confirm Pip changes from Walk to Run; Run leans
   forward and shows cyan dash lines. Internal 30/60/90 WPM core bands remain covered by
   deterministic tests and make Walk/pulse tempo progress smoothly.
3. Complete a baseline session by waiting at least 30 seconds.
4. Start a faster session and exceed the stored peak.
5. Confirm Pip jumps, turns green and cheers once, then does not loop while the value
   remains above the old record.
6. Establish a complete 30-second and then 60-second baseline, start a faster session,
   and confirm each sustained record uses the same one-shot Jump/Cheer celebration.
7. Enable **Reduce motion** in macOS Accessibility settings and repeat. Values must
   remain readable without continuous motion.
8. Tired activates only after 90 aggregate active-typing minutes in one session; use a
   real long-session check rather than synthetic input before release.

## Multiple displays

1. Complete the focused-window routing cases in `focused-display.md`.
2. Attach a second display and repeat all four positions.
3. Disconnect the display currently hosting the overlay while it is visible.
4. Change scaling or resolution on the remaining display.

Expected: with Accessibility granted the overlay follows the focused window's display.
Without it, or after revocation/disconnection, it moves into the primary available
display's useful area and never remains at stale coordinates.

## Result

- macOS version:
- hardware/displays:
- build or commit:
- focus retained: PASS / FAIL
- click-through: PASS / FAIL
- fade and timeout: PASS / FAIL
- positions/sizes/content: PASS / FAIL
- Walk / Run brand behavior: PASS / FAIL
- Tired long-session behavior: PASS / FAIL / NOT RUN
- one-shot record celebration: PASS / FAIL
- 30/60-second records: PASS / FAIL
- reduced motion: PASS / FAIL
- multi-monitor fallback: PASS / FAIL
- notes (no typed content):
