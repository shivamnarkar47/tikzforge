#!/usr/bin/env bash
# Prepare the bundled Tectonic engine for TikzForge.
#
# Downloads a self-contained Tectonic binary (~100MB) and stages it under
# src-tauri/tectonic/ so Tauri's resource bundler ships it inside the
# AppImage / NSIS installer. Tectonic handles its own dependency downloads
# — no separate TeX Live, no Perl, no CTAN installer needed.
#
# Usage:
#   ./scripts/prepare-texlive.sh           # prepare for current platform
#   ./scripts/prepare-texlive.sh linux     # prepare Linux bundle
#   ./scripts/prepare-texlive.sh windows   # prepare Windows bundle
#   ./scripts/prepare-texlive.sh all       # prepare both
#
# On CI this runs before `tauri build`. The resulting tree is cached by the
# workflow keyed on the Tectonic version.

set -euo pipefail

TECTONIC_VERSION="0.17.0"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAGE_DIR="$REPO_ROOT/src-tauri/tectonic"

log() { echo "[prepare-engine] $*"; }

prepare_linux() {
  local dest="$STAGE_DIR"
  mkdir -p "$dest"

  if [[ -f "$dest/tectonic" ]]; then
    log "Tectonic already staged at $dest/tectonic — skipping."
    return 0
  fi

  log "Installing Tectonic for Linux x86_64..."

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' RETURN

  # Tectonic's drop-sh installer extracts the binary into the current
  # directory. Run it from the temp dir, then copy the binary out.
  ( cd "$tmpdir" && curl --proto '=https' --tlsv1.2 -fsSL https://drop-sh.fullyjustified.net | sh )

  cp "$tmpdir/tectonic" "$dest/tectonic"
  chmod +x "$dest/tectonic"

  log "Linux engine (Tectonic) staged at $dest/tectonic"
}

prepare_windows() {
  local dest="$STAGE_DIR"
  mkdir -p "$dest"

  if [[ -f "$dest/tectonic.exe" ]]; then
    log "Tectonic already staged at $dest/tectonic.exe — skipping."
    return 0
  fi

  log "Downloading Tectonic v${TECTONIC_VERSION} for Windows x86_64..."

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' RETURN

  curl -fsSL --retry 3 --retry-delay 5 \
    "https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic%40${TECTONIC_VERSION}/tectonic-${TECTONIC_VERSION}-x86_64-pc-windows-msvc.zip" \
    -o "$tmpdir/tectonic.zip"

  unzip -q "$tmpdir/tectonic.zip" -d "$tmpdir"

  find "$tmpdir" -name 'tectonic.exe' -exec cp {} "$dest/" \;

  log "Windows engine (Tectonic) staged at $dest/tectonic.exe"
}

case "${1:-current}" in
  linux)    prepare_linux ;;
  windows)  prepare_windows ;;
  all)      prepare_linux; prepare_windows ;;
  current)
    case "$(uname -s)" in
      Linux*)  prepare_linux ;;
      MINGW*|MSYS*|CYGWIN*) prepare_windows ;;
      *) echo "Unknown platform $(uname -s)"; exit 1 ;;
    esac
    ;;
  *)
    echo "Usage: $0 [linux|windows|all|current]"
    exit 1
    ;;
esac

log "Done. Sample of staged tree:"
find "$STAGE_DIR" -type f 2>/dev/null | head -20 || true
