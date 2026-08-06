# Architecture

QRY is a local-first macOS application built around one rule: keyboard identity
must disappear before data reaches the portable core, UI, storage, or logs.

## System overview

```text
macOS CGEvent tap
       │
       ▼
private input filter ── discards key identity and auto-repeat
       │
       ▼
TypingActivity(Instant only)
       │
       ▼
portable Rust engine ── rolling WPM, sessions, animation, records
       │
       ├──► aggregate repository ──► SQLite
       │
       └──► desktop controller ──► menu bar, Pip, Today, Statistics
```

The application never creates a serializable raw-key model. This prevents a
key code, character, or word from crossing the platform boundary by accident.

## Technology

- Tauri 2 owns the application lifecycle, menu bar, windows, and packaging.
- Rust owns input adapters, concurrency, metrics, sessions, and persistence.
- Vanilla TypeScript, HTML, and CSS render the interface.
- SQLite stores local aggregate history behind a repository contract.
- GitHub Actions runs checks and builds separate Apple Silicon and Intel assets.

The frontend deliberately has no framework. A UI dependency should be added
only when it removes more complexity than it introduces.

## Workspace boundaries

### `typepulse-core`

The portable domain layer contains WPM calculation, session rules, record
detection, aggregate models, dates, and repository interfaces. It has no Tauri,
SQLite, or operating-system dependency.

`TypingEngine` receives an injected clock. Production uses monotonic system
time; tests use a manual clock and do not sleep. Metric semantics are recorded
in [ADR 0005](decisions/0005-core-metrics-and-sessions.md).

### `typepulse-platform-desktop`

The desktop boundary exposes one privacy-safe monitor and focused-display API.
Its target-gated macOS adapter owns Input Monitoring, the passive event tap,
private key filtering, and optional focused-window geometry. Windows native
input is intentionally added behind the same API so the application and core
never branch on raw operating-system events.

The event tap is `ListenOnly`, runs on a dedicated thread, and sends bounded
`TypingActivity` messages without blocking the callback. Auto-repeat and
navigation/function input are removed privately. The repetition guard may
compare ephemeral key codes, but it emits only a monotonic occurrence time.

The Accessibility adapter reads only focused-window position and size. It
reduces them immediately to a center point used for display selection. It does
not read or expose an application name, window title, URL, or content.

### `typepulse-storage-sqlite`

The storage adapter implements the core repository contract with embedded,
ordered migrations. Its schema contains completed-session summaries, fixed
aggregate buckets, records, onboarding state, and preferences—never individual
input events.

Before a non-empty older database is migrated, the adapter creates a local
sibling backup. Local civil dates are assigned by the desktop layer so midnight
rollover is deterministic and historical days remain queryable.

### `src-tauri`

The composition root coordinates adapters, permissions, the single-instance
lifecycle, the menu bar, windows, overlay placement, commands, and local state.
It may combine prepared domain values, but WPM formulas do not belong here.

The permission gate is created before the normal shell. Missing required Input
Monitoring displays onboarding only; valid access creates the accessory shell
and starts monitoring. Runtime revocation stops live components immediately.

The login-item preference is independent of the current monitor state. Startup
reconciles the persisted preference with the real LaunchAgent only after the
required permission has been validated.

### `src`

The frontend displays prepared DTOs and sends user intentions through a narrow
Tauri command surface. It cannot query SQLite or receive raw input data.

## Runtime and concurrency

- the macOS event-tap callback performs only filtering, timestamp creation,
  atomic counters, and non-blocking channel delivery;
- a Rust relay owns the engine and database writes;
- the WebView consumes aggregate snapshots and does not drive metric meaning;
- overlay visibility is delayed until the third accepted activity;
- after activity stops, Pip breathes until the configured 1–15 second delay has
  elapsed, while the session itself closes under its separate timeout;
- a single-instance guard ensures onboarding relaunch, login launch, and manual
  open converge on one process.

## Data and permission boundaries

| Boundary                  | Allowed                                        | Forbidden                                |
| ------------------------- | ---------------------------------------------- | ---------------------------------------- |
| Platform → core           | monotonic occurrence time                      | key code, character, text, active app    |
| Core → UI                 | WPM, bands, records, aggregate session state   | raw activities or focused-window data    |
| Core → storage            | sessions, minute buckets, records, preferences | per-key rows or reconstructable text     |
| Accessibility → placement | temporary window center point                  | title, value, URL, app identity          |
| Application → network     | explicit release/update navigation only        | telemetry or automatic statistics upload |

## Failure behavior

- no Input Monitoring: show the permission gate, do not create the normal shell;
- permission denial or onboarding timeout: stop and exit cleanly;
- Accessibility unavailable: place Pip on the primary-display fallback;
- Secure Input active: accept missing activity rather than bypass protection;
- slow metric consumer: drop bounded activity messages instead of blocking input;
- migration failure: return a categorized error and retain the source database;
- invalid login setup: remove the stale LaunchAgent and clear its preference.

## Dependency direction

```text
frontend
   ↓ commands/events
src-tauri ──→ platform-macos
   │                │
   ├──→ storage     │
   └────────────────┴──→ core
```

Dependencies must point toward the portable core, never from the core toward a
platform or UI. See the complete [decision log](decisions/README.md) before
changing a system boundary.
