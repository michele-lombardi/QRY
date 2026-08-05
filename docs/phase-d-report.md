# Fase D — Report persistenza e aggregazioni

- Data: 5 agosto 2026
- Ambito: persistenza locale, data civile, CSV e avvio automatico richiesto
- Esito: completata
- Gate D: chiuso automaticamente; prove macOS reali indicate come TODO manuale

## Risultato

QRY collega il monitor macOS al core metrico e salva in SQLite soltanto
sessioni concluse, bucket aggregati da 60 secondi e una preferenza locale. La
vista diagnostica mostra WPM live e riepilogo del giorno corrente.

Il cambio giorno non esegue una cancellazione pianificata. Ogni record ha una
data civile locale: quando la data cambia, “oggi” interroga la nuova data e
parte automaticamente da zero, mentre i giorni precedenti restano disponibili
per lo storico e il CSV. Questo evita perdita dati e rende il comportamento
deterministico.

## Output implementati

- `LocalDate` Gregoriano validato, ordinabile e serializzato `YYYY-MM-DD`;
- `StatisticsRepository` nel core e repository in memoria per i test;
- modelli portabili per sessione, bucket, riepilogo e preferenze;
- export CSV cronologico con formato numerico indipendente dal locale;
- SQLite bundled tramite `rusqlite`;
- migrazione iniziale embedded tramite `rusqlite_migration`;
- WAL, busy timeout e copia `.bak` prima di migrare un database esistente;
- salvataggio idempotente delle sessioni e upsert pesato dei bucket;
- riepilogo giornaliero e sequenze recenti con giorni vuoti espliciti;
- reset transazionale limitato a una singola data;
- relay monitor → `TypingEngine` → repository fuori dal callback globale;
- chiusura esplicita della sessione allo stop e al passaggio di data;
- DTO Tauri per oggi, ultimi giorni, CSV e reset del giorno;
- checkbox `Start automatically` con login item macOS e monitor automatico;
- errore non bloccante quando login item o Input Monitoring non sono disponibili.

## Schema persistito

```text
completed_sessions
  local_date, started_at_unix_ms, ended_at_unix_ms,
  estimated_character_count, estimated_word_count,
  average_wpm, peak_wpm, active_typing_ms

metric_buckets
  local_date, interval_start_unix_ms, interval_duration_ms,
  estimated_character_count, average_wpm, peak_wpm

app_preferences
  auto_start_enabled, menu_bar_wpm_enabled,
  overlay_enabled, overlay_position, overlay_size, overlay_content
```

Non esiste una tabella eventi. Lo schema non può rappresentare key code, testo,
contenuto, applicazione attiva o titolo finestra. Un test enumera le colonne di
tutte le tabelle applicative e rifiuta nomi sensibili.

## Semantica dell'avvio automatico

La preferenza ha un significato unico e visibile:

1. quando viene selezionata, registra il `LaunchAgent` e prova ad avviare subito
   il monitor;
2. a ogni apertura successiva, riconcilia il login item e avvia il monitor;
3. quando viene deselezionata, rimuove il login item ma non forza lo stop di una
   sessione già in corso;
4. non concede e non aggira il permesso TCC di macOS.

## Verifiche automatiche

Il gate locale esegue:

```bash
./scripts/check.sh
```

Risultato al completamento:

- Prettier ed ESLint: pass;
- TypeScript e build Vite: pass;
- Rustfmt e Clippy con warning negati: pass;
- 48 test Rust passati;
- 1 benchmark manuale ignorato come previsto;
- bundle debug `QRY.app` costruito e avviato correttamente;
- database reale inizializzato a schema 1 con preferenza automatica disattivata;
- persistenza dopo riapertura, migrazione, backup, rollover, reset isolato, CSV
  e audit privacy coperti da test.

Lo schema corrente è v3: `0002` aggiunge le preferenze overlay e `0003` la sola
preferenza booleana del WPM nella menu bar. Migrazioni da v1 e v2 conservano i
valori esistenti, creano il backup previsto e abilitano il numero per default.

## TODO manuali lasciati aperti

- concedere Input Monitoring e chiudere le checklist della Fase B;
- verificare login item dopo logout/login o riavvio reale;
- osservare un passaggio naturale di mezzanotte;
- compilare titolare copyright e contatto pubblico in `NOTICE.md`.

La procedura completa è in
`TypePulse/tests/manual/phase-d-persistence-startup.md`. Nessuno di questi TODO
nasconde codice mancante della Fase D: richiedono consenso o stato reale del Mac.
