# Backtracking & Recovery — TikzForge GPUI Migration

This document describes how to roll back the GPUI migration to a previous phase, restore the original Tauri app, and perform emergency recovery.

## Roll back to a previous phase

1. Identify the target tag from `docs/agent-state.md` (Rollback points / Tag-to-phase mapping), e.g. `gpui-phase0-complete`.
2. Ensure working tree is clean:
   ```bash
   git status
   ```
   Stash or commit any in-progress work first.
3. Reset the migration branch to the tag (local only):
   ```bash
   git checkout gpui-migration
   git reset --hard gpui-phaseN-complete
   ```
4. If the reset must be shared, force-push (coordinate with team first):
   ```bash
   git push --force-with-lease origin gpui-migration
   ```
5. Verify:
   ```bash
   git log --oneline -5
   git describe --tags
   ```

> Prefer `git reset --hard` on a feature branch only. Never reset `master`/`main` without explicit approval.

## Restore the Tauri app from `archive/tauri-react-0.2.1`

The archived Tauri implementation lives on the branch `archive/tauri-react-0.2.1` (also `origin/archive/tauri-react-0.2.1`), at commit `3a34d066c1320bc29fa5fb824d5fa791b545380c`.

Option A — checkout the whole tree (new worktree):
```bash
git worktree add /tmp/opencode/tauri-restore archive/tauri-react-0.2.1
```

Option B — restore specific files into current branch:
```bash
git checkout archive/tauri-react-0.2.1 -- src-tauri/ src/ package.json
```

Option C — reset current branch entirely to the Tauri baseline:
```bash
git checkout gpui-migration
git reset --hard archive/tauri-react-0.2.1
```

The tag `v0.2.1-tauri` also points at the same commit and can be used interchangeably:
```bash
git reset --hard v0.2.1-tauri
```

## Emergency recovery steps

1. **Stop and assess**: `git status`, `git log --oneline -10`, `git branch -a`, `git tag`.
2. **Preserve work**: stash uncommitted changes
   ```bash
   git stash push -m "emergency-$(date +%s)"
   ```
3. **Find last good commit**: use `git reflog` to locate a known-good SHA; cross-check against tags in `docs/agent-state.md`.
4. **Restore**:
   - Soft reset (keep changes): `git reset --soft <sha>`
   - Hard reset (discard): `git reset --hard <sha>`
5. **Recover deleted branches/tags**: `git reflog` + `git checkout -b recovered <sha>`; recreate tags with `git tag <name> <sha>`.
6. **Restore Tauri app** if GPUI migration is abandoned: follow "Restore the Tauri app" above; the `archive/tauri-react-0.2.1` branch is the source of truth for the Tauri implementation.
7. **Push recovery** (only if needed, with lease):
   ```bash
   git push --force-with-lease origin <branch>
   ```
8. **Update state**: append a note to `docs/agent-state.md` Blockers/Notes recording the incident and the SHA restored to.

## Checkpoint convention

- After each phase completes, create `gpui-phaseN-complete` tag at the phase's final commit and update `docs/agent-state.md` (completed tasks, rollback points, tag-to-phase mapping).
- Push tags: `git push origin gpui-phaseN-complete`.
- Never delete phase tags without recording the deletion in `docs/agent-state.md`.
