# Fase B — Report dello spike macOS

- Data: 5 agosto 2026
- Ambito: permesso Input Monitoring e monitor globale passivo
- Implementazione: completata
- Gate runtime su eventi reali: TODO, richiede consenso TCC dell'utente

## Risultato

TypePulse dispone di un adapter macOS funzionante a livello di build e test che
legge/richiede il permesso, apre la sezione corretta di System Settings, installa
un event tap passivo, filtra gli eventi e distrugge l'identità del tasto prima
del confine pubblico.

La finestra temporanea di Fase B permette di eseguire la prova manuale senza
strumenti esterni. Il frontend riceve soltanto un conteggio crescente e metriche
di salute aggregate. Non riceve timestamp individuali, key code, testo,
applicazione attiva o finestra.

## Output prodotti

- `TypingActivity` monotono e non serializzabile nel core;
- controllo e richiesta del permesso tramite API Core Graphics;
- deep link a Privacy & Security → Input Monitoring;
- event tap `Session` + `ListenOnly` su thread e run loop dedicati;
- filtro privato per lettere, numeri, spazio, punteggiatura e keypad;
- esclusione di shortcut con Command, Control, Option o Fn;
- esclusione dell'auto-repeat Core Graphics;
- guard effimera che consente la doppia lettera e scarta dalla terza pressione
  identica fino a cambio tasto o pausa di un secondo;
- canale bounded non bloccante con contatore dei drop;
- stop con join del worker, revoca rilevata e riattivazione del tap;
- comandi e snapshot diagnostici Tauri a cadenza di un secondo, senza eventi
  per singola attività né metadati di input;
- UI diagnostica accessibile per consenso, avvio, stop e metriche;
- ADR tecnica e checklist manuali riproducibili;
- macOS minimo fissato a 10.15.

## Verifiche automatiche

Passano:

```bash
npm run lint
npm run build
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
```

I test coprono il tipo privacy-safe, filtro tasti/modificatori, auto-repeat,
sequenze identiche, stringhe degli
stati, URL delle impostazioni, stati atomici del monitor, metriche vuote e DTO
Tauri.

## Riferimento prestazioni

Comando:

```bash
cargo test -p typepulse-platform-macos --release \
  typing_callback_hot_path_reference -- --ignored --nocapture
```

Risultato locale su Apple M1 Pro, arm64, macOS 26.5.2:

```text
250.000 attività, 12 ns/attività nel percorso filtro + Instant + try_send + atomiche
```

Questo è un microbenchmark del percorso Rust, non una promessa universale né la
durata completa imposta da Core Graphics. La UI espone media e massimo misurati
nel callback reale per la verifica manuale. Il callback non esegue I/O, log,
query, rendering o chiamate Tauri.

## Evidenza locale e TODO TCC

Sul Mac di sviluppo il 5 agosto 2026:

- `CGPreflightListenEventAccess`: `denied`;
- apertura del deep link System Settings: exit code `0`;
- build, test e microbenchmark: completati;
- ricezione globale reale: **TODO manuale**, perché Codex non concede permessi
  privacy al posto dell'utente;
- revoca/ripristino durante il monitor: **TODO manuale** dopo la concessione;
- comportamento in un campo Secure Input: **TODO manuale** dopo la concessione.

Le procedure esatte sono in `TypePulse/tests/manual/`. Questi TODO non sono
codice mancante: sono criteri runtime che richiedono una decisione esplicita del
proprietario del Mac.

## Stato Gate B

Il gate di implementazione è chiuso: l'architettura conta solo attività anonima
e non espone contenuto. Il Gate B end-to-end resta `TODO manuale` finché la
checklist TCC non viene firmata con esito positivo.

## TODO ancora aperti

- eseguire e registrare le tre prove manuali TCC;
- compilare titolare del copyright e contatto pubblico in `NOTICE.md`;
- verificare la CI dopo il primo push GitHub;
- sostituire icone e bundle identifier prima della release.
