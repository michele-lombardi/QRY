# Fase E — Shell Tauri, tray e overlay

- Data completamento implementazione: 5 agosto 2026
- Stato codice: completato
- Gate E reale: checklist manuale aperta finché manca il consenso Input Monitoring

## Risultato

QRY dispone ora di una seconda finestra Tauri dedicata all'overlay. La
finestra viene creata nascosta e usa queste proprietà native:

- trasparente e senza decorazioni;
- sempre in primo piano e visibile su tutti gli spazi;
- esclusa dal Dock e non focalizzabile;
- configurata con `set_ignore_cursor_events(true)` per lasciar passare i click;
- mostrata e nascosta dal controller Rust in base allo stato del core.

L'uso della trasparenza su macOS abilita `macOSPrivateApi` di Tauri. QRY
non è destinata al Mac App Store, quindi questa scelta è coerente con la
distribuzione open source tramite GitHub Release e Homebrew.

## Controller e display

Il controller dell'overlay campiona lo snapshot già aggregato del core senza
toccare il callback dell'event tap. Inoltra al frontend soltanto:

- visibilità;
- WPM smussato e fascia visuale;
- dimensione e modalità contenuto;
- sequenza numerica della celebrazione record.

La posizione è calcolata nell'area utile del display contenente il centro della
finestra focalizzata, rispettando menu bar e Dock. Le quattro opzioni sono
`top-left`, `top-right`, `bottom-left` e `bottom-right`. Il controller rivaluta
il target prima della comparsa e durante nuova attività, con limite di 250 ms;
la topologia viene comunque rivalutata ogni due secondi. Permesso Accessibilità
assente o geometria temporaneamente non disponibile conservano il monitor
corrente quando esiste; all'avvio il fallback resta il display principale. Una
lettura focalizzata valida torna immediatamente ad avere precedenza.

Il posizionamento usa coordinate logiche derivate dalla scala del monitor di
destinazione. Questo evita che un salto diretto da un Retina 2× verso il terzo
monitor 1× dimezzi una coordinata negativa e collochi il PiP sul secondo monitor.

Il nuovo adapter macOS legge soltanto `AXPosition` e `AXSize` della finestra
focalizzata e restituisce un centro globale effimero. Non legge o conserva nome
applicazione, titolo, ruolo, valore o contenuto. Input Monitoring e Accessibilità
restano due consensi distinti nell'interfaccia.

## Presentazione

La UI `overlay.html` implementa:

- fade-in con lieve spostamento e fade-out prima dell'hide nativo;
- WPM live;
- Pip in SVG conforme all'identity con Breathe, Walk, Run e Tired;
- Jump/Cheer verde one-shot per il nuovo record;
- celebrazione breve quando il core supera un record già esistente;
- contenuto `wpm`, `animation` o `both`;
- dimensioni `small`, `medium` e `large`;
- rispetto di `prefers-reduced-motion`.

Il record iniziale del motore viene caricato dal massimo storico qualificato
delle sessioni concluse. Il warm-up live dei primi 3 secondi non può modificare
media, picco o best score. La prima sessione in assoluto stabilisce il riferimento senza una
celebrazione artificiale; un record successivo incrementa una sequenza e viene
animato una sola volta.

## Persistenza

La migrazione SQLite `0002_overlay_preferences.sql` aggiunge esclusivamente
preferenze visuali, con vincoli sui valori ammessi. I database schema v1 vengono
copiati in backup prima dell'upgrade e ricevono default sicuri:

- overlay abilitato;
- posizione in alto a destra;
- dimensione media;
- WPM e animazione.

Nessuna identità di tasto, testo, applicazione o finestra entra nei DTO o nel
database.

La successiva migrazione v3 aggiunge `menu_bar_wpm_enabled`, un booleano visuale
indipendente dal contenuto del PiP.

## Stato task

| Task | Stato | Evidenza |
| --- | --- | --- |
| APP-01…APP-04 | Done | shell menu bar e percorso metriche già integrati |
| OVR-01 | Done | finestra trasparente, top-most e click-through |
| OVR-02 | Done | calcolo unit-tested delle quattro posizioni |
| OVR-03 | Done nel codice | segue display focalizzato, fallback e rivalutazione; smoke test fisico ancora manuale |
| OVR-04 | Done | transizioni da 150/180 ms senza focus |
| OVR-05 | Done | DTO riceve il valore smussato del core |
| OVR-06 | Done | fasce core tradotte nei comportamenti Pip supportati |
| OVR-07 | Done | sequenza record monotona, effetto non ripetuto |
| OVR-08 | Done | tre dimensioni e tre modalità contenuto persistenti |

## Gate E

L'implementazione automatizzabile è completa. La chiusura end-to-end richiede
le checklist `TypePulse/tests/manual/phase-e-overlay.md` e
`TypePulse/tests/manual/focused-display.md`, perché soltanto una
sessione macOS reale può dimostrare che scrivere in un'altra applicazione non
perde focus, che i click attraversano davvero la finestra e che il cambio di
monitor funziona sull'hardware dell'utente.

## Verifiche eseguite

- `./scripts/check.sh`: superato;
- frontend: Prettier, ESLint, TypeScript e bundle Vite multipagina superati;
- Rust: rustfmt, Clippy con warning negati, check e test superati;
- 70 test Rust passati e un benchmark manuale ignorato;
- audit privacy: schema, capability e DTO overlay superati;
- migrazioni simulate fino allo schema v4, inclusa v3 → v4 con backup: superate;
- bundle debug macOS creato e verificato con `codesign --verify --deep --strict`;
- smoke test di avvio: processo background avviato senza panic e poi terminato;
- focus, click-through, TCC e due display: restano verifiche manuali, non simulate.
