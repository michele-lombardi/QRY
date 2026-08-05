# Distribuzione

## Obiettivo

TypePulse è open source e viene sviluppato prima per macOS. Il codice sorgente è
sempre disponibile su GitHub; i binari sono una comodità aggiuntiva e non devono
diventare un requisito per contribuire.

Canali previsti:

1. compilazione locale dal repository;
2. artefatti macOS nelle GitHub Releases;
3. tap Homebrew personale;
4. pacchetto `.deb` quando inizierà il supporto Linux;
5. eventuale repository APT dopo la stabilizzazione della versione Linux.

## macOS: compilazione locale

Lo sviluppo avviene da VS Code e terminale. Tauri deve produrre `TypePulse.app`
su macOS con un comando ripetibile. L'ambiente di build necessita comunque del
toolchain e dell'SDK macOS, ma non richiede l'uso quotidiano dell'interfaccia di
Xcode.

La CLI Tauri è una dipendenza npm locale. I comandi attuali sono:

```bash
npm run tauri dev
npm run tauri build -- --bundles app
```

## GitHub Releases

Ogni tag stabile crea tramite GitHub Actions:

- app bundle per Apple Silicon;
- app bundle per Intel, finché supportato;
- checksum SHA-256;
- archivio adatto al download da Homebrew Cask;
- release notes e collegamento al codice sorgente del tag.

La scelta tra due artefatti distinti e una Universal Binary resta aperta finché
non viene misurato il costo in dimensione e tempo di build.

## Firma macOS

Open source e firma del binario sono decisioni indipendenti.

Per le prime release:

- non è necessario iscriversi al programma Apple per pubblicare il sorgente;
- le build locali possono essere eseguite senza Developer ID;
- gli artefatti possono usare una firma ad-hoc gratuita quando tecnicamente
  utile;
- la documentazione deve spiegare che Gatekeeper può richiedere un'approvazione
  manuale al primo avvio.

Una firma Developer ID e la notarizzazione potranno essere aggiunte in futuro
per rendere l'installazione più fluida. Non cambieranno licenza o apertura del
codice.

## Homebrew

TypePulse è un'app grafica, quindi il pacchetto appropriato è un **cask**.

Durante le prime versioni viene mantenuto un tap personale:

```text
github.com/<owner>/homebrew-typepulse
└── Casks/
    └── typepulse.rb
```

Installazione prevista:

```bash
brew tap <owner>/typepulse
brew install --cask typepulse
```

Il cask contiene versione, URL immutabile della GitHub Release, checksum e nome
dell'app bundle. Un workflow di release deve aggiornare il cask soltanto dopo
che l'artefatto definitivo è stato pubblicato.

L'ingresso nel repository ufficiale `homebrew/cask` è una possibilità futura,
non un requisito della V1: dipende anche dai criteri di accettazione e dalla
diffusione del progetto.

## Linux: pacchetto Debian

Quando l'adapter Linux sarà pronto, Tauri potrà generare un `.deb`. Il primo
metodo di installazione sarà intenzionalmente semplice:

```bash
sudo apt install ./typepulse_<version>_<architettura>.deb
```

Questo non equivale ancora a `sudo apt install typepulse`: quel comando richiede
che APT conosca un repository contenente il pacchetto.

## Repository APT futuro

Un repository APT di terze parti richiede almeno:

- pacchetti `.deb` per le architetture supportate;
- indici `Packages` e metadati `Release`;
- checksum;
- firma dei metadati del repository;
- chiave pubblica e istruzioni di configurazione sicure;
- strategia di aggiornamento e conservazione delle versioni.

Non verrà creato finché TypePulse non avrà una release Linux stabile. In una
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

La pipeline Linux verrà aggiunta senza modificare il flusso macOS già stabile.

## Riferimenti

- [Tauri: distribuzione](https://v2.tauri.app/distribute/)
- [Tauri: macOS application bundle](https://v2.tauri.app/distribute/macos-application-bundle/)
- [Homebrew: Adding Software](https://docs.brew.sh/Adding-Software-to-Homebrew)
- [Homebrew: Acceptable Casks](https://docs.brew.sh/Acceptable-Casks)
- [Debian Repository](https://wiki.debian.org/DebianRepository)
- [Debian: repository di terze parti](https://wiki.debian.org/DebianRepository/UseThirdParty)
