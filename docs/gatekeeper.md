# Gatekeeper e build macOS non notarizzate

## Stato della distribuzione

TypePulse è open source e gli artefatti iniziali sono firmati **ad-hoc** con
l'identità `-`. `codesign` può così verificare la coerenza interna del bundle,
ma la firma non autentica l'autore: un terzo potrebbe sostituire e rifirmare il
file. Provenienza e corruzione del download vanno quindi controllate rispetto
alla Release ufficiale e al relativo SHA-256. Non è una notarizzazione.

Di conseguenza macOS può bloccare il primo avvio di un archivio scaricato da
GitHub o installato con Homebrew. È un comportamento previsto, non un errore da
aggirare disabilitando globalmente Gatekeeper.

## Primo avvio sicuro

1. Scarica l'artefatto esclusivamente dalla GitHub Release ufficiale del tag.
2. Verifica il file `.sha256`:

   ```bash
   shasum -a 256 -c TypePulse-0.1.0-aarch64.app.zip.sha256
   ```

3. Installa tramite il cask oppure sposta `TypePulse.app` in Applicazioni.
4. Prova ad aprire TypePulse.
5. Se macOS lo blocca, apri **System Settings → Privacy & Security**, verifica
   che il messaggio nomini TypePulse e scegli **Open Anyway**.
6. Conferma nuovamente l'apertura quando richiesto.

Non eseguire comandi che rimuovono ricorsivamente la quarantena da cartelle
generiche e non disattivare Gatekeeper per tutto il sistema.

## Input Monitoring è separato

L'approvazione Gatekeeper consente di avviare l'app, ma non concede il permesso
di osservare attività di digitazione. TypePulse deve comparire separatamente in
**Privacy & Security → Input Monitoring** e il consenso resta sempre manuale.

## Limite noto

Per eliminare normalmente questo percorso manuale servono un certificato
Developer ID Application e la notarizzazione Apple. Non sono requisiti per
pubblicare il codice GPL, ma restano un possibile miglioramento distributivo.
