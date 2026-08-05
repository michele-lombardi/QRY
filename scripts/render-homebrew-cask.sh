#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 4 ]]; then
  printf 'Usage: %s OWNER VERSION ARM64_SHA256 X86_64_SHA256\n' "$0" >&2
  exit 2
fi

github_owner="$1"
release_version="$2"
arm64_sha="$3"
x86_64_sha="$4"

if [[ ! "$github_owner" =~ ^[A-Za-z0-9_.-]+$ ]]; then
  printf 'Invalid GitHub owner: %s\n' "$github_owner" >&2
  exit 1
fi
if [[ ! "$release_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  printf 'Invalid release version: %s\n' "$release_version" >&2
  exit 1
fi
for checksum in "$arm64_sha" "$x86_64_sha"; do
  if [[ ! "$checksum" =~ ^[0-9a-f]{64}$ ]]; then
    printf 'Invalid SHA-256: %s\n' "$checksum" >&2
    exit 1
  fi
done

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "$script_dir/.." && pwd)"
template="$project_root/packaging/homebrew/Casks/qry.rb.template"
release_dir="$project_root/release"
output="$release_dir/qry.rb"

mkdir -p "$release_dir"
sed \
  -e "s/__OWNER__/$github_owner/g" \
  -e "s/__VERSION__/$release_version/g" \
  -e "s/__ARM64_SHA256__/$arm64_sha/g" \
  -e "s/__X86_64_SHA256__/$x86_64_sha/g" \
  "$template" > "$output"

ruby -c "$output"
printf 'Rendered %s\n' "$output"
