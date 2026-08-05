# Roadmap macOS V1 e Linux successivo

Questa è la vista sintetica delle milestone. Task, dipendenze, priorità e criteri
di completamento sono mantenuti nel [`working plan`](working-plan.md).

## 0. Fondazioni

- decidere nome definitivo e versione minima di macOS;
- creare scaffold Tauri 2 con frontend TypeScript;
- configurare Cargo workspace e `typepulse-core`;
- creare il crate `typepulse-platform-macos`;
- configurare tray, finestra nascosta e target di test;
- avviare repository GitHub con licenza open source e contribution guide.

**Uscita:** l'app si avvia su macOS, appare nella menu bar e `cargo test` gira in
locale e in CI.

## 1. Segnale e metrica

- adapter macOS, permesso e filtro privacy-safe: implementati nella Fase B;
- verifica end-to-end TCC/revoca/Secure Input: TODO manuale del proprietario;
- implementare finestra mobile, smoothing e sessioni;
- aggiungere test deterministici per soglie e inattività.

**Uscita:** una vista diagnostica temporanea mostra WPM affidabili e nessun dato
sensibile viene persistito o loggato.

## 2. Esperienza principale

- costruire overlay non cliccabile e sempre in primo piano;
- aggiungere quattro posizioni, tre dimensioni e ritardo configurabile;
- implementare i quattro stati dell'animazione e la celebrazione record;
- completare menu bar e pausa del tracking.

**Uscita:** iniziando a scrivere l'overlay compare rapidamente e scompare dopo
l'inattività configurata.

## 3. Dati locali

- salvare sessioni e bucket aggregati;
- calcolare riepilogo giornaliero e ultimi sette giorni;
- creare schermata statistiche e grafico;
- implementare reset di oggi ed export CSV.

**Uscita:** i valori sopravvivono al riavvio e il CSV rispetta lo schema pubblico.

## 4. Onboarding e distribuzione macOS

- realizzare onboarding in tre passaggi;
- aggiungere accesso diretto alle impostazioni macOS;
- implementare avvio al login e preferenza aspetto;
- verificare multi-monitor, revoca permessi, sospensione e riavvio;
- controllare accessibilità e consumo CPU/memoria;
- produrre l'app bundle tramite GitHub Actions;
- pubblicare sorgenti e artefatto in una GitHub Release;
- creare un tap Homebrew personale e il relativo cask;
- documentare chiaramente l'avviso Gatekeeper per build senza Developer ID.

**Uscita:** tutti i sette punti dell'MVP sono verificati su una release installabile
con il tap Homebrew personale.

## 5. Linux

- validare la portabilità del core su CI Linux;
- implementare un adapter X11 senza modificare le regole di dominio;
- adattare tray e overlay al desktop Linux;
- generare un pacchetto `.deb` con Tauri;
- documentare installazione locale con `apt install ./pacchetto.deb`;
- valutare un repository APT firmato solo dopo release Linux stabili;
- studiare Wayland separatamente senza ricorrere a privilegi invasivi.

**Uscita:** TypePulse funziona su una distribuzione Linux/X11 dichiarata e viene
installato da un pacchetto `.deb` riproducibile.

## Criteri trasversali

- nessun evento individuale finisce su disco o nei log;
- il monitoraggio può essere fermato in modo immediato;
- i calcoli principali hanno test unitari;
- perdita del permesso e problemi di persistenza hanno stati UI comprensibili;
- ogni nuova funzione resta entro lo scope dichiarato della V1;
- la release macOS non viene ritardata per ottenere subito compatibilità Linux.
