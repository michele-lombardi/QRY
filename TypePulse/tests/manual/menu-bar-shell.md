# Manual check — macOS menu-bar shell

This checklist verifies UI behavior that cannot be reproduced by the Rust test runtime.
It does not require typing or recording any keyboard input.

## Setup

1. Run `npm run tauri dev` from `TypePulse/`, or open the debug bundle.
2. Do not enable **Start automatically** merely for this checklist.

## Acceptance checks

- [ ] TypePulse starts without opening a large window.
- [ ] An idle TypePulse flatline glyph appears in the upper-right macOS menu bar.
- [ ] TypePulse has no Dock icon.
- [ ] TypePulse does not appear in `Cmd + Tab`.
- [ ] Left-clicking the status icon opens and focuses the TypePulse window.
- [ ] Closing the window hides it and leaves the status icon running.
- [ ] Right-clicking the status icon shows Open, Start monitoring, Pause monitoring and
      Quit TypePulse.
- [ ] Start/Pause affect the same monitor state shown in the window.
- [ ] Typing changes the flatline to the three-beat Pulse mark and shows rounded live
      WPM beside it; stopping returns it to the flatline without a stale number.
- [ ] Quit TypePulse removes the status icon and terminates the process.
- [ ] Reopening TypePulse recreates exactly one status icon.
- [ ] Light and dark menu bars both render the glyph as a native monochrome template.

## Result

- Date / macOS / architecture: `TODO`
- Bundle or dev command used: `TODO`
- Result: `TODO manuale`
- Notes: `TODO`
