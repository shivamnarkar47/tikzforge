#!/usr/bin/env bash
# Generate tauri.conf.json with bundled engine resources.
#
# CI runs this AFTER prepare-texlive.sh so the texlive/ and tectonic/ dirs
# exist. The generated config is written to tauri.conf.json, which Tauri
# reads at build time. The base tauri.conf.json (without resources) is kept
# in version control for local dev where the engine may not be downloaded.
#
# Usage:
#   ./scripts/generate-tauri-config.sh           # generate if engine exists
#   ./scripts/generate-tauri-config.sh --force  # always generate (CI)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_CONF="$REPO_ROOT/src-tauri/tauri.conf.json"
DEST_CONF="$REPO_ROOT/src-tauri/tauri.conf.json"
TL_DIR="$REPO_ROOT/src-tauri/texlive"
TECTONIC_DIR="$REPO_ROOT/src-tauri/tectonic"

FORCE="${1:-}"

if [[ "$FORCE" != "--force" ]]; then
  if [[ ! -d "$TL_DIR" && ! -d "$TECTONIC_DIR" ]]; then
    echo "[generate-tauri-config] No engine dir found — skipping resource injection."
    echo "  Run ./scripts/prepare-texlive.sh first, or pass --force."
    exit 0
  fi
fi

# Read the base config and inject resources using jq (available on GitHub Actions).
if ! command -v jq &>/dev/null; then
  echo "[generate-tauri-config] jq not found — skipping."
  exit 0
fi

# Build the resources array from whatever engine dirs exist.
# - Linux: full TeX Live tree (pdflatex + texmf-dist)
# - Windows: single self-contained tectonic.exe binary
RESOURCES="[]"
if [[ -d "$TL_DIR/linux" ]]; then
  RESOURCES=$(echo "$RESOURCES" | jq '. + ["texlive/linux/bin/x86_64-linux/*", "texlive/linux/texmf-dist/*"]')
fi
if [[ -d "$TECTONIC_DIR" ]]; then
  RESOURCES=$(echo "$RESOURCES" | jq '. + ["tectonic/*"]')
fi

# Inject into the bundle object.
TMP=$(mktemp)
jq --argjson resources "$RESOURCES" '.bundle.resources = $resources' "$SRC_CONF" > "$TMP"
mv "$TMP" "$DEST_CONF"

echo "[generate-tauri-config] Generated tauri.conf.json with resources:"
echo "$RESOURCES" | jq .
