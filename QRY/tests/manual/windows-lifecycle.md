# Windows startup and lifecycle checklist

Run on a clean Windows 11 x64 virtual machine using a release installer. Record the
Windows build, QRY commit, installer checksum and result for every row.

## Fresh install and local data

- [ ] Confirm no QRY AppData directory or login registration exists before installation.
- [ ] Install QRY for the current user and complete onboarding.
- [ ] Confirm Today and Statistics begin at zero and no demo session, bucket, CSV, log
      or pre-populated SQLite file shipped inside the installer.
- [ ] Confirm `typepulse.sqlite3` is created under QRY's resolved roaming AppData
      directory only after the installed application starts.
- [ ] Enter activity, restart QRY and confirm aggregate statistics and all three
      personal records survive with the same schema as macOS.
- [ ] Export CSV, then reset today. Confirm the export contains aggregate rows only and
      the reset affects today without deleting earlier days.

## Autostart and single instance

- [ ] Enable launch at login, sign out and back in. QRY must start hidden with the tray
      and monitor active.
- [ ] Disable launch at login, sign out and back in. QRY must not start.
- [ ] Toggle the preference repeatedly and confirm Windows has at most one QRY startup
      registration.
- [ ] With QRY running, launch it again and confirm there is still one process and the
      appropriate QRY surface is brought forward.
- [ ] Force monitor initialization to fail and confirm QRY removes an inconsistent
      autostart registration and returns to its setup gate.
- [ ] Update and reinstall QRY, then confirm the saved preference and native
      registration do not drift or duplicate.

## System transitions and recovery

- [ ] Lock and unlock Windows while QRY is monitoring; typing must resume without
      duplicate events or an extra process.
- [ ] Sleep and wake the machine; confirm the tray, overlay and input monitor recover
      and accumulated idle time is not counted as active typing.
- [ ] Switch users and return; QRY must not cross user sessions or expose data from
      another Windows account.
- [ ] Change the active display and DPI configuration after wake; the overlay must
      reposition inside a valid work area.
- [ ] Terminate QRY during active typing and restart it; the database must open cleanly,
      completed aggregates must remain readable and no partial input data may exist.
- [ ] Quit from the tray and confirm all QRY threads and windows terminate.
