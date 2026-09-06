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

## Bundled TeX Live

The app ships a minimal TeX Live (`pdflatex` + TikZ/PGF + deps, ~300MB) so
compilation works on first launch with no download.

CI builds it via:

```bash
bash scripts/prepare-texlive.sh linux      # or `windows`
bash scripts/generate-tauri-config.sh --force
bun run tauri build
```

`prepare-texlive.sh` downloads a trimmed TeX Live profile and stages it under
`src-tauri/texlive/<platform>/`. `generate-tauri-config.sh` injects the
resource globs into `tauri.conf.json` so Tauri's bundler ships the tree inside
the AppImage / NSIS installer.

The TeX Live tree is keyed by platform and cached in CI. Commit the
`texlive/` directory if you want offline-local builds without re-downloading.

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
