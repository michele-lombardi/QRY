# Manual check — Phase D persistence, rollover and automatic startup

This checklist changes the current user's macOS login-item configuration. Use dummy text
only and disable the option again at the end if you do not want QRY to open at login.

## Prerequisites

1. Complete the Input Monitoring checklist and confirm `Access: granted`.
2. Run `npm run tauri dev` from `TypePulse/`.
3. Keep the diagnostic window open and type dummy text in TextEdit.

## Persistence and daily summary

- [ ] **Start** reaches `running` and live WPM changes while typing.
- [ ] Stop monitoring, or wait 30 seconds after typing, so the active session is
      completed.
- [ ] Today's estimated words, average/peak WPM, active time and session count become
      non-zero.
- [ ] Quit and reopen the app; today's completed values are unchanged.
- [ ] Confirm the database exists in the QRY application-data directory as
      `typepulse.sqlite3`.
- [ ] Inspect only its schema: tables are `completed_sessions`, `metric_buckets`,
      `app_preferences` plus SQLite metadata; no key, text, application or window column
      exists.

Do not attach a copy of a personal database to an issue. It contains only aggregates,
but they are still personal activity data.

## Automatic startup

- [ ] Enable **Start automatically**.
- [ ] Confirm QRY appears in macOS System Settings → General → Login Items.
- [ ] Stop monitoring, quit QRY, then open it normally; monitoring reaches `running`
      without pressing **Start**.
- [ ] Log out and back in, or restart the Mac; QRY opens and monitoring starts after
      macOS grants access.
- [ ] Disable **Start automatically** and confirm the login item is removed.
- [ ] Quit and reopen QRY; monitoring remains stopped until **Start** is selected.

If TCC consent is missing, automatic launch may work while monitoring reports a
permission error. That is expected: the preference never bypasses macOS consent.

## Local-day rollover

The automated suite verifies Gregorian boundaries and date-isolated queries. For one
real midnight test:

- [ ] Note today's completed summary before local midnight.
- [ ] Leave QRY open across midnight without changing the system clock.
- [ ] Confirm the displayed date advances and the new day's summary starts at zero.
- [ ] Use the seven-day/CSV backend in a development check to confirm the prior day
      remains present; rollover must not delete history.

Do not change the Mac clock merely to force this test: TCC, filesystem and other
applications can behave unexpectedly. Mark this item `TODO manuale` until a natural date
change is observed.

## Result

- Date / macOS / architecture: `TODO`
- Persistence after reopen: `TODO`
- Login item after logout/login: `TODO`
- Natural local-day rollover: `TODO`
- Notes containing aggregates only: `TODO`
