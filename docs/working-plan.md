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
- Fase B: codice, test e diagnostica implementati;
- Gate B end-to-end: `TODO manuale`, il Mac di sviluppo non ha ancora concesso
  Input Monitoring;
- Fase C: core metrico e Gate C completati con test deterministici;
- Fase D: persistenza, aggregazioni, CSV e Gate D completati;
- Fase E: shell e overlay `APP-01`–`OVR-09` implementati e testati nel codice;
  Gate E end-to-end resta una checklist manuale con consenso TCC;
- brand identity v0.1: nome, mark, icone, token e comportamenti Pip supportati
  integrati; Dance, Sleep, messaggi v2 e superfici finali hanno task dedicati;
- Fase F: avviata dal pannello menu bar e dal ciclo finestra in background;
- Fase G: automazione qualità/release avviata; audit privacy, SemVer, packaging,
  workflow draft Release e template cask implementati;
- rollover: automatico sulla data civile locale, con storico conservato;
- avvio automatico: preferenza e login item macOS implementati in anticipo;
- licenza open source: GNU GPLv3 only; restano placeholder anagrafici in
  `NOTICE.md`;
- prossimo lavoro: completare le schermate della Fase F e svolgere le checklist
  reali TCC/overlay. Gate G rimane aperto fino a queste dipendenze, alle prove
  manuali e alla configurazione del remote GitHub.

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
│       ├── 0003-local-storage.md
│       ├── 0004-macos-input-monitor.md
│       ├── 0005-core-metrics-and-sessions.md
│       ├── 0006-local-day-and-automatic-startup.md
│       └── 0007-focused-display-accessibility.md
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
│   │   │       ├── config.rs
│   │   │       ├── metrics/
│   │   │       │   ├── animation.rs
│   │   │       │   ├── rolling_wpm.rs
│   │   │       │   └── smoothing.rs
│   │   │       ├── session/
│   │   │       │   ├── engine.rs
│   │   │       │   └── model.rs
│   │   ├── typepulse-storage-sqlite/  # Persistenza locale
│   │   │   ├── Cargo.toml
│   │   │   ├── migrations/
│   │   │   └── src/lib.rs
│   │   └── typepulse-platform-macos/  # API e permessi macOS
│   │       ├── Cargo.toml
│   │       └── src/
│   │           ├── lib.rs
│   │           ├── event_filter.rs
│   │           ├── focused_window.rs
│   │           ├── monitor.rs
│   │           └── permissions.rs
│   └── tests/
│       ├── fixtures/
│       └── manual/
│           ├── focused-display.md
│           ├── input-monitoring.md
│           ├── phase-e-overlay.md
│           └── release-quality.md
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
| FND-01 | Done | GPL-3.0-only sincronizzata in LICENSE, npm, Cargo e documentazione |
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
| MAC-10 | Proteggere la metrica da ripetizioni artificiali — **Done** | P0 | M | MAC-05 | auto-repeat escluso; doppie lettere ammesse; terza pressione identica filtrata senza esporre key code |

**Gate B:** TypePulse conta attività globale su macOS senza esporre o registrare
contenuto. Se questo gate fallisce, si ferma lo sviluppo UI e si rivaluta
l'approccio tecnico.

Stato Fase B:

| Task | Stato | Evidenza / residuo |
| --- | --- | --- |
| MAC-01 | Done | ADR 0004; binding Rust diretti, nessun bridge Swift necessario |
| MAC-02 | Done code | preflight reale restituisce `denied`; passaggio a `granted` TODO proprietario |
| MAC-03 | Done | deep link eseguito localmente con exit code 0 |
| MAC-04 | Done code | tap `Session`/`ListenOnly`; ricezione esterna TODO dopo consenso TCC |
| MAC-05 | Done | filtro privato con test automatici su categorie e shortcut |
| MAC-06 | Done | API pubblica emette solo `TypingActivity` non serializzabile |
| MAC-07 | Done code | stop/revoca/re-enable implementati; prova revoca reale TODO manuale |
| MAC-08 | Done design | comportamento non invasivo documentato; prova Secure Input TODO manuale |
| MAC-09 | Done | benchmark release 12 ns/hot path; misura callback reale TODO manuale |
| MAC-10 | Done | guard atomica effimera e test auto-repeat/doppia lettera/streak/reset |

Il codice della Fase B è completo. Il Gate B end-to-end resta `TODO manuale`:
TCC è `denied` sul Mac di sviluppo e il permesso non può essere auto-concesso.
Le checklist sono in `TypePulse/tests/manual/` e il dettaglio è nel report di
Fase B.

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
| CORE-11 | Rendere reattivo il warm-up WPM — **Done** | P0 | M | CORE-02, CORE-04 | prima stima dopo ≥250 ms, lookback 10 s e limite 300 WPM testati |
| CORE-12 | Proteggere best score dai picchi warm-up — **Done** | P0 | M | CORE-11, CORE-08 | rampa iniziale 1 s e record qualificato dopo 3 s con regressione testata |

**Gate C:** il core riceve solo timestamp e produce tutto lo stato necessario
all'interfaccia con test deterministici.

Stato Fase C:

| Task | Stato | Evidenza |
| --- | --- | --- |
| CORE-01 | Done | `Clock`, `SystemClock`, `ManualClock`, `TypingActivity` |
| CORE-02 | Done | lookback 10 s con warm-up adattivo; test vuoto/lento/60 WPM/burst |
| CORE-03 | Done | formula a 5 attività per parola sulla durata realmente osservata |
| CORE-04 | Done | EMA configurabile, test di stabilizzazione e reset |
| CORE-05 | Done | fasce e confini esatti 30/60/90 testati |
| CORE-06 | Done | state machine idle/visible/hidden/completed deterministica |
| CORE-07 | Done | caratteri, parole, media, picco e tempo attivo riproducibili |
| CORE-08 | Done | record storico opzionale, evento una volta per sessione |
| CORE-09 | Done | default V1 centralizzati e configurazione validata |
| CORE-10 | Done | casi patologici e sequenza deterministica da 10.000 eventi |
| CORE-11 | Done | prima stima affidabile in 250–400 ms, cap 300 WPM, EMA dal primo campione valido |
| CORE-12 | Done | warm-up progressivo 1 s; media, picco e record esclusi fino a 3 s |

Il Gate C è chiuso. Semantica e verifiche sono descritte nell'ADR 0005 e nel
report di Fase C.

### Fase D — Persistenza e aggregazioni

| ID | Task | Stato | Prio | Taglia | Dipende da | Evidenza di completamento |
| --- | --- | --- | --- | --- | --- | --- |
| DB-01 | Definire trait del repository nel core | Done | P0 | S | CORE-07 | trait portabile e repository in memoria testato |
| DB-02 | Disegnare schema SQLite iniziale | Done | P0 | M | DB-01 | solo sessioni, bucket aggregati e preferenza necessaria |
| DB-03 | Creare prima migrazione | Done | P0 | M | DB-02 | migrazione embedded e `user_version = 1` testato |
| DB-04 | Salvare sessioni concluse | Done | P0 | M | DB-03, CORE-06 | idempotenza e riapertura database testate |
| DB-05 | Salvare bucket da 60 secondi | Done | P0 | M | DB-03, CORE-07 | upsert pesato; nessun evento individuale nello schema |
| DB-06 | Implementare riepilogo giornaliero | Done | P0 | M | DB-04, DB-05 | conteggi, parole, media, picco e tempo attivo testati |
| DB-07 | Implementare ultimi sette giorni | Done | P0 | S | DB-06 | sequenza cronologica con giorni mancanti vuoti |
| DB-08 | Implementare rollover automatico del giorno locale | Done | P1 | M | DB-06 | nuova data vuota senza cancellare lo storico; sessioni/bucket non mescolati |
| DB-09 | Implementare export CSV | Done | P0 | M | DB-06 | header, decimali e ordinamento stabili testati |
| DB-10 | Definire strategia backup/migrazione fallita | Done | P1 | Spike | DB-03 | copia `.bak` pre-migrazione ed errore categorizzato |
| DB-11 | Persistenza visibilità WPM menu bar — **Done** | P1 | S | DB-10, APP-03 | schema v3, default attivo e migrazione v1/v2 con backup testati |

**Gate D: chiuso.** Statistiche e preferenze sopravvivono alla riapertura;
l'audit automatico dello schema SQLite esclude tasti, testo, app e titoli di
finestre. La prova reale del login item resta nella checklist manuale.

### Fase E — Shell Tauri, tray e overlay

| ID | Task | Prio | Taglia | Dipende da | Criterio di accettazione |
| --- | --- | --- | --- | --- | --- |
| APP-01 | Configurare app senza finestra principale all'avvio — **Done** | P0 | M | FND-03 | finestra principale creata nascosta |
| APP-02 | Nascondere app da Dock e `Cmd + Tab` — **Done** | P0 | M | APP-01 | activation policy macOS `Accessory` e `skipTaskbar` |
| APP-03 | Creare icona tray e menu minimo — **Done** | P0 | M | APP-01 | click apre l'app; menu permette start, pausa e quit |
| APP-04 | Collegare monitor, core e stato applicazione — **Done anticipato in D** | P0 | L | MAC-07, CORE-07 | evento reale aggiorna stato diagnostico, bucket e sessioni |
| OVR-01 | Creare finestra overlay trasparente — **Done** | P0 | L | APP-01 | niente bordo, always-on-top e click-through |
| OVR-02 | Implementare quattro posizioni — **Done** | P0 | M | OVR-01 | coordinate corrette su area utile del display principale |
| OVR-03 | Gestire multi-monitor e cambio display — **Done nel codice; manuale aperto** | P1 | L | OVR-02 | topologia rivalutata e fallback al display principale |
| OVR-04 | Implementare fade-in e fade-out — **Done** | P0 | M | OVR-01, CORE-06 | transizioni rispettano timeout e non chiedono focus |
| OVR-05 | Mostrare WPM live — **Done** | P0 | S | APP-04, OVR-01 | valore UI segue lo stato smussato del core |
| OVR-06 | Implementare stati animati misurabili — **Done** | P0 | M | CORE-05, OVR-01 | fasce core tradotte in Walk/Run; Breathe, Tired e record gestiti esplicitamente |
| OVR-07 | Aggiungere celebrazione record — **Done** | P1 | M | CORE-08, OVR-06 | sequenza monotona ed effetto breve non ripetuto |
| OVR-08 | Implementare dimensioni e contenuti configurabili — **Done** | P1 | M | OVR-05, OVR-06 | tre dimensioni e tre modalità persistono e si applicano live |
| OVR-09 | Seguire il display della finestra focalizzata — **Done nel codice; manuale aperto** | P0 | L | OVR-03, MAC-03 | consenso Accessibilità separato, solo centro geometrico effimero, fallback principale e refresh ≤250 ms durante attività |
| OVR-10 | Conservare il display valido su errore AX transitorio — **Done nel codice; manuale aperto** | P0 | S | OVR-09 | monitor corrente precede il fallback principale e ha test di regressione |
| OVR-11 | Correggere salto diretto Retina → terzo display — **Done nel codice; manuale aperto** | P0 | S | OVR-09 | posizione logica usa la scala del target; layout 2×/1× con coordinate negative testato |

**Gate E: implementazione completata, prova reale aperta.** Test automatici
coprono coordinate, preset, DTO, fasce e persistenza. Le checklist manuali devono
confermare focus, click-through e cambio monitor con Input Monitoring e
Accessibilità concessi, oltre al fallback senza Accessibilità.

### Brand identity — integrazione e backlog

| ID | Task | Stato | Prio | Dipende da | Criterio di accettazione |
| --- | --- | --- | --- | --- | --- |
| BRD-01 | Adottare TypePulse e Pulse mark come identità canonica | Done | P0 | identity v0.1 | nome e geometria unici nel prodotto |
| BRD-02 | Generare icona app e glifi menu bar | Done | P0 | BRD-01 | template adattivi; titolo WPM nativo opzionale con slot fisso a tre cifre |
| BRD-03 | Applicare palette, SF Pro/system stack e voce | Done sulla UI corrente | P0 | BRD-01 | token e copy correnti rispettano l'identity |
| BRD-04 | Implementare anatomia Pip | Done | P0 | OVR-06 | cerchio, occhi, piedi; niente bocca, ombre o accessori |
| BRD-05 | Implementare Breathe, Walk, Run, Jump/Cheer e Tired | Done nel renderer | P0 | CORE-05, CORE-08 | trigger misurabili, record one-shot e tempo attivo aggregato |
| BRD-06 | Applicare pulse dinamico e reduced motion | Done | P1 | BRD-05 | 1.6–0.4 s; modalità ridotta senza moto continuo |
| BRD-07 | Misurare stabilità del ritmo e attivare Dance | Todo futuro | P1 | nuovo task core | trigger deterministico dopo oltre due minuti stabili |
| BRD-08 | Progettare Sleep a cinque minuti | Todo futuro | P1 | lifecycle/UI-02 | nessun conflitto con hide 2 s e fine sessione 30 s |
| BRD-09 | Trasformare il Pulse mark nel grafico live | Todo F | P1 | UI-04 | wave accessibile senza libreria chart |
| BRD-10 | Messaggi compagno, massimo tre al giorno | Todo v2 | P2 | UI-02 | solo popover, nessuna notifica di sistema |
| BRD-11 | Onboarding e insights conformi all'identity | Todo F | P0 | UI-03, UI-09 | token, Pip e voce applicati alle schermate finali |
| BRD-12 | Confermare endorsement legale Micro-Y nell'About | Todo owner | P1 | dato esterno | solo testo in-app, formulazione autorizzata |
| BRD-13 | Verificare nome, dominio e marchio TypePulse | Todo pre-release | P0 | ricerca/owner | decisione registrata prima della V1 |
| BRD-14 | Preparare sito, lockup e GIF prodotto | Todo futuro | P2 | sito pubblico | clear-space e motion identity rispettati |
| BRD-15 | Visual regression e contrast audit | Todo RC | P0 | Gate F | light/dark, tray, overlay e reduce motion verificati |

### Fase F — Schermate e preferenze

| ID | Task | Prio | Taglia | Dipende da | Criterio di accettazione |
| --- | --- | --- | --- | --- | --- |
| UI-01 | Definire token visuali e tema | P1 | S | FND-03 | colori, spaziature e tipografia non sono duplicati |
| UI-02 | Completare pannello tray — **In progress** | P0 | M | APP-03, DB-06 | shell e azioni pronte; riepilogo compatto ancora da integrare |
| UI-03 | Creare finestra statistiche | P0 | L | DB-06, DB-07 | quattro metriche e ultimi sette giorni visibili |
| UI-04 | Creare grafico giornaliero | P1 | L | DB-05 | bucket visualizzati con assi leggibili |
| UI-05 | Creare settings General | P0 | M | APP-03 | pausa e preferenze persistono |
| UI-06 | Creare settings Overlay | P0 | M | OVR-08 | posizione, dimensione, hide-after e contenuto applicati live |
| UI-07 | Creare settings Appearance | P1 | S | UI-01 | System, Light e Dark funzionano |
| UI-08 | Implementare avvio al login — **Done anticipato in D** | P1 | M | UI-05 | login item e monitor automatico seguono la stessa preferenza |
| UI-09 | Creare onboarding in tre passaggi | P0 | L | MAC-03, MAC-07, OVR-09 | primo avvio spiega privacy e i due permessi distinti |
| UI-10 | Gestire permesso negato/revocato | P0 | M | UI-09, MAC-07 | stato chiaro, nessun dato simulato |
| UI-11 | Aggiungere dialog export CSV | P0 | M | DB-09 | utente sceglie destinazione e riceve esito |
| UI-12 | Verificare accessibilità di base | P1 | M | UI-02–UI-11 | tastiera, contrasto e reduced motion verificati |

**Gate F:** un nuovo utente può installare, concedere il permesso, usare TypePulse
e comprendere i dati senza istruzioni esterne.

### Fase G — Qualità, sicurezza e release macOS

| ID | Task | Prio | Taglia | Dipende da | Criterio di accettazione |
| --- | --- | --- | --- | --- | --- |
| QA-01 | Audit log e database — **Done automatico** | P0 | M | DB-09, APP-04 | script nel gate: niente log eventi, capability minima, schema/DTO aggregati |
| QA-02 | Profilare CPU e memoria idle/typing — **Ready, TCC/manuale** | P0 | M | OVR-06 | overlay pronto; baseline finale TODO manuale |
| QA-03 | Test sospensione, logout e riavvio — **Blocked da UI-10/TCC** | P0 | M | UI-10 | checklist pronta; prova reale TODO manuale |
| QA-04 | Test timezone e cambio giorno — **Done automatico** | P1 | M | DB-07 | offset opposti e date isolate testati; mezzanotte naturale resta smoke test |
| QA-05 | Eseguire smoke test multi-monitor — **Ready, manuale** | P1 | M | OVR-09 | routing finestra focalizzata e fallback pronti; prova su due display TODO manuale |
| REL-01 | Definire versioning e changelog — **Done** | P0 | S | FND-07 | SemVer, changelog e audit coerenza metadati |
| REL-02 | Creare workflow release macOS — **Implemented, non pubblicata** | P0 | L | REL-01, Gate F | due architetture, ZIP, checksum e draft Release su tag |
| REL-03 | Applicare firma ad-hoc dove utile — **Done nel packaging** | P1 | M | REL-02 | identità `-` e `codesign --verify` obbligatorio |
| REL-04 | Scrivere istruzioni Gatekeeper — **Done** | P0 | S | REL-02 | limiti e apertura sicura documentati senza bypass globale |
| REL-05 | Creare tap `homebrew-typepulse` — **Template done, remote blocked** | P0 | M | REL-02 | struttura cask pronta; owner/repository TODO |
| REL-06 | Creare e validare cask — **Blocked da REL-05/release reale** | P0 | M | REL-05 | renderer valida SemVer/hash; install/upgrade/uninstall TODO manuale |
| REL-07 | Pubblicare release candidate — **Blocked** | P0 | M | QA-01–QA-05, REL-06 | Gate F, QA manuale e remote mancanti |
| REL-08 | Pubblicare V1 macOS — **Blocked** | P0 | S | REL-07 | nessuna release stabile autorizzata |

**Gate G: aperto.** Il codice di audit e distribuzione è pronto, ma nessun utente
può ancora installare dal tap: mancano Gate F, prove reali, remote GitHub,
Release pubblica e repository `homebrew-typepulse`.

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
| Licenza | completata | GPL-3.0-only; TODO titolare e contatto in NOTICE.md |
| Versione minima macOS | completata | macOS 10.15 |
| Frontend build tool | durante FND-03 | Vite + TypeScript senza framework UI |
| Binding macOS | durante MAC-01 | Rust diretto; bridge minimo solo se necessario |
| SQLite crate | completata | `rusqlite` bundled + `rusqlite_migration` |
| Bucket metriche | completata | 60 secondi per la V1 |
| Artefatto macOS | completata | `.app.zip` separato per architettura + SHA-256 |
| Architetture macOS | completata nel workflow | Apple Silicon e Intel |

Ogni decisione che cambia un confine architetturale viene registrata in un ADR,
non soltanto in una issue o in una conversazione.
