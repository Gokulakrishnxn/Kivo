#!/usr/bin/env bash
# Package Kivo.app into a distributable .dmg.
#
# Built with hdiutil rather than Tauri's bundler, which drives Finder over
# AppleScript to style the window and therefore needs a logged-in GUI session.
# This works headless, in CI, and over SSH.
#
# Usage:  ./scripts/package-macos.sh [path/to/Kivo.app]
set -euo pipefail

APP="${1:-src-tauri/target/universal-apple-darwin/release/bundle/macos/Kivo.app}"
[ -d "$APP" ] || { echo "No app at $APP" >&2; exit 1; }

VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$APP/Contents/Info.plist")"
ARCHS="$(lipo -archs "$APP/Contents/MacOS/kivo" | tr ' ' '-')"
OUT="dist/Kivo-${VERSION}-${ARCHS}.dmg"
mkdir -p dist
rm -f "$OUT"

STAGE="$(mktemp -d)"
trap 'rm -rf "$STAGE"' EXIT

cp -R "$APP" "$STAGE/"
# The drag-to-install target. A symlink needs no AppleScript.
ln -s /Applications "$STAGE/Applications"

# An unsigned app is quarantined on download, and macOS 15 removed the old
# Control-click bypass — so the instructions have to be version-aware or people
# follow steps that silently do nothing. Ship them inside the disk image.
cp "$(dirname "$0")/../docs/first-launch.txt" "$STAGE/READ ME FIRST.txt"

hdiutil create \
  -volname "Kivo" \
  -srcfolder "$STAGE" \
  -ov -format UDZO \
  -quiet \
  "$OUT"

echo "$OUT"
shasum -a 256 "$OUT" | tee "$OUT.sha256"
