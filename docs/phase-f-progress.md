# Fase F — Stato schermate e preferenze

- Data avvio: 5 agosto 2026
- Stato: implementazione principale completata; onboarding e audit manuale aperti
- Incremento corrente: QRY, pannello giornaliero, Impostazioni e Statistiche

## Decisione UX

QRY non usa il Dock come punto di accesso permanente. Vive nella parte
destra della menu bar di macOS come le altre utility in background. La finestra
principale è uno strumento aperto su richiesta, non la presenza costante
dell'app.

## Completato

- `APP-01`: finestra principale nascosta all'avvio;
- `APP-02`: activation policy `Accessory`, niente Dock o `Cmd + Tab`;
- `APP-03`: icona menu bar con tooltip e menu nativo;
- click sinistro sul Pulse per aprire/chiudere il pannello giornaliero compatto;
- click destro per Oggi, Statistiche, Impostazioni, WPM, monitor e uscita;
- chiusura finestra intercettata e trasformata in hide;
- quit esplicito con stop del monitor e flush degli aggregati;
- test unitario della mappatura degli ID del menu;
- checklist manuale macOS dedicata.
- stato, richiesta e collegamento Impostazioni per il permesso Accessibilità;
- spiegazione distinta dei permessi Input Monitoring e Accessibilità;
- fallback esplicito del PiP quando Accessibilità non è concessa.
- icona menu bar mantenuta come template anche nei cambi idle/attivo, con logo e
  titolo WPM adattivi fra aspetto Light e Dark.
- voce checkabile **Show WPM in menu bar**, persistente e indipendente dal PiP;
- slot nativo fisso a tre cifre per evitare spostamenti del Pulse.
- nome visibile, bundle e artefatti rinominati in QRY senza migrare
  identificatore/database legacy;
- pannello giornaliero con WPM live, Pulse, parole, record, streak e ultima
  attività;
- finestra Impostazioni in stile macOS con sezioni General, Appearance,
  Permissions e Privacy;
- controlli persistenti per login, WPM menu bar, overlay, posizione, dimensione
  e contenuto;
- finestra Statistiche con viste Oggi, 7 giorni, 30 giorni e Anno;
- riepilogo parole/media/picco/tempo, grafico aggregato, tabella, insight, reset
  dati completati e copia CSV;
- aggiornamento live del pannello e refresh su focus/ogni cinque secondi delle
  Statistiche;
- chiusura automatica del pannello quando perde il focus;
- rollover a mezzanotte anche senza nuova digitazione.

## Fase F ancora aperta

- onboarding e stati permesso negato/revocato;
- dialog macOS per scegliere dove salvare il CSV; la copia negli appunti è già
  disponibile;
- selezione forzata Light/Dark oltre al tema System già adattivo;
- audit accessibilità, contrasto e navigazione da tastiera su bundle reale.

La checklist dedicata è `QRY/tests/manual/qry-interface.md`.

## Verifiche

- `./scripts/check.sh`: pass;
- suite Rust/frontend: da rieseguire sul commit finale;
- bundle atteso: `QRY.app`;
- Launch Services: `ApplicationType = UIElement`, quindi app accessoria senza
  presenza ordinaria nel Dock;
- preferenza `auto_start_enabled = 0` durante la prova: nessun login item
  attivato dal test;
- presenza e interazioni reali della status icon: checklist manuale aperta.
