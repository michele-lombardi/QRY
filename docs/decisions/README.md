# Architecture decision records

Architecture decision records explain choices that are difficult or costly to
reverse. Accepted decisions apply to new contributions unless a later ADR
explicitly supersedes them.

| ADR                                             | Decision                                                        |
| ----------------------------------------------- | --------------------------------------------------------------- |
| [0001](0001-desktop-stack.md)                   | Tauri, Rust, and a small web frontend for the desktop stack.    |
| [0002](0002-input-privacy-boundary.md)          | Discard keyboard identity at the platform boundary.             |
| [0003](0003-local-storage.md)                   | Keep SQLite behind a portable repository interface.             |
| [0004](0004-macos-input-monitor.md)             | Use a passive macOS event tap with bounded delivery.            |
| [0005](0005-core-metrics-and-sessions.md)       | Define deterministic WPM, session, and record semantics.        |
| [0006](0006-local-day-and-automatic-startup.md) | Use local-day rollover and a user-controlled login item.        |
| [0007](0007-focused-display-accessibility.md)   | Reduce focused-window accessibility data to temporary geometry. |
| [0008](0008-permission-gated-lifecycle.md)      | Gate the runtime on required permission and relaunch cleanly.   |

Use the next four-digit sequence for a new record. Include context, decision,
privacy impact, and consequences. Do not silently rewrite an accepted decision
after implementation; add a superseding ADR instead.
