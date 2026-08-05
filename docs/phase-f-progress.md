# Fase F — Stato schermate e preferenze

- Data avvio: 5 agosto 2026
- Stato: in corso
- Primo incremento: shell macOS nella menu bar

## Decisione UX

TypePulse non userà il Dock come punto di accesso permanente. Vive nella parte
destra della menu bar di macOS come le altre utility in background. La finestra
principale è uno strumento aperto su richiesta, non la presenza costante
dell'app.

## Completato in questo incremento

- `APP-01`: finestra principale nascosta all'avvio;
- `APP-02`: activation policy `Accessory`, niente Dock o `Cmd + Tab`;
- `APP-03`: icona menu bar con tooltip e menu nativo;
- click sinistro per riaprire e focalizzare la finestra;
- click destro per Open, Start monitoring, Pause monitoring e Quit;
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

## Fase F ancora aperta

- token visuali e temi;
- riepilogo compatto nel pannello tray;
- schermata statistiche, ultimi sette giorni e grafico;
- settings General, Overlay e Appearance;
- onboarding e stati permesso negato/revocato;
- dialog di export CSV;
- audit accessibilità.

Il backend e i controlli diagnostici delle impostazioni Overlay sono ora
disponibili grazie alla Fase E. In Fase F resta da inserirli nella schermata
Settings definitiva insieme agli altri pannelli.

## Verifiche

- `./scripts/check.sh`: pass;
- 69 test Rust passati e un benchmark manuale ignorato;
- bundle debug `TypePulse.app`: costruito e avviato;
- Launch Services: `ApplicationType = UIElement`, quindi app accessoria senza
  presenza ordinaria nel Dock;
- preferenza `auto_start_enabled = 0` durante la prova: nessun login item
  attivato dal test;
- presenza e interazioni reali della status icon: checklist manuale aperta.
