#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  printf 'Usage: %s vMAJOR.MINOR.PATCH[-PRERELEASE]\n' "$0" >&2
  exit 2
fi

release_tag="$1"
if [[ ! "$release_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  printf 'Release tag is not valid SemVer: %s\n' "$release_tag" >&2
  exit 1
fi
release_version="${release_tag#v}"

# WiX maps a SemVer prerelease to the fourth MSI version field. Tauri requires
# that value to be a single integer in the Windows Installer range.
if [[ "$release_version" == *-* ]]; then
  prerelease_version="${release_version#*-}"
  if [[ ! "$prerelease_version" =~ ^(0|[1-9][0-9]*)$ ]] ||
    [[ ${#prerelease_version} -gt 5 ]] ||
    { [[ ${#prerelease_version} -eq 5 ]] && [[ "$prerelease_version" > "65535" ]]; }; then
    printf 'Windows MSI prerelease identifier must be one integer from 0 to 65535: %s\n' "$prerelease_version" >&2
    exit 1
  fi
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "$script_dir/.." && pwd)"
app_root="$project_root/QRY"

package_version="$(node -p "require('$app_root/package.json').version")"
tauri_version="$(node -p "require('$app_root/src-tauri/tauri.conf.json').version")"
cargo_versions="$(
  cd "$app_root"
  cargo metadata --no-deps --format-version 1 \
    | node -e 'let s="";process.stdin.on("data",d=>s+=d).on("end",()=>console.log([...new Set(JSON.parse(s).packages.map(p=>p.version))].join("\n")))'
)"

for actual in "$package_version" "$tauri_version" $cargo_versions; do
  if [[ "$actual" != "$release_version" ]]; then
    printf 'Version mismatch: tag=%s metadata=%s\n' "$release_version" "$actual" >&2
    exit 1
  fi
done

if ! grep -Eq "^## \\[$release_version\\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$" "$project_root/CHANGELOG.md"; then
  printf 'CHANGELOG.md has no dated section for %s\n' "$release_version" >&2
  exit 1
fi

if grep -Eqr 'TODO_OWNER|TODO_[A-Z0-9_]+' "$project_root/.github/workflows"; then
  printf 'Release workflow contains unresolved placeholders.\n' >&2
  exit 1
fi

node - "$app_root/src-tauri/tauri.windows.conf.json" <<'NODE'
const fs = require("node:fs");
const config = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const targets = config.bundle?.targets;
const webview = config.bundle?.windows?.webviewInstallMode?.type;
const installMode = config.bundle?.windows?.nsis?.installMode;
if (JSON.stringify(targets) !== JSON.stringify(["nsis", "msi"])) {
  throw new Error(`unexpected Windows bundle targets: ${targets}`);
}
if (webview !== "downloadBootstrapper" || installMode !== "currentUser") {
  throw new Error(`unsafe Windows installer defaults: webview=${webview} installMode=${installMode}`);
}
NODE

printf 'Release metadata is coherent for %s.\n' "$release_tag"
