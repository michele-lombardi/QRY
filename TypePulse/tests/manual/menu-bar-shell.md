# Manual check — macOS menu-bar shell

This checklist verifies UI behavior that cannot be reproduced by the Rust test runtime.
It does not require typing or recording any keyboard input.

## Setup

1. Run `npm run tauri dev` from `TypePulse/`, or open the debug bundle.
2. Do not enable **Start automatically** merely for this checklist.

## Acceptance checks

- [ ] QRY starts without opening a large window.
- [ ] An idle QRY flatline glyph appears in the upper-right macOS menu bar.
- [ ] QRY has no Dock icon.
- [ ] QRY does not appear in `Cmd + Tab`.
- [ ] Left-clicking the status icon opens the compact QRY Today panel.
- [ ] Closing the window hides it and leaves the status icon running.
- [ ] Right-clicking the status icon shows Today, Statistics, Settings, Show WPM in menu
      bar, Start monitoring, Pause monitoring and Quit QRY.
- [ ] Start/Pause affect the same monitor state shown in the window.
- [ ] Typing changes the flatline to the three-beat Pulse mark and shows rounded live
      WPM beside it; stopping returns it to the flatline without a stale number.
- [ ] While WPM changes across one-, two- and three-digit values, the fixed three-digit
      slot keeps the Pulse mark stationary.
- [ ] Unchecking **Show WPM in menu bar** removes the numeric slot but leaves WPM in the
      PiP; checking it restores the slot, and the choice survives Quit/reopen.
- [ ] Quit QRY removes the status icon and terminates the process.
- [ ] Reopening QRY recreates exactly one status icon.
- [ ] With QRY idle, switch macOS from Light to Dark appearance and back: the flatline
      becomes light on the dark menu bar and dark on the light menu bar.
- [ ] Repeat the appearance switch while typing: both the Pulse glyph and native WPM
      title change contrast automatically without restarting QRY or flashing a
      stale-color icon.

## Result

- Date / macOS / architecture: `TODO`
- Bundle or dev command used: `TODO`
- Result: `TODO manuale`
- Notes: `TODO`
