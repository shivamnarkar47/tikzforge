# GPUI Migration — Phase 6 Manual QA Checklist

## Environment
- OS: Linux (Wayland/X11), target hardware
- Binary: `target/release/tikzforge-gpui` (20 MB, release build ~10m)
- LaTeX engine: Tectonic (bundled or dev-staged)

## Smoke / window
- [ ] App launches without panic; window stays open.
- [ ] Window title shows "TikzForge".
- [ ] Kanagawa dark theme renders on first launch.

## Editor
- [ ] Default template loads in editor.
- [ ] Typing updates buffer and marks document dirty (•).
- [ ] Backspace, delete, enter (auto-indent/extra indent on `{`/`[`/`(`) work.
- [ ] Bracket auto-close and overtype work.
- [ ] Line numbers render; current line highlighted.
- [ ] LaTeX syntax highlighting renders (commands, comments).
- [ ] Bracket matching highlights pairs.

## Compile / PDF
- [ ] Ctrl+Enter (Compile) triggers compile; status shows "Compiling...".
- [ ] Successful compile renders PDF in preview pane.
- [ ] Page navigation Prev/Next works; page counter updates.
- [ ] Zoom +/- and Fit toggle work.
- [ ] SyncTeX click on PDF jumps source line (status shows line).
- [ ] Compile errors appear in error list and red line gutter.

## Tabs
- [ ] New document creates tab; tab switching works.
- [ ] Dirty document close prompts discard dialog.
- [ ] Close tab removes tab; last tab creates new untitled.

## Theming
- [ ] Toggle Theme switches dark/light; editor + chrome update.
- [ ] Theme persists across restart.

## Persistence
- [ ] View mode (Split/Editor/Preview) persists across restart.
- [ ] Divider position persists across restart.
- [ ] Window geometry persists across restart.
- [ ] Recent files list persists and "Open Recent" works.

## Shortcuts
- [ ] Ctrl+S saves (Save dialog for untitled).
- [ ] Ctrl+O opens file dialog.
- [ ] Ctrl+K toggles command palette; filtering works; Enter executes.
- [ ] Ctrl+Enter compiles.

## Auto-compile
- [ ] Editing triggers debounced auto-compile after 2s (when enabled).
- [ ] Toggle Auto-compile in palette flips behavior.

## Log panel
- [ ] Toggle Log Panel shows/hides compile output.

## Updater
- [ ] Check Updates reports status (network dependent).

## Performance
- [ ] Large document (1k+ lines) remains responsive.
- [ ] Multi-page PDF (10+ pages) renders without excessive lag.

## Automated checks (Phase 6)
- `cargo test --workspace` — PASS (20 tests: core 11, gpui 5, tauri 4)
- `cargo clippy --workspace -- -D warnings` — PASS
- `cargo fmt --check` — PASS
- `cargo build --release -p tikzforge-gpui` — PASS (20 MB, ~10m)
- Frontend: `bunx tsc --noEmit && bunx vitest run` — verification only (not run here)
