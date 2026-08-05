# Working plan

## 1. Obiettivo operativo

Realizzare una prima release open source di TypePulse per macOS che:

1. vive nella menu bar;
2. rileva attività di scrittura globale con consenso esplicito;
3. non conserva tasti o testo;
4. mostra WPM e animazione in un overlay discreto;
5. salva solo sessioni e aggregati locali;
6. mostra statistiche essenziali;
7. esporta un riepilogo CSV;
8. può essere installata da un tap Homebrew personale.

Linux viene preparato a livello architetturale, ma implementato soltanto dopo
una release macOS stabile.

## 2. Regole del piano

### Stato al 5 agosto 2026

- specifica, architettura, privacy, distribuzione e roadmap: completate;
- Fase A: implementata e verificata localmente;
- licenza open source: `TODO`, decisione del proprietario necessaria;
- funzionalità di prodotto: non iniziata, come previsto dal Gate A;
- prossimo lavoro: fase B, spike sul monitoraggio globale macOS.

### Priorità

- **P0:** indispensabile per la V1.
- **P1:** importante, ma può essere semplificato per la prima release.
- **P2:** miglioramento successivo.

### Dimensioni

- **S:** modifica piccola e circoscritta.
- **M:** task completo di sviluppo e test.
- **L:** task con integrazione di sistema o più componenti.
- **Spike:** esplorazione limitata nel tempo che deve produrre una decisione.

Le dimensioni sono relative, non equivalgono a una promessa in ore o giorni.
Ogni task `L` va diviso se durante l'implementazione emergono due risultati
verificabili indipendenti.

### Stati

Ogni issue GitHub usa uno di questi stati:

```text
backlog → ready → in progress → review → done
                         ↘ blocked
```

Un task entra in `ready` solo quando dipendenze e criterio di accettazione sono
chiari.

## 3. Struttura di sviluppo prevista

```text
progetto1/
├── README.md
├── LICENSE
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── SECURITY.md
├── docs/
│   ├── product-spec.md
│   ├── architecture.md
│   ├── privacy.md
│   ├── distribution.md
│   ├── roadmap.md
│   ├── working-plan.md
│   └── adr/
│       ├── 0001-desktop-stack.md
│       ├── 0002-input-privacy-boundary.md
│       └── 0003-local-storage.md
├── TypePulse/
│   ├── Cargo.toml                     # Workspace Rust
│   ├── Cargo.lock
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   ├── index.html
│   ├── src/                           # Frontend TypeScript
│   │   ├── main.ts
│   │   ├── api/                       # Wrapper dei comandi/eventi Tauri
│   │   ├── components/
│   │   │   ├── overlay/
│   │   │   ├── menu-bar/
│   │   │   ├── statistics/
│   │   │   ├── settings/
│   │   │   └── onboarding/
│   │   ├── stores/                    # Stato UI, senza logica WPM
│   │   ├── styles/                    # Token, temi e layout
│   │   └── types/                     # DTO scambiati con Rust
│   ├── public/
│   │   ├── icons/
│   │   └── animation/
│   ├── src-tauri/                     # Composizione applicazione
│   │   ├── Cargo.toml
│   │   ├── build.rs
│   │   ├── tauri.conf.json
│   │   ├── capabilities/
│   │   │   └── default.json
│   │   ├── Entitlements.plist
│   │   └── src/
│   │       ├── main.rs
│   │       ├── lib.rs
│   │       ├── app_state.rs
│   │       ├── commands/
│   │       │   ├── export.rs
│   │       │   ├── permissions.rs
│   │       │   ├── settings.rs
│   │       │   └── statistics.rs
│   │       └── desktop/
│   │           ├── overlay.rs
│   │           └── tray.rs
│   ├── crates/
│   │   ├── typepulse-core/            # Dominio portabile
│   │   │   ├── Cargo.toml
│   │   │   └── src/
│   │   │       ├── lib.rs
│   │   │       ├── activity.rs
│   │   │       ├── clock.rs
│   │   │       ├── metrics/
│   │   │       │   ├── rolling_wpm.rs
│   │   │       │   └── smoothing.rs
│   │   │       ├── session/
│   │   │       │   ├── engine.rs
│   │   │       │   └── state.rs
│   │   │       ├── summary.rs
│   │   │       └── export.rs
│   │   ├── typepulse-storage-sqlite/  # Persistenza locale
│   │   │   ├── Cargo.toml
│   │   │   ├── migrations/
│   │   │   └── src/lib.rs
│   │   └── typepulse-platform-macos/  # API e permessi macOS
│   │       ├── Cargo.toml
│   │       └── src/
│   │           ├── lib.rs
│   │           ├── event_filter.rs
│   │           ├── monitor.rs
│   │           └── permissions.rs
│   └── tests/
│       ├── fixtures/
│       └── manual/
│           ├── input-monitoring.md
│           ├── multi-monitor.md
│           └── release-smoke-test.md
├── packaging/
│   ├── homebrew/
│   │   └── typepulse.rb               # Template sincronizzato nel tap
│   └── debian/                        # Creato solo nella fase Linux
├── scripts/
│   ├── check.sh
│   ├── build-macos.sh
│   └── update-homebrew-cask.sh
└── .github/
    ├── ISSUE_TEMPLATE/
    │   ├── bug.yml
    │   └── feature.yml
    ├── pull_request_template.md
    └── workflows/
        ├── ci.yml
        └── release-macos.yml
```

### Responsabilità dei livelli

| Livello | Può conoscere | Non deve conoscere |
| --- | --- | --- |
| `typepulse-core` | eventi astratti, tempo, sessioni, metriche | Tauri, WebView, macOS, SQLite |
| `platform-macos` | API macOS, permessi, filtro eventi | UI, statistiche, SQL |
| `storage-sqlite` | modelli persistibili, migrazioni | eventi grezzi, UI, permessi |
| `src-tauri` | tutti gli adapter e il ciclo di vita | formule WPM duplicate |
| frontend | DTO, comandi ed eventi Tauri | key code, callback globali, query SQL |

### Regola privacy strutturale

L'adapter macOS può osservare l'evento solo per decidere se conta come attività
di scrittura. Verso il core invia esclusivamente:

```text
TypingActivity { occurred_at }
```

Non esiste un campo per carattere, key code, applicazione o finestra attiva. Il
tipo stesso deve rendere difficile una futura persistenza accidentale del
contenuto.

## 4. Sequenza e dipendenze

```text
Fondazioni
   ↓
Spike macOS ───────────────┐
   ↓                       │
Core metriche              │
   ↓                       │
Sessioni + storage ←───────┘
   ↓
Tray + overlay
   ↓
Statistiche + settings + onboarding
   ↓
Hardening privacy e prestazioni
   ↓
GitHub Release
   ↓
Homebrew Cask
   ↓
Linux/X11
```

Lo spike macOS è anticipato perché il monitoraggio globale è il rischio tecnico
maggiore. Non si costruisce l'intera UI prima di sapere che il segnale necessario
è ottenibile in modo affidabile e rispettoso della privacy.

## 5. Piano dettagliato dei task

### Fase A — Repository e fondazioni

| ID | Task | Prio | Taglia | Dipende da | Criterio di accettazione |
| --- | --- | --- | --- | --- | --- |
| FND-01 | Scegliere licenza open source | P0 | S | — | `LICENSE` presente; README e package metadata coerenti |
| FND-02 | Aggiungere contribution guide, codice di condotta e security policy | P1 | S | FND-01 | un contributor può fare setup/PR e segnalare vulnerabilità in modo corretto |
| FND-03 | Creare scaffold Tauri 2 con TypeScript | P0 | M | FND-01 | `npm run tauri dev` apre l'app su macOS |
| FND-04 | Convertire `TypePulse/` in Cargo workspace | P0 | M | FND-03 | tutti i crate compilano con un solo comando |
| FND-05 | Creare crate core, storage e macOS vuoti | P0 | S | FND-04 | dipendenze rispettano la tabella dei livelli |
| FND-06 | Configurare formatting e lint | P0 | S | FND-03 | `cargo fmt --check`, `cargo clippy` e controllo frontend passano |
| FND-07 | Creare workflow CI macOS | P0 | M | FND-06 | push e PR eseguono build, lint e test |
| FND-08 | Scrivere ADR iniziali | P1 | S | FND-04 | stack, confine privacy e SQLite risultano motivati e datati |
| FND-09 | Aggiungere template issue/PR | P1 | S | FND-02 | bug e feature raccolgono versione, OS e passi riproducibili |

**Gate A:** repository compilabile, CI verde, nessuna funzionalità di prodotto
ancora richiesta.

Stato Fase A:

| Task | Stato | Nota |
| --- | --- | --- |
| FND-01 | TODO | placeholder `LICENSE`; scelta OSI riservata al proprietario |
| FND-02 | Done | contribution guide, condotta e security policy presenti |
| FND-03 | Done | Tauri 2 + Vite + TypeScript avviabili tramite npm |
| FND-04 | Done | workspace Cargo unico in `TypePulse/Cargo.toml` |
| FND-05 | Done | core, storage e adapter macOS compilano separatamente |
| FND-06 | Done | Prettier, ESLint, TypeScript, rustfmt e Clippy configurati |
| FND-07 | Done | workflow CI macOS definito; esecuzione remota dopo il push |
| FND-08 | Done | ADR stack, privacy boundary e storage presenti |
| FND-09 | Done | template bug, feature e pull request presenti |

Il Gate A locale è chiuso. La verifica remota del workflow resta naturalmente
in attesa della pubblicazione e del primo push su GitHub.

### Fase B — Spike monitoraggio macOS

| ID | Task | Prio | Taglia | Dipende da | Criterio di accettazione |
| --- | --- | --- | --- | --- | --- |
| MAC-01 | Valutare binding Rust per API macOS | P0 | Spike | FND-05 | nota tecnica confronta binding diretto e bridge minimo |
| MAC-02 | Leggere lo stato del permesso Input Monitoring | P0 | M | MAC-01 | stato `granted/denied/unknown` verificabile manualmente |
| MAC-03 | Aprire le impostazioni macOS corrette | P0 | S | MAC-02 | pulsante diagnostico apre la sezione prevista |
| MAC-04 | Creare event tap globale sperimentale | P0 | L | MAC-01 | eventi ricevuti fuori dall'app senza bloccare la tastiera |
| MAC-05 | Prototipare il filtro degli eventi | P0 | M | MAC-04 | modificatori e shortcut non aumentano il conteggio |
| MAC-06 | Eliminare key code al confine dell'adapter | P0 | S | MAC-05 | API pubblica emette soltanto `TypingActivity` |
| MAC-07 | Verificare revoca e ripristino permesso | P0 | M | MAC-02, MAC-04 | il monitor si ferma e riparte senza crash |
| MAC-08 | Verificare Secure Input e casi non osservabili | P1 | Spike | MAC-04 | comportamento documentato senza tentativi di aggiramento |
| MAC-09 | Misurare overhead del callback | P0 | M | MAC-05 | nessun I/O o lavoro UI nel callback; misura di riferimento salvata |

**Gate B:** TypePulse conta attività globale su macOS senza esporre o registrare
contenuto. Se questo gate fallisce, si ferma lo sviluppo UI e si rivaluta
l'approccio tecnico.

### Fase C — Core delle metriche

| ID | Task | Prio | Taglia | Dipende da | Criterio di accettazione |
| --- | --- | --- | --- | --- | --- |
| CORE-01 | Definire `TypingActivity` e clock iniettabile | P0 | S | FND-05 | test possono avanzare il tempo senza attese reali |
| CORE-02 | Implementare finestra mobile da 10 secondi | P0 | M | CORE-01 | casi vuoto, lento, veloce e burst hanno test |
| CORE-03 | Implementare formula WPM a 5 caratteri | P0 | S | CORE-02 | esempi noti producono risultati attesi |
| CORE-04 | Implementare smoothing configurabile | P0 | M | CORE-03 | valore stabile senza ritardo eccessivo, con test |
| CORE-05 | Definire fasce animazione | P0 | S | CORE-03 | soglie 30/60/90 gestite ai confini |
| CORE-06 | Creare state machine della sessione | P0 | L | CORE-01 | idle, active, overlay timeout e session timeout testati |
| CORE-07 | Calcolare media, picco e tempo attivo | P0 | M | CORE-06 | aggregati riproducibili da una sequenza finta |
| CORE-08 | Rilevare nuovo record | P1 | S | CORE-07 | celebrazione emessa una sola volta per record |
| CORE-09 | Rendere parametri configurabili | P1 | S | CORE-02, CORE-04, CORE-06 | default V1 centralizzati e validati |
| CORE-10 | Aggiungere property test o casi limite estesi | P1 | M | CORE-07 | nessun NaN, overflow o WPM negativo |

**Gate C:** il core riceve solo timestamp e produce tutto lo stato necessario
all'interfaccia con test deterministici.

### Fase D — Persistenza e aggregazioni

| ID | Task | Prio | Taglia | Dipende da | Criterio di accettazione |
| --- | --- | --- | --- | --- | --- |
| DB-01 | Definire trait del repository nel core | P0 | S | CORE-07 | core testabile con repository in memoria |
| DB-02 | Disegnare schema SQLite iniziale | P0 | M | DB-01 | schema contiene solo sessioni, bucket e preferenze necessarie |
| DB-03 | Creare prima migrazione | P0 | M | DB-02 | database nuovo e aggiornato arrivano alla stessa versione |
| DB-04 | Salvare sessioni concluse | P0 | M | DB-03, CORE-06 | riavvio dell'app conserva il riepilogo |
| DB-05 | Salvare bucket da 30/60 secondi | P0 | M | DB-03, CORE-07 | nessun evento individuale è presente nel database |
| DB-06 | Implementare riepilogo giornaliero | P0 | M | DB-04, DB-05 | parole, media, picco e minuti corrispondono ai fixture |
| DB-07 | Implementare ultimi sette giorni | P0 | S | DB-06 | giorni mancanti e cambio data gestiti |
| DB-08 | Implementare reset di oggi | P1 | M | DB-06 | elimina solo dati del giorno locale selezionato |
| DB-09 | Implementare export CSV | P0 | M | DB-06 | header, locale, escaping e ordinamento sono testati |
| DB-10 | Definire strategia backup/migrazione fallita | P1 | Spike | DB-03 | errore non distruttivo e messaggio UI documentati |

**Gate D:** statistiche e CSV sopravvivono al riavvio; ispezionando SQLite non
si trovano tasti, testo, app o titoli di finestre.

### Fase E — Shell Tauri, tray e overlay

| ID | Task | Prio | Taglia | Dipende da | Criterio di accettazione |
| --- | --- | --- | --- | --- | --- |
| APP-01 | Configurare app senza finestra principale all'avvio | P0 | M | FND-03 | avvio silenzioso e nessuna finestra grande |
| APP-02 | Nascondere app da Dock e `Cmd + Tab` | P0 | M | APP-01 | comportamento verificato dopo avvio e riapertura |
| APP-03 | Creare icona tray e menu minimo | P0 | M | APP-01 | menu permette pausa, settings e quit |
| APP-04 | Collegare monitor, core e stato applicazione | P0 | L | MAC-07, CORE-07 | evento reale aggiorna uno stato diagnostico |
| OVR-01 | Creare finestra overlay trasparente | P0 | L | APP-01 | niente bordo, always-on-top e click-through |
| OVR-02 | Implementare quattro posizioni | P0 | M | OVR-01 | coordinate corrette su display principale |
| OVR-03 | Gestire multi-monitor e cambio display | P1 | L | OVR-02 | overlay ricollocato se un display scompare |
| OVR-04 | Implementare fade-in e fade-out | P0 | M | OVR-01, CORE-06 | transizioni rispettano timeout e non rubano focus |
| OVR-05 | Mostrare WPM live | P0 | S | APP-04, OVR-01 | valore UI segue lo stato smussato del core |
| OVR-06 | Implementare quattro stati animati | P0 | M | CORE-05, OVR-01 | ogni fascia produce lo stato visuale corretto |
| OVR-07 | Aggiungere celebrazione record | P1 | M | CORE-08, OVR-06 | effetto breve, non ripetuto continuamente |
| OVR-08 | Implementare dimensioni e contenuti configurabili | P1 | M | OVR-05, OVR-06 | small/medium/large e modalità contenuto funzionano |

**Gate E:** scrivendo in un'altra applicazione, overlay e tray funzionano senza
rubare focus o intercettare click.

### Fase F — Schermate e preferenze

| ID | Task | Prio | Taglia | Dipende da | Criterio di accettazione |
| --- | --- | --- | --- | --- | --- |
| UI-01 | Definire token visuali e tema | P1 | S | FND-03 | colori, spaziature e tipografia non sono duplicati |
| UI-02 | Completare pannello tray | P0 | M | APP-03, DB-06 | mostra riepilogo di oggi e azioni previste |
| UI-03 | Creare finestra statistiche | P0 | L | DB-06, DB-07 | quattro metriche e ultimi sette giorni visibili |
| UI-04 | Creare grafico giornaliero | P1 | L | DB-05 | bucket visualizzati con assi leggibili |
| UI-05 | Creare settings General | P0 | M | APP-03 | pausa e preferenze persistono |
| UI-06 | Creare settings Overlay | P0 | M | OVR-08 | posizione, dimensione, hide-after e contenuto applicati live |
| UI-07 | Creare settings Appearance | P1 | S | UI-01 | System, Light e Dark funzionano |
| UI-08 | Implementare avvio al login | P1 | M | UI-05 | attivazione e disattivazione sono idempotenti |
| UI-09 | Creare onboarding in tre passaggi | P0 | L | MAC-03, MAC-07 | primo avvio spiega privacy e permesso |
| UI-10 | Gestire permesso negato/revocato | P0 | M | UI-09, MAC-07 | stato chiaro, nessun dato simulato |
| UI-11 | Aggiungere dialog export CSV | P0 | M | DB-09 | utente sceglie destinazione e riceve esito |
| UI-12 | Verificare accessibilità di base | P1 | M | UI-02–UI-11 | tastiera, contrasto e reduced motion verificati |

**Gate F:** un nuovo utente può installare, concedere il permesso, usare TypePulse
e comprendere i dati senza istruzioni esterne.

### Fase G — Qualità, sicurezza e release macOS

| ID | Task | Prio | Taglia | Dipende da | Criterio di accettazione |
| --- | --- | --- | --- | --- | --- |
| QA-01 | Audit log e database | P0 | M | DB-09, APP-04 | nessun key code, testo, app o titolo finestra presente |
| QA-02 | Profilare CPU e memoria idle/typing | P0 | M | OVR-06 | baseline documentata e nessun loop continuo inutile |
| QA-03 | Test sospensione, logout e riavvio | P0 | M | UI-10 | sessioni chiuse o riprese coerentemente |
| QA-04 | Test timezone e cambio giorno | P1 | M | DB-07 | riepiloghi non si mescolano tra date |
| QA-05 | Eseguire smoke test multi-monitor | P1 | M | OVR-03 | checklist manuale completata |
| REL-01 | Definire versioning e changelog | P0 | S | FND-07 | tag SemVer genera note coerenti |
| REL-02 | Creare workflow release macOS | P0 | L | REL-01, Gate F | tag produce artefatto e checksum |
| REL-03 | Applicare firma ad-hoc dove utile | P1 | M | REL-02 | comportamento documentato su Apple Silicon e Intel |
| REL-04 | Scrivere istruzioni Gatekeeper | P0 | S | REL-02 | utente comprende il limite delle build senza Developer ID |
| REL-05 | Creare tap `homebrew-typepulse` | P0 | M | REL-02 | repository tap pubblico e installabile |
| REL-06 | Creare e validare cask | P0 | M | REL-05 | install, upgrade e uninstall provati su macchina pulita |
| REL-07 | Pubblicare release candidate | P0 | M | QA-01–QA-05, REL-06 | checklist completa e problemi bloccanti chiusi |
| REL-08 | Pubblicare V1 macOS | P0 | S | REL-07 | tag stabile, release, checksum e cask disponibili |

**Gate G:** un utente può installare TypePulse dal tap personale, completare
l'onboarding, usarlo e disinstallarlo seguendo la documentazione pubblica.

### Fase H — Linux dopo la V1

| ID | Task | Prio | Taglia | Dipende da | Criterio di accettazione |
| --- | --- | --- | --- | --- | --- |
| LNX-01 | Aggiungere build/test core su Linux CI | P0 | S | REL-08 | core e storage compilano senza codice macOS |
| LNX-02 | Definire distribuzione Linux di riferimento | P0 | Spike | LNX-01 | distro, desktop e sessione X11 dichiarati |
| LNX-03 | Creare crate `platform-linux` | P0 | M | LNX-02 | implementa la stessa API astratta del monitor macOS |
| LNX-04 | Implementare monitor X11 | P0 | L | LNX-03 | attività globale conteggiata senza contenuto persistito |
| LNX-05 | Adattare tray e overlay | P0 | L | LNX-03 | comportamento verificato sul desktop scelto |
| LNX-06 | Verificare privacy e privilegi | P0 | M | LNX-04 | nessun requisito `root` o accesso diretto invasivo ai device |
| LNX-07 | Generare `.deb` | P0 | M | LNX-05 | installazione e rimozione con APT locale funzionano |
| LNX-08 | Pubblicare release Linux sperimentale | P1 | M | LNX-06, LNX-07 | limiti X11/Wayland chiaramente dichiarati |
| LNX-09 | Studiare Wayland per compositor | P1 | Spike | LNX-08 | matrice GNOME/KDE/portal con esito fattibile o non supportato |
| LNX-10 | Creare repository APT firmato | P2 | L | release Linux stabili | update e verifica firma funzionano |

## 6. Ordine consigliato per le prime iterazioni

### Iterazione 1 — App vuota ma sana

Task: `FND-01` → `FND-07`.

Risultato: repository open source compilabile, app Tauri avviabile e CI verde.

### Iterazione 2 — Riduzione del rischio principale

Task: `MAC-01` → `MAC-09`.

Risultato: prova tecnica del monitoraggio globale con confine privacy verificato.

### Iterazione 3 — Motore testato

Task: `CORE-01` → `CORE-09`.

Risultato: WPM e sessioni funzionano su eventi simulati, senza UI.

### Iterazione 4 — Primo percorso end-to-end

Task: `APP-01` → `APP-04`, `OVR-01`, `OVR-04`, `OVR-05`.

Risultato: scrivere in un'altra app fa comparire un numero WPM reale.

### Iterazione 5 — MVP visuale

Task: resto della fase E.

Risultato: overlay completo, posizionabile e reattivo.

### Iterazione 6 — Dati e interfacce

Task: fase D, poi fase F.

Risultato: statistiche, preferenze, onboarding ed export.

### Iterazione 7 — Release

Task: fase G.

Risultato: prima V1 macOS installabile da Homebrew.

## 7. Strategia Git e GitHub

- branch principale: `main`, sempre compilabile;
- branch brevi: `feat/<issue>-descrizione`, `fix/<issue>-descrizione`;
- una issue corrisponde preferibilmente a un task del piano;
- PR piccole, con test e nota privacy quando toccano input o persistenza;
- commit descrittivi, senza imporre squash durante il lavoro locale;
- tag release secondo SemVer: `v0.1.0`, `v0.2.0`, `v1.0.0`;
- release candidate: `v1.0.0-rc.1`.

Label GitHub suggerite:

```text
area:core
area:macos
area:frontend
area:storage
area:release
area:linux
privacy
good first issue
blocked
```

## 8. Definition of Done globale

Un task è `done` quando:

- il criterio di accettazione specifico è verificato;
- formatting, lint, build e test passano;
- non introduce dati vietati dai documenti privacy;
- errori e stati degradati sono gestiti;
- la documentazione interessata è aggiornata;
- non lascia codice diagnostico che registra eventi di input;
- la PR può essere compresa senza dipendere da conversazioni esterne.

Per una feature UI si aggiungono:

- verifica Light e Dark quando applicabile;
- verifica con reduced motion quando usa animazioni;
- nessun furto del focus durante la scrittura;
- screenshot o breve registrazione nella PR.

Per una modifica al monitoraggio si aggiungono:

- test o checklist di permesso negato e revocato;
- conferma esplicita che key code e contenuto non attraversano l'adapter;
- misurazione dell'impatto sul callback globale.

## 9. Criteri di rilascio V1

La V1 macOS può essere pubblicata quando:

- tutti i task P0 delle fasi A–G sono completati;
- nessun bug aperto causa perdita dati, crash frequenti o blocco tastiera;
- il database contiene soltanto lo schema autorizzato;
- CPU idle e comportamento durante digitazione sono stati profilati;
- onboarding e revoca del permesso sono stati provati da zero;
- installazione e disinstallazione Homebrew sono state verificate;
- README, privacy e limitazioni Gatekeeper sono aggiornati;
- è disponibile una procedura di build dal sorgente riproducibile.

Non sono requisiti per la V1:

- Developer ID e notarizzazione;
- ingresso nel repository ufficiale `homebrew/cask`;
- Linux, Wayland o Windows;
- updater automatico;
- temi o personaggi multipli;
- analytics o telemetria.

## 10. Registro delle decisioni ancora necessarie

| Decisione | Quando serve | Default proposto |
| --- | --- | --- |
| Licenza | prima dello scaffold pubblico | MIT, da confermare |
| Versione minima macOS | prima di FND-03 | scegliere in base al Mac di sviluppo e alle API richieste |
| Frontend build tool | durante FND-03 | Vite + TypeScript senza framework UI |
| Binding macOS | durante MAC-01 | Rust diretto; bridge minimo solo se necessario |
| SQLite crate | prima di DB-03 | valutare semplicità, migrazioni e build Tauri |
| Bucket metriche | prima di DB-05 | 60 secondi per la V1 |
| Artefatto macOS | prima di REL-02 | `.app.tar.gz` o `.app.zip` per il cask |
| Architetture macOS | prima di REL-02 | Apple Silicon prima; Intel se la CI resta semplice |

Ogni decisione che cambia un confine architetturale viene registrata in un ADR,
non soltanto in una issue o in una conversazione.
