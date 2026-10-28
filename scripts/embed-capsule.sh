#!/usr/bin/env bash
set -euo pipefail
if [[ $# -lt 1 || $# -gt 2 ]]; then printf 'Usage: %s QRY_APP_BUNDLE [ARCH]\n' "$0" >&2; exit 2; fi
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "$script_dir/.." && pwd)"
identifier="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$1/Contents/Info.plist")"
[[ "$identifier" == app.typepulse.desktop ]] || { printf 'Expected a QRY bundle.\n' >&2; exit 1; }
architecture="${2:-$(uname -m)}"
bash "$script_dir/build-capsule.sh" "$architecture"
mkdir -p "$1/Contents/Helpers"
ditto "$project_root/build/macos-capsule/$architecture/QRY Capsule.app" "$1/Contents/Helpers/QRY Capsule.app"
printf 'Embedded capsule: %s\n' "$1/Contents/Helpers/QRY Capsule.app"
printf 'Re-sign the parent QRY bundle before verification or distribution.\n'
