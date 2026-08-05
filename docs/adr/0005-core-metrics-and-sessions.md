# ADR 0005 — Semantica delle metriche e delle sessioni

- Data: 5 agosto 2026
- Stato: accettata per la V1

## Contesto

La Fase C deve trasformare il solo segnale `TypingActivity { occurred_at }` in
uno stato completo per overlay, sessioni e persistenza futura. Il risultato deve
essere deterministico, portabile e indipendente dalla frequenza con cui la UI
richiede uno snapshot.

## Decisione

### Tempo

- usare esclusivamente `Instant` nel percorso live;
- iniettare un trait `Clock` nel `TypingEngine`;
- fornire `SystemClock` per produzione e `ManualClock` per test senza sleep;
- rifiutare attività o tick precedenti all'ultima attività osservata.

La conversione verso data/ora civile appartiene alla composizione e alla
persistenza della Fase D, non al calcolo WPM.

### WPM live

- lookback mobile massimo di 10 secondi;
- convenzione di cinque attività per parola;
- durante il warm-up, formula:
  `(intervalli osservati / 5) / durata fra prima e ultima attività`;
- almeno 250 ms fra prima e ultima attività prima di pubblicare una stima;
- dopo il warm-up la stessa formula usa le attività rimaste nel lookback;
- limite difensivo live di 300 WPM;
- un evento esattamente sul limite inferiore della finestra è scaduto;
- EMA con fattore `0,25`, inizializzata dalla prima stima affidabile senza
  incorporare gli zeri del warm-up;
- smoothing aggiornato sulle attività, non sui tick UI.

Il lookback rende la metrica indipendente dal polling senza imporre dieci secondi
di attesa. A 60 WPM una sequenza con un'attività ogni 200 ms produce la prima
stima dopo circa 400 ms; sequenze simultanee non producono una velocità infinita.
Il valore resta una stima convenzionale basata su cinque attività per parola.

### Protezione dalle ripetizioni

- scartare gli eventi marcati da macOS come auto-repeat;
- permettere due pressioni consecutive dello stesso tasto per non penalizzare le
  doppie lettere;
- scartare dalla terza pressione identica fino a un tasto conteggiabile diverso
  o una pausa di almeno un secondo;
- mantenere tasto precedente, streak e tempo soltanto nell'adapter privato;
- non inoltrare mai questa identità al core, ai DTO, ai log o allo storage.

### Fasce visuali

- `Still`: meno di 30 WPM;
- `Steady`: da 30 a meno di 60;
- `Fast`: da 60 a meno di 90;
- `Intense`: almeno 90.

Le soglie sono configurabili, finite, non negative e strettamente crescenti.

### Sessioni

- prima attività: apertura sessione e overlay visibile;
- 2 secondi senza attività: sessione ancora attiva, overlay nascosto;
- 30 secondi senza attività: sessione conclusa;
- l'orario logico di fine è l'ultima attività, non la fine del timeout;
- il tempo attivo è la somma dei gap consecutivi non superiori a 2 secondi;
- la media è la media aritmetica dei WPM visualizzati campionati sulle attività;
- il picco è il massimo WPM visualizzato della sessione;
- parole stimate = attività / 5, mantenendo la parte frazionaria nel dominio.

Una nuova attività arrivata al timeout esatto completa la vecchia sessione e
ne apre una nuova nello stesso aggiornamento.

### Record

La celebrazione esiste solo se è stato fornito un record storico. Viene emessa
al primo superamento e al massimo una volta per sessione. In assenza di storico,
la prima sessione stabilisce il riferimento senza celebrare ogni incremento.

## Conseguenze

- test completi possono avanzare minuti senza attese reali;
- core, macOS, Tauri, SQLite e WebView restano disaccoppiati;
- nessun output contiene caratteri, key code, applicazioni o finestre;
- `Instant` e i tipi live non vengono serializzati direttamente;
- la Fase D associa il riepilogo monotono a `LocalDate` nel layer applicativo e
  persiste le parole frazionarie come `REAL`, esportandole con due decimali.
