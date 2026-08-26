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

# An unsigned app is quarantined on download, and the failure mode — "Kivo is
# damaged and can't be opened" — reads like a corrupt file rather than a
# missing signature. Ship the explanation inside the disk image.
cat > "$STAGE/READ ME FIRST.txt" <<'NOTE'
Kivo — first launch on macOS
============================

Kivo is not signed with an Apple Developer ID, so macOS will refuse to open
it the first time and may say the app is "damaged". It is not damaged. That
message is what macOS shows for any app it cannot trace to a paid Apple
developer account.

To open it:

  1. Drag Kivo into Applications.
  2. Right-click (or Control-click) Kivo in Applications.
  3. Choose "Open", then "Open" again in the dialog.

You only do this once.

If that does not work, remove the download quarantine flag directly:

  xattr -dr com.apple.quarantine /Applications/Kivo.app

Verify what you downloaded before running any of the above. The SHA-256 of
this disk image is published on the release page it came from.

Kivo needs two permissions:
  * Microphone   — asked for on your first dictation
  * Accessibility — System Settings > Privacy & Security > Accessibility.
                    This is how Kivo sees your hotkey and inserts text.
                    It reads nothing from your screen.

Source: https://github.com/Gokulakrishnxn/Kivo
NOTE

hdiutil create \
  -volname "Kivo" \
  -srcfolder "$STAGE" \
  -ov -format UDZO \
  -quiet \
  "$OUT"

echo "$OUT"
shasum -a 256 "$OUT" | tee "$OUT.sha256"
