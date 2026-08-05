# Security policy

## Supported versions

Security fixes currently apply to the `0.1.x` beta line and the `main` branch.
Older development artifacts are unsupported.

## Reporting a vulnerability

Do not publish sensitive exploit details in a public issue.

Use [GitHub private vulnerability reporting](https://github.com/michele-lombardi/QRY/security/advisories/new).
Do not include vulnerability details in a public issue. During the beta,
reports are handled on a best-effort basis and no response-time SLA is offered.

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

QRY must never persist or log individual keys, key codes, written text,
passwords, active applications, window titles or visited sites.

The local and CI quality gate runs `scripts/audit-privacy.sh` to verify the
aggregate-only schema/DTO boundary, narrow Tauri capability and absence of
unexpected runtime input logging. Dependency update checks are configured for
npm, Cargo and GitHub Actions through Dependabot.
