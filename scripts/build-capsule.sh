#!/usr/bin/env bash
set -euo pipefail

if [[ $# -gt 1 ]]; then
  printf 'Usage: %s [arm64|x86_64]\n' "$0" >&2
  exit 2
fi
[[ "$(uname -s)" == Darwin ]] || { printf 'The capsule requires macOS.\n' >&2; exit 1; }
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
project_root="$(cd "$script_dir/.." && pwd)"
architecture="${1:-$(uname -m)}"
case "$architecture" in
  arm64|x86_64) ;;
  *) printf 'Unsupported capsule architecture: %s\n' "$architecture" >&2; exit 2 ;;
esac
bundle="$project_root/build/macos-capsule/$architecture/QRY Capsule.app"
version="$(node -p "require(process.argv[1]).version" "$project_root/QRY/package.json")"
mkdir -p "$bundle/Contents/MacOS"
swiftc -O -swift-version 5 -target "$architecture-apple-macosx14.0" -framework AppKit -framework SwiftUI \
  "$project_root/integrations/native/QRYBridge.swift" \
  "$project_root/integrations/native/QRYPipView.swift" \
  "$project_root/integrations/macos-capsule/Sources/main.swift" \
  -o "$bundle/Contents/MacOS/QRYCapsule"
cat > "$bundle/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>app.typepulse.desktop.capsule</string>
<key>CFBundleName</key><string>QRY Capsule</string>
<key>CFBundleDisplayName</key><string>QRY Capsule</string>
<key>CFBundleExecutable</key><string>QRYCapsule</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>$version</string>
<key>LSMinimumSystemVersion</key><string>14.0</string>
<key>LSUIElement</key><true/>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
codesign --force --sign - "$bundle"
codesign --verify --strict "$bundle"
[[ "$(lipo -archs "$bundle/Contents/MacOS/QRYCapsule")" == "$architecture" ]]
if [[ "$architecture" == "$(uname -m)" ]]; then
  "$bundle/Contents/MacOS/QRYCapsule" --self-test
else
  printf 'Cross-compiled capsule; run --self-test on a %s Mac.\n' "$architecture"
fi
printf 'Built: %s\n' "$bundle"
