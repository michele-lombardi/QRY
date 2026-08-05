# ADR 0007 — Display focalizzato tramite Accessibilità macOS

- Stato: accepted
- Data: 5 agosto 2026

## Contesto

Il PiP deve comparire sul display in cui l'utente sta scrivendo, non sempre sul
display principale. La posizione del puntatore non è un segnale corretto: il
mouse può trovarsi su uno schermo diverso dalla finestra che riceve la tastiera.
L'event tap di Input Monitoring segnala attività di digitazione, ma non fornisce
in modo privacy-safe la finestra destinataria.

macOS espone la finestra focalizzata e la sua geometria tramite Accessibility
API. Questo richiede un consenso TCC separato da Input Monitoring.

## Decisione

`typepulse-platform-macos` usa l'elemento Accessibility di sistema e richiede
soltanto questi quattro attributi:

1. `AXFocusedApplication`, necessario per raggiungere l'elemento focalizzato;
2. `AXFocusedWindow`;
3. `AXPosition`;
4. `AXSize`.

L'adapter riduce subito posizione e dimensione a un singolo `ScreenPoint`, il
centro della finestra. `src-tauri` passa il punto a `monitor_from_point`, che su
macOS usa lo stesso spazio globale Core Graphics, e colloca il PiP nell'angolo
configurato dell'area utile di quel monitor.

Il controllo avviene:

- prima di mostrare il PiP;
- durante nuova attività di digitazione, al massimo ogni 250 ms;
- durante la rivalutazione periodica della topologia display.

Il permesso non viene richiesto all'avvio. L'utente lo concede esplicitamente
dalla finestra TypePulse o da Impostazioni di Sistema. Senza consenso, con una
finestra non compatibile o dopo la rimozione del display, l'app usa il display
principale e poi il primo display disponibile.

## Vincoli privacy

- non richiedere `AXTitle`, `AXValue`, testo selezionato, ruolo o URL;
- non ricavare né esporre nome, bundle ID o PID dell'applicazione;
- non inviare la geometria al frontend;
- non registrarla nei log, nel database, nei backup o nel CSV;
- conservarla soltanto nello stack della singola operazione di posizionamento;
- trattare errore, revoca e attributo non supportato come normale fallback.

## Conseguenze

Il comportamento è accurato anche quando il puntatore si trova altrove e non
lega il dominio portabile alle API macOS. In cambio TypePulse presenta due
permessi distinti e la funzione “segui display” non è disponibile senza
Accessibilità. Linux dovrà implementare una sorgente equivalente oppure
dichiarare il fallback previsto dalla propria sessione grafica.
