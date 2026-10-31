# Releasing TikzForge

TikzForge ships GPUI binaries for Linux (x86_64) and Windows (x86_64).
Releases are driven entirely by **git tags** — pushing a `v*` tag triggers
the `Release GPUI` GitHub Actions workflow, which builds both platforms and
publishes self-contained archives to a GitHub Release.

## Versioning

Semantic versioning (`MAJOR.MINOR.PATCH`). A **minor** bump signals new
features; a patch bump signals fixes. Keep the version in sync in:

| File | Field |
|------|-------|
| `tikzforge-core/Cargo.toml` | `version =` |
| `tikzforge-gpui/Cargo.toml` | `version =` |
| `Cargo.lock` | `version =` under `tikzforge-core` / `tikzforge-gpui` |

The archived Tauri app under `legacy/` keeps its own version and is not part
of a GPUI release.

## Release checklist

1. **Bump the version** in the files above and run:
   ```bash
   cargo check --workspace
   ```
   This updates `Cargo.lock`.
2. **Commit** the bump:
   ```bash
   git add Cargo.lock tikzforge-core/Cargo.toml tikzforge-gpui/Cargo.toml
   git commit -m "chore: bump version to 0.4.0"
   ```
3. **Tag** the commit with a `v`-prefixed version:
   ```bash
   git tag v0.4.0
   ```
4. **Push** the commit and the tag:
   ```bash
   git push origin master --tags
   ```
   Pushing the tag triggers `.github/workflows/release-gpui.yml`.

## What the workflow does

`.github/workflows/release-gpui.yml` runs on `push: tags: v*` across two
matrix platforms (`ubuntu-latest`, `windows-latest`):

1. Checks out the repo and sets up stable Rust.
2. Installs Linux system dependencies (GTK, Wayland, XKB, Vulkan, etc.).
3. Builds the GPUI binary (`cargo build -p tikzforge-gpui --release`).
4. Downloads and stages the bundled Tectonic engine
   (`scripts/prepare-texlive.sh linux|windows`).
5. Packages the binary and `tectonic/` into an archive.
6. Publishes the archive via `softprops/action-gh-release`:
   - Linux: `tikzforge-gpui-<tag>-linux-x86_64.tar.gz`
   - Windows: `tikzforge-gpui-<tag>-windows-x86_64.zip`

Extract an archive and keep `tectonic/` next to the binary. This is the
layout `resolve_engine(None)` probes at runtime.

## Installers

- `install.sh` installs the Linux archive under
  `~/.local/share/tikzforge`, links `~/.local/bin/tikzforge`, and writes a
  desktop entry. It supports `--check` and an explicit version.
- `install.ps1` installs the Windows zip under
  `%LOCALAPPDATA%\TikzForge` and creates a Start Menu shortcut. It supports
  `-Check` and `-Version`.

Both installers resolve the latest release from the GitHub API, so they do
not need updating for every version.

## AUR (Arch Linux)

`packaging/aur/` builds the GPUI app from the tagged source and depends on
the official `tectonic` package at runtime. After tagging:

1. Update `pkgver` and `sha256sums` in `packaging/aur/PKGBUILD`.
2. Regenerate the metadata:
   ```bash
   cd packaging/aur
   makepkg --printsrcinfo > .SRCINFO
   ```
3. Push `PKGBUILD`, `.SRCINFO`, and `tikzforge.install` to the AUR git
   remote separately.

## Legacy Tauri releases

The Tauri + React app is archived under `legacy/` (tag
`v0.2.1-tauri`). Its old AppImage/NSIS release workflow has been removed;
only GPUI releases are produced from tags now.
