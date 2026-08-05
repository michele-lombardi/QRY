# Manual check — reactive live WPM and repetition guard

Run this check with Input Monitoring granted. Use a disposable field and record only
aggregate WPM/count observations; never record the typed characters or key identities.

## Reactive estimate

1. Pause for at least two seconds, then type normally at a steady pace.
2. Observe the PiP and, if enabled, the menu-bar WPM.
3. Repeat with a deliberately slower and then faster short phrase.
4. Start a new session, type a fast burst shorter than three seconds, then stop
   monitoring so the session is saved.
5. Confirm that the short warm-up burst did not increase today's peak; type steadily for
   more than three seconds and confirm that qualified values can update it.

Expected: the first reliable estimate appears after a short 250 ms observation span,
normally within two to four counted characters. It must not wait for the 10-second
lookback to fill. During the first second it grows progressively instead of jumping to a
full projected rate. Later values should remain smoother than individual key intervals
and must never exceed 300 WPM. No value from the first three seconds may update the
session average, peak or personal best.

## Repetition protection

1. Hold one textual key down for several seconds.
2. Release it, pause, and manually press the same key four times quickly.
3. Type a normal word containing a double letter.
4. Switch between different textual keys.

Expected:

- operating-system auto-repeat does not increase typing activities;
- the first two identical manual presses count, while the third and later presses in the
  uninterrupted run do not;
- a different counted key or a one-second pause resets the guard;
- a legitimate double letter still counts twice.

## Result

- macOS / keyboard layout:
- build or commit:
- first estimate latency: PASS / FAIL
- one-second progressive ramp: PASS / FAIL
- three-second record qualification: PASS / FAIL
- slow/fast response: PASS / FAIL
- held-key auto-repeat rejected: PASS / FAIL
- identical-run guard: PASS / FAIL
- double-letter behavior: PASS / FAIL
- notes without input content:
