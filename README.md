# TikzForge

A native desktop app for editing TikZ/LaTeX diagrams with live PDF preview.

## Features

- **Live Preview** — Edit TikZ code, see the PDF update instantly
- **Syntax Highlighting** — Custom LaTeX stream language for CodeMirror 6
- **Error Display** — Clickable error list with line numbers parsed from compilation logs
- **Auto-Compile** — Configurable debounce (default 2s) after last keystroke
- **Multi-Document Tabs** — Open multiple `.tex` files, each with its own compile state
- **Unsaved Changes Protection** — Prompts on tab close when work hasn't been saved
- **Recent Files** — Persisted list of recently opened files
- **Keyboard Shortcuts** — `Ctrl+O` (open), `Ctrl+S` (save)
- **Auto-Save** — Periodic save to temp file
- **Dark/Light Theme** — Follows system theme by default, user override persisted
- **State Persistence** — Window size, position, and divider position restored on relaunch
- **SyncTeX Forward Sync** — `Ctrl+Click` on PDF to jump to source line
- **LaTeX Engine Management** — Detect existing installation or download minimal TeX Live
- **Auto-Updates** — Tauri updater checks for new versions on startup

## Roadmap

Near-term (hooks exist, wiring in progress — see #10):

- **Auto-Compile rollout** — connect the editor to the auto-compile hook: 2s debounce, queue-while-busy, toolbar toggle (on by default)
- **SyncTeX end-to-end** — pass `--synctex` to the engine so `Ctrl+Click` jump-to-source works on real output
- **Offline first compile** — ship a pre-warmed engine bundle; today the first build downloads engine data once, then caches
- **Finish file workflows** — wire Auto-Save + unclean-shutdown restore offer, `Ctrl+O` / `Ctrl+S` shortcuts, and the tab-close guard into the UI
- **Real auto-updater** — backend currently reports "no update"; connect Tauri updater + `.tex` file association
- **Template library** — starter TikZ examples beyond the default template
- **Close #10** — bundled-engine acceptance across AppImage + NSIS installers

Later:

- **Multi-file documents** — `\include` / `\input` project support
- **macOS support** — out of scope for v0.1 (Linux + Windows first)

## Tech Stack

- **UI**: GPUI (Rust-native, GPU-accelerated)
- **Backend**: Rust (`tikzforge-core` + `tikzforge-gpui`)
- **Editor**: Rust text buffer (ropey) + tree-sitter-latex
- **PDF**: pdfium
- **Testing**: cargo test

The legacy Tauri + React implementation is archived under `legacy/` for one release cycle.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Platform dependencies for GPUI (Linux: gtk/wayland/vulkan dev packages; see CI)

### Installation

```bash
# Clone the repo
git clone https://github.com/shivamnarkar47/tikzforge.git
cd tikzforge
```

### Development

```bash
# Build and run the GPUI app
cargo run -p tikzforge-gpui
```

### Testing

```bash
# Run all Rust tests
cargo test --workspace

# Run GPUI tests only
cargo test -p tikzforge-gpui

# Type check
cargo check --workspace
```

### Building

```bash
# Build for production
cargo build -p tikzforge-gpui --release
```

Outputs:
- Binary: `target/release/tikzforge-gpui` (or `.exe` on Windows)

### Legacy (Tauri) build

The archived Tauri + React implementation lives under `legacy/`. To build it:

```bash
cd legacy/src-tauri
cargo tauri build
```

## Project Structure

```
tikzforge-core/     # Pure Rust backend (compile, engine, synctex, persistence)
tikzforge-gpui/     # GPUI app (UI, editor, PDF viewer)
legacy/             # Archived Tauri + React app (one release cycle)
```

## Architecture

TikzForge is a native Rust application built on GPUI:

1. **Core** (`tikzforge-core`) handles compile pipeline, engine detection, SyncTeX, persistence.
2. **UI** (`tikzforge-gpui`) handles rendering, editing, PDF preview, theming.

## License

MIT
