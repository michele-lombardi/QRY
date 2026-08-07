# Windows shell and overlay checklist

Run this checklist on a clean Windows 11 virtual machine after the automated workspace
checks pass. Record the Windows build number, QRY commit and result for every row in the
release evidence.

## Focused display and DPI

- [ ] With one 100% display, type in Notepad and confirm the overlay uses the configured
      corner of that display.
- [ ] Repeat at 125%, 150% and 200% scaling; the overlay must remain fully inside the
      work area and keep the same logical size.
- [ ] Connect a second display to the left of the primary display so its global
      coordinates are negative. Move the focused editor between displays and confirm the
      overlay follows it.
- [ ] Repeat with a display above the primary display and with mixed scaling.
- [ ] Focus a window whose DWM frame geometry is unavailable and confirm QRY falls back
      without crashing or moving off-screen.
- [ ] Temporarily make the foreground window unavailable (desktop, lock/unlock, or app
      transition) and confirm the current display is retained before the primary-display
      fallback is used.

## Overlay interaction

- [ ] Confirm the overlay never receives keyboard focus while typing.
- [ ] Click through every visible part of the overlay and confirm the underlying
      application receives the click.
- [ ] Confirm the overlay stays above ordinary windows without appearing in the taskbar
      or Alt+Tab list.
- [ ] Verify all four configured corners and all three size presets.
- [ ] Confirm the overlay appears only after three qualifying activities, breathes
      during the configured quiet interval, and then disappears.

## Tray and windows

- [ ] Confirm the QRY tray icon appears after setup and has an accessible tooltip.
- [ ] Left-click the icon twice and confirm the dashboard opens and closes.
- [ ] Open Today, Statistics and Settings from the menu; only the requested window
      should remain visible and focused.
- [ ] Toggle **Show WPM**, pause monitoring, resume monitoring and confirm each state is
      reflected without restarting QRY.
- [ ] Close every ordinary QRY window and confirm the background process and tray remain
      active.
- [ ] Choose **Quit QRY** and confirm the monitor, overlay, windows and tray all
      terminate cleanly.
