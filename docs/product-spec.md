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
4. Alla prima attività di scrittura compare l'overlay sul display della finestra
   focalizzata quando l'utente ha concesso Accessibilità; altrimenti usa il
   display principale.
5. WPM e animazione reagiscono al ritmo corrente.
6. Dopo due secondi senza digitazione l'overlay scompare.
7. Dopo circa trenta secondi di inattività la sessione termina.

## Overlay

La card è larga indicativamente 180–220 pt, usa un materiale traslucido macOS,
non intercetta i click ed è sempre sopra le finestre normali. L'utente può
scegliere uno dei quattro angoli dello schermo. Il monitor di destinazione segue
il centro della finestra focalizzata e non la posizione del mouse.

L'ingresso usa fade-in e un lieve spostamento verticale di circa 150 ms. In
uscita l'animazione rallenta e la card svanisce.

| Trigger | Stato Pip |
| ---: | --- |
| 0 WPM | Breathe; normalmente non visibile perché l'overlay idle è nascosto |
| 1–69 WPM | Walk; passo più rapido nella fascia core fast |
| 70+ WPM | Run, inclinazione e dash lines |
| 90+ minuti attivi nella sessione | Tired, senza giudicare l'utente |
| nuovo record | Jump e Cheer verde, una sola volta |

Per la V1 basta un solo Pip con questi comportamenti misurabili. Dance e Sleep
sono backlog esplicito perché richiedono rispettivamente una metrica di stabilità
e una decisione sul lifecycle idle.

## Menu bar

TypePulse vive come icona di stato nella parte destra della menu bar di macOS,
insieme alle altre applicazioni in background. Non mantiene un'icona nel Dock e
non compare in `Cmd + Tab`. Il click apre il pannello; la chiusura del pannello
non termina il monitoraggio. Un menu contestuale offre almeno start, pausa,
visibilità del WPM nella menu bar, impostazioni e uscita completa. Quando il
numero è visibile occupa uno slot stabile di tre cifre, così il Pulse non si
sposta durante gli aggiornamenti; disabilitarlo non modifica il PiP.

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
- WPM nella menu bar visibile o nascosto;
- pausa del monitoraggio;
- reset delle statistiche di oggi;
- overlay attivo, posizione, dimensione e ritardo di scomparsa;
- contenuto overlay: WPM, animazione o entrambi;
- aspetto System, Light o Dark;
- collegamento alle impostazioni di Input Monitoring.
- stato e collegamento separati al permesso opzionale Accessibilità per seguire
  il display focalizzato.

Le statistiche di “oggi” cambiano automaticamente alla mezzanotte della data
locale. Il nuovo giorno parte vuoto e i giorni precedenti restano disponibili
per storico ed export; non è un'eliminazione automatica dei dati.

## Calcolo WPM

Sono contati soltanto gli eventi compatibili con la scrittura: lettere, numeri,
spazio e punteggiatura. Modificatori, frecce, tasti funzione, scorciatoie e
auto-repeat del sistema sono ignorati. Due pressioni identiche consecutive sono
ammesse per le doppie lettere; una sequenza continua dal terzo tasto identico
viene ignorata fino a cambio tasto o pausa di un secondo.

La velocità live usa un lookback mobile massimo di 10 secondi e la convenzione
di 5 caratteri per parola. Durante l'avvio usa subito la durata realmente
osservata tra prima e ultima attività:

```text
WPM = ((attività osservate - 1) / 5) / minuti osservati
```

La prima stima richiede almeno 250 ms di osservazione ed è limitata a 300 WPM;
normalmente compare dopo 2–4 caratteri. Una media esponenziale attenua i salti
successivi. I default validati dalla Fase C sono lookback massimo di 10 secondi
e fattore EMA 0,25. Overlay,
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
