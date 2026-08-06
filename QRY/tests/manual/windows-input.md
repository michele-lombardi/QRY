# Windows global-input parity check

Run this checklist on a clean Windows 10/11 x64 machine after the automated Windows job
is green. Do not record raw keyboard-event logs.

## Preconditions

- Build a debug or release QRY executable from the same commit under test.
- Start with no QRY process running.
- Keep Task Manager available to verify that only one instance remains.

## Lifecycle

1. Start QRY and confirm the monitor reaches `running` within three seconds.
2. Pause and start monitoring ten times; confirm no hang or duplicate activity.
3. Quit QRY while typing; confirm the process exits promptly.
4. Start QRY twice; confirm the second launch does not create a second monitor.

## Input parity

For Notepad, a browser textarea, a terminal, and an Office editor:

1. Type one and two characters: Pip must remain hidden.
2. Type the third valid character: Pip must appear.
3. Hold a letter: operating-system repeat must not inflate activity.
4. Type a legitimate double letter: both presses must count.
5. Use arrows, function keys, Backspace, Delete, Enter, and Tab: none count.
6. Use Ctrl, Alt, and Windows shortcuts: shortcut keys do not count.
7. Use Shift with letters/punctuation: the text key counts.
8. On an Italian layout, type AltGr characters: the text key counts once.
9. Repeat with a US layout, dead keys, and an IME composition sequence.

## System boundaries

1. Lock and unlock Windows, then confirm normal input resumes.
2. Suspend and resume the machine, then confirm the monitor remains usable.
3. Test inside a Remote Desktop session and record only pass/fail aggregates.
4. Open a UAC secure-desktop prompt: QRY must not bypass or monitor the secure desktop;
   normal monitoring may resume after returning to the user desktop.

## Privacy review

- No raw key, scan code, virtual key, entered text, app identity, window title, device
  handle, or `HWND` appears in logs, DTOs, IPC, SQLite, or CSV.
- Diagnostics contain only lifecycle state, event counts, drop counts, and aggregate
  callback timing.
