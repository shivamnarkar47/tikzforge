# Building TikzForge

## Prerequisites

- **bun** (package manager, script runner)
- **Rust** (stable toolchain)
- **Tauri CLI** (`bun add -d @tauri-apps/cli`)
- **System deps** (Linux): `libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

## Quick Start

```bash
bun install
bun run tauri dev          # dev mode (Vite + Tauri)
bun run tauri build        # production build
```

## Output

Production builds produce:

| Format | Path |
|--------|------|
| `.deb` | `src-tauri/target/release/bundle/deb/*.deb` |
| `.rpm` | `src-tauri/target/release/bundle/rpm/*.rpm` |
| AppImage | `src-tauri/target/release/bundle/appimage/*.AppImage` |

## Bundled Tectonic engine

The app ships a self-contained Tectonic binary (~60MB, XeTeX-based, TikZ
included) so compilation works on first launch with no separate LaTeX install.

Stage it before building:

```bash
bash scripts/prepare-texlive.sh linux      # or `windows`, or `all`
bash scripts/generate-tauri-config.sh      # verifies the staged binary
bun run tauri build
```

`prepare-texlive.sh` downloads the pinned prebuilt release asset
(Tectonic v0.17.0) and stages it under `src-tauri/tectonic/`.
`tauri.conf.json` declares `tectonic/*` as bundled resources, so Tauri ships
the binary inside the AppImage / NSIS installer. `generate-tauri-config.sh`
fails fast if the binary is missing — without it the installer builds fine
but the app reports "LaTeX not found" on launch.

The staged `src-tauri/tectonic/` directory is gitignored and cached in CI.
`tauri dev` picks the staged binary up automatically (it probes the source
tree, since dev mode never bundles resources); installing Tectonic on PATH
is only a last-resort fallback.

Note: Tectonic fetches its TeX package bundle from the network on first
compile and caches it locally, so the very first compile needs connectivity
even though engine detection works offline.

## AppImage builds on local Linux

The AppImage step uses `linuxdeploy`, which Tauri caches at
`~/.cache/tauri/linuxdeploy-x86_64.AppImage`. On newer Linux distros you may
see:

```
ERROR: Strip call failed: ... unknown type [0x13] section `.relr.dyn'
failed to run /home/<user>/.cache/tauri/linuxdeploy-x86_64.AppImage
```

The cached `linuxdeploy` contains an older `strip` that doesn't recognize
RELR relocations (`.relr.dyn`) in modern shared libraries (libxml2, libxslt,
libyuv, libzstd). The `.deb` and `.rpm` bundles still succeed — only AppImage
is affected.

Workarounds:

1. **Let CI produce the AppImage.** GitHub's `ubuntu-latest` runners have
   older library versions where the cached linuxdeploy works fine. Tag a
   release and grab the AppImage from the GitHub release.
2. **Delete the cached binary** so Tauri re-downloads the latest:
   ```bash
   rm ~/.cache/tauri/linuxdeploy-x86_64.AppImage
   ```
3. **Install linuxdeploy from your package manager** and symlink it into
   the cache path.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+S | Save |
| Ctrl+O | Open file |
| Ctrl+Enter | Compile |
| Ctrl+Shift+Enter | Force recompile |
| Ctrl+Click on PDF | Jump to source (SyncTeX) |
