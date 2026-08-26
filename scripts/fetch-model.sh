#!/usr/bin/env bash
# Fetch the Parakeet TDT model Kivo uses for recognition.
#
# Kivo does not download this for you at runtime yet. Until the in-app model
# manager lands, this script is the documented path — and it is deliberately
# something you run yourself, so the one network request Kivo depends on is
# visible rather than implied.
#
# Usage:  ./scripts/fetch-model.sh
set -euo pipefail

# sherpa-onnx publishes Parakeet already exported to ONNX. NVIDIA's original
# weights are CC-BY-4.0, which permits redistribution.
NAME="sherpa-onnx-nemo-parakeet-tdt-0.6b-v2-int8"
URL="https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/${NAME}.tar.bz2"

case "$(uname -s)" in
  Darwin) DEST="${HOME}/Library/Application Support/Kivo/models/parakeet-tdt-0.6b" ;;
  *)      DEST="${XDG_DATA_HOME:-$HOME/.local/share}/Kivo/models/parakeet-tdt-0.6b" ;;
esac

echo "Model : ${NAME}"
echo "From  : ${URL}"
echo "Into  : ${DEST}"
echo
read -r -p "Download roughly 600 MB? [y/N] " reply
[[ "${reply}" =~ ^[Yy]$ ]] || { echo "Cancelled."; exit 0; }

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT

echo "Downloading…"
curl -fL --progress-bar "${URL}" -o "${TMP}/model.tar.bz2"

echo
echo "SHA-256 of what was downloaded:"
shasum -a 256 "${TMP}/model.tar.bz2" | awk '{print "  " $1}'
echo
echo "  Verify this against the checksum published on the sherpa-onnx release"
echo "  page before trusting it. Kivo pins this value once the in-app model"
echo "  manager ships; until then the check is yours to make."
echo
read -r -p "Continue and install? [y/N] " reply
[[ "${reply}" =~ ^[Yy]$ ]] || { echo "Stopped. Nothing was installed."; exit 0; }

mkdir -p "${DEST}"
tar -xjf "${TMP}/model.tar.bz2" -C "${TMP}"
SRC="$(find "${TMP}" -maxdepth 1 -type d -name "${NAME}*" | head -1)"
[[ -n "${SRC}" ]] || { echo "Could not find the extracted model directory." >&2; exit 1; }

# Kivo expects four artefacts with fixed names.
copy_one() {
  local pattern="$1" target="$2" found
  found="$(find "${SRC}" -maxdepth 1 -name "${pattern}" | head -1)"
  [[ -n "${found}" ]] || { echo "Missing ${pattern} in the archive." >&2; exit 1; }
  cp "${found}" "${DEST}/${target}"
  echo "  ${target}  <-  $(basename "${found}")"
}

echo "Installing:"
copy_one 'encoder*.onnx' encoder.onnx
copy_one 'decoder*.onnx' decoder.onnx
copy_one 'joiner*.onnx'  joiner.onnx
copy_one 'tokens.txt'    tokens.txt

echo
echo "Done. Open Kivo and press Reload in Settings."
