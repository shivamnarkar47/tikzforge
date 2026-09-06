#!/usr/bin/env bash
# Generate tauri.conf.json with bundled TeX Live resources.
#
# CI runs this AFTER prepare-texlive.sh so the texlive/ tree exists.
# The generated config is written to tauri.conf.json, which Tauri reads
# at build time. The base tauri.conf.json (without resources) is kept in
# version control for local dev where TeX Live may not be downloaded.
#
# Usage:
#   ./scripts/generate-tauri-config.sh           # generate if texlive exists
#   ./scripts/generate-tauri-config.sh --force  # always generate (CI)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_CONF="$REPO_ROOT/src-tauri/tauri.conf.json"
DEST_CONF="$REPO_ROOT/src-tauri/tauri.conf.json"
TEXLIVE_DIR="$REPO_ROOT/src-tauri/texlive"

FORCE="${1:-}"

if [[ "$FORCE" != "--force" ]]; then
  if [[ ! -d "$TEXLIVE_DIR/linux" && ! -d "$TEXLIVE_DIR/windows" ]]; then
    echo "[generate-tauri-config] No texlive/ dir found — skipping resource injection."
    echo "  Run ./scripts/prepare-texlive.sh first, or pass --force."
    exit 0
  fi
fi

# Read the base config and inject resources using jq (available on GitHub Actions).
if ! command -v jq &>/dev/null; then
  echo "[generate-tauri-config] jq not found — skipping."
  exit 0
fi

# Build the resources array from whatever platform dirs exist.
RESOURCES="[]"
if [[ -d "$TEXLIVE_DIR/linux" ]]; then
  RESOURCES=$(echo "$RESOURCES" | jq '. + ["texlive/linux/**/*"]')
fi
if [[ -d "$TEXLIVE_DIR/windows" ]]; then
  RESOURCES=$(echo "$RESOURCES" | jq '. + ["texlive/windows/**/*"]')
fi

# Inject into the bundle object.
TMP=$(mktemp)
jq --argjson resources "$RESOURCES" '.bundle.resources = $resources' "$SRC_CONF" > "$TMP"
mv "$TMP" "$DEST_CONF"

echo "[generate-tauri-config] Generated tauri.conf.json with resources:"
echo "$RESOURCES" | jq .
