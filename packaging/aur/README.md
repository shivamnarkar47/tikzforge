# TikzForge AUR package

This directory contains the Arch Linux package definition:

```bash
cd packaging/aur
makepkg -si
```

The package builds the GPUI app from the tagged GitHub source and installs
it as `/usr/bin/tikzforge`, with a desktop entry and icons. Tectonic is a
runtime dependency (`tectonic` in the official repositories), so the app
finds it on `PATH`.

For an AUR submission, upload `PKGBUILD`, `.SRCINFO`, and
`tikzforge.install` to the `tikzforge` AUR repository. Build dependencies
are `rust`, `clang`, `pkgconf`, and `curl`; the PKGBUILD does not install
toolchains with `rustup`.
