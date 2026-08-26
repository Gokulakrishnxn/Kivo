#!/usr/bin/env bash
# Sign (and optionally notarise) Kivo.app.
#
# Three modes, chosen by what you have available:
#
#   ./scripts/sign-macos.sh
#       Ad-hoc signature. No certificate, no Apple account, no cost. The app
#       is valid and runs on this Mac. It will NOT run on anyone else's
#       without them right-clicking to bypass Gatekeeper.
#
#   APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)" \
#   ./scripts/sign-macos.sh
#       A real Developer ID signature. Distributable, but macOS will still
#       warn on first launch until it is also notarised.
#
#   APPLE_SIGNING_IDENTITY="..." NOTARY_PROFILE=kivo \
#   ./scripts/sign-macos.sh
#       Signed and notarised. This is the shippable artefact.
#
# This script never asks for, reads, or stores your Apple ID password. See
# docs/SIGNING.md for how to store notarisation credentials yourself.
set -euo pipefail

# Note: macOS ships bash 3.2, where expanding an empty array under `set -u`
# is an error — hence the ${arr[@]+"${arr[@]}"} guards below.

# Accepts an explicit bundle path; defaults to the universal build, then the
# host-arch one.
APP="${1:-}"
if [ -z "$APP" ]; then
  for candidate in \
    "src-tauri/target/universal-apple-darwin/release/bundle/macos/Kivo.app" \
    "src-tauri/target/release/bundle/macos/Kivo.app"; do
    [ -d "$candidate" ] && { APP="$candidate"; break; }
  done
fi
[ -d "$APP" ] || { echo "No app found — run 'pnpm tauri build --bundles app' first." >&2; exit 1; }
echo "Bundle: $APP"

IDENTITY="${APPLE_SIGNING_IDENTITY:--}"
ENTITLEMENTS="src-tauri/Entitlements.plist"

if [ "$IDENTITY" = "-" ]; then
  echo "Signing ad-hoc (no certificate)."
  RUNTIME_FLAG=()
else
  echo "Signing as: $IDENTITY"
  # The hardened runtime is required for notarisation and rejected without a
  # real identity, so it is only applied when one is present.
  RUNTIME_FLAG=(--options runtime)
fi

# Nested code must be signed before the bundle that contains it, innermost
# first, or the outer signature seals a mismatched hash.
echo "Signing nested libraries…"
find "$APP/Contents/Frameworks" -name '*.dylib' -print0 2>/dev/null | while IFS= read -r -d '' lib; do
  codesign --force --timestamp=none ${RUNTIME_FLAG[@]+"${RUNTIME_FLAG[@]}"} --sign "$IDENTITY" "$lib"
  printf '  %s\n' "$(basename "$lib")"
done

echo "Signing the bundle…"
codesign --force --timestamp=none ${RUNTIME_FLAG[@]+"${RUNTIME_FLAG[@]}"} \
         --entitlements "$ENTITLEMENTS" \
         --sign "$IDENTITY" "$APP"

echo "Verifying…"
codesign --verify --deep --strict --verbose=2 "$APP" 2>&1 | sed 's/^/  /'

if [ "$IDENTITY" = "-" ]; then
  echo
  echo "Done — ad-hoc signed. Gatekeeper assessment will fail by design:"
  spctl --assess --type execute "$APP" 2>&1 | sed 's/^/  /' || true
  echo
  echo "To distribute this to anyone else you need a Developer ID."
  echo "See docs/SIGNING.md."
  exit 0
fi

if [ -z "${NOTARY_PROFILE:-}" ]; then
  echo
  echo "Signed. Set NOTARY_PROFILE to also notarise. See docs/SIGNING.md."
  exit 0
fi

echo "Notarising (this uploads the app to Apple and can take several minutes)…"
ZIP="$(mktemp -d)/Kivo.zip"
ditto -c -k --keepParent "$APP" "$ZIP"
xcrun notarytool submit "$ZIP" --keychain-profile "$NOTARY_PROFILE" --wait

echo "Stapling the ticket…"
xcrun stapler staple "$APP"
xcrun stapler validate "$APP"

echo "Final assessment:"
spctl --assess --type execute --verbose=2 "$APP" 2>&1 | sed 's/^/  /'
echo
echo "Done. This app will open cleanly on any Mac."
