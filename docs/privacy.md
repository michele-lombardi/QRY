# Privacy e trattamento dei dati

## Impegno di prodotto

TypePulse conta attività compatibili con la digitazione per stimarne la velocità.
Non registra ciò che l'utente scrive.

Testo breve previsto nell'app:

> TypePulse counts keyboard activity to estimate typing speed. It never stores
> individual keys, words, passwords or written content. All statistics remain
> locally on your Mac.

## Dati ammessi

- timestamp necessari al calcolo effimero;
- conteggi aggregati di caratteri e parole stimate;
- WPM medio e massimo;
- durata della digitazione attiva;
- inizio e fine delle sessioni;
- preferenze locali dell'app.

## Dati vietati

- singoli tasti o key code persistiti;
- parole, testo, password o contenuto degli appunti;
- applicazione attiva e titolo delle finestre;
- siti visitati;
- identificatori di account o dati cloud nella V1.

## Regole tecniche

1. Filtrare gli eventi dentro l'adapter specifico della piattaforma.
2. Non esporre caratteri o key code al core, a Tauri o al frontend.
3. Conservare in memoria solo timestamp/conteggi necessari alla finestra mobile.
4. Scrivere su disco esclusivamente sessioni e aggregati.
5. Non aggiungere logging degli eventi di tastiera, nemmeno in build di debug.
6. L'export contiene unicamente il riepilogo giornaliero documentato.
7. Pausa e revoca del permesso devono fermare immediatamente l'acquisizione.
8. L'adapter Linux futuro non deve richiedere accesso privilegiato ai dispositivi
   di input per aggirare le protezioni del desktop.

## Garanzie implementate nella Fase B

- il tap macOS è passivo (`ListenOnly`) e non può modificare la digitazione;
- key code e modificatori sono confinati in `event_filter.rs`, che non è
  pubblico;
- `TypingActivity` contiene solo un `Instant` monotono e non implementa
  serializzazione;
- il frontend diagnostico riceve solo conteggi e metriche aggregate;
- il callback usa un canale limitato non bloccante e non effettua I/O o log;
- Secure Input viene rispettato: dati non osservabili restano non osservati;
- revoca del permesso ferma il worker e non genera dati simulati.

## Garanzie implementate nella Fase C

- il motore accetta soltanto `TypingActivity` con tempo monotono;
- i timestamp individuali restano nella rolling window in memoria e vengono
  eliminati allo scadere dei 10 secondi o alla fine della sessione;
- snapshot e riepiloghi contengono solo conteggi, durate, WPM e stati;
- nessun tipo live implementa una serializzazione automatica verso frontend o
  disco;
- test patologici verificano che non escano `NaN`, infiniti o valori negativi.

## Garanzie implementate nella Fase D

- SQLite contiene soltanto `completed_sessions`, `metric_buckets` e
  `app_preferences`;
- lo schema non ha colonne per key code, testo, contenuto, applicazione o titolo
  della finestra, e un test automatico ne impedisce l'introduzione accidentale;
- i bucket persistiti sono aggregati da 60 secondi, non eventi individuali;
- il CSV legge soltanto riepiloghi giornalieri ed espone valori aggregati;
- il giorno nuovo parte automaticamente da zero tramite una nuova chiave data,
  senza cancellare lo storico;
- il login item salva un booleano locale e non amplia i dati osservati;
- le copie `.bak` pre-migrazione contengono gli stessi soli aggregati locali del
  database sorgente.

## Garanzie implementate nella Fase E

- l'evento frontend dell'overlay contiene soltanto visibilità, WPM aggregato,
  fascia di animazione, preferenze visuali e un contatore di celebrazione;
- le nuove colonne SQLite rappresentano esclusivamente abilitazione, posizione,
  dimensione e contenuto visuale dell'overlay;
- il controller di posizionamento osserva geometria e scala dei display, mai
  applicazione attiva, titolo finestra o contenuto digitato;
- l'audit DTO include anche le preferenze dell'overlay.

## Checklist per ogni modifica

- Il nuovo dato è davvero necessario per una funzione promessa?
- Può essere sostituito da un aggregato meno sensibile?
- Dove nasce, quanto vive e quando viene eliminato?
- Compare in log, crash report, analytics, backup o export?
- L'interfaccia spiega in modo fedele ciò che avviene?

Qualunque funzione futura che richieda dati oggi vietati va trattata come una
nuova decisione di prodotto e privacy, non come una semplice estensione tecnica.

## Audit di release

La Fase G aggiunge `scripts/audit-privacy.sh` al gate ordinario. Il controllo
fallisce in presenza di logging runtime inatteso, capability Tauri più ampie di
`core:default`, colonne SQLite sensibili o DTO non aggregati. Il solo output di
profilazione ammesso è CPU/RSS aggregato; database personali, eventi e testo non
devono essere allegati a issue o release report.
