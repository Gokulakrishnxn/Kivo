#!/usr/bin/env bash
# Fetch and verify Kivo's speech model.
#
# Kivo does not download this for you at runtime yet. Until the in-app model
# manager lands this script is the documented path, and it is deliberately
# something you run yourself so the one network request Kivo depends on stays
# visible rather than implied.
#
# The archive is mirrored on Kivo's own releases so the hash below can be
# pinned. The weights are NVIDIA's Parakeet TDT 0.6B v2 under CC-BY-4.0; the
# ONNX export is sherpa-onnx's. Neither is Kivo's work.
#
# Usage:  ./scripts/fetch-model.sh
set -euo pipefail

TAG="models-parakeet-tdt-0.6b-v2"
ASSET="parakeet-tdt-0.6b-v2-int8.tar.bz2"
URL="https://github.com/Gokulakrishnxn/Kivo/releases/download/${TAG}/${ASSET}"
EXPECTED="157c157bc51155e03e37d2466522a3a737dd9c72bb25f36eb18912964161e1ad"

case "$(uname -s)" in
  Darwin) DEST="${HOME}/Library/Application Support/Kivo/models/parakeet-tdt-0.6b" ;;
  *)      DEST="${XDG_DATA_HOME:-$HOME/.local/share}/Kivo/models/parakeet-tdt-0.6b" ;;
esac

if [ -f "${DEST}/encoder.onnx" ] && [ "${1:-}" != "--force" ]; then
  echo "A model is already installed at:"
  echo "  ${DEST}"
  echo "Re-run with --force to replace it."
  exit 0
fi

echo "Model : Parakeet TDT 0.6B v2, int8 (NVIDIA, CC-BY-4.0)"
echo "From  : ${URL}"
echo "Into  : ${DEST}"
echo "Size  : 460 MB download, 631 MB installed"
echo
read -r -p "Download? [y/N] " reply
[[ "${reply}" =~ ^[Yy]$ ]] || { echo "Cancelled."; exit 0; }

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

echo "Downloading…"
curl -fL --progress-bar "${URL}" -o "${TMP}/model.tar.bz2"

echo "Verifying…"
ACTUAL="$(shasum -a 256 "${TMP}/model.tar.bz2" | cut -d' ' -f1)"
if [ "${ACTUAL}" != "${EXPECTED}" ]; then
  echo >&2
  echo "SHA-256 MISMATCH — refusing to install." >&2
  echo "  expected ${EXPECTED}" >&2
  echo "  got      ${ACTUAL}" >&2
  exit 1
fi
echo "  sha-256 ok"

tar -xjf "${TMP}/model.tar.bz2" -C "${TMP}"
SRC="$(find "${TMP}" -maxdepth 1 -type d -name 'sherpa-onnx-*' | head -1)"
[ -n "${SRC}" ] || { echo "Could not find the extracted model directory." >&2; exit 1; }

mkdir -p "${DEST}"
copy_one() {
  local pattern="$1" target="$2" found
  found="$(find "${SRC}" -maxdepth 1 -name "${pattern}" | head -1)"
  [ -n "${found}" ] || { echo "Missing ${pattern} in the archive." >&2; exit 1; }
  cp "${found}" "${DEST}/${target}"
  printf '  %-14s <- %s\n' "${target}" "$(basename "${found}")"
}

echo "Installing:"
copy_one 'encoder*.onnx' encoder.onnx
copy_one 'decoder*.onnx' decoder.onnx
copy_one 'joiner*.onnx'  joiner.onnx
copy_one 'tokens.txt'    tokens.txt

echo
echo "Done. Open Kivo and press Reload in Settings."
