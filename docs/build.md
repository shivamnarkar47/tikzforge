# Building TikzForge

## Prerequisites

- **Rust** (stable toolchain)
- **System deps** (Linux): `gtk3`, `libxkbcommon-x11`, `wayland`, Vulkan
  loader, `fontconfig`, `freetype2`, `harfbuzz`, `graphite`, `icu`,
  `libpng`, `openssl`, `zlib`, `pkgconf`, `clang`
- **Windows**: MSVC Rust toolchain

## Quick Start

```bash
cargo run -p tikzforge-gpui
```

Production build:

```bash
cargo build -p tikzforge-gpui --release
```

Output: `target/release/tikzforge-gpui` (or `.exe` on Windows).

## Bundled Tectonic engine

Releases ship a self-contained Tectonic binary so compilation works without
a separate LaTeX install. Stage it before packaging:

```bash
bash scripts/prepare-texlive.sh linux      # or `windows`, or `all`
```

The script downloads the pinned prebuilt Tectonic release (v0.17.0) and
stages it under `legacy/src-tauri/tectonic/`. For a portable build, place it
next to the app binary:

```bash
mkdir -p target/release/tectonic
cp legacy/src-tauri/tectonic/tectonic target/release/tectonic/
```

At runtime `resolve_engine` probes `tectonic/` next to the executable, the
source tree used by `cargo run`, then `tectonic` on `PATH`.

Tectonic fetches its TeX package bundle from the network on first compile
and caches it locally, so the first compile needs connectivity even though
engine detection works offline.

## Release packaging

`.github/workflows/release-gpui.yml` builds the app, stages Tectonic, and
produces:

| Platform | Asset |
|----------|-------|
| Linux | `tikzforge-gpui-<tag>-linux-x86_64.tar.gz` |
| Windows | `tikzforge-gpui-<tag>-windows-x86_64.zip` |

Both archives contain the binary, the `tectonic/` directory, and an icon.
Extract and keep `tectonic/` next to the binary.

`install.sh` and `install.ps1` install these archives for users. See
`docs/release.md` for the release checklist and AUR packaging.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+S | Save |
| Ctrl+O | Open file |
| Ctrl+Enter | Compile |
| Ctrl+Shift+Enter | Force recompile |
| Ctrl+Click on PDF | Jump to source (SyncTeX) |
