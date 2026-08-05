# ADR 0001: Tauri, Rust and TypeScript desktop stack

- Status: Accepted
- Date: 2026-08-05

## Context

TypePulse targets macOS first and Linux later. The product needs a tray, a small
always-on-top overlay, local storage and a platform-specific global input adapter.
A Swift-only application would optimize the first platform but require a second
implementation for Linux.

## Decision

Use Tauri 2 for the desktop shell, Rust for the core and platform adapters, and
vanilla TypeScript/HTML/CSS for the small frontend. Keep the Tauri CLI as a local
npm dependency. Use a Cargo workspace to enforce module boundaries.

## Consequences

- WPM, sessions, aggregation and CSV can be reused on Linux.
- macOS input integration remains platform-specific.
- the UI uses the system WebView rather than SwiftUI.
- contributors need both Node and Rust toolchains.
- a framework UI is deferred until the vanilla frontend proves insufficient.
