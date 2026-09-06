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

## Tech Stack

- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Tauri v2 (Rust)
- **Editor**: CodeMirror 6 with custom LaTeX highlighting
- **State**: Zustand
- **Styling**: Tailwind CSS v4 + shadcn/ui components
- **Testing**: Vitest + React Testing Library

## Getting Started

### Prerequisites

- [Bun](https://bun.sh/) (package manager)
- [Rust](https://www.rust-lang.org/tools/install) (for Tauri)
- [Tauri CLI](https://v2.tauri.app/start/prerequisites/) platform dependencies

### Installation

```bash
# Clone the repo
git clone https://github.com/shivamnarkar47/tikzforge.git
cd tikzforge

# Install dependencies
bun install
```

### Development

```bash
# Start dev server with hot reload
bun run tauri dev
```

### Testing

```bash
# Run all tests
bunx vitest run

# Run single test file
bunx vitest run src/path/to/test.test.ts

# Type check
bunx tsc --noEmit
```

### Building

```bash
# Build for production
bun run tauri build
```

Outputs:
- Linux: `.AppImage` and `.deb` packages
- Windows: `.exe` NSIS installer

## Project Structure

```
src/
├── components/     # React UI components
│   ├── editor.tsx       # CodeMirror editor
│   ├── pdf-viewer.tsx   # PDF preview
│   ├── error-list.tsx   # Compilation errors
│   └── compile-button.tsx
├── hooks/          # React hooks
│   ├── use-compile.ts
│   ├── use-auto-compile.ts
│   ├── use-keyboard-shortcuts.ts
│   ├── use-system-theme.ts
│   ├── use-synctex.ts
│   ├── use-tab-close-guard.ts
│   └── use-updater.ts
├── lib/            # Pure functions and Tauri wrappers
│   ├── compilation.ts
│   ├── compilation-parser.ts
│   ├── synctex-parser.ts
│   ├── latex-engine.ts
│   ├── updater.ts
│   └── persistence.ts
├── store/          # Zustand stores
│   ├── document-store.ts
│   ├── tab-store.ts
│   ├── theme-store.ts
│   └── latex-engine-store.ts
└── test/           # Test setup
    └── setup.tsx
```

## Architecture

TikzForge uses a Tauri v2 architecture:

1. **Frontend** (React) handles UI, state, and user interactions
2. **Backend** (Rust/Tauri) handles:
   - Process management (pdflatex invocation)
   - File system operations
   - Update checks
   - SyncTeX parsing

Communication between frontend and backend uses Tauri's `invoke` API with typed TypeScript interfaces.

## License

MIT
