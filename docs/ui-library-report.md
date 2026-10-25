# UI Library Feasibility Report — TikzForge (Tauri v2)

**Date:** 2026-10-25
**Repo:** `shivamnarkar47/tikzforge`
**Scope:** Whether and how web UI libraries (shadcn/ui, 21st.dev, and alternatives) can be used in this Tauri v2 desktop app, and what to actually do about the "UI/UX is shit" complaint.

---

## 1. Executive summary (TL;DR)

- **Yes. Tauri imposes zero restrictions on UI libraries.** The frontend is a standard web app served into a system webview. Anything that runs in a browser (React, Vue, Svelte, Solid, plain HTML) runs in Tauri. There is no "Tauri UI kit" you must use, and no native widget bridge required.
- **You are already using shadcn/ui + Radix + Tailwind v4.** `components.json`, `@radix-ui/react-*` (via the monolithic `radix-ui` package), `tailwindcss`, `class-variance-authority`, `clsx`, `tailwind-merge`, and `lucide-react` are all in `package.json`. The spec (`docs/plans/spec.md`) explicitly chose shadcn/ui.
- **So "can I use shadcn/21st.dev?" is already answered: you are.** The real problem is not library choice — it is **incomplete adoption**. Only 9 shadcn primitives exist in `src/components/ui/`, while `App.tsx` still uses raw `<select>`, hand-rolled tab buttons, and inline Tailwind for things shadcn already ships (Select, Tabs, Dialog, DropdownMenu, Card, Badge, Alert, Separator, Tooltip).
- **Recommendation: stay on shadcn + Radix + Tailwind v4, and pull polished pieces from 21st.dev** (shadcn-registry format, drop-in). Do **not** migrate to MUI/Ant Design/Chakra — they bring a second design system, heavier bundles, and theming friction for no gain in a desktop app.
- **Biggest real risks are Tauri/Linux-specific, not library-specific:** WebKitGTK rendering quirks (backdrop-blur, DMABUF/NVIDIA), no SSR (SPA only), and desktop affordances (native menus/dialogs) that web libs don't provide.

---

## 2. How Tauri runs the UI (why any lib works)

- Tauri v2 is a **static web host**. In dev it points at `http://localhost:1420` (Vite); in prod it inlines the built `dist/` into the binary. `tauri.conf.json` → `build.frontendDist = "../dist"`.
- The window contains **one webview** rendering HTML/CSS/JS. Tauri does not bundle Chromium — it uses the OS webview:
  - **Windows:** WebView2 (Edge/Chromium) — recent, auto-updating.
  - **macOS:** WebKit.
  - **Linux:** WebKitGTK.
- Therefore every React component library that targets the DOM works. The only constraints are **web-platform** constraints of those engines (see §6), not Tauri constraints.

Implication: choosing a UI lib is a pure frontend decision. Tauri only cares that `bun run build` produces static files.

---

## 3. Current stack audit

### Already installed (from `package.json`)

| Concern | Package | Status |
|---|---|---|
| Components (headless) | `radix-ui` (monolithic) | ✅ present |
| Styling | `tailwindcss` v4 + `@tailwindcss/vite` | ✅ present |
| Variants | `class-variance-authority` | ✅ present |
| Class merge | `clsx`, `tailwind-merge`, plus `cn` package | ⚠️ duplicated (see §4) |
| Icons | `lucide-react`, `@radix-ui/react-icons` | ✅ present |
| Toasts/overlays | — | ❌ none |
| Animation | — | ❌ none (only Tailwind transitions) |

### `src/components/ui/` inventory (9 files)

`button`, `input`, `label`, `resizable`, `scroll-area`, `tabs`, `toggle`, `toggle-group`, `tooltip`.

### shadcn config (`components.json`)

- Style `new-york`, base color `neutral`, CSS variables, Tailwind v4 (`config: ""`), aliases `@/components`, `@/lib/utils`, `@/components/ui`, `@/hooks`. Icon library `lucide`. This is a standard, healthy shadcn setup. `shadcn@latest add <component>` will work out of the box.

### What's NOT wired up despite being installed

- `react-resizable-panels` is used (good), but the panel group has no persisted size wired to the store's `dividerPosition` (spec wants it persisted).
- `radix-ui` monolithic package is used by some components (`button` uses `Slot` from `radix-ui`, `label` uses `LabelPrimitive`, etc.) — inconsistent with `@radix-ui/react-*` individual packages, but functional. shadcn's newer "new-york" style uses the monolithic `radix-ui` package; that's fine and intended.

---

## 4. Why the UI feels "shit" — concrete findings

These are code-level, not library-level. Fixing them costs less than a migration and yields most of the perceived quality gain.

1. **Raw `<select>` in the toolbar instead of shadcn `Select`.** `App.tsx` uses two native `<select>` elements (Templates, Recent files). They don't match the design tokens, don't get dark-mode styling consistently, and break the visual rhythm of the header. shadcn `Select` is not even installed.
2. **Hand-rolled tab bar.** The open-documents tab strip is built from raw `<button>`s with hand-concatenated class strings instead of shadcn `Tabs`. Inconsistent hover/focus states.
3. **Hand-rolled confirm dialog.** The "close tab with unsaved changes?" bar is an inline `role="alertdialog"` div with raw buttons instead of shadcn `AlertDialog`/`Dialog`. No focus trap, no escape-to-close, no overlay.
4. **No `Card`/`Alert`/`Badge`/`Separator`.** First-launch gate and error list reimplement cards/alerts with ad-hoc Tailwind (`rounded-lg border bg-card p-8 shadow-sm`). Error items are plain buttons with no severity badge.
5. **No `DropdownMenu`.** Open/Save/Recent/Template actions are scattered across the header instead of grouped into a compact menu — header is cluttered (`flex flex-wrap` with many items).
6. **No `Sheet`/`Dialog` for logs.** The log panel is a collapsible band; a resizable `Sheet`/`Drawer` (vaul) would feel more desktop-native and reclaim vertical space.
7. **PDF viewer is a bare `<iframe>`.** No toolbar (page nav, zoom, fit-to-width) despite the spec requiring it. `FileText` placeholder is fine but the empty state has no action.
8. **`cn` import inconsistency.** `button.tsx`, `input.tsx`, `label.tsx`, `tabs.tsx`, `toggle.tsx`, `toggle-group.tsx`, `tooltip.tsx`, `scroll-area.tsx` import `cn` from the `"cn"` package, while `resizable.tsx` and `lib/utils.ts` define their own. Both resolve to the same clsx+twMerge result, but having two sources is a latent footgun (version drift, bundle duplication). Pick `@/lib/utils` and remove the `cn` dependency, or vice versa — not both.
9. **No command palette / keyboard-first UX.** For a developer tool, a `Command` (cmdk) palette bound to Ctrl+K would massively improve perceived quality and is a shadcn component away.
10. **No toasts.** Save/open/compile success/failure feedback is silent. `sonner` is the shadcn-recommended toast and is a one-line add.
11. **Header wraps** (`flex flex-wrap`) instead of a fixed toolbar with overflow menu — looks broken on narrow windows.
12. **No skeleton/loading states** for compile, file open, or first launch.

None of these require abandoning shadcn. They require *using* it.

---

## 5. Library landscape (options matrix)

| Library | Tauri OK? | Design system | Bundle | Theming | Verdict for TikzForge |
|---|---|---|---|---|---|
| **shadcn/ui (current)** | ✅ | Your own (tokens in `index.css`) | Small (copy-paste, tree-shaken) | CSS vars, dark/light built-in | **Keep. Primary.** |
| **21st.dev** (registry) | ✅ | shadcn-compatible, reads your tokens | Same as shadcn (source) | Inherits your tokens | **Use for specific polished components/blocks.** |
| **Radix primitives** (already via `radix-ui`) | ✅ | Headless | Small | You style | Already the base. |
| **Headless UI** (Tailwind Labs) | ✅ | Headless | Small | Tailwind | Redundant with Radix. Skip. |
| **Mantine** | ✅ | Own (Mantine theme) | Medium | MantineProvider | Adds a second design system. Skip. |
| **Chakra UI v3** | ✅ | Own | Medium | Panda CSS | Heavy migration. Skip. |
| **Ant Design** | ✅ | Own (enterprise) | Large | Less vars | Wrong aesthetic for a dev tool. Skip. |
| **MUI (Material)** | ✅ | Material | Large (CSS-in-JS or zero-runtime) | ThemeProvider | Wrong aesthetic + perf on WebKitGTK. Skip. |
| **HeroUI / NextUI** | ✅ | Own | Medium | Tailwind-based | Possible, but duplicates shadcn. Skip. |
| **DaisyUI** | ✅ | Tailwind plugin | Tiny | CSS vars | Could layer on, but shadcn already covers it. Skip. |
| **Tailwind UI (paid)** | ✅ | Tailwind | Copy-paste | Tokens | Good blocks; 21st.dev is the free-ish equivalent. Optional. |
| **Magic UI / Aceternity** | ✅ | shadcn-compatible | Small | Tokens | Nice for marketing/landing; overkill for an editor. Use sparingly. |
| **vaul** (drawer) | ✅ | shadcn-friendly | Tiny | Tokens | **Add** for bottom/side drawers (logs, settings). |

Rule of thumb for a desktop app: **prefer copy-paste, token-driven, headless-primitive-based libs** (shadcn, Radix, 21st.dev, vaul) over bundled all-in-one libs. Bundled libs ship their own CSS runtime, fight your tokens, and bloat the (small) Tauri binary's web assets.

---

## 6. Tauri-specific constraints & gotchas (the things that actually bite)

### 6.1 Webview engine differences (Linux is the weak link)

- **WebKitGTK on Linux is older and pickier than Chromium.** Features that work in your dev browser (or on Windows WebView2) may silently fail on Linux:
  - `backdrop-filter` / `backdrop-blur` — known broken on WebKitGTK (tailwindlabs#13844, tauri#14040). **Your header uses `backdrop-blur` (`bg-background/80 ... backdrop-blur supports-[backdrop-filter]:bg-background/60`).** This is likely rendering as a flat bar on Linux. Either remove blur or provide a solid fallback.
  - CSS `:has()`, container queries, newer color functions — check WebKitGTK version.
- **NVIDIA + WebKitGTK DMABUF renderer** causes blank windows / flicker on resize (tauri linux-graphics docs). Workarounds: `WEBKIT_DISABLE_DMABUF_RENDERER=1`, `WEBKIT_DISABLE_COMPOSITING_MODE=1`. Not a UI-lib issue, but it masquerades as "UI is broken."
- **Dev vs prod mismatch:** you develop in Chromium (browser/Vite) but ship WebKitGTK on Linux. Always test the real `tauri dev`/`tauri build` on a Linux machine, not just `bun run dev` in Chrome.

### 6.2 No SSR, no server components

- Tauri serves static files. **No Next.js SSR, no React Server Components.** `components.json` already has `rsc: false` — correct. Don't adopt libs that assume a server (some Next.js-oriented shadcn blocks). 21st.dev components are SPA-safe; verify before installing any that import `next/*`.

### 6.3 Bundle size still matters (web assets are inlined into the binary)

- Tauri's binary is small because there's no Chromium, but `dist/` is embedded. Heavy libs (MUI, AntD, moment, lodash) inflate the installer. shadcn copy-paste keeps only used code. Prefer that.

### 6.4 Desktop affordances web libs don't cover

- **Native menus, tray, dialogs, notifications** — use Tauri plugins (`@tauri-apps/plugin-dialog`, menu APIs), not web modals, for OS-level actions (Open/Save file dialogs you already use via `plugin-dialog`). Web `Dialog` is for in-app confirmations only.
- **Window chrome / title bar** — Tauri windows have native title bars by default. If you want a custom title bar, use Tauri's window decorations + drag region, not a web component.
- **File drag-and-drop** — use Tauri's drag-drop events, not HTML5 DnD, for OS file drops (spec wants drag `.tex` onto app).
- **Keyboard shortcuts** that conflict with OS (Ctrl+Q, etc.) — register via Tauri global shortcuts or be careful.

### 6.5 Accessibility & focus

- Desktop users expect Tab/Enter/Escape to work in dialogs. Radix (shadcn base) handles focus trapping. Raw `<div role="alertdialog">` (your close-tab confirm) does not. This is a real UX defect, fixable by switching to shadcn `AlertDialog`.

### 6.6 Theming vs OS theme

- You already follow system theme via `useSystemTheme` and apply `.dark` class. shadcn tokens respond to `.dark`. Keep this; don't introduce a lib with its own theme provider that bypasses it.

### 6.7 Performance on WebKitGTK

- `backdrop-blur` is not just broken on Linux, it's a perf killer everywhere (shadcn#327). Remove from hot paths.
- Virtualize long lists (logs, error lists, recent files) if they can grow — `react-virtual` or TanStack Virtual. Your `ScrollArea` + full render of `compilationLog` could be heavy for huge logs.

---

## 7. Recommendation

**Stay on shadcn/ui + Radix + Tailwind v4. Add 21st.dev components selectively. Do not migrate frameworks.**

Rationale:
1. You've already paid the setup cost (config, tokens, 9 components, tests).
2. The spec mandates shadcn + CSS-variable theming + dark/light.
3. Your complaints are about *coverage* (missing Select/Dialog/DropdownMenu/Card/etc.) and *consistency*, not about shadcn being wrong.
4. 21st.dev is not a competing library — it's a **catalog of shadcn-registry components**. Installing one is `npx shadcn@latest add "https://21st.dev/r/<author>/<slug>"`. It lands as source in `src/components/ui/`, reads your tokens, and is editable. It's the cheapest way to get polished, designer-made components without a rewrite.
5. Migrating to MUI/Chakra/etc. would invalidate your tests' class/role expectations, your token system, and your dark-mode wiring, for no user-visible benefit.

### What to pull from 21st.dev specifically

- A polished **toolbar/header** block (or app shell) to replace the cluttered `flex flex-wrap` header.
- **Command palette** (`cmdk`-based) component if not using shadcn's `Command`.
- **Data table / list** patterns for errors and recent files.
- **Empty states** with illustrations for the PDF viewer and editor.
- **Toast** (but prefer `sonner` directly — it's the shadcn standard).
- Themes to refresh the token palette if desired (21st.dev ships shadcn themes).

Caveats for 21st.dev:
- Free tier: 2 component copies/day; unlimited requires membership. Budget this.
- Some components depend on `framer-motion`, `next`, etc. Check `npm dependencies` listed on each component page before installing; avoid `next/*` imports (no SSR in Tauri).
- Requires an API key for CLI installs (`API_KEY_21ST`). Browsing is free.

---

## 8. Phased action plan (concrete, test-friendly)

Phase A — **Consistency & quick wins** (no new deps except shadcn adds):
1. Resolve the `cn` duplication: standardize all `src/components/ui/*` on `@/lib/utils`, remove the `cn` package dependency.
2. `bunx shadcn@latest add select dialog alert-dialog dropdown-menu card badge separator sonner command sheet`.
3. Replace the two raw `<select>`s with shadcn `Select`.
4. Replace the hand-rolled tab strip with shadcn `Tabs` (or keep custom but use tokens consistently).
5. Replace the close-tab confirm band with `AlertDialog`.
6. Remove `backdrop-blur` from the header (or gate behind `supports-[backdrop-filter]` + solid fallback); test on Linux/WebKitGTK.
7. Add `sonner` toasts for save/open/compile outcomes.

Phase B — **UX structure**:
8. Restructure header into a compact toolbar: logo + filename input + (Templates/Recent via `DropdownMenu` or `Select`) on left; status + actions grouped on right; overflow menu on narrow widths.
9. Move the log panel into a `Sheet` (right/bottom) with a toggle; keep the inline band as fallback if preferred.
10. Add PDF viewer toolbar (page nav, zoom, fit-to-width) per spec user story 19/20.
11. Add a `Command` palette (Ctrl+K) for open/save/compile/theme/template actions.
12. Persist divider position via `react-resizable-panels` `onLayout` → store.

Phase C — **Polish**:
13. Adopt 2-3 21st.dev components (app shell, empty states, data table for errors) — verify no `next/*` imports.
14. Add skeleton/loading states for compile and file open.
15. Virtualize the log panel if logs grow large.
16. Test the real webview on Linux (WebKitGTK) + Windows (WebView2) for every visual change; don't trust Chrome.

Phase D — **Verification**:
17. Update tests that assert on raw DOM (e.g., `.pdf-viewer`, `.cm-editor`) — shadcn components render Radix portals; tests may need `screen` queries instead of `container.firstChild` (see `error-list.test.tsx` "renders nothing when no errors" — `container.firstChild` becomes null still OK, but portal-based overlays need care).
18. Run `bunx tsc --noEmit` and `bunx vitest run` after each component swap.

---

## 9. Risks & open questions

- **WebKitGTK backdrop-filter** is a hard blocker for the glassy header aesthetic on Linux. Decide early whether to drop blur entirely (recommended) or accept platform divergence.
- **21st.dev free-tier limits** may slow adoption if you plan to install many components in one session. Batch installs or get membership.
- **Test churn:** swapping raw elements for Radix-based shadcn components changes the DOM (portals, data-slot attributes). Existing tests mostly query by text/role, which is good, but `container.firstChild` assertions and `.error-list` class assertions need review.
- **Monolithic `radix-ui` vs individual packages:** shadcn's newer CLI emits imports from the monolithic `radix-ui` package (as seen in your `button.tsx`). Keep that convention; don't mix in `@radix-ui/react-dialog` separately unless needed.
- **No ADR/CONTEXT.md exists** in this repo, so this report is the decision record. Consider adding `docs/adr/0001-ui-stack-shadcn.md` to lock in "shadcn + Radix + Tailwind v4, 21st.dev for blocks, no framework migration" so future agents don't re-litigate.

---

## 10. Appendix — component inventory

### Installed (`src/components/ui/`)
button, input, label, resizable, scroll-area, tabs, toggle, toggle-group, tooltip.

### Missing but recommended (shadcn)
select, dialog, alert-dialog, dropdown-menu, card, badge, separator, sonner, command, sheet, skeleton, progress, avatar, accordion, collapsible, popover, hover-card, context-menu, menubar, navigation-menu, scroll-area (✓), textarea, checkbox, radio-group, switch, slider, calendar, chart, table, data-table, form, field, input-group, input-otp, kbd, empty, spinner, typography, breadcrumb, pagination, carousel, resizable (✓).

### Hand-rolled in `App.tsx` that should become shadcn
- 2× `<select>` → `Select`
- tab strip → `Tabs` (or keep custom)
- close-tab confirm band → `AlertDialog`
- header layout → `Toolbar`/custom + `DropdownMenu`

### Consumers of `cn`
`button`, `input`, `label`, `tabs`, `toggle`, `toggle-group`, `tooltip`, `scroll-area` import from `"cn"` (package); `resizable` imports from `@/lib/utils`. Unify.

---

**Bottom line:** You don't need a new UI library. You need to *finish* the one you chose. shadcn + 21st.dev blocks, applied consistently, will fix 80% of the "shit UI" feeling. The remaining 20% is Tauri/Linux webview reality (backdrop-blur, DMABUF) which no library can paper over.
