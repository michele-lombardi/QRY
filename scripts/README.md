# Script

Spazio riservato ad automazioni ripetibili, per esempio lint, build, test,
aggiornamento checksum Homebrew e creazione di una release. Gli script verranno
aggiunti insieme allo scaffold Tauri e dovranno essere richiamabili sia da VS
Code sia da GitHub Actions.

## Script disponibili

- `check.sh`: esegue il gate locale completo della Fase A — frontend, Rust, test
  e controllo workspace.

Gli script di bundle e release verranno aggiunti soltanto nelle relative fasi,
quando formato degli artefatti e canale GitHub/Homebrew saranno definiti.
