# TikzForge GPUI Migration Journey

**Start date:** 2026-10-26 (Phase 0 baseline capture)
**Branch:** `gpui-migration` (working branch)
**Status:** In progress — Phase 1 complete; Phase 2 (GPUI shell) not started.
**Owner:** Orchestrator + sub-agents (state-maintainer, documentation, …)

## Branch strategy

- `gpui-migration` is the active migration branch; all migration work lands here.
- The Tauri implementation is preserved untouched on `archive/tauri-react-0.2.1` (and tag `v0.2.1-tauri`) at commit `3a34d066c1320bc29fa5fb824d5fa791b545380c` — the rollback source of truth.
- `master`/`main` keep shipping the Tauri app until a later cutover phase.
- After each phase, a lightweight tag `gpui-phaseN-complete` is created at the phase's final commit and pushed (see `docs/agent-state.md`).
- Recovery procedure: `docs/backtracking.md`.

## Phase tracker

| Phase | Name | Status | Start | End | Notes |
|-------|------|--------|-------|-----|-------|
| 0 | Baseline capture | Complete | 2026-10-26 | 2026-10-28 | Tauri app intact; branch + tags + state docs. Commit `3a34d06`. Tag `gpui-phase0-complete`. |
| 1 | Backend parity (extract Rust backend into `tikzforge-core`) | Complete | 2026-10-28 | 2026-10-28 | Pure Rust crate `tikzforge-core` (compile, engine detection, synctex). Workspace root `Cargo.toml`. Commit `5336d50`. CI workflow `edad2f0`. |
| 2 | GPUI shell prototype | Not started | — | — | Build minimal GPUI window (toolbar + text area); validate rendering/input. Planned crate `tikzforge-gpui`. |
| 3 | Editor port | Not started | — | — | LaTeX highlighting + basic editing in GPUI (or embed Rust editor crate). |
| 4 | PDF viewer | Not started | — | — | Rust PDF renderer + SyncTeX mapping. |
| 5 | Feature parity | Not started | — | — | Tabs, theming, persistence, updater, dialogs. |
| 6 | Cutover | Not started | — | — | Replace Tauri frontend entirely; retire webview. |

## Decision log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-10-26 | Migrate from Tauri + React to GPUI (Rust-native) | Performance (no webview), native feel, single language, smaller binaries. Phased to manage risk. See ADR-0002. |
| 2026-10-26 | Keep Tauri app as shipping implementation during migration; do not delete `src-tauri/` until cutover phase | Avoids losing a working product; allows incremental validation. |
| 2026-10-26 | Archive Tauri implementation on `archive/tauri-react-0.2.1` + tags before migration | Guaranteed restore point; cheap insurance. |
| 2026-10-28 | Extract pure Rust backend into `tikzforge-core` workspace crate before any GPUI UI work | Makes compile/engine/synctex logic reusable and unit-testable independent of Tauri; unblocks Phase 2 without UI coupling. |
| 2026-10-28 | Use Cargo workspace with `default-members = ["src-tauri"]` | Keeps `cargo build`/Tauri dev workflow unchanged while adding the core crate. |
| 2026-10-28 | Add GitHub Actions CI for workspace + draft GPUI release workflow | Catch regressions in extracted core; prepare (disabled) GPUI release path. |
| 2026-10-28 | Spike GPUI separately (`tikzforge-gpui-spike`, gpui 0.2.2) | Validate GPUI builds/runs on this machine before committing migration architecture. |

## Blockers / risks encountered

- GPUI ecosystem is young; no drop-in equivalent to CodeMirror 6 (highlighting, autocomplete, linting) — Phase 3 is the highest-risk phase.
- PDF rendering requires replacing PDF.js with a Rust PDF renderer (e.g. `pdfium`/`mupdf`) plus SyncTeX mapping — significant integration work.
- `tikzforge-gpui` crate does not exist yet; Phase 2 has not started.
- `docs/agent-state.md` predates Phase 1 and still reports "Phase 1 NOT STARTED" — to be reconciled by state-maintainer.
- Migration is a large, multi-phase effort; risk of long-lived branch drift from `master`.

## Links

- Plan file: [`docs/plans/gpui-migration.md`](plans/gpui-migration.md) (mirrors the Migration Strategy in `docs/technical-documentation.tex`, chapter "Migrating to GPUI (In Progress)").
- ADR: [`docs/adr/0002-gpui-migration.md`](adr/0002-gpui-migration.md).
- State: [`docs/agent-state.md`](agent-state.md).
- Recovery: [`docs/backtracking.md`](backtracking.md).
- Changelog: [`docs/migration-changelog.md`](migration-changelog.md).
