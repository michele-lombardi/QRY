# ADR 0002: Discard keyboard identity at the platform boundary

- Status: Accepted
- Date: 2026-08-05

## Context

macOS exposes details that could be used to reconstruct what a person types.
TypePulse promises to measure rhythm without recording content. Relying only on
developer discipline would make accidental logging or persistence too easy.

## Decision

The macOS adapter determines whether an event counts as typing and immediately
discards character and key identity. Its public API emits only an occurrence
timestamp. The core, Tauri commands, frontend and storage have no raw-key type.

## Consequences

- tests can use synthetic activity timestamps without macOS permission;
- content cannot accidentally enter SQLite or frontend telemetry;
- filtering rules must remain within the platform adapter;
- features such as per-key heatmaps and accuracy are structurally unsupported;
- changes to this boundary require a new ADR and privacy review.
