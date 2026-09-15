#!/usr/bin/env bash
# Prepare the bundled Tectonic engine for TikzForge.
#
# Builds Tectonic from crates.io via `cargo install` and stages the resulting
# binary under src-tauri/tectonic/ so Tauri's resource bundler ships it inside
# the AppImage / NSIS installer.
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

check_cargo() {
  if ! command -v cargo &>/dev/null; then
    log "cargo not found — installing Rust via rustup..."

    local tmpdir
    tmpdir="$(mktemp -d)"

    case "$(uname -s)" in
      Linux*)
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
          | sh -s -- -y --no-modify-path
        ;;
      MINGW*|MSYS*|CYGWIN*)
        curl --proto '=https' --tlsv1.2 -sSf \
          -o "$tmpdir/rustup-init.exe" \
          https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe
        "$tmpdir/rustup-init.exe" -y --no-modify-path
        rm -rf "$tmpdir"
        ;;
      *)
        echo "[prepare-engine] Error: unsupported platform $(uname -s) for automatic Rust install."
        echo "  Install Rust manually from https://rustup.rs"
        exit 1
        ;;
    esac

    # Make cargo available in the current shell session.
    # shellcheck source=/dev/null
    source "${HOME}/.cargo/env"

    if ! command -v cargo &>/dev/null; then
      echo "[prepare-engine] Error: rustup ran but cargo is still not on PATH."
      echo "  Try opening a new shell or running: source ~/.cargo/env"
      exit 1
    fi
    log "Rust installed successfully."
  fi
  log "cargo found at $(command -v cargo) ($(cargo --version))"
}

prepare_linux() {
  check_cargo
  local dest="$STAGE_DIR"
  mkdir -p "$dest"

  if [[ -f "$dest/tectonic" ]]; then
    log "Tectonic already staged at $dest/tectonic — skipping."
    return 0
  fi

  log "Building Tectonic v${TECTONIC_VERSION} via cargo install (this takes a few minutes)..."

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' RETURN

  cargo install "tectonic@${TECTONIC_VERSION}" --root "$tmpdir"

  cp "$tmpdir/bin/tectonic" "$dest/tectonic"
  chmod +x "$dest/tectonic"

  log "Linux engine staged at $dest/tectonic ($(du -h "$dest/tectonic" | cut -f1))"
}

prepare_windows() {
  check_cargo

  if [[ -z "${VCPKG_ROOT:-}" ]]; then
    echo "[prepare-engine] Error: Windows Tectonic builds require VCPKG_ROOT."
    echo "  Install Tectonic's native dependencies with vcpkg and set"
    echo "  TECTONIC_DEP_BACKEND=vcpkg, VCPKG_ROOT, and VCPKGRS_TRIPLET."
    exit 1
  fi
  export TECTONIC_DEP_BACKEND="${TECTONIC_DEP_BACKEND:-vcpkg}"
  export VCPKGRS_TRIPLET="${VCPKGRS_TRIPLET:-x64-windows-static}"
  export RUSTFLAGS="${RUSTFLAGS:--Ctarget-feature=+crt-static}"

  local dest="$STAGE_DIR"
  mkdir -p "$dest"

  if [[ -f "$dest/tectonic.exe" ]]; then
    log "Tectonic already staged at $dest/tectonic.exe — skipping."
    return 0
  fi

  log "Building Tectonic v${TECTONIC_VERSION} via cargo install (this takes a few minutes)..."

  local tmpdir
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "$tmpdir"' RETURN

  cargo install "tectonic@${TECTONIC_VERSION}" --root "$tmpdir"

  cp "$tmpdir/bin/tectonic.exe" "$dest/tectonic.exe"

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
