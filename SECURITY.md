# Security policy

## Supported versions

TypePulse has no public release yet. Security fixes currently apply to the
`main` branch only.

## Reporting a vulnerability

Do not publish sensitive exploit details in a public issue.

TODO before the repository becomes public:

1. enable GitHub private vulnerability reporting;
2. add the final repository URL to this document;
3. add an optional private security email controlled by the maintainer;
4. define response targets after the first stable release.

Until private reporting is configured, create a public issue containing only
the phrase “Security contact requested” and no technical details. A maintainer
must then provide a private channel before receiving the report.

## Sensitive areas

Changes to the following components require explicit privacy and security
review:

- global keyboard monitoring;
- macOS permissions and event taps;
- logs and diagnostics;
- SQLite schema and migrations;
- CSV export;
- Tauri capabilities and commands;
- release and Homebrew automation.

TypePulse must never persist or log individual keys, key codes, written text,
passwords, active applications, window titles or visited sites.

The local and CI quality gate runs `scripts/audit-privacy.sh` to verify the
aggregate-only schema/DTO boundary, narrow Tauri capability and absence of
unexpected runtime input logging. Dependency update checks are configured for
npm, Cargo and GitHub Actions through Dependabot; a private reporting channel
is still a release blocker because no final GitHub repository is configured.
