# TikzForge v0.1 — Product Spec

## Problem

LaTeX users working with TikZ diagrams must compile from the command line, parse cryptic error logs, and constantly switch between editor and PDF viewer. This friction makes the inherently trial-and-error process of building TikZ figures painfully slow.

## Solution

A native desktop app where you open a `.tex` file, edit it, and see the PDF update automatically 2 seconds after you stop typing. Click the PDF to jump to the source line that produced it. Errors appear as a clickable list that jumps to the offending line.

---

## User Stories

1. As a LaTeX user, I want to open a `.tex` file from my file manager, so that I can start working immediately
2. As a LaTeX user, I want the app to pre-load with a TikZ template on first launch, so that I can see a working example before I understand how to use the app
3. As a LaTeX user, I want syntax highlighting in the editor, so that I can distinguish commands, environments, and text at a glance
4. As a LaTeX user, I want my document to auto-compile 2 seconds after I stop typing, so that I see the result of my changes without clicking a button
5. As a LaTeX user, I want a toggle to turn off auto-compile, so that I can work on large documents without constant recompilation
6. As a LaTeX user, I want to see a spinner and status text while compiling, so that I know the app is working
7. As a LaTeX user, I want the PDF to refresh automatically after a successful compile, so that I always see the latest output
8. As a LaTeX user, I want to click on the PDF and have the editor jump to the source line that produced it (forward SyncTeX), so that I can debug TikZ positioning errors
9. As a LaTeX user, I want compilation errors shown as a clickable list, so that I can jump directly to the line causing the problem
10. As a LaTeX user, I want to see the raw compilation log when the parsed errors aren't enough, so that I can diagnose tricky problems
11. As a LaTeX user, I want to save my `.tex` file with a keyboard shortcut, so that I don't lose work
12. As a LaTeX user, I want to export the compiled PDF to a location of my choice via a save dialog, so that I can put the output where I need it
13. As a LaTeX user, I want to have multiple `.tex` files open in tabs, so that I can switch between documents without closing them
14. As a LaTeX user, I want a prompt asking whether to save when I close a tab or the app with unsaved changes, so that I don't lose work accidentally
15. As a LaTeX user, I want the app to auto-save my work periodically and offer to restore it if the app crashes, so that I never lose significant progress
16. As a LaTeX user, I want the window size, position, and editor/PDF divider position to persist between sessions, so that I don't rearrange my workspace every time
17. As a LaTeX user, I want a list of recently opened files, so that I can quickly return to my current projects
18. As a LaTeX user, I want the app to follow my system dark/light theme by default but let me override it, so that it matches my desktop environment
19. As a LaTeX user, I want standard PDF viewer controls (page navigation, zoom, fit-to-width), so that I can inspect my TikZ figures comfortably
20. As a LaTeX user, I want the PDF to default to fit-to-width when it first opens, so that I can see the full page without adjusting
21. As a LaTeX user, I want to drag a `.tex` file onto the app to open it, so that I can launch documents from my file manager quickly
22. As a LaTeX user, I want to install LaTeX automatically on first launch with a progress indicator, so that I don't need to manage it myself
23. As a LaTeX user, I want the app to check for updates automatically and install them, so that I always have the latest version
24. As a LaTeX user, I want the app to register itself as the default handler for `.tex` files, so that double-clicking opens it in TikzForge
25. As a LaTeX user, I want keyboard shortcuts for compile, force recompile, and PDF-to-source navigation, so that I can work without touching the mouse
26. As a LaTeX user, I want the window title to show the filename and app name, so that I can identify the window in my taskbar

---

## Architecture Decisions

### Platform & Distribution
- **Tauri v2** — small binary, low memory, web UI with Rust backend
- **Linux** (AppImage) and **Windows** (NSIS installer)
- `.tex` file association registered on both platforms
- Tauri updater for automatic updates on startup

### Frontend Stack
- **React + TypeScript + Tailwind CSS**
- **CodeMirror 6** — lighter than Monaco (~200KB vs ~5MB), faster startup, sufficient for LaTeX editing
- **PDF.js** — consistent cross-platform rendering, no extra architectural cost since the UI is already web-based

### Backend (Rust)
- Compilation engine (pdflatex invocation)
- Error log parsing (hybrid: regex + raw fallback)
- File operations (save, auto-save, drag-and-drop)
- SyncTeX parsing (forward sync: PDF position → source line)
- State persistence (window geometry, recent files, theme preference)

### LaTeX Engine
- **Minimal TeX Live** with `pdflatex` + TikZ packages only (~300MB)
- Same engine bundled for Linux and Windows for consistency
- Downloaded on first launch with a progress indicator
- Flags: `-interaction=nonstopmode`, `-shell-escape`, `-synctex=1`

### Layout
- Side-by-side: editor left, PDF right
- Divider position persisted between sessions
- Window title: `document.tex — TikzForge`

---

## Feature Details

### Auto-Compile
- 2-second debounce after last keystroke
- Queue behavior: new changes wait for current compile to finish, then start a fresh timer
- Toggle in toolbar to enable/disable
- On by default

### SyncTeX (Forward Sync)
- Click PDF → cursor jumps to source line that produced it
- Uses `-synctex=1` flag to generate `.synctex.gz` file
- Parses the synctex file to map PDF positions to source lines

### Error Display
- Hybrid parsing: regex for common errors, raw log fallback
- Parsed errors shown as clickable list (file:line:message)
- Raw log available in collapsible panel below
- Clicking an error jumps cursor to that line

### Multi-Document
- Tabs for multiple open files
- Tab close prompts if unsaved changes
- Recent files list persisted

### Crash Recovery
- Auto-save to temp file periodically
- On launch, detect unclean shutdown and offer to restore

### Theme
- Follows system dark/light setting by default
- User can override; choice persisted

### First-Run Experience
- Minimal TikZ template pre-loaded in the editor
- User can immediately hit compile to see a figure

### PDF Viewer
- Standard navigation: page up/down, zoom in/out, fit-to-width
- Default zoom: fit-to-width
- Save dialog for PDF export

### Unsaved Changes
- Prompt on close: "Save? / Don't Save / Cancel"
- Never silently discard work

### Keyboard Shortcuts
- Ctrl+S: Save
- Ctrl+O: Open file
- Ctrl+Enter: Compile
- Ctrl+Shift+Enter: Force recompile
- Ctrl+Click on PDF: Jump to source (SyncTeX)

---

## Testing Strategy

Tests verify external behavior, not implementation details. A test passes if the user-observable outcome is correct.

### Unit Tests (Rust Backend)
- Compilation logic: does pdflatex get invoked with correct flags?
- Error parsing: does the regex extract the right file:line:message?
- File operations: does save write the correct content?
- SyncTeX parsing: does a PDF coordinate map to the right source line?

### Integration Tests
- Full compile pipeline: given a `.tex` file with a TikZ figure, does the app produce a PDF?
- Error pipeline: given a `.tex` file with a syntax error, does the error panel show the correct line number?

### CI/CD
- GitHub Actions: build on push, create releases with artifacts for both platforms

### License
- MIT

---

## Out of Scope

- macOS support (v0.1 targets Linux + Windows only)
- Project folder support / multi-file documents (single file only)
- Auto-recompile on external file change
- File explorer sidebar
- Template library
- Document outline / table of contents
- Cloud sync
- Collaboration features
- Backward SyncTeX (scroll editor → scroll PDF)
- Tectonic or other LaTeX engine alternatives
- MiKTeX detection on Windows
- E2E UI tests
- Flatpak or .deb distribution
- Custom app icon

---

## Future Notes

- The 300MB TeX Live download on first launch is a known friction point. Future versions could explore Tectonic (~100MB) if compatibility gaps are acceptable.
- SyncTeX forward sync is the only sync direction in v0.1. Backward sync is a common v0.2 request.
- The app is offline-first. All compilation happens locally. No cloud dependency.
