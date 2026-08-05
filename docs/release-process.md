# Processo di release macOS

## Stato

La pipeline è implementata ma non è ancora autorizzata a pubblicare una V1:
Gate F è aperto, non esiste un remote GitHub configurato e mancano le prove
manuali indicate nel report di Fase G.

## Versioning

QRY usa Semantic Versioning. I tag hanno forma `vMAJOR.MINOR.PATCH` oppure
`vMAJOR.MINOR.PATCH-PRERELEASE`. Prima del tag, lo stesso numero deve comparire
in:

- `TypePulse/package.json`;
- `TypePulse/src-tauri/tauri.conf.json`;
- `[workspace.package].version` in `TypePulse/Cargo.toml`;
- una sezione datata di `CHANGELOG.md`.

Verifica:

```bash
./scripts/release-audit.sh v0.1.0
```

## Pipeline GitHub

Un push di un tag `v*` esegue `.github/workflows/release-macos.yml`:

1. verifica metadati, qualità e privacy;
2. costruisce separatamente `aarch64-apple-darwin` e
   `x86_64-apple-darwin`;
3. applica e verifica la firma ad-hoc configurata da Tauri;
4. crea ZIP determinati per versione/architettura e checksum SHA-256;
5. crea una **draft GitHub Release** con entrambi gli artefatti.

La bozza deve essere ispezionata e provata prima della pubblicazione. La
pipeline non aggiorna automaticamente Homebrew e non pubblica una release
stabile senza una decisione umana.

## Dry-run locale

Sul Mac di sviluppo:

```bash
./scripts/package-macos.sh aarch64-apple-darwin
```

Gli output finiscono in `release/`, che è ignorata da Git. Su hardware e CI
compatibili va ripetuto anche per `x86_64-apple-darwin`.

## Homebrew

Il template è `packaging/homebrew/Casks/qry.rb.template`. Dopo la
pubblicazione della Release, copia i due hash dai file `.sha256` e genera il
cask:

```bash
./scripts/render-homebrew-cask.sh \
  TODO_OWNER \
  0.1.0 \
  ARM64_SHA256 \
  X86_64_SHA256
```

Prima di pubblicarlo, copia il file generato nel futuro repository
`homebrew-qry/Casks/qry.rb`. Con il tap installato, valida il cask
tramite il suo token:

```bash
ruby -c release/qry.rb
brew style --cask TODO_OWNER/qry/qry
brew audit --cask --strict TODO_OWNER/qry/qry
```

Poi verificare su un account/macchina pulita:

```bash
brew tap TODO_OWNER/qry
brew install --cask qry
brew uninstall --cask qry
```

`TODO_OWNER`, URL del repository, checksum reali e contatto security non possono
essere compilati finché il repository e la prima Release non esistono.

## Pubblicazione

Una draft può diventare release candidate solo quando tutti i task P0 richiesti
sono chiusi e la checklist manuale è allegata senza log di input. La V1 stabile
richiede inoltre Gate F chiuso, installazione Homebrew provata e nessun problema
bloccante di privacy, perdita dati o consumo risorse.
