# Distribuzione

## Obiettivo

QRY è open source e viene sviluppato prima per macOS. Il codice sorgente è
sempre disponibile su GitHub; i binari sono una comodità aggiuntiva e non devono
diventare un requisito per contribuire.

Canali previsti:

1. compilazione locale dal repository;
2. artefatti macOS nelle GitHub Releases;
3. tap Homebrew personale;
4. pacchetto `.deb` quando inizierà il supporto Linux;
5. eventuale repository APT dopo la stabilizzazione della versione Linux.

## macOS: compilazione locale

Lo sviluppo avviene da VS Code e terminale. Tauri deve produrre `QRY.app`
su macOS con un comando ripetibile. L'ambiente di build necessita comunque del
toolchain e dell'SDK macOS, ma non richiede l'uso quotidiano dell'interfaccia di
Xcode.

La CLI Tauri è una dipendenza npm locale. I comandi attuali sono:

```bash
npm run tauri dev
npm run tauri build -- --bundles app
```

## GitHub Releases

La workflow `.github/workflows/release-macos.yml` reagisce ai tag `v*`, esegue
il gate completo e crea una draft GitHub Release contenente:

- app bundle per Apple Silicon;
- app bundle per Intel;
- un checksum SHA-256 per ogni archivio;
- archivio adatto al download da Homebrew Cask;
- release notes e collegamento al codice sorgente del tag.

La V1 usa due artefatti distinti:

```text
QRY-VERSION-aarch64.app.zip
QRY-VERSION-x86_64.app.zip
```

Questa scelta evita una fase `lipo` non ancora necessaria e permette al cask di
selezionare architettura e checksum corretti. La Release resta in bozza finché
le checklist manuali non sono completate.

## Firma macOS

Open source e firma del binario sono decisioni indipendenti.

Per le prime release:

- non è necessario iscriversi al programma Apple per pubblicare il sorgente;
- le build locali possono essere eseguite senza Developer ID;
- gli artefatti usano la firma ad-hoc `-`, verificata prima dell'archivio;
- la documentazione deve spiegare che Gatekeeper può richiedere un'approvazione
  manuale al primo avvio.

Una firma Developer ID e la notarizzazione potranno essere aggiunte in futuro
per rendere l'installazione più fluida. Non cambieranno licenza o apertura del
codice.

La firma ad-hoc non identifica l'autore e non sostituisce la notarizzazione. La
procedura utente è in [`gatekeeper.md`](gatekeeper.md).

## Homebrew

QRY è un'app grafica, quindi il pacchetto appropriato è un **cask**.

Durante le prime versioni viene mantenuto un tap personale:

```text
github.com/michele-lombardi/homebrew-qry
└── Casks/
    └── qry.rb
```

Installazione prevista:

```bash
brew tap michele-lombardi/qry
brew install --cask qry
```

Il cask contiene versione, URL immutabile della GitHub Release, checksum per
architettura e nome dell'app bundle. Il template versionato è in
`packaging/homebrew/Casks/qry.rb.template`; lo script di rendering accetta
solo owner semplice, SemVer e hash SHA-256 da 64 cifre. Viene generato soltanto
dopo che gli artefatti definitivi sono stati pubblicati.

L'ingresso nel repository ufficiale `homebrew/cask` è una possibilità futura,
non un requisito della V1: dipende anche dai criteri di accettazione e dalla
diffusione del progetto.

## Linux: pacchetto Debian

Quando l'adapter Linux sarà pronto, Tauri potrà generare un `.deb`. Il primo
metodo di installazione sarà intenzionalmente semplice:

```bash
sudo apt install ./qry_<version>_<architettura>.deb
```

Questo non equivale ancora a `sudo apt install qry`: quel comando richiede
che APT conosca un repository contenente il pacchetto.

## Repository APT futuro

Un repository APT di terze parti richiede almeno:

- pacchetti `.deb` per le architetture supportate;
- indici `Packages` e metadati `Release`;
- checksum;
- firma dei metadati del repository;
- chiave pubblica e istruzioni di configurazione sicure;
- strategia di aggiornamento e conservazione delle versioni.

Non verrà creato finché QRY non avrà una release Linux stabile. In una
fase ancora successiva si potrà valutare l'inclusione nei repository ufficiali
delle distribuzioni.

## Automazione desiderata

```text
tag Git
  ↓
test Rust + test frontend
  ↓
build su runner macOS
  ↓
GitHub Release + checksum
  ↓
aggiornamento del tap Homebrew
```

La pipeline fino alla draft Release è implementata. L'aggiornamento del tap è
intenzionalmente manuale per la prima versione e resta bloccato finché non
esistono owner GitHub, repository remoto e checksum pubblici. La procedura
operativa completa è in [`release-process.md`](release-process.md).

La pipeline Linux verrà aggiunta senza modificare il flusso macOS già stabile.

## Riferimenti

- [Tauri: distribuzione](https://v2.tauri.app/distribute/)
- [Tauri: macOS application bundle](https://v2.tauri.app/distribute/macos-application-bundle/)
- [Homebrew: Adding Software](https://docs.brew.sh/Adding-Software-to-Homebrew)
- [Homebrew: Acceptable Casks](https://docs.brew.sh/Acceptable-Casks)
- [Debian Repository](https://wiki.debian.org/DebianRepository)
- [Debian: repository di terze parti](https://wiki.debian.org/DebianRepository/UseThirdParty)
