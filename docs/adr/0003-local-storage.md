# ADR 0003: SQLite behind a repository boundary

- Status: Accepted in principle; crate selection remains TODO
- Date: 2026-08-05

## Context

TypePulse stores sessions, short aggregate buckets and preferences locally. It
needs deterministic migrations and simple daily queries. The future Linux build
should share the same persistence model.

## Decision

Use SQLite through a dedicated Rust crate. Define the repository interface in
the portable domain layer and implement it in `typepulse-storage-sqlite`. Choose
the concrete SQLite library during Phase D after evaluating Tauri packaging,
migration support and maintenance.

## Consequences

- storage remains local and portable;
- schema changes require explicit migrations;
- the domain can be tested with an in-memory repository;
- the macOS adapter cannot write directly to the database;
- selecting the Rust SQLite crate remains a documented TODO.
