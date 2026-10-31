# 0002 — Migrate TikzForge's UI from Tauri + React to GPUI

Status: accepted

## Context
TikzForge v0.2.1 ships as a Tauri v2 app with a React/TypeScript frontend (CodeMirror 6 editor, PDF.js viewer). The team wants a fully native Rust desktop app for lower overhead, native feel, a single language, and smaller binaries. GPUI (Zed's Rust UI framework) is the chosen vehicle. The decision is hard to reverse and carries real lock-in, so it is recorded here.

## Decision
Migrate the UI to GPUI in phases, keeping the Tauri app shipping until feature parity is reached:
1. Extract pure Rust backend logic into a reusable `tikzforge-core` crate (done, Phase 1).
2. Build a GPUI shell prototype, then port the editor and PDF viewer.
3. Reach feature parity (tabs, theming, persistence, updater, dialogs), then cut over and retire the webview.

The Tauri implementation is preserved on `archive/tauri-react-0.2.1` and tagged commits for rollback.

## Considered options
- **Keep Tauri + React (status quo):** maximizes iteration speed and leverages CodeMirror/PDF.js. Rejected as the long-term direction because of webview overhead and the TS/Rust boundary, though retained as the shipping implementation during migration.
- **Hybrid (GPUI chrome, webview editor):** reduces editor port cost but adds complexity. Rejected for now; revisit if the editor port proves infeasible.
- **Other Rust UI crates (egui/iced):** less mature for desktop native feel. Rejected in favor of GPUI's rendering quality.

## Consequences
- A long-lived migration branch and phased cutover; risk of drift from `master`.
- Highest-risk phases are the editor port (no drop-in CodeMirror equivalent) and PDF viewer (Rust PDF renderer + SyncTeX).
- Backend logic becomes shared and unit-testable via `tikzforge-core`.

## References
- Plan: `docs/plans/gpui-migration.md`.
- Migration strategy: `docs/technical-documentation.tex` (chapter "Migrating to GPUI (In Progress)").
- State/recovery: `docs/agent-state.md`, `docs/backtracking.md`.
- Journey/changelog: `docs/migration-journey.md`, `docs/migration-changelog.md`.
