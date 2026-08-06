# Manual check — macOS Input Monitoring

Do not attach keyboard event logs or type a real password during this check.

## Setup

1. Run `npm run tauri dev` from `QRY/`.
2. In the diagnostic window, select **Check**.
3. If access is denied, select **Request access**, then **Open Settings**.
4. Enable QRY (or the development terminal when macOS identifies the executable that
   way), quit the debug app, and start it again.
5. Confirm that the status is `granted` before selecting **Start**.

Unsigned debug executables can appear as a new TCC identity after a rebuild. Remove
stale entries only through System Settings, never by editing the TCC database.

## Acceptance checks

- [ ] With onboarding incomplete, QRY shows only the permission gate and no menu bar or
      overlay.
- [ ] **Exit QRY**, the red window button, and the timeout terminate the process without
      leaving orphan helpers.
- [ ] With required permission missing, a stale login item is removed and **Start QRY at
      login** returns to disabled.
- [ ] Granting Input Monitoring enables **Continue**; completing the optional
      Accessibility step produces exactly one relaunch.
- [ ] After relaunch, QRY creates one instance, starts the monitor, and does not repeat
      onboarding.
- [ ] Revoking permission stops the monitor and overlay and reopens the gate.
- [ ] Skipping Accessibility keeps the primary-display fallback.
- [ ] Status changes between `denied` and `granted` truthfully.
- [ ] **Open Settings** opens Privacy & Security → Input Monitoring.
- [ ] **Start** reaches `running` without freezing input.
- [ ] Letters, digits, punctuation, space and keypad keys increase the count in a
      different application such as TextEdit.
- [ ] Shift plus a writing key increases the count.
- [ ] Command, Control, Option and Fn shortcuts do not increase the count.
- [ ] Return, Tab, Delete, Escape, arrows and function keys do not increase the count.
- [ ] **Stop** prevents further increases and returns promptly.
- [ ] Removing permission while running reaches `permission-revoked` without a crash or
      invented activity.
- [ ] After granting permission again, stop/start (or app restart when requested by
      macOS) returns to `running`.
- [ ] Dropped activities remain zero under ordinary typing.
- [ ] Callback average/max remain comfortably below 1 ms during ordinary use.

## Current local result — 5 August 2026

- preflight: `denied`;
- settings deep link: opened successfully;
- remaining boxes: TODO by the Mac owner after granting TCC access.

Record only pass/fail, OS, architecture and aggregate callback figures. Never record the
keys used.
