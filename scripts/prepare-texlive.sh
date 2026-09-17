#!/usr/bin/env bash
# Prepare the bundled Tectonic engine for TikzForge.
#
# Downloads a self-contained, prebuilt Tectonic binary (~20MB) and stages it
# under src-tauri/tectonic/ so Tauri's resource bundler ships it inside the
# AppImage / NSIS installer. Tectonic handles its own TeX package downloads
# at compile time — no separate TeX Live, no Perl, no CTAN installer needed.
#
# Prefer prebuilt release assets over `cargo install tectonic`: building from
# source takes 10+ minutes, needs OpenSSL/fontconfig/harfbuzz dev headers
# (plus vcpkg on Windows), and was the main source of CI flakes. A pinned
# GitHub release download takes seconds and needs only curl + tar/unzip.
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
TECTONIC_BASE="https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic%40${TECTONIC_VERSION}"
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

  log "Downloading Tectonic v${TECTONIC_VERSION} for Linux x86_64..."

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' RETURN

  curl -fsSL --retry 3 --retry-delay 5 \
    "${TECTONIC_BASE}/tectonic-${TECTONIC_VERSION}-x86_64-unknown-linux-gnu.tar.gz" \
    -o "$tmpdir/tectonic.tar.gz"

  tar -xzf "$tmpdir/tectonic.tar.gz" -C "$tmpdir"

  local matches
  matches="$(find "$tmpdir" -maxdepth 2 -name 'tectonic' -type f)"
  if [[ "$(echo "$matches" | wc -l)" -ne 1 ]]; then
    echo "[prepare-engine] Error: expected exactly one tectonic binary in the archive, found:"
    echo "$matches"
    exit 1
  fi
  cp "$matches" "$dest/tectonic"
  chmod +x "$dest/tectonic"

  log "Linux engine staged at $dest/tectonic ($(du -h "$dest/tectonic" | cut -f1))"
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
    "${TECTONIC_BASE}/tectonic-${TECTONIC_VERSION}-x86_64-pc-windows-msvc.zip" \
    -o "$tmpdir/tectonic.zip"

  unzip -q "$tmpdir/tectonic.zip" -d "$tmpdir"

  local matches
  matches="$(find "$tmpdir" -maxdepth 2 -name 'tectonic.exe' -type f)"
  if [[ "$(echo "$matches" | wc -l)" -ne 1 ]]; then
    echo "[prepare-engine] Error: expected exactly one tectonic.exe in the archive, found:"
    echo "$matches"
    exit 1
  fi
  cp "$matches" "$dest/tectonic.exe"

  log "Windows engine staged at $dest/tectonic.exe ($(du -h "$dest/tectonic.exe" | cut -f1))"
}

case "${1:-current}" in
  linux)    prepare_linux ;;
  windows)  prepare_windows ;;
  all)      prepare_linux; prepare_windows ;;
  current)
    case "$(uname -s)" in
      Linux*)              prepare_linux ;;
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
