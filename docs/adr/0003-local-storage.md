# ADR 0003: SQLite behind a repository boundary

- Status: Accepted and implemented
- Date: 2026-08-05

## Context

QRY stores sessions, short aggregate buckets and preferences locally. It
needs deterministic migrations and simple daily queries. The future Linux build
should share the same persistence model.

## Decision

Use SQLite through the dedicated `typepulse-storage-sqlite` crate. The portable
`typepulse-core` crate owns the `StatisticsRepository` contract and an in-memory
implementation for deterministic tests.

The concrete adapter uses `rusqlite` with the bundled SQLite library and
`rusqlite_migration`. Schema changes are ordered SQL files embedded in the
binary. Before upgrading a non-empty, older on-disk database, the adapter makes
a timestamped sibling `.bak` copy. A failed migration returns a categorized
error and never intentionally deletes the source database.

The first schema contains only:

- completed-session aggregates;
- fixed 60-second metric buckets;
- the singleton application preference for automatic startup.

Daily identity is the local civil date (`YYYY-MM-DD`) observed by the desktop
layer. Opening a new local date produces an empty current-day summary while
keeping older dates queryable. This is a rollover, not deletion.

## Consequences

- storage remains local and portable;
- schema changes require explicit migrations;
- the domain can be tested with an in-memory repository;
- the macOS adapter cannot write directly to the database;
- SQLite is included in the application build rather than assumed on the host;
- session and bucket writes occur on the metrics relay, never in the macOS
  event-tap callback;
- database backups may remain beside the database after a migration and are
  documented as local aggregate data.
