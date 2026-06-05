# End-to-end tests — deferred to Spec #3

Spec #2 (Tasks 26–28) planned `tauri-driver` + Playwright E2E coverage. That work
is **deferred to Spec #3** via the plan's documented, time-boxed fallback. Spec #2
ships with Vitest + React Testing Library coverage only (35 unit/component tests),
plus the Rust core/CLI/bridge test suites.

## Why deferred

- **`tauri-driver` has no macOS support.** It drives `WebKitWebDriver` (Linux) and
  Microsoft Edge WebDriver (Windows). macOS `WKWebView` exposes no WebDriver
  endpoint, so the harness cannot launch the app on the primary (macOS-only) target.
- The harness was also un-runnable in the headless build environment used to land
  Spec #2 (no display server).

Per the Spec #2 plan, the acceptance gate (Task 32) accepts the Vitest+RTL-only path.

## Spec #3 prerequisite — "E2E via tauri-driver: harness setup"

To add real E2E later:

1. Run E2E on a **Linux CI runner** (Ubuntu) with `webkit2gtk` + `WebKitWebDriver`
   installed, building `kanban-tauri` for Linux. This is the supported tauri-driver path.
2. Implement `ui/e2e/launch.ts` (spawn `tauri-driver`, point Playwright at port 4444,
   set the Tauri binary path) and the flows the plan sketched:
   - `happy-path.spec.ts`: create project → create issue → drag across columns →
     open detail → edit title.
   - `theme-persistence.spec.ts`: toggle theme to dark → reload → assert `html.dark`
     persists (backed by the `workspace_settings` row).
3. Wire `test:e2e` into CI **only** on the Linux job (it is intentionally excluded
   from the Spec #2 CI workflow).

The `playwright.config.ts` and this directory are kept as scaffolding for that work.
