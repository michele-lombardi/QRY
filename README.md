# TypePulse

TypePulse è una piccola app macOS da menu bar che rende visibile il ritmo di
scrittura senza registrare ciò che viene scritto.

> **See your typing rhythm. Not what you type.**

## Stato del progetto

Le Fasi A, B e C sono implementate: fondazioni, monitor macOS privacy-safe e core
portabile di WPM/sessioni sono presenti. Il Gate C è chiuso con test
deterministici. Sul Mac di sviluppo il consenso TCC è ancora negato, quindi il
Gate B end-to-end resta una checklist manuale prima dell'integrazione.

La licenza è ancora un `TODO`. Finché non viene scelta una licenza OSI, il codice
è visibile ma non concede diritti di utilizzo o redistribuzione.

Decisioni attuali:

- progetto destinato a diventare open source su GitHub dopo la scelta licenza;
- macOS è la prima piattaforma e la V1 di riferimento;
- Linux verrà affrontato dopo una release macOS stabile;
- Tauri 2 per l'app desktop;
- Rust per dominio, metriche e integrazioni di sistema;
- HTML, CSS e TypeScript per l'interfaccia;
- VS Code come ambiente di lavoro principale;
- GitHub Releases e tap Homebrew personale come primo canale di distribuzione.

## Obiettivo della V1

La prima versione è completa quando:

1. vive nella menu bar e non compare nel Dock;
2. rileva attività di digitazione globale con il consenso dell'utente;
3. mostra un piccolo overlay reattivo;
4. calcola i WPM in tempo reale senza salvare i tasti;
5. nasconde l'overlay dopo un breve periodo di inattività;
6. mostra statistiche giornaliere essenziali;
7. esporta un riepilogo giornaliero in CSV.

Account, cloud, classifiche, AI, analisi per applicazione e distribuzione sul
Mac App Store non fanno parte della V1. La compatibilità Linux è una fase
successiva, non un requisito bloccante per la prima release macOS.

## Struttura

Struttura implementata nella Fase A:

```text
.
├── TypePulse/
│   ├── src/                         # UI HTML/CSS/TypeScript
│   ├── src-tauri/                   # Shell Tauri e configurazione desktop
│   ├── crates/
│   │   ├── typepulse-core/          # Confine del dominio portabile
│   │   ├── typepulse-platform-macos/ # Monitoraggio e API specifiche macOS
│   │   └── typepulse-storage-sqlite/ # Confine della persistenza locale
│   └── tests/                       # Fixture e checklist manuali future
├── .github/                         # CI e template di collaborazione
├── docs/                            # Specifiche e decisioni di progetto
└── scripts/                         # Build e release ripetibili
```

La responsabilità di ogni area è descritta in
[`TypePulse/README.md`](TypePulse/README.md). Prima di implementare, consulta
anche:

- [`docs/product-spec.md`](docs/product-spec.md)
- [`docs/architecture.md`](docs/architecture.md)
- [`docs/privacy.md`](docs/privacy.md)
- [`docs/roadmap.md`](docs/roadmap.md)
- [`docs/distribution.md`](docs/distribution.md)
- [`docs/working-plan.md`](docs/working-plan.md)
- [`docs/development.md`](docs/development.md)
- [`docs/phase-a-report.md`](docs/phase-a-report.md)
- [`docs/phase-b-report.md`](docs/phase-b-report.md)
- [`docs/phase-c-report.md`](docs/phase-c-report.md)

## Sviluppo locale

```bash
cd TypePulse
npm install
npm run tauri dev
```

Controllo completo dalla root:

```bash
./scripts/check.sh
```

## Principi

- **Privacy strutturale:** elaborare eventi effimeri, salvare solo aggregati.
- **Presenza discreta:** menu bar e overlay, nessuna finestra grande all'avvio.
- **Core portabile:** il calcolo non dipende dalle API della piattaforma.
- **Adapter sottili:** macOS e Linux implementano soltanto ciò che è specifico
  del sistema operativo.
- **Poche dipendenze:** usare API di sistema e librerie Rust mirate.
- **Testabilità:** tempo, eventi di input e persistenza devono essere iniettabili.
- **Scope ridotto:** una sola animazione con quattro livelli di intensità.

## Prossimi passi tecnici

1. Concedere Input Monitoring e completare le checklist manuali della Fase B.
2. Iniziare la Fase D: repository, SQLite e aggregazioni locali.
3. Scegliere la licenza OSI prima di rendere pubblico il repository.

Il piano aggiornato è in [`docs/working-plan.md`](docs/working-plan.md).
