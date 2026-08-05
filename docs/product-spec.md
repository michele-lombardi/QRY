# Specifica di prodotto — V1

## Visione

TypePulse è un piccolo oggetto digitale che vive nella menu bar e reagisce al
ritmo di scrittura. Deve sembrare più vicino a un tamagotchi della tastiera che
a uno strumento analitico.

La promessa è: **See your typing rhythm. Not what you type.**

## Piattaforme

La V1 è progettata, verificata e distribuita per macOS. TypePulse nasce però
come progetto open source e il core delle metriche deve restare portabile.

Linux è previsto dopo una versione macOS stabile. Il supporto iniziale potrà
essere limitato a X11; Wayland non è incluso nei criteri di completamento della
V1. Windows non è pianificato, ma non deve essere impedito da scelte inutilmente
specifiche della piattaforma.

## Esperienza principale

1. Al primo avvio viene mostrato un onboarding di massimo tre passaggi.
2. L'utente concede a TypePulse il permesso di monitoraggio input di macOS.
3. L'app resta in background, fuori dal Dock e da `Cmd + Tab`.
4. Alla prima attività di scrittura compare l'overlay.
5. WPM e animazione reagiscono al ritmo corrente.
6. Dopo due secondi senza digitazione l'overlay scompare.
7. Dopo circa trenta secondi di inattività la sessione termina.

## Overlay

La card è larga indicativamente 180–220 pt, usa un materiale traslucido macOS,
non intercetta i click ed è sempre sopra le finestre normali. L'utente può
scegliere uno dei quattro angoli dello schermo.

L'ingresso usa fade-in e un lieve spostamento verticale di circa 150 ms. In
uscita l'animazione rallenta e la card svanisce.

| WPM | Stato visuale |
| ---: | --- |
| 0–29 | quasi fermo |
| 30–59 | movimento regolare |
| 60–89 | movimento rapido |
| 90+ | movimento intenso |
| nuovo record | breve celebrazione |

Per la V1 basta un solo personaggio con quattro stati, più l'effetto record.

## Menu bar

Il pannello compatto mostra:

- media WPM di oggi;
- picco WPM;
- parole stimate;
- tempo di digitazione attiva;
- accesso alle statistiche e alle impostazioni;
- controllo overlay, pausa, export CSV e chiusura dell'app.

## Statistiche

La finestra principale si apre solo su richiesta. Contiene quattro valori di
oggi, un grafico WPM nel tempo e un elenco sintetico degli ultimi sette giorni.
Le misurazioni vengono aggregate ogni 30 o 60 secondi: non è necessario
conservare ogni evento.

## Impostazioni

- `Start automatically`: avvio al login e avvio immediato del monitor ogni volta
  che TypePulse viene aperto;
- icona menu bar visibile;
- pausa del monitoraggio;
- reset delle statistiche di oggi;
- overlay attivo, posizione, dimensione e ritardo di scomparsa;
- contenuto overlay: WPM, animazione o entrambi;
- aspetto System, Light o Dark;
- collegamento alle impostazioni di Input Monitoring.

Le statistiche di “oggi” cambiano automaticamente alla mezzanotte della data
locale. Il nuovo giorno parte vuoto e i giorni precedenti restano disponibili
per storico ed export; non è un'eliminazione automatica dei dati.

## Calcolo WPM

Sono contati soltanto gli eventi compatibili con la scrittura: lettere, numeri,
spazio e punteggiatura. Modificatori, frecce, tasti funzione e scorciatoie sono
ignorati.

La velocità live usa una finestra mobile indicativa di 10 secondi e la
convenzione di 5 caratteri per parola:

```text
WPM = (caratteri nella finestra / 5) / durata finestra in minuti
```

Una media esponenziale attenua i salti del valore mostrato. I default validati
dalla Fase C sono finestra fissa di 10 secondi e fattore EMA 0,25. Overlay,
sessione e tempo attivo usano rispettivamente 2, 30 e 2 secondi.

## Dati salvati

Per ogni sessione:

- data e orari di inizio/fine;
- caratteri e parole stimati;
- WPM medio e massimo;
- tempo effettivo di digitazione.

L'export V1 contiene solo riepiloghi giornalieri:

```csv
date,estimated_words,average_wpm,peak_wpm,typing_minutes
2026-08-05,3840.00,58.2,104.0,52.00
```

## Fuori ambito

Account, cloud, sincronizzazione, notifiche, obiettivi, classifiche, AI,
accuratezza, dati per applicazione, heatmap, temi multipli, marketplace e Mac
App Store non rientrano nella prima versione.

Anche il supporto Linux, il repository APT e la compatibilità Wayland restano
fuori dalla V1 macOS.
