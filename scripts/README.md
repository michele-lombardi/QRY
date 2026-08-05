# Script

Automazioni ripetibili richiamabili da VS Code, terminale e GitHub Actions.

## Script disponibili

- `check.sh`: gate completo frontend, Rust e privacy;
- `audit-privacy.sh`: vieta logging runtime degli eventi, verifica capability,
  schema SQLite e DTO aggregati;
- `release-audit.sh vX.Y.Z`: controlla SemVer, versioni Cargo/npm/Tauri,
  changelog e placeholder nella pipeline;
- `package-macos.sh TARGET`: costruisce l'app firmata ad-hoc, verifica la firma,
  crea lo ZIP e il checksum SHA-256 per `aarch64` o `x86_64`;
- `render-homebrew-cask.sh OWNER VERSION ARM_SHA INTEL_SHA`: trasforma il
  template in un cask con checksum validati dentro `release/`.
- `sample-resources.sh PID [SAMPLES] [INTERVAL]`: raccoglie soltanto CPU e RSS
  aggregati per le prove idle/typing, senza osservare eventi di input.

`release/` è generata e ignorata da Git. Gli artefatti pubblici devono provenire
da un tag verificato dalla workflow, non da file locali committati.
