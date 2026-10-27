# Migration Changelog

Running log of completed migration phases. Append a new entry when a phase completes.

## Phase 0 — Baseline capture (2026-10-26 → 2026-10-28)

- Created `gpui-migration` branch from Tauri baseline commit `3a34d066c1320bc29fa5fb824d5fa791b545380c`.
- Archived Tauri implementation on `archive/tauri-react-0.2.1` and tag `v0.2.1-tauri`.
- Tagged baseline as `gpui-phase0-complete`.
- Added `docs/agent-state.md` and `docs/backtracking.md`.

## Phase 1 — Backend parity: extract Rust backend into `tikzforge-core` (2026-10-28)

- Added workspace root `Cargo.toml` with members `tikzforge-core`, `src-tauri`; `default-members = ["src-tauri"]`.
- Created `tikzforge-core` crate exposing compile, engine detection, synctex, and persistence helpers (with unit tests).
- Refactored `src-tauri` to depend on `tikzforge-core`.
- Added GitHub Actions CI (workspace rust + frontend) and a draft (manual) GPUI release workflow.
- Commits: `5336d50` (Phase 1), `edad2f0` (CI).

## Phase 2 — GPUI shell prototype

_(not started)_

## Phase 3 — Editor port

_(not started)_

## Phase 4 — PDF viewer

_(not started)_

## Phase 5 — Feature parity

_(not started)_

## Phase 6 — Cutover

_(not started)_
