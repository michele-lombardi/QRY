# ADR 0006: Local-day rollover and automatic startup

- Status: Accepted and implemented
- Date: 2026-08-05

## Context

QRY must show “today” without requiring a manual reset. It also needs one
user-controlled option that both opens the app at macOS login and starts input
monitoring whenever the app process opens.

## Decision

The application assigns each completed session and metric bucket to the local
civil date observed on macOS. Queries for “today” resolve the date again on each
request. At midnight the new date therefore starts at zero automatically;
records from previous dates remain stored for history and CSV export.

If an active session crosses the date boundary, it is closed before the first
activity of the new day. An idle session finishes under the normal 30-second
timeout and remains assigned to the date on which it started. Buckets are fixed
60-second intervals and cannot mix two local dates.

The `Start automatically` checkbox persists one boolean preference and uses the
Tauri autostart plugin with a macOS `LaunchAgent` login item:

- enabled: register the login item and start monitoring immediately;
- later app open: reconcile the login item and start monitoring automatically;
- disabled: unregister the login item; the current monitor is not forcibly
  stopped, because stopping is a separate user action.

Failure to register a login item or obtain Input Monitoring permission is shown
as a runtime error and does not prevent the application window from opening.

## Consequences

- daily rollover needs no scheduler or destructive midnight job;
- changing timezone changes which local date is considered “today”, but does
  not rewrite historical rows;
- history is retained until an explicit reset command deletes one selected day;
- enabling automatic startup changes macOS login-item state and must be tested
  manually on a real user session;
- login startup cannot bypass macOS Input Monitoring consent.
