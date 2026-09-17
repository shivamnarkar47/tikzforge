#!/usr/bin/env bash
# Verify the staged Tectonic engine before a Tauri build.
#
# `src-tauri/tauri.conf.json` declares `tectonic/*` as bundled resources, so
# a build without a staged binary fails late inside Tauri with a bare
# `GlobPathNotFound`. This script fails early instead, with a message that
# tells the developer exactly what to run.
#
# CI runs this after `prepare-texlive.sh` and before `tauri build`.
# The `--force` flag is accepted for backwards compatibility with existing
# workflow files and behaves the same as a bare invocation.
#
# Usage:
#   ./scripts/generate-tauri-config.sh [--force]

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TECTONIC_DIR="$REPO_ROOT/src-tauri/tectonic"
CONF="$REPO_ROOT/src-tauri/tauri.conf.json"

if [[ ! -f "$TECTONIC_DIR/tectonic" && ! -f "$TECTONIC_DIR/tectonic.exe" ]]; then
  echo "[generate-tauri-config] Error: no staged Tectonic binary found in $TECTONIC_DIR."
  echo "  Run ./scripts/prepare-texlive.sh first (linux | windows | all)."
  echo "  Without it, the installer ships no engine and the app reports 'LaTeX not found' on launch."
  exit 1
fi

if ! grep -q '"tectonic/\*"' "$CONF"; then
  echo "[generate-tauri-config] Error: $CONF does not declare tectonic/* resources."
  exit 1
fi

echo "[generate-tauri-config] OK. Staged engine:"
find "$TECTONIC_DIR" -type f 2>/dev/null
