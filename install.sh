#!/usr/bin/env bash
# Install TikzForge on Linux.
#
# Downloads the latest release from GitHub and installs it:
#   - AppImage → ~/.local/bin/tikzforge.AppImage + desktop entry
#   - Falls back to .deb if AppImage unavailable
#
# Usage:
#   ./install.sh                 # install latest
#   ./install.sh v0.1.0          # install specific version
#   ./install.sh --check         # print latest version, don't install

set -euo pipefail

REPO="shivamnarkar47/tikzforge"
INSTALL_DIR="$HOME/.local/bin"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/512x512/apps"
VERSION="${1:-latest}"

if [[ "$VERSION" == "--check" ]]; then
  VERSION="latest"
fi

log() { echo "[install-tikzforge] $*"; }

need_cmd() {
  if ! command -v "$1" &>/dev/null; then
    log "ERROR: $1 is required. Install it and retry."
    exit 1
  fi
}

need_cmd curl
need_cmd jq

resolve_version() {
  if [[ "$VERSION" == "latest" ]]; then
    log "Fetching latest release..."
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | jq -r '.tag_name'
  else
    echo "$VERSION"
  fi
}

download_release() {
  local ver="$1"
  log "Downloading TikzForge $ver..."

  local release_json
  release_json="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/tags/$ver")"

  local download_url=""
  local filename=""

  # Prefer AppImage on Linux.
  local urls
  urls="$(echo "$release_json" | jq -r '.assets[] | "\(.name)\t\(.browser_download_url)"')"

  while IFS=$'\t' read -r name url; do
    if [[ "$name" == *.AppImage ]]; then
      filename="$name"
      download_url="$url"
      break
    fi
  done <<< "$urls"

  # Fall back to .deb
  if [[ -z "$download_url" ]]; then
    while IFS=$'\t' read -r name url; do
      if [[ "$name" == *.deb ]]; then
        filename="$name"
        download_url="$url"
        break
      fi
    done <<< "$urls"
  fi

  if [[ -z "$download_url" ]]; then
    log "ERROR: No AppImage or .deb asset found in release $ver."
    exit 1
  fi

  log "Found asset: $filename"

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' RETURN

  curl -fSL "$download_url" -o "$tmpdir/$filename"

  echo "$tmpdir/$filename"
}

install_appimage() {
  local filepath="$1"
  mkdir -p "$INSTALL_DIR"

  local dest="$INSTALL_DIR/tikzforge.AppImage"
  cp "$filepath" "$dest"
  chmod +x "$dest"

  # Extract the AppImage to get the icon and desktop file.
  # If extraction fails, create a minimal desktop entry.
  local extract_dir
  extract_dir="$(mktemp -d)"
  trap 'rm -rf "$extract_dir"' RETURN

  ( cd "$extract_dir" && "$dest" --appimage-extract &>/dev/null ) || true

  mkdir -p "$DESKTOP_DIR" "$ICON_DIR"

  # Try to use the extracted icon.
  local squashfs_root="$extract_dir/squashfs-root"
  if [[ -d "$squashfs_root" ]]; then
    local icon_src
    icon_src="$(find "$squashfs-root" -maxdepth 2 -name '*.png' | head -1)"
    if [[ -n "$icon_src" ]]; then
      cp "$icon_src" "$ICON_DIR/tikzforge.png"
    fi

    # Use the extracted .desktop if present.
    local desktop_src
    desktop_src="$(find "$squashfs-root" -maxdepth 2 -name '*.desktop' | head -1)"
    if [[ -n "$desktop_src" ]]; then
      sed -e "s|Exec=.*|Exec=$dest %f|" \
          -e "s|Icon=.*|Icon=tikzforge|" \
          "$desktop_src" > "$DESKTOP_DIR/tikzforge.desktop"
      chmod +x "$DESKTOP_DIR/tikzforge.desktop"
    fi
  fi

  # Fallback desktop entry if extraction didn't produce one.
  if [[ ! -f "$DESKTOP_DIR/tikzforge.desktop" ]]; then
    cat > "$DESKTOP_DIR/tikzforge.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=TikzForge
Comment=Edit TikZ diagrams with live preview
Exec=$dest %f
Icon=tikzforge
Terminal=false
Categories=Development;Graphics;
MimeType=text/x-tex;
DESKTOP
    chmod +x "$DESKTOP_DIR/tikzforge.desktop"
  fi

  # Update desktop database so the entry shows up in menus.
  command -v update-desktop-database &>/dev/null && \
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true

  log "Installed to $dest"
  log "Desktop entry: $DESKTOP_DIR/tikzforge.desktop"
  log "Run with: $dest"
  log "Or launch from your application menu."
}

install_deb() {
  local filepath="$1"
  log "Installing .deb package via apt..."

  if command -v sudo &>/dev/null; then
    sudo dpkg -i "$filepath" || sudo apt-get install -f -y
  elif command -v doas &>/dev/null; then
    doas dpkg -i "$filepath" || doas apt-get install -f -y
  else
    log "ERROR: Neither sudo nor doas found. Install manually: $filepath"
    exit 1
  fi

  log "Installed via dpkg. Run with: tikzforge"
}

main() {
  log "TikzForge installer for Linux"

  local ver
  ver="$(resolve_version)"
  log "Version: $ver"

  if [[ "${1:-}" == "--check" ]]; then
    echo "Latest version: $ver"
    exit 0
  fi

  local filepath
  filepath="$(download_release "$ver")"

  if [[ "$filepath" == *.AppImage ]]; then
    install_appimage "$filepath"
  elif [[ "$filepath" == *.deb ]]; then
    install_deb "$filepath"
  else
    log "ERROR: Unknown file type: $filepath"
    exit 1
  fi

  log "Done."
}

main "$@"
