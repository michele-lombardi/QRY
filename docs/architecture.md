# Architettura tecnica

## Obiettivi

- mantenere il percorso evento → overlay molto rapido;
- impedire per design la persistenza del contenuto digitato;
- separare API di sistema, calcoli e interfaccia per rendere tutto testabile;
- completare prima macOS senza legare il dominio alla piattaforma;
- permettere un adapter Linux successivo senza riscrivere metriche e dati;
- mantenere il progetto semplice da compilare da VS Code e da CI.

## Stack deciso

- Tauri 2 per ciclo di vita desktop, tray, finestre e packaging;
- Rust per dominio, concorrenza, persistenza e adapter di sistema;
- HTML e CSS per la presentazione;
- TypeScript per stato UI e comunicazione con i comandi Tauri;
- Cargo workspace per separare i crate;
- SQLite dietro un repository Rust per sessioni e aggregati locali;
- VS Code come editor principale;
- GitHub Actions per test, build e artefatti di release.

Per la V1 il frontend resta intenzionalmente piccolo e non richiede un framework
complesso. Una libreria UI potrà essere introdotta solo se riduce realmente la
complessità.

## Flusso dei dati

```text
Evento piattaforma
    ↓ filtro immediato
PlatformKeyboardMonitor ── evento semanticamente "typing activity"
    ↓
TypingEngine ── WPM live + stato sessione
    ├──→ Tauri event ──→ Overlay UI
    └──→ Aggregator ──→ Repository SQLite ──→ Statistics / CSV
```

Il monitor non espone il carattere digitato. Il suo unico output è
`TypingActivity`, che contiene un `Instant` monotono non serializzabile. Il key
code viene usato solo nel modulo privato del filtro e distrutto prima del
confine pubblico.

## Componenti

### `typepulse-core`

Contiene modelli e regole indipendenti dal sistema operativo. Espone trait per
clock, origine degli eventi e persistenza, così i test possono utilizzare
implementazioni controllate.

Non importa Tauri e non conosce finestre, tray o permessi macOS.

### PlatformKeyboardMonitor

- verifica e comunica lo stato del permesso, quando la piattaforma lo prevede;
- avvia e ferma il monitoraggio globale;
- scarta modificatori, navigazione, tasti funzione e scorciatoie;
- emette soltanto attività conteggiabile, mai key code o testo verso gli altri
  componenti.

L'implementazione macOS vive in `typepulse-platform-macos`: `core-graphics`
gestisce event tap e run loop; `objc2-core-graphics` espone le API del permesso.
La Fase B ha dimostrato che non serve un bridge Swift/Objective-C.

### TypingEngine

- mantiene la finestra mobile degli ultimi eventi;
- calcola e smussa i WPM;
- apre una sessione al primo evento;
- segnala inattività overlay dopo il ritardo configurato;
- conclude la sessione dopo circa 30 secondi;
- determina fascia di animazione e nuovo record.

Tempo e scheduler devono essere iniettabili per testare i timeout senza attese
reali.

La Fase C implementa `TypingEngine<C: Clock>`. Il motore riceve soltanto
`TypingActivity`, usa una finestra fissa di 10 secondi e produce `EngineUpdate`
con snapshot, eventuale sessione conclusa ed eventuale nuovo record. I clock di
produzione e test sono separati; nessun polling UI modifica la semantica delle
metriche.

Gli aggregati di sessione sono definiti nell'ADR 0005. Restano monotoni nel core;
il layer applicativo della Fase D associa la data civile locale tramite un
`LocalDate` portabile, senza portare `chrono` nel dominio live.

### DesktopShell

Il layer `src-tauri` crea la tray e le finestre, registra i comandi e inoltra gli
eventi del core al frontend. L'overlay è una finestra Tauri trasparente, senza
decorazioni, sempre in primo piano e configurata per ignorare il cursore. Dove
il comportamento differisce tra sistemi, il dettaglio rimane nell'adapter.

L'UI riceve uno stato già pronto e non calcola le metriche.

### Persistence

`StatisticsRepository` vive nel core e ha implementazioni in memoria e SQLite.
L'adapter usa `rusqlite` bundled, `rusqlite_migration`, WAL e busy timeout. Tutte
le scritture avvengono nel relay delle metriche, fuori dal callback critico
dell'event tap. Una migrazione SQL esplicita accompagna ogni modifica dello
schema; prima di migrare un database non vuoto viene creata una copia `.bak`.

Schema logico minimo:

```text
TypingSession
- id
- startedAt
- endedAt
- estimatedCharacterCount
- estimatedWordCount
- averageWPM
- peakWPM
- activeTypingDuration

MetricBucket
- intervalStart
- intervalDuration
- estimatedCharacterCount
- averageWPM
- peakWPM

AppPreferences
- autoStartEnabled
```

Il database applicativo macOS è `typepulse.sqlite3` nella directory dati
risolta da Tauri. Il cambio della data locale non cancella righe: la query di
“oggi” passa semplicemente alla nuova data, che parte vuota. La sessione attiva
viene chiusa prima di registrare attività sul nuovo giorno; i bucket da 60
secondi non attraversano la data civile.

### CSVExporter

Legge aggregati giornalieri e produce il formato pubblico documentato. Non deve
accedere agli eventi live.

### Automatic startup

Una singola preferenza controlla sia il login item macOS (`LaunchAgent`) sia
l'avvio del monitor quando TypePulse si apre. L'abilitazione avvia subito il
monitor; la disabilitazione rimuove il login item ma non interrompe una sessione
già attiva. Errori del login item o del permesso vengono mostrati senza impedire
l'apertura dell'app.

## Confini di concorrenza

Il callback di input deve fare il minimo lavoro possibile. Il conteggio viene
inoltrato a un task Rust dedicato tramite un canale limitato; gli aggiornamenti
UI passano attraverso eventi Tauri e la persistenza lavora separatamente. Nessuna
operazione su disco o WebView deve bloccare il callback globale.

Su macOS il tap è `Session` e `ListenOnly`, vive su un thread con `CFRunLoop` e
usa `try_send`. Un consumer lento causa un drop misurabile, mai un blocco. Il
worker controlla la revoca del permesso e riabilita il tap dopo le notifiche di
timeout/disabilitazione del sistema.

## Confini di piattaforma

### macOS V1

- target principale e unico criterio di rilascio iniziale;
- versione minima: macOS 10.15;
- monitoraggio globale tramite API macOS e permesso Input Monitoring;
- build `.app` prodotta su un runner o computer macOS;
- distribuzione tramite GitHub Releases e Homebrew Cask personale.

### Linux successivo

- riutilizza interamente `typepulse-core`;
- aggiunge tray, overlay e monitoraggio nell'adapter Linux;
- parte da X11;
- tratta Wayland come capacità separata, perché non permette normalmente a
  un'app generica di osservare passivamente tutta la tastiera;
- produce inizialmente un `.deb`, poi eventualmente un repository APT.

## Errori e stati degradati

- permesso assente: niente monitoraggio, onboarding o impostazioni indicano come
  abilitarlo;
- permesso revocato: interrompere il monitoraggio e non simulare statistiche;
- archivio non disponibile: continuare la metrica live, mostrare un errore
  discreto per statistiche/export;
- display rimosso: ricollocare l'overlay sul display principale.

Su una piattaforma che non consente il monitoraggio globale, l'app deve dichiarare
la funzione non disponibile: non deve chiedere privilegi elevati o leggere
direttamente dispositivi di input senza un modello di consenso comprensibile.

## Decisioni ancora aperte

1. Formato dell'artefatto GitHub: `.app.zip`, `.tar.gz` o entrambi.
2. Risultato manuale end-to-end del Gate B dopo il consenso TCC.
3. Titolare copyright e contatto pubblico in `NOTICE.md`.

## Riferimenti tecnici

- [Tauri 2](https://v2.tauri.app/)
- [Tauri system tray](https://v2.tauri.app/learn/system-tray/)
- [Tauri macOS application bundle](https://v2.tauri.app/distribute/macos-application-bundle/)
- [XDG Desktop Portal Input Capture](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.InputCapture.html)
