# Plan: Migrate TikzForge from Tauri + React to GPUI

## Problem
TikzForge currently ships as a Tauri v2 app with a React/TypeScript frontend (CodeMirror 6 editor, PDF.js viewer). The goal is a fully native Rust app via GPUI (Zed's UI framework) for performance, native feel, single-language codebase, and smaller binaries.

## Goals
- Keep the Tauri app shipping until feature parity is reached.
- Extract pure Rust backend logic first (done) so it is reusable by the GPUI frontend.
- Build GPUI shell, port editor and PDF viewer, reach feature parity, then cut over.

## Non-goals
- Replacing the LaTeX engine (Tectonic stays).
- Changing product scope (no new user-facing features during migration).

## Phases

### Phase 0 — Baseline capture (COMPLETE, 2026-10-26)
- Branch `gpui-migration`, archive Tauri on `archive/tauri-react-0.2.1`, tags.

### Phase 1 — Backend parity (COMPLETE, 2026-10-28)
- Extract compile/engine/synctex logic into `tikzforge-core` workspace crate.
- Workspace root `Cargo.toml`; `default-members = ["src-tauri"]`.
- CI for workspace + draft GPUI release.

### Phase 2 — GPUI shell prototype
- Create `tikzforge-gpui` crate depending on `tikzforge-core`.
- Minimal window: toolbar + text area; validate rendering/input.

### Phase 3 — Editor port
- LaTeX highlighting + basic editing in GPUI, or embed a Rust editor crate.

### Phase 4 — PDF viewer
- Rust PDF renderer + SyncTeX mapping.

### Phase 5 — Feature parity
- Tabs, theming, persistence, updater, dialogs.

### Phase 6 — Cutover
- Replace Tauri frontend; retire webview; tag `gpui-phase6-complete`.

## Risks
- GPUI ecosystem young; no drop-in CodeMirror/PDF.js equivalents.
- Editor and PDF viewer ports are the dominant cost.
- Long-lived branch drift from `master`.

## References
- `docs/technical-documentation.tex` (chapter "Migrating to GPUI (In Progress)").
- `docs/adr/0002-gpui-migration.md`.
- `docs/agent-state.md`, `docs/backtracking.md`.
