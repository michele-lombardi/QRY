#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  printf 'Usage: %s aarch64-apple-darwin|x86_64-apple-darwin\n' "$0" >&2
  exit 2
fi

target_triple="$1"
case "$target_triple" in
  aarch64-apple-darwin)
    artifact_arch="aarch64"
    binary_arch="arm64"
    ;;
  x86_64-apple-darwin)
    artifact_arch="x86_64"
    binary_arch="x86_64"
    ;;
  *)
    printf 'Unsupported macOS target: %s\n' "$target_triple" >&2
    exit 1
    ;;
esac

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "$script_dir/.." && pwd)"
app_root="$project_root/TypePulse"
release_dir="$project_root/release"
release_version="$(node -p "require('$app_root/package.json').version")"
bundle_path="$app_root/target/$target_triple/release/bundle/macos/TypePulse.app"
binary_path="$bundle_path/Contents/MacOS/typepulse-app"
info_plist="$bundle_path/Contents/Info.plist"
archive_path="$release_dir/TypePulse-$release_version-$artifact_arch.app.zip"

mkdir -p "$release_dir"
target_lib="$(rustc --print sysroot)/lib/rustlib/$target_triple"
if [[ ! -d "$target_lib" ]]; then
  if command -v rustup >/dev/null 2>&1; then
    rustup target add "$target_triple"
  else
    printf 'Rust target %s is not installed and rustup is unavailable.\n' "$target_triple" >&2
    exit 1
  fi
fi

cd "$app_root"
npm run tauri -- build --bundles app --target "$target_triple"

bundle_version="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$info_plist")"
if [[ "$bundle_version" != "$release_version" ]]; then
  printf 'Bundle version mismatch: expected=%s actual=%s\n' "$release_version" "$bundle_version" >&2
  exit 1
fi
binary_archs="$(lipo -archs "$binary_path")"
if [[ " $binary_archs " != *" $binary_arch "* ]]; then
  printf 'Bundle architecture mismatch: expected=%s actual=%s\n' "$binary_arch" "$binary_archs" >&2
  exit 1
fi

codesign --verify --deep --strict --verbose=2 "$bundle_path"
ditto -c -k --sequesterRsrc --keepParent "$bundle_path" "$archive_path"
shasum -a 256 "$archive_path" \
  | awk -v name="$(basename "$archive_path")" '{ print $1 "  " name }' \
  > "$archive_path.sha256"

printf 'Created %s and %s\n' "$archive_path" "$archive_path.sha256"
