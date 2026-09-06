#!/usr/bin/env bash
# Build a minimal TeX Live tarball for bundling into TikzForge.
#
# Run this ONCE locally when you want to refresh the TeX Live version.
# Commit the resulting tarball via Git LFS, then CI extracts it instead of
# downloading TeX Live (fast, deterministic, no flaky installer downloads).
#
# Usage:
#   ./scripts/build-texlive-tarball.sh linux     # build linux tarball
#   ./scripts/build-texlive-tarball.sh windows   # build windows tarball
#   ./scripts/build-texlive-tarball.sh all       # build both
#
# Prerequisites:
#   - Linux: install-tl (from texlive.net installer)
#   - Windows: install-tl-windows.exe (from texlive.org)
#   - Git LFS installed and initialized (`git lfs install`)
#
# Output:
#   scripts/texlive-linux.tar.gz
#   scripts/texlive-windows.tar.gz

set -euo pipefail

TL_VERSION="2025"
TL_MIRROR="https://mirror.ctan.org/systems/texlive/tlnet"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPTS_DIR="$REPO_ROOT/scripts"

# Minimal profile: pdflatex + TikZ/PGF + hard dependencies only.
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

log() { echo "[build-texlive-tarball] $*"; }

build_linux() {
  local output="$SCRIPTS_DIR/texlive-linux.tar.gz"

  if [[ -f "$output" ]]; then
    log "Linux tarball already exists at $output — remove it to rebuild."
    return 0
  fi

  log "Building minimal TeX Live ${TL_VERSION} for Linux x86_64..."

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' RETURN

  curl -fsSL "$TL_MIRROR/install-tl-unx.tar.gz" | tar -xz -C "$tmpdir"
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

  ( cd "$installer" && ./install-tl -no-gui -profile "$tmpdir/tikzforge.profile" -repository "$TL_MIRROR" )

  if [[ -n "$TL_PACKAGES" ]]; then
    local tlmgr="$tmpdir/texlive/bin/x86_64-linux/tlmgr"
    "$tlmgr" install --no-auto-install --no-doc --no-src $TL_PACKAGES || true
  fi

  # Remove man pages and docs to keep the tarball small.
  rm -rf "$tmpdir/texlive/bin/x86_64-linux/man"
  rm -rf "$tmpdir/texlive/texmf-dist/doc"

  log "Compressing to $output..."
  tar -czf "$output" -C "$tmpdir/texlive" .

  log "Linux tarball built: $output ($(du -h "$output" | cut -f1))"
}

build_windows() {
  local output="$SCRIPTS_DIR/texlive-windows.tar.gz"

  if [[ -f "$output" ]]; then
    log "Windows tarball already exists at $output — remove it to rebuild."
    return 0
  fi

  log "Building minimal TeX Live ${TL_VERSION} for Windows x86_64..."

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' RETURN

  curl -fsSL "$TL_MIRROR/install-tl-windows.exe" -o "$tmpdir/install-tl-windows.exe"

  cat > "$tmpdir/tikzforge.profile" <<PROFILE
selected_scheme scheme-minimal
TEXDIR $tmpdir/texlive
TEXMFLOCAL $tmpdir/texlive/texmf-local
TEXMFSYSCONFIG $tmpdir/texlive/texmf-config
TEXMFSYSVAR $tmpdir/texlive/texmf-var
TEXMFHOME ~/texmf
TEXMFCONFIG ~/.texlive/texmf-config
TEXMFVAR ~/.texlive/texmf-var
binary_win32 1
instopt_adjustpath 0
instopt_adjustrepo 1
instopt_letter 0
instopt_portable 0
instopt_write18_restricted 1
tlpdbopt_autobackup 0
tlpdbopt_install_docfiles 0
tlpdbopt_install_srcfiles 0
PROFILE

  ( cd "$tmpdir" && ./install-tl-windows.exe -no-gui -profile tikzforge.profile -repository "$TL_MIRROR" < /dev/null )

  if [[ -n "$TL_PACKAGES" ]]; then
    local tlmgr="$tmpdir/texlive/bin/win32/tlmgr"
    "$tlmgr" install --no-auto-install --no-doc --no-src $TL_PACKAGES < /dev/null || true
  fi

  rm -rf "$tmpdir/texlive/bin/win32/man"
  rm -rf "$tmpdir/texlive/texmf-dist/doc"

  log "Compressing to $output..."
  tar -czf "$output" -C "$tmpdir/texlive" .

  log "Windows tarball built: $output ($(du -h "$output" | cut -f1))"
}

case "${1:-all}" in
  linux)    build_linux ;;
  windows)  build_windows ;;
  all)      build_linux; build_windows ;;
  *)
    echo "Usage: $0 [linux|windows|all]"
    exit 1
    ;;
esac

log "Done. Commit the tarball(s) via Git LFS:"
log "  git lfs install"
log "  git add scripts/.gitattributes scripts/texlive-*.tar.gz"
log "  git commit -m \"chore: add pre-built TeX Live tarball\""
