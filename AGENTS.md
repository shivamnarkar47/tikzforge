## Agent skills

### Issue tracker

Issues and specs live as GitHub issues. See `docs/agents/issue-tracker.md`.

### Triage labels

Five canonical triage roles mapped to label strings. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `CONTEXT.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## Development workflow

### Package management

Use **bun** for all package installations and script execution:

```bash
# Install dependencies
bun add <package>
bun add -d <dev-package>

# Run tests
bunx vitest run

# Type check
bunx tsc --noEmit

# Build
bun run build
```

### Test-Driven Development (TDD)

All feature work follows the **red → green** TDD loop:

1. **Red** — Write a failing test at a public seam (never against internals)
2. **Green** — Write only enough code to pass the test
3. **Refactor** — Clean up (only after green)

**Rules:**
- One test → one implementation → repeat (vertical slices, not horizontal)
- Tests verify behavior through public interfaces, not implementation details
- Expected values come from an independent source of truth (spec, literal, worked example)
- No speculative generality — don't add features the spec doesn't ask for

**Seams under test:**
- Document Store (Zustand): state transitions, actions
- Pure functions: `formatWindowTitle`, `DEFAULT_TEMPLATE`
- Components: render behavior, prop changes, user interactions

**Test infrastructure:**
- Vitest + React Testing Library + jsdom
- Run single file: `bunx vitest run src/path/to/test.test.ts`
- Run full suite: `bunx vitest run`

### Implementation workflow

1. Pick a ticket from the frontier (unblocked issues)
2. Agree on seams with the user before writing tests
3. TDD loop: red → green for each seam
4. Typecheck: `bunx tsc --noEmit`
5. Full test suite: `bunx vitest run`
6. Code review: `/code-review` against the ticket's fixed point
7. Commit and push
