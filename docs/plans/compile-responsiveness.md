# Plan: Make LaTeX compilation non-blocking and responsive

## Problem
During a compile (especially the first one, which downloads the Tectonic
bundle/fonts), the app can appear frozen: the window stops repainting,
typing is laggy, and there is no progress feedback beyond a spinner.

## Root causes
1. `compile_tex` is a synchronous Tauri command. Tauri v2 runs sync commands
   on a small blocking thread pool; a 10-minute compile occupies one worker
   and can starve other commands (file dialogs, cancel, engine detection).
2. The frontend awaits `compileDocument` in `useCompile` and only flips
   `isCompiling` at the end, so a slow compile keeps the spinner with no
   intermediate progress beyond streamed log lines.
3. Auto-compile (`useAutoCompile`) has no queue-while-busy logic: a keystroke
   during a compile schedules a new `compile()` that supersedes the running
   one via the generation counter, causing churn/cancellation storms.
4. First compile downloads ~300MB+ with no byte-level progress; only raw
   log lines are shown.
5. `run_with_timeout` polls every 50ms on the worker thread; acceptable but
   combined with log-join can delay settle.

## Goals
- UI stays fully responsive (60fps typing/scrolling) during any compile.
- Cancel works within ~100ms.
- Auto-compile debounces and queues: never cancels an in-flight compile;
  waits for it to finish, then recompiles if content changed.
- First compile shows download progress (bytes) and an ETA-ish message.
- No compile can run longer than 10 minutes; timed-out/cancelled state is
  reported clearly.

## Plan

### Phase 1 — Backend (Rust)
- [ ] Convert `compile_tex` to an `async` command and run the engine via
      `tokio::process` (or `spawn_blocking`), so the blocking pool is never
      occupied. Keep the generation counter for cancellation.
- [ ] Replace 50ms poll loop with `tokio::select!` on child exit / cancel
      token / timeout, so cancel settles promptly.
- [ ] Add a `compile-progress` event carrying `{ bytes_downloaded, phase }`
      parsed from Tectonic's stderr ("downloading", "unpacking").
- [ ] Ensure pipes are closed and threads joined on every exit path
      (success, timeout, cancel, error) — add a test harness or at least
      manual verification.
- [ ] Keep `cancel_compile` as a cheap, non-blocking command.

### Phase 2 — Frontend compile state
- [ ] Make `useCompile` track phases: `idle | queued | compiling | cancelling`.
- [ ] Consume `compile-progress` events and surface bytes in the status bar.
- [ ] Auto-compile queue-while-busy: if a compile is in flight, set a
      `pendingRecompile` flag; on settle, if flag set, restart the 2s timer.
      Never call `compile()` while `isCompiling`.
- [ ] Wire the existing Cancel button to `cancelling` state with spinner.

### Phase 3 — UX
- [ ] Status bar shows "Compiling… (Ns)", "Downloading (X MB)…", or
      "Cancelled"/"Timed out".
- [ ] Disable the manual Compile button while compiling; show Cancel instead.
- [ ] Persist "first compile done" flag so subsequent sessions skip the
      download messaging.

### Phase 4 — Tests
- [ ] Rust: unit test generation counter / cancellation logic (extract pure
      part); integration smoke via `cargo test`.
- [ ] Frontend: tests for queue-while-busy (fake timers + mocked compile),
      phase transitions, cancel.
- [ ] Manual: compile a 10-page doc, type during compile, hit cancel; verify
      no freeze and cancel < 100ms.

### Phase 5 — Release
- [ ] Bump version, tag, push; verify CI green (no version mismatch).

## Non-goals
- Replacing Tectonic or changing the engine.
- Full download manager (just progress + cancel).

## Risks
- Async Tauri commands need careful lifetime management of the child process
  and log threads; use `Arc<Mutex<Child>>` shared with cancel.
- Tectonic's download progress format may change; parse defensively and fall
  back to raw lines.
