#!/usr/bin/env bash
# Prepare the bundled minimal TeX Live for TikzForge.
#
# Downloads a trimmed TeX Live (pdflatex + TikZ/PGF + deps only) and stages it
# under src-tauri/texlive/<platform>/ so Tauri's resource bundler ships it
# inside the AppImage / NSIS installer.
#
# Usage:
#   ./scripts/prepare-texlive.sh           # prepare for current platform
#   ./scripts/prepare-texlive.sh linux     # prepare Linux bundle
#   ./scripts/prepare-texlive.sh windows   # prepare Windows bundle
#   ./scripts/prepare-texlive.sh all       # prepare both
#
# On CI this runs before `tauri build`. The resulting tree is cached by the
# workflow keyed on the TeX Live version + package list hash.

set -euo pipefail

TL_VERSION="2025"
TL_MIRROR="https://mirror.ctan.org/systems/texlive/tlnet"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAGE_DIR="$REPO_ROOT/src-tauri/texlive"

# Minimal profile: pdflatex + TikZ/PGF + hard dependencies only.
# ~300MB installed. Do NOT add luatex/xetex — they bloat the bundle
# and are not needed for pdflatex compilation.
TL_PACKAGES="\
  scheme-minimal \
  latex-bin \
  latex-fonts \
  latexconfig \
  pgf \
  tikz-cd \
  xcolor \
  graphics \
  graphics-cfg \
  graphics-def \
  l3kernel \
  l3packages \
  l3backend \
  l3experimental \
  amsmath \
  amscls \
  amsfonts \
  babel \
  babel-english \
  ec \
  etoolbox \
  fontspec \
  geometry \
  glyphlist \
  hyph-utf8 \
  hyphen-base \
  iftex \
  inputenx \
  kvoptions \
  kvsetkeys \
  oberdiek \
  pdftexcmds \
  psnfss \
  tikzfill \
  translator \
  unicode-data \
  upquote \
  url \
  xkeyval \
"

log() { echo "[prepare-texlive] $*"; }

prepare_linux() {
  local dest="$STAGE_DIR/linux/bin/x86_64-linux"
  mkdir -p "$dest"

  if [[ -f "$dest/pdflatex" ]]; then
    log "Linux pdflatex already staged at $dest/pdflatex — skipping download."
    return 0
  fi

  log "Downloading minimal TeX Live ${TL_VERSION} for Linux x86_64..."

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' RETURN

  # Use the direct TUG FTP mirror — mirror.ctan.org redirects to random
  # mirrors (like mirror.clarkson.edu) that may be down or slow.
  # --retry 3 handles transient network failures on CI.
  curl -fsSL --retry 3 --retry-delay 5 "https://ftp.tug.org/texlive/tlnet/install-tl-unx.tar.gz" | tar -xz -C "$tmpdir"
  local installer
  installer="$(find "$tmpdir" -maxdepth 2 -type d -name 'install-tl-*' | head -1)"

  cat > "$tmpdir/tikzforge.profile" <<PROFILE
selected_scheme scheme-minimal
TEXDIR $tmpdir/texlive
TEXMFLOCAL $tmpdir/texlive/texmf-local
TEXMFSYSCONFIG $tmpdir/texlive/texmf-config
TEXMFSYSVAR $tmpdir/texlive/texmf-var
TEXMFHOME ~/texmf
TEXMFCONFIG ~/.texlive/texmf-config
TEXMFVAR ~/.texlive/texmf-var
binary_x86_64-linux 1
instopt_adjustpath 0
instopt_adjustrepo 1
instopt_letter 0
instopt_portable 0
instopt_write18_restricted 1
tlpdbopt_autobackup 0
tlpdbopt_install_docfiles 0
tlpdbopt_install_srcfiles 0
PROFILE

  ( cd "$installer" && timeout 900 ./install-tl -no-gui -profile "$tmpdir/tikzforge.profile" -repository "$TL_MIRROR" )

  if [[ -n "$TL_PACKAGES" ]]; then
    log "Installing additional packages: $TL_PACKAGES"
    local tlmgr="$tmpdir/texlive/bin/x86_64-linux/tlmgr"
    # shellcheck disable=SC2086
    "$tlmgr" install --no-auto-install --no-doc --no-src $TL_PACKAGES || true
  fi

  cp -a "$tmpdir/texlive/bin/x86_64-linux/." "$dest/"

  # Remove man pages — they bloat the bundle and may contain symlinks that
  # break Tauri's resource validator.
  rm -rf "$dest/man"

  mkdir -p "$STAGE_DIR/linux/texmf-dist"
  cp -a "$tmpdir/texlive/texmf-dist/." "$STAGE_DIR/linux/texmf-dist/" 2>/dev/null || true

  log "Linux bundle staged at $dest"
}

prepare_windows() {
  local dest="$STAGE_DIR/tectonic"
  mkdir -p "$dest"

  if [[ -f "$dest/tectonic.exe" ]]; then
    log "Tectonic already staged at $dest/tectonic.exe — skipping."
    return 0
  fi

  log "Downloading Tectonic for Windows x86_64..."

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' RETURN

  # Tectonic ships a single self-contained .zip per release.
  # Pin a specific version — the "latest" redirect sometimes 404s on GitHub
  # Actions runners due to rate limiting.
  curl -fsSL "https://github.com/tectonic-typesetting/tectonic/releases/download/tectonic%400.17.0/tectonic-0.17.0-x86_64-pc-windows-msvc.zip" \
    -o "$tmpdir/tectonic.zip"

  unzip -q "$tmpdir/tectonic.zip" -d "$tmpdir"

  # Copy the binary into the staging dir.
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
