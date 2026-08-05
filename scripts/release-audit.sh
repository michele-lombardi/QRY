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

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "$script_dir/.." && pwd)"
app_root="$project_root/TypePulse"

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

if ! rg -q "^## \\[$release_version\\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$" "$project_root/CHANGELOG.md"; then
  printf 'CHANGELOG.md has no dated section for %s\n' "$release_version" >&2
  exit 1
fi

if rg -q 'TODO_OWNER|TODO_[A-Z0-9_]+' "$project_root/.github/workflows"; then
  printf 'Release workflow contains unresolved placeholders.\n' >&2
  exit 1
fi

printf 'Release metadata is coherent for %s.\n' "$release_tag"
