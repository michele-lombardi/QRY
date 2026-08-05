# Focused-display PiP checklist

This check requires macOS, two active displays, Input Monitoring access and the optional
Accessibility access. Record only pass/fail and display arrangement; never record app
names, window titles or typed content.

## Permission and fallback

1. Start QRY, open Settings and confirm the two permissions are shown separately.
2. Leave Accessibility denied, start monitoring and type in a disposable document on the
   secondary display.
3. Confirm the PiP still works and falls back to the primary display.
4. Select **Request access**, enable QRY under Privacy & Security → Accessibility, then
   quit and reopen the app if macOS requests it.
5. Confirm the Accessibility status changes to `granted`.

After installing or rebuilding an ad-hoc-signed copy, verify `granted` again. macOS may
treat the changed binary as a new identity; remove the stale QRY entry, add the exact
app being tested, enable it and restart QRY if needed.

Expected: denied access never blocks typing metrics and never causes a crash or repeated
permission prompt.

## Follow the focused window

1. Put a normal editable window entirely on the secondary display and keep the pointer
   on the primary display.
2. Begin typing. The PiP must appear directly on the secondary display, in the
   configured corner and useful area.
3. During the same live session, focus an editable window on the primary display and
   type. The PiP must move there no later than the next short typing burst.
4. Repeat with the window mostly across both displays. Record which display contains the
   window center; the PiP must use that display.
5. Repeat all four overlay corner settings and at least two different display scale
   factors.
6. With three displays in a horizontal chain, jump directly from a Retina 2× display to
   the farthest 1× display. The PiP must not stop on the intermediate display.

Expected: the focused window—not the mouse and not the configured primary
display—selects the PiP display when Accessibility is granted.

## Degraded cases

1. Revoke Accessibility while QRY is running and type again.
2. Test an app or system surface that does not expose a focused window.
3. Disconnect the display hosting the PiP while it is visible.

Expected: a transient read failure keeps the PiP on its current valid display. A cold
start without a valid focused target uses the primary/available display. QRY remains
responsive and does not invent focused-window data.

## Privacy observation

1. Use the QRY UI and exported CSV after the test.
2. Inspect only the documented aggregate database schema if needed.

Expected: no app name, window title, window rectangle or typed content is shown,
exported or stored.

## Result

- macOS version:
- hardware/display arrangement:
- build or commit:
- denied-permission fallback: PASS / FAIL
- secondary focused display: PASS / FAIL
- active-session display switch: PASS / FAIL
- mixed scaling/corners: PASS / FAIL
- direct Retina-to-third-display jump: PASS / FAIL
- revocation/unsupported-window fallback: PASS / FAIL
- privacy observation: PASS / FAIL
- notes without app, title or content data:
