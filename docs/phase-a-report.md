# Fase A — Report di completamento

- Data: 5 agosto 2026
- Ambito: repository e fondazioni
- Esito tecnico locale: completato
- Funzionalità di prodotto: intenzionalmente non incluse

## Risultato

QRY dispone ora di una base macOS compilabile e riproducibile con Tauri 2,
Rust e TypeScript. L'applicazione mostra una finestra diagnostica di fondazione,
ma non richiede permessi e non osserva ancora la tastiera.

Il Gate A ingegneristico è chiuso. La licenza, inizialmente lasciata aperta, è
stata successivamente scelta dal proprietario come `GPL-3.0-only` e sincronizzata
dopo la Fase C.

## Output prodotti

### Applicazione

- scaffold Tauri 2 generato dal template ufficiale vanilla TypeScript;
- frontend dimostrativo del template sostituito da una schermata QRY;
- plugin opener e relativo permesso rimossi perché non necessari;
- Content Security Policy esplicita;
- bundle target limitato all'app macOS nella fase corrente;
- CLI Tauri installata localmente tramite npm.

### Workspace Rust

- `typepulse-core`: confine portabile del dominio;
- `typepulse-platform-macos`: confine per permessi e input macOS;
- `typepulse-storage-sqlite`: confine per la persistenza locale;
- `src-tauri`: composition root dell'applicazione;
- lint condivisi e dipendenze orientate verso il core;
- lockfile Cargo unico per il workspace.

I crate contengono soltanto health check e test di collegamento. WPM, event tap e
SQLite non appartengono alla Fase A e non sono stati simulati con implementazioni
finte.

### Qualità

- Prettier per formattazione frontend e configurazioni;
- ESLint con regole JavaScript/TypeScript;
- TypeScript in modalità strict;
- `rustfmt` e Clippy con warning trattati come errori;
- test Rust di ogni confine architetturale;
- script locale unico `scripts/check.sh`;
- workflow GitHub Actions su runner macOS.

### Collaborazione

- contribution guide;
- code of conduct;
- security policy;
- template issue bug e feature;
- template pull request con checklist privacy;
- configurazione consigliata VS Code;
- ADR per stack, confine privacy e storage.

## Verifiche eseguite

Ambiente locale:

```text
Rust 1.95.0
Cargo 1.95.0
Node.js 24.6.0
npm 11.5.1
Tauri 2.11.5 risolto dal lockfile Cargo
Vite 6.4.3 risolto dal lockfile npm
```

Comando completo:

```bash
./scripts/check.sh
```

Esito:

- Prettier: passato;
- ESLint: passato;
- TypeScript e build Vite: passati;
- `cargo fmt --all --check`: passato;
- Clippy workspace con `-D warnings`: passato;
- test workspace: 3 passati, 0 falliti;
- `cargo check --workspace --all-targets`: passato;
- audit npm: 0 vulnerabilità note al momento dell'installazione.

Avvio applicazione:

```bash
npm run tauri dev
```

Vite ha aperto la porta 1420, Cargo ha compilato i quattro package del workspace
e il binario `typepulse-app` è stato avviato. Il processo di sviluppo è stato poi
terminato senza lasciare processi Vite o QRY in background.

Bundle prodotto:

```text
QRY/target/debug/bundle/macos/QRY.app
```

Il bundle è un artefatto locale di debug ed è correttamente escluso da Git. Non è
una release e non deve essere pubblicato come tale.

## Stato dei task

| Task | Esito |
| --- | --- |
| FND-01 | completato successivamente: GPL-3.0-only |
| FND-02 | completato |
| FND-03 | completato e avviato localmente |
| FND-04 | completato |
| FND-05 | completato |
| FND-06 | completato |
| FND-07 | workflow definito; esecuzione remota attende il primo push |
| FND-08 | completato |
| FND-09 | completato |

## TODO aperti intenzionalmente

### Completato per la beta pubblica

- copyright e repository compilati in `NOTICE.md`;
- URL Security Advisories configurato;
- repository remoto definito;
- identità QRY e icone applicate.

### Prima della prima release

- eseguire le checklist manuali TCC e Gatekeeper sugli artefatti beta;
- verificare la prima CI e la build Intel.

### Fasi successive

- monitoraggio Input Monitoring ed event tap: Fase B;
- motore WPM e sessioni: Fase C;
- SQLite e statistiche: Fase D;
- tray e overlay: Fase E;
- onboarding e impostazioni: Fase F;
- GitHub Release e Homebrew: Fase G;
- Linux/X11: Fase H.

## Stato successivo

Le Fasi B–G sono state implementate dopo questo report. Monitor, core metrico,
persistenza, tray, overlay e interfacce sono integrati; le prove TCC, login item,
Gatekeeper e multi-monitor restano checklist manuali sul Mac del proprietario.
