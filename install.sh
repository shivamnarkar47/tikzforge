#!/usr/bin/env bash
# Install TikzForge on Linux.
#
# Downloads a release archive, installs the app and its bundled Tectonic
# engine under ~/.local/share/tikzforge, and links the launcher into
# ~/.local/bin.
#
# Usage:
#   ./install.sh                 # install latest
#   ./install.sh v0.4.0          # install specific version
#   ./install.sh --check         # print latest version, don't install

set -euo pipefail

REPO="shivamnarkar47/tikzforge"
MODE="install"
VERSION="latest"

if [[ "${1:-}" == "--check" ]]; then
  MODE="check"
elif [[ -n "${1:-}" ]]; then
  VERSION="$1"
fi

DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
BIN_HOME="${XDG_BIN_HOME:-$HOME/.local/bin}"
INSTALL_DIR="$DATA_HOME/tikzforge"
APP_BIN="$INSTALL_DIR/tikzforge-gpui"
LAUNCHER="$BIN_HOME/tikzforge"
DESKTOP_DIR="$DATA_HOME/applications"
ICON_DIR="$DATA_HOME/icons/hicolor/512x512/apps"
TMP_DIR=""

log() { echo "[install-tikzforge] $*" >&2; }
die() { log "ERROR: $*"; exit 1; }
need_cmd() { command -v "$1" >/dev/null 2>&1 || die "$1 is required."; }

need_cmd curl
need_cmd jq
need_cmd tar

resolve_version() {
  if [[ "$VERSION" == "latest" ]]; then
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | jq -r '.tag_name'
  else
    printf '%s\n' "$VERSION"
  fi
}

download_archive() {
  local ver="$1" json url name
  json="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/tags/$ver")" \
    || die "Release $ver not found."
  url="$(printf '%s' "$json" | jq -r '.assets[] | select(.name | endswith("-linux-x86_64.tar.gz")) | .browser_download_url' | head -n1)"
  [[ -n "$url" && "$url" != "null" ]] || die "No Linux x86_64 archive found in release $ver."
  name="$(basename "$url")"
  log "Downloading $name..."
  curl -fSL "$url" -o "$TMP_DIR/$name"
  printf '%s\n' "$TMP_DIR/$name"
}

main() {
  local ver archive
  ver="$(resolve_version)"
  [[ -n "$ver" && "$ver" != "null" ]] || die "Could not resolve version."

  if [[ "$MODE" == "check" ]]; then
    echo "Latest version: $ver"
    exit 0
  fi

  log "Version: $ver"
  TMP_DIR="$(mktemp -d)"
  trap 'rm -rf "$TMP_DIR"' EXIT

  archive="$(download_archive "$ver")"
  tar -xzf "$archive" -C "$TMP_DIR"
  [[ -x "$TMP_DIR/tikzforge-gpui" ]] || die "Archive does not contain tikzforge-gpui."
  [[ -x "$TMP_DIR/tectonic/tectonic" ]] || die "Archive does not contain the bundled Tectonic engine."

  log "Installing to $INSTALL_DIR..."
  rm -rf "$INSTALL_DIR"
  mkdir -p "$INSTALL_DIR" "$BIN_HOME" "$DESKTOP_DIR" "$ICON_DIR"
  install -Dm755 "$TMP_DIR/tikzforge-gpui" "$APP_BIN"
  cp -a "$TMP_DIR/tectonic" "$INSTALL_DIR/"
  ln -sfn "$APP_BIN" "$LAUNCHER"

  if [[ -f "$TMP_DIR/tikzforge.png" ]]; then
    install -Dm644 "$TMP_DIR/tikzforge.png" "$ICON_DIR/tikzforge.png"
  else
    curl -fsSL "https://raw.githubusercontent.com/$REPO/$ver/legacy/src-tauri/icons/icon.png" \
      -o "$ICON_DIR/tikzforge.png" || rm -f "$ICON_DIR/tikzforge.png"
  fi

  cat > "$DESKTOP_DIR/tikzforge.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=TikzForge
Comment=Edit TikZ diagrams with live preview
Exec="$LAUNCHER" %f
Icon=tikzforge
Terminal=false
Categories=Development;Graphics;
MimeType=text/x-tex;
EOF
  chmod +x "$DESKTOP_DIR/tikzforge.desktop"

  command -v update-desktop-database >/dev/null 2>&1 \
    && update-desktop-database "$DESKTOP_DIR" >/dev/null 2>&1 || true

  log "Installed to $INSTALL_DIR"
  log "Run with: $LAUNCHER"
  log "Uninstall: rm -rf '$INSTALL_DIR' '$LAUNCHER' '$DESKTOP_DIR/tikzforge.desktop' '$ICON_DIR/tikzforge.png'"
}

main "$@"
