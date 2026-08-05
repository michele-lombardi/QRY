# Fase C — Report core delle metriche

- Data: 5 agosto 2026
- Ambito: metriche live, sessioni e output portabile
- Esito: completata
- Gate C: chiuso

## Risultato

`typepulse-core` trasforma esclusivamente timestamp monotoni anonimi in WPM,
stato overlay, fascia visuale, aggregati di sessione e notifica record. Non
importa Tauri, macOS, SQLite o codice frontend.

Al momento della chiusura della Fase C il core era pronto per `APP-04` e restava
testabile senza permessi macOS o thread reali. Il collegamento monitor → core è
stato poi completato nella Fase D senza modificare quel confine.

## Output implementati

- `Clock`, `SystemClock` e `ManualClock` condivisibile;
- `CoreConfig` con validazione di ogni relazione tra timeout e soglie;
- lookback WPM massimo e memoria effimera degli ultimi 10 secondi;
- warm-up adattivo con prima stima dopo almeno 250 ms e limite 300 WPM;
- formula standard a cinque attività per parola;
- smoothing esponenziale configurabile;
- fasce `Still`, `Steady`, `Fast`, `Intense` a 30/60/90 WPM;
- `TypingEngine<C>` con clock generico;
- stati `Idle`, `ActiveVisible`, `ActiveHidden`;
- completamento sessione e avvio simultaneo della successiva;
- conteggi, parole stimate, media, picco e tempo attivo;
- record opzionale emesso una volta per sessione;
- snapshot e update completi per i futuri adapter applicativi;
- errori espliciti per configurazione, record e tempo non monotono.

## Default V1

| Parametro | Valore |
| --- | ---: |
| Finestra WPM | 10 s |
| Osservazione minima warm-up | 250 ms |
| Limite live difensivo | 300 WPM |
| Caratteri stimati per parola | 5 |
| Fattore EMA | 0,25 |
| Hide overlay | 2 s |
| Fine sessione | 30 s |
| Gap massimo di tempo attivo | 2 s |
| Fasce | 30 / 60 / 90 WPM |

La semantica completa è formalizzata nell'ADR 0005.

## Verifiche

Il crate core contiene 33 test. Coprono:

- clock manuale senza attese;
- configurazioni valide e relazioni invalide;
- finestra vuota, warm-up reattivo, digitazione lenta, 60/120 WPM e burst da
  1.000 attività;
- smoothing senza ritardo iniziale e attenuazione progressiva;
- `NaN`, infinito e valori negativi;
- confini esatti 30/60/90;
- transizioni a 2 e 30 secondi;
- attività immediatamente prima e sul timeout;
- esclusione dei gap lunghi dal tempo attivo;
- media, picco, conteggi e parole stimate;
- record singolo e assenza di falsa celebrazione iniziale;
- tempo non monotono;
- sequenza pseudo-casuale deterministica di 10.000 attività con invarianti
  numerici e temporali.

Gate locale eseguito:

```bash
./scripts/check.sh
```

Risultato: frontend valido, Rustfmt valido, Clippy senza warning, tutti i test
workspace passati. Il benchmark manuale della Fase B resta ignorato nel test run
ordinario come previsto.

## Confine privacy verificato

Gli input pubblici del motore sono `TypingActivity` e tempo monotono. Gli output
contengono soltanto conteggi, durate, WPM, stati e aggregati. Nessun tipo del core
ha campi per contenuto, identità del tasto, applicazione o finestra.

## Attività demandate e stato successivo

- Fase D: repository, SQLite, data civile e CSV — completati;
- `APP-04`: receiver macOS collegato al `TypingEngine` e ai DTO diagnostici —
  completato in anticipo nella Fase D;
- Fase E: restano tray e overlay definitivi;
- TODO manuale precedente: completare Gate B con consenso TCC.
