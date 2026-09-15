# TikzForge AUR package

This directory contains the release package definition for Arch Linux:

```bash
cd packaging/aur
makepkg -si
```

The package builds TikzForge from the tagged GitHub source and compiles
Tectonic `0.17.0` with Cargo during the package build. The resulting Tectonic
binary is included in the installed package, so users do not need a separate
TeX Live installation.

For an AUR submission, upload `PKGBUILD` and the generated `.SRCINFO` file to
the `tikzforge` AUR repository. The package build dependencies (`bun`,
`cargo`, `rust`, `jq`, `patchelf`, and the native Tauri dependencies) must be
available on the build machine; the PKGBUILD does not install toolchains with
`rustup`.
