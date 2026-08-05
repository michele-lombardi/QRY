# Fase G — Qualità, sicurezza e release macOS

- Data avvio: 5 agosto 2026
- Stato: sorgenti pronti per beta 0.1.0; Gate stabile aperto
- Pubblicazioni esterne effettuate: nessuna

## Risultato implementato

### Qualità e privacy

- audit ripetibile integrato in `scripts/check.sh`;
- blocco di logging runtime non autorizzato nei layer input/app/UI;
- capability Tauri verificata esattamente come `core:default`;
- audit automatico delle colonne SQLite e dei DTO pubblici;
- test di date civili diverse per lo stesso istante in offset opposti;
- script CPU/RSS aggregato senza osservazione degli input;
- checklist per idle/typing, sleep/wake, logout/login, mezzanotte, multi-monitor,
  Gatekeeper e Homebrew;
- Dependabot settimanale per npm, Cargo e GitHub Actions.

### Release

- SemVer sincronizzato fra npm, Tauri e tutti i package Cargo;
- `CHANGELOG.md` e audit del tag/versione;
- firma ad-hoc Tauri (`signingIdentity = "-"`);
- packaging ripetibile per Apple Silicon e Intel;
- verifica `codesign`, ZIP e SHA-256 obbligatori;
- workflow tag → gate → due build → draft GitHub prerelease;
- guida Gatekeeper che non suggerisce di disabilitare le protezioni globali;
- template Homebrew Cask arch-aware e renderer con validazione degli hash.

## Stato task

| Task | Stato | Motivo residuo |
| --- | --- | --- |
| QA-01 | Done automatico | audit incluso nel gate |
| QA-02 | Ready, manuale | overlay pronto; servono TCC e baseline reale |
| QA-03 | Blocked | onboarding/revoca e prova lifecycle reale mancanti |
| QA-04 | Done automatico | smoke test mezzanotte naturale ancora manuale |
| QA-05 | Ready, manuale | controller pronto; servono due display reali |
| REL-01 | Done | SemVer, changelog e audit pronti |
| REL-02 | Done code | remote GitHub configurato; primo tag ancora da creare |
| REL-03 | Done nel packaging | verifica Intel ancora affidata a CI/hardware compatibile |
| REL-04 | Done | guida Gatekeeper disponibile |
| REL-05 | Template only | repository `homebrew-qry` ancora da creare |
| REL-06 | Blocked | servono URL e checksum di una Release reale |
| REL-07 | Blocked | dipendenze QA/Gate F non chiuse |
| REL-08 | Blocked | release candidate non pubblicata |

## Perché Gate G resta aperto

La pipeline non sostituisce il prodotto completo. Gate F, Gate E manuale e i
test TCC sono ancora aperti. `v0.1.0` può quindi essere pubblicata soltanto come
beta prerelease, non come V1 stabile.

## Verifiche locali eseguite

- `./scripts/check.sh`: pass, incluso audit privacy;
- 74 test Rust passati e un benchmark manuale ignorato;
- `./scripts/release-audit.sh v0.1.0`: metadati coerenti;
- YAML delle workflow e Dependabot: parsing riuscito;
- renderer Cask con owner/hash fittizi validi: sintassi Ruby riuscita;
- build release Apple Silicon: riuscita;
- firma ad-hoc: `codesign --verify --deep --strict` riuscito;
- archivio Apple Silicon: test ZIP e verifica SHA-256 riusciti;
- campionatore CPU/RSS: smoke test riuscito, ma non costituisce la baseline
  finale QA-02.

La build Intel non è stata prodotta localmente: il toolchain Rust installato via
Homebrew contiene soltanto la libreria target Apple Silicon e non include
`rustup`. La matrice CI installa esplicitamente entrambi i target; l'esito Intel
resta quindi TODO al primo tag eseguito su GitHub Actions.

## TODO che richiedono dati o autorità esterna

- repository pubblico `homebrew-qry`;
- consenso TCC e test lifecycle/risorse su macOS reale;
- Gate F completato e Gate E verificato manualmente;
- checksum dei due artefatti creati dalla prima Release;
- decisione umana di pubblicare release candidate e, successivamente, V1.
