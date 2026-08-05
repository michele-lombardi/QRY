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

## Checklist per ogni modifica

- Il nuovo dato è davvero necessario per una funzione promessa?
- Può essere sostituito da un aggregato meno sensibile?
- Dove nasce, quanto vive e quando viene eliminato?
- Compare in log, crash report, analytics, backup o export?
- L'interfaccia spiega in modo fedele ciò che avviene?

Qualunque funzione futura che richieda dati oggi vietati va trattata come una
nuova decisione di prodotto e privacy, non come una semplice estensione tecnica.
