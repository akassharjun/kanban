# Kanban v2 — Spec #2 GUI Shell Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a macOS Tauri desktop kanban that wraps the existing `kanban-core`, delivering Workspace + Theme (E01), Projects (E02), Issues CRUD (E04), Kanban Board (E05), and the markdown Detail Panel (E06), every behaviour test-first.

**Architecture:** A new `kanban-tauri` Cargo crate owns the Rust↔TS bridge: a single `apply(Operation)` mutator command + ~7 read commands, error types mapped to a TS-friendly tagged union, and `tauri-specta`-generated bindings. A new `ui/` pnpm package (Vite + React 19 + Tailwind v4 + shadcn/ui) consumes the bindings via TanStack Query with optimistic updates, react-router v7 for the sidebar/board/detail tree, `@dnd-kit` for column drag, and `react-markdown` + `remark-gfm` for the panel body. macOS-only first ship; unsigned bundle.

**Tech Stack:** Tauri 2, `tauri-specta` + `specta`, `tokio`. React 19, Vite 5, Tailwind v4, shadcn/ui, react-router 7, TanStack Query 5, react-hook-form + zod, `@dnd-kit/core` + `@dnd-kit/sortable`, react-markdown + remark-gfm, lucide-react, pnpm. Vitest + React Testing Library, Playwright + `tauri-driver`.

**Spec:** `docs/superpowers/specs/2026-05-05-kanban-v2-spec-2-gui-shell-design.md`

---

## Phase Map

| Phase | Tasks | Topic |
|-------|-------|-------|
| 0 | 1–3 | Workspace bootstrap: kanban-tauri crate + ui/ pnpm scaffold + tooling |
| 1 | 4 | Core: `0002_workspace_settings` migration + `get_setting`/`set_setting` |
| 2 | 5–10 | Rust bridge: ApiError, AppState, read commands, apply, settings, tauri-specta export |
| 3 | 11–14 | React app shell: providers, router, theme, sidebar |
| 4 | 15–17 | Data layer in TS: ops builders, queries, `useApply` with optimistic dispatch |
| 5 | 18–21 | Board: layout, dnd-kit, sortKey midpoint, inline create |
| 6 | 22–25 | Detail panel + markdown + edit-in-place + close UX |
| 7 | 26 | tauri-driver harness (with fallback) |
| 8 | 27–28 | Playwright E2E (smoke + theme persistence) |
| 9 | 29–31 | CI workflow, release workflow, docs |
| 10 | 32 | Acceptance gate |

---

## Phase 0 — Workspace bootstrap

### Task 1: Add `kanban-tauri` crate to the Cargo workspace

**Files:**
- Modify: `Cargo.toml`
- Create: `crates/kanban-tauri/Cargo.toml`
- Create: `crates/kanban-tauri/src/main.rs`
- Create: `crates/kanban-tauri/build.rs`
- Create: `crates/kanban-tauri/tauri.conf.json`
- Create: `crates/kanban-tauri/icons/.gitkeep`

- [ ] **Step 1: Add the new crate to workspace members and workspace deps**

Edit `Cargo.toml` — add `crates/kanban-tauri` to `members` and add the Tauri-side deps to `[workspace.dependencies]`:

```toml
[workspace]
resolver = "2"
members = ["crates/kanban-core", "crates/kanban-cli", "crates/kanban-tauri"]

# … existing workspace.package, workspace.lints stay unchanged …

[workspace.dependencies]
# existing deps unchanged
rusqlite = { version = "0.31", features = ["bundled", "serde_json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v7", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
clap = { version = "4", features = ["derive", "env"] }
anyhow = "1"
assert_cmd = "2"
insta = { version = "1", features = ["yaml", "filters"] }
proptest = "1"
tempfile = "3"

# new for kanban-tauri
tauri = { version = "2", features = [] }
tauri-build = { version = "2", features = [] }
tauri-specta = { version = "2.0.0-rc.21", features = ["derive", "typescript"] }
specta = { version = "2.0.0-rc.22", features = ["derive"] }
specta-typescript = "0.0.9"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "sync"] }
tauri-plugin-shell = "2"
```

- [ ] **Step 2: Create `crates/kanban-tauri/Cargo.toml`**

```toml
[package]
name = "kanban-tauri"
version = "0.0.1"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "Tauri desktop GUI for kanban-core."

[lints]
workspace = true

[[bin]]
name = "kanban-tauri"
path = "src/main.rs"

[build-dependencies]
tauri-build = { workspace = true }

[dependencies]
kanban-core   = { path = "../kanban-core" }
tauri         = { workspace = true }
tauri-specta  = { workspace = true }
specta        = { workspace = true }
specta-typescript = { workspace = true }
tauri-plugin-shell = { workspace = true }
tokio         = { workspace = true }
serde         = { workspace = true }
serde_json    = { workspace = true }
thiserror     = { workspace = true }
anyhow        = { workspace = true }
uuid          = { workspace = true }
chrono        = { workspace = true }
```

- [ ] **Step 3: Create `crates/kanban-tauri/src/main.rs`**

Minimal Tauri app that just opens a window pointing at the dev URL. Real handlers come in Phase 2.

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Create `crates/kanban-tauri/build.rs`**

```rust
fn main() {
    tauri_build::build();
}
```

- [ ] **Step 5: Create `crates/kanban-tauri/tauri.conf.json`**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "kanban",
  "version": "0.0.1",
  "identifier": "com.kanban.app",
  "build": {
    "beforeDevCommand": "pnpm --dir ../../ui dev",
    "beforeBuildCommand": "pnpm --dir ../../ui build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../../ui/dist"
  },
  "app": {
    "windows": [
      {
        "title": "kanban",
        "width": 1200,
        "height": 760,
        "minWidth": 720,
        "minHeight": 480,
        "decorations": false,
        "titleBarStyle": "Overlay",
        "trafficLightPosition": { "x": 16, "y": 18 }
      }
    ],
    "security": {
      "csp": null
    }
  },
  "bundle": {
    "active": true,
    "targets": ["dmg", "app"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.icns"
    ],
    "macOS": {
      "minimumSystemVersion": "11.0"
    }
  }
}
```

- [ ] **Step 6: Stub icons directory**

```bash
mkdir -p crates/kanban-tauri/icons
touch crates/kanban-tauri/icons/.gitkeep
```

A placeholder Tauri icon set is generated automatically by `pnpm tauri icon path/to/source.png` in a later task; the `.gitkeep` keeps the dir present for the conf reference.

- [ ] **Step 7: Verify workspace still compiles (the Tauri crate won't link without icons but `cargo check` is enough)**

Run: `cargo check --workspace`
Expected: success — `kanban-tauri` compiles up to the `tauri::generate_context!` call. (The macro requires icons to bundle; if it fails, supply a 1×1 PNG by running `printf '\x89PNG\r\n\x1a\n...' > crates/kanban-tauri/icons/icon.png` or grab any existing PNG and copy to all three `bundle.icon` paths.)

- [ ] **Step 8: Commit**

```bash
git add Cargo.toml Cargo.lock crates/kanban-tauri
git commit -m "chore(tauri): scaffold kanban-tauri crate in workspace"
```

---

### Task 2: Bootstrap the `ui/` pnpm package (Vite + React + Tailwind v4)

**Files:**
- Create: `ui/package.json`
- Create: `ui/pnpm-workspace.yaml` (single package, but pinning the pnpm version is useful)
- Create: `ui/index.html`
- Create: `ui/vite.config.ts`
- Create: `ui/tsconfig.json`
- Create: `ui/tsconfig.node.json`
- Create: `ui/src/main.tsx`
- Create: `ui/src/App.tsx`
- Create: `ui/src/styles.css`
- Modify: `.gitignore`

- [ ] **Step 1: Create `ui/package.json`**

```json
{
  "name": "kanban-ui",
  "private": true,
  "version": "0.0.1",
  "type": "module",
  "packageManager": "pnpm@9.12.0",
  "scripts": {
    "dev": "vite --port 1420 --strictPort",
    "build": "tsc -b && vite build",
    "preview": "vite preview --port 1420 --strictPort",
    "typecheck": "tsc --noEmit",
    "lint": "eslint . --max-warnings 0",
    "test:unit": "vitest run",
    "test:unit:watch": "vitest",
    "test:e2e": "playwright test",
    "tauri": "tauri"
  },
  "dependencies": {
    "@dnd-kit/core": "^6.1.0",
    "@dnd-kit/sortable": "^8.0.0",
    "@tanstack/react-query": "^5.59.0",
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0",
    "lucide-react": "^0.451.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "react-hook-form": "^7.53.0",
    "react-markdown": "^9.0.1",
    "react-router": "^7.0.0",
    "remark-gfm": "^4.0.0",
    "uuid": "^10.0.0",
    "zod": "^3.23.8"
  },
  "devDependencies": {
    "@playwright/test": "^1.48.0",
    "@tailwindcss/vite": "^4.0.0",
    "@tauri-apps/cli": "^2.0.0",
    "@testing-library/jest-dom": "^6.5.0",
    "@testing-library/react": "^16.0.1",
    "@testing-library/user-event": "^14.5.2",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "@types/uuid": "^10.0.0",
    "@vitejs/plugin-react": "^4.3.2",
    "eslint": "^9.12.0",
    "eslint-plugin-react-hooks": "^5.0.0",
    "eslint-plugin-react-refresh": "^0.4.12",
    "happy-dom": "^15.7.4",
    "tailwindcss": "^4.0.0",
    "typescript": "^5.6.2",
    "typescript-eslint": "^8.8.0",
    "vite": "^5.4.8",
    "vitest": "^2.1.2"
  }
}
```

- [ ] **Step 2: Create `ui/index.html`**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>kanban</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

- [ ] **Step 3: Create `ui/vite.config.ts`**

```ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwind from "@tailwindcss/vite";
import path from "node:path";

export default defineConfig({
  plugins: [react(), tailwind()],
  resolve: {
    alias: { "@": path.resolve(__dirname, "src") },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: false,
    hmr: { protocol: "ws", host: "localhost", port: 1421 },
  },
});
```

- [ ] **Step 4: Create `ui/tsconfig.json`**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "useDefineForClassFields": true,
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "verbatimModuleSyntax": true,
    "baseUrl": ".",
    "paths": { "@/*": ["src/*"] }
  },
  "include": ["src", "tests", "e2e"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

- [ ] **Step 5: Create `ui/tsconfig.node.json`**

```json
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true,
    "strict": true
  },
  "include": ["vite.config.ts"]
}
```

- [ ] **Step 6: Create `ui/src/styles.css`** (Tailwind v4 single-file import + theme tokens)

```css
@import "tailwindcss";

@theme {
  --color-accent: oklch(67% 0.18 250);
  --color-accent-fg: white;
}

.dark {
  --color-accent: oklch(72% 0.16 250);
}

html, body, #root { height: 100%; }
body { margin: 0; }
```

- [ ] **Step 7: Create `ui/src/main.tsx`**

```tsx
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "./styles.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
```

- [ ] **Step 8: Create `ui/src/App.tsx`** (placeholder, replaced in Phase 3)

```tsx
export default function App() {
  return <div className="p-6 text-sm">kanban — bootstrapping…</div>;
}
```

- [ ] **Step 9: Append Vite/Tauri build artefacts to `.gitignore`**

```
ui/node_modules/
ui/dist/
ui/.vite/
ui/.turbo/
ui/playwright-report/
ui/test-results/
target/release/bundle/
```

- [ ] **Step 10: Verify install + dev server**

Run: `pnpm --dir ui install`
Expected: success, lockfile generated.

Run: `pnpm --dir ui typecheck`
Expected: success, no errors.

Run: `pnpm --dir ui dev` (Ctrl-C after the server announces "Local: http://localhost:1420")
Expected: success.

- [ ] **Step 11: Commit**

```bash
git add ui/package.json ui/pnpm-lock.yaml ui/index.html ui/vite.config.ts ui/tsconfig.json ui/tsconfig.node.json ui/src .gitignore
git commit -m "chore(ui): bootstrap Vite + React + Tailwind v4 scaffold"
```

---

### Task 3: ESLint + Vitest + Playwright config

**Files:**
- Create: `ui/eslint.config.js`
- Create: `ui/vitest.config.ts`
- Create: `ui/playwright.config.ts`
- Create: `ui/tests/setup.ts`
- Create: `ui/e2e/.gitkeep`

- [ ] **Step 1: Create `ui/eslint.config.js`** (flat config)

```js
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";

export default tseslint.config(
  { ignores: ["dist", "src/data/bindings.ts"] },
  {
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    files: ["**/*.{ts,tsx}"],
    plugins: { "react-hooks": reactHooks, "react-refresh": reactRefresh },
    rules: {
      ...reactHooks.configs.recommended.rules,
      "react-refresh/only-export-components": ["warn", { allowConstantExport: true }],
      "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
    },
  },
);
```

- [ ] **Step 2: Create `ui/vitest.config.ts`**

```ts
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import tailwind from "@tailwindcss/vite";
import path from "node:path";

export default defineConfig({
  plugins: [react(), tailwind()],
  resolve: { alias: { "@": path.resolve(__dirname, "src") } },
  test: {
    environment: "happy-dom",
    setupFiles: ["./tests/setup.ts"],
    include: ["tests/**/*.test.{ts,tsx}"],
    globals: true,
  },
});
```

- [ ] **Step 3: Create `ui/tests/setup.ts`**

```ts
import "@testing-library/jest-dom/vitest";
```

- [ ] **Step 4: Create `ui/playwright.config.ts`**

```ts
import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  expect: { timeout: 5_000 },
  fullyParallel: false,
  reporter: "list",
  use: {
    actionTimeout: 5_000,
    trace: "on-first-retry",
  },
});
```

- [ ] **Step 5: Stub the e2e directory**

```bash
mkdir -p ui/e2e && touch ui/e2e/.gitkeep
```

- [ ] **Step 6: Verify lint + unit harness boot**

Run: `pnpm --dir ui lint`
Expected: success — no files yet beyond App.tsx and main.tsx, no warnings.

Run: `pnpm --dir ui test:unit`
Expected: success — "No test files found, exiting with code 0" (vitest passes when `passWithNoTests` defaults to false; if it errors, append `passWithNoTests: true` to vitest config and re-run).

- [ ] **Step 7: Commit**

```bash
git add ui/eslint.config.js ui/vitest.config.ts ui/playwright.config.ts ui/tests ui/e2e
git commit -m "chore(ui): add eslint, vitest, playwright config"
```

---

## Phase 1 — Core schema bump

### Task 4: `0002_workspace_settings` migration + `Workspace::get_setting`/`set_setting`

**Files:**
- Create: `crates/kanban-core/migrations/0002_workspace_settings.sql`
- Modify: `crates/kanban-core/src/lib.rs` (or wherever `Workspace` is defined — confirm with `grep -n "pub fn open" crates/kanban-core/src/`)
- Test: `crates/kanban-core/tests/settings.rs`

- [ ] **Step 1: Write the failing integration test**

Create `crates/kanban-core/tests/settings.rs`:

```rust
use kanban_core::Workspace;

#[test]
fn theme_setting_defaults_to_system_after_migrations() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(dir.path().join("data.db")).unwrap();
    let theme = ws.get_setting("theme").unwrap();
    assert_eq!(theme.as_deref(), Some("system"));
}

#[test]
fn set_setting_then_get_setting_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(dir.path().join("data.db")).unwrap();
    ws.set_setting("theme", "dark").unwrap();
    assert_eq!(ws.get_setting("theme").unwrap().as_deref(), Some("dark"));
    assert_eq!(ws.get_setting("nonexistent").unwrap(), None);
}

#[test]
fn set_setting_overwrites_existing_value() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(dir.path().join("data.db")).unwrap();
    ws.set_setting("theme", "dark").unwrap();
    ws.set_setting("theme", "light").unwrap();
    assert_eq!(ws.get_setting("theme").unwrap().as_deref(), Some("light"));
}
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `cargo test -p kanban-core --test settings`
Expected: FAIL — `get_setting` / `set_setting` are not defined on `Workspace`.

- [ ] **Step 3: Add the migration file**

Create `crates/kanban-core/migrations/0002_workspace_settings.sql`:

```sql
CREATE TABLE workspace_settings (
  key        TEXT PRIMARY KEY NOT NULL,
  value      TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

INSERT INTO workspace_settings (key, value, updated_at)
  VALUES ('theme', 'system', datetime('now'));
```

- [ ] **Step 4: Wire the migration into the runner**

Find the migrations array in `kanban-core` (search for `0001_init.sql` with `grep -n "0001_init" crates/kanban-core/src/`). Append the new migration with `include_str!`:

```rust
const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_init", include_str!("../migrations/0001_init.sql")),
    ("0002_workspace_settings", include_str!("../migrations/0002_workspace_settings.sql")),
];
```

(Adjust the line number to match the existing format. The exact constant name may differ — match what's already there.)

- [ ] **Step 5: Add `get_setting` and `set_setting` to `Workspace`**

These are read/write methods on `Workspace` that bypass `Operation` (documented exception per the spec). Add to `crates/kanban-core/src/workspace.rs` (or wherever `impl Workspace` lives):

```rust
impl Workspace {
    pub fn get_setting(&self, key: &str) -> crate::Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM workspace_settings WHERE key = ?1")?;
        let result = stmt
            .query_row([key], |row| row.get::<_, String>(0))
            .optional()?;
        Ok(result)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> crate::Result<()> {
        use chrono::Utc;
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO workspace_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            (key, value, Utc::now().to_rfc3339()),
        )?;
        Ok(())
    }
}
```

(The `self.conn` access pattern must match the existing crate; adjust the lock/borrow style to match.)

You will also need `use rusqlite::OptionalExtension;` near the top of the file for `.optional()`.

- [ ] **Step 6: Run the tests**

Run: `cargo test -p kanban-core --test settings`
Expected: PASS, 3 tests.

Run: `cargo test --workspace`
Expected: PASS — existing tests unaffected; the new migration runs cleanly on every test fixture.

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS, no warnings.

- [ ] **Step 7: Commit**

```bash
git add crates/kanban-core/migrations/0002_workspace_settings.sql crates/kanban-core/src crates/kanban-core/tests/settings.rs
git commit -m "feat(core): add workspace_settings table and get/set_setting accessors"
```

---

## Phase 2 — Rust bridge

### Task 5: `ApiError` + `From<KanbanError>`

**Files:**
- Create: `crates/kanban-tauri/src/error.rs`
- Modify: `crates/kanban-tauri/src/main.rs`
- Test: `crates/kanban-tauri/src/error.rs` (inline `#[cfg(test)]`)

- [ ] **Step 1: Write the failing test**

Create `crates/kanban-tauri/src/error.rs`:

```rust
use serde::Serialize;
use specta::Type;
use thiserror::Error;

#[derive(Debug, Error, Serialize, Type)]
#[serde(tag = "kind")]
pub enum ApiError {
    #[error("{resource} not found: {key}")]
    NotFound { resource: String, key: String },
    #[error("validation: {field}: {message}")]
    Validation { field: String, message: String },
    #[error("conflict: {message}")]
    Conflict { message: String },
    #[error("internal: {message}")]
    Internal { message: String },
}

impl From<kanban_core::Error> for ApiError {
    fn from(e: kanban_core::Error) -> Self {
        use kanban_core::{EntityKind, Error};
        match e {
            Error::NotFound { kind, id } => ApiError::NotFound {
                resource: match kind {
                    EntityKind::Project => "project",
                    EntityKind::Issue => "issue",
                    EntityKind::Label => "label",
                    EntityKind::Status => "status",
                }
                .to_string(),
                key: id,
            },
            Error::Validation(v) => ApiError::Validation {
                field: v.field,
                message: v.reason,
            },
            Error::Conflict(msg) => ApiError::Conflict { message: msg },
            Error::Db(e) => ApiError::Internal { message: format!("db: {e}") },
            Error::Io(e) => ApiError::Internal { message: format!("io: {e}") },
            Error::Serde(e) => ApiError::Internal { message: format!("serde: {e}") },
            Error::InvalidSnapshot(s) => ApiError::Validation {
                field: "snapshot".to_string(),
                message: s,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanban_core::{EntityKind, Error, ValidationError};

    #[test]
    fn maps_not_found_with_resource_string() {
        let api: ApiError = Error::NotFound { kind: EntityKind::Issue, id: "AUTH-12".into() }.into();
        match api {
            ApiError::NotFound { resource, key } => {
                assert_eq!(resource, "issue");
                assert_eq!(key, "AUTH-12");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn maps_validation_with_field() {
        let api: ApiError = Error::Validation(ValidationError {
            field: "prefix".into(),
            reason: "must be uppercase".into(),
        })
        .into();
        match api {
            ApiError::Validation { field, message } => {
                assert_eq!(field, "prefix");
                assert_eq!(message, "must be uppercase");
            }
            _ => panic!("wrong variant"),
        }
    }
}
```

- [ ] **Step 2: Wire the module from `main.rs`**

Replace `crates/kanban-tauri/src/main.rs`:

```rust
mod error;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Run the test**

Run: `cargo test -p kanban-tauri`
Expected: PASS, 2 tests.

- [ ] **Step 4: Commit**

```bash
git add crates/kanban-tauri/src/error.rs crates/kanban-tauri/src/main.rs
git commit -m "feat(tauri): add ApiError + From<KanbanError> mapping"
```

---

### Task 6: `AppState` + `Workspace::open_default` wiring

**Files:**
- Create: `crates/kanban-tauri/src/state.rs`
- Modify: `crates/kanban-tauri/src/main.rs`

- [ ] **Step 1: Create `state.rs`**

```rust
use std::sync::Mutex;

use kanban_core::Workspace;

pub struct AppState {
    pub workspace: Mutex<Workspace>,
}

impl AppState {
    pub fn from_default() -> Result<Self, kanban_core::Error> {
        let workspace = Workspace::open_default()?;
        Ok(Self { workspace: Mutex::new(workspace) })
    }
}
```

- [ ] **Step 2: Manage AppState from `main.rs`**

```rust
mod error;
mod state;

use state::AppState;

fn main() {
    let app_state = AppState::from_default()
        .expect("failed to open kanban workspace");

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p kanban-tauri`
Expected: success.

- [ ] **Step 4: Commit**

```bash
git add crates/kanban-tauri/src/state.rs crates/kanban-tauri/src/main.rs
git commit -m "feat(tauri): add AppState wrapping Workspace under a Mutex"
```

---

### Task 7: Read commands (list/get/search) + `spawn_blocking` wrapper

**Files:**
- Create: `crates/kanban-tauri/src/commands.rs`
- Modify: `crates/kanban-tauri/src/main.rs`
- Test: `crates/kanban-tauri/tests/commands_smoke.rs`

- [ ] **Step 1: Write the failing integration test**

Create `crates/kanban-tauri/tests/commands_smoke.rs`. This exercises the underlying functions used by the Tauri commands directly (without the Tauri runtime) so we can TDD without needing tauri-driver yet:

```rust
use kanban_core::Workspace;
use kanban_tauri::commands::{list_projects_inner, list_issues_inner};

#[test]
fn list_projects_returns_empty_on_fresh_db() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(dir.path().join("data.db")).unwrap();
    let projects = list_projects_inner(&ws).unwrap();
    assert!(projects.is_empty());
}

#[test]
fn list_issues_returns_empty_for_unknown_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(dir.path().join("data.db")).unwrap();
    let issues = list_issues_inner(&ws, "AUTH").unwrap();
    assert!(issues.is_empty());
}
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `cargo test -p kanban-tauri --test commands_smoke`
Expected: FAIL — `kanban_tauri::commands` module not found.

- [ ] **Step 3: Make `kanban-tauri` a library too**

The crate currently only has a binary. Add `[lib]` in `crates/kanban-tauri/Cargo.toml`:

```toml
[lib]
name = "kanban_tauri"
path = "src/lib.rs"

[[bin]]
name = "kanban-tauri"
path = "src/main.rs"
```

Create `crates/kanban-tauri/src/lib.rs`:

```rust
pub mod commands;
pub mod error;
pub mod settings;
pub mod state;
```

(The `settings` module lands in Task 9; placeholder ok for now — we'll create an empty `settings.rs` here too if needed to keep `cargo check` happy.)

Create empty `crates/kanban-tauri/src/settings.rs`:

```rust
// stub — populated in Task 9
```

- [ ] **Step 4: Implement read commands in `commands.rs`**

```rust
use kanban_core::{Workspace, query};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::State;
use tokio::task;

use crate::error::ApiError;
use crate::state::AppState;

// === Inner functions (sync, exposed for tests) ===

pub fn list_projects_inner(ws: &Workspace) -> Result<Vec<query::ProjectRow>, ApiError> {
    Ok(query::list_projects(ws)?)
}

pub fn get_project_inner(ws: &Workspace, prefix: &str) -> Result<query::ProjectRow, ApiError> {
    Ok(query::get_project_by_prefix(ws, prefix)?)
}

pub fn list_issues_inner(ws: &Workspace, prefix: &str) -> Result<Vec<query::IssueRow>, ApiError> {
    Ok(query::list_issues_for_project(ws, prefix)?)
}

pub fn get_issue_inner(ws: &Workspace, key: &str) -> Result<query::IssueDetail, ApiError> {
    Ok(query::get_issue_by_key(ws, key)?)
}

pub fn list_statuses_inner(ws: &Workspace, prefix: &str) -> Result<Vec<query::StatusRow>, ApiError> {
    Ok(query::list_statuses_for_project(ws, prefix)?)
}

pub fn list_labels_inner(ws: &Workspace, prefix: &str) -> Result<Vec<query::LabelRow>, ApiError> {
    Ok(query::list_labels_for_project(ws, prefix)?)
}

// === Tauri commands (async wrappers) ===

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ListIssuesFilter {
    pub status: Option<String>,
    pub label: Option<String>,
    pub search: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<query::ProjectRow>, ApiError> {
    let arc = state.inner();
    let ws = arc.workspace.lock().unwrap().clone_handle();
    task::spawn_blocking(move || list_projects_inner(&ws))
        .await
        .map_err(|e| ApiError::Internal { message: e.to_string() })?
}

#[tauri::command]
#[specta::specta]
pub async fn get_project(
    state: State<'_, AppState>,
    prefix: String,
) -> Result<query::ProjectRow, ApiError> {
    let ws = state.inner().workspace.lock().unwrap().clone_handle();
    task::spawn_blocking(move || get_project_inner(&ws, &prefix))
        .await
        .map_err(|e| ApiError::Internal { message: e.to_string() })?
}

#[tauri::command]
#[specta::specta]
pub async fn list_issues(
    state: State<'_, AppState>,
    project: String,
    _filter: ListIssuesFilter,
) -> Result<Vec<query::IssueRow>, ApiError> {
    let ws = state.inner().workspace.lock().unwrap().clone_handle();
    task::spawn_blocking(move || list_issues_inner(&ws, &project))
        .await
        .map_err(|e| ApiError::Internal { message: e.to_string() })?
}

#[tauri::command]
#[specta::specta]
pub async fn get_issue(
    state: State<'_, AppState>,
    key: String,
) -> Result<query::IssueDetail, ApiError> {
    let ws = state.inner().workspace.lock().unwrap().clone_handle();
    task::spawn_blocking(move || get_issue_inner(&ws, &key))
        .await
        .map_err(|e| ApiError::Internal { message: e.to_string() })?
}

#[tauri::command]
#[specta::specta]
pub async fn list_statuses(
    state: State<'_, AppState>,
    project: String,
) -> Result<Vec<query::StatusRow>, ApiError> {
    let ws = state.inner().workspace.lock().unwrap().clone_handle();
    task::spawn_blocking(move || list_statuses_inner(&ws, &project))
        .await
        .map_err(|e| ApiError::Internal { message: e.to_string() })?
}

#[tauri::command]
#[specta::specta]
pub async fn list_labels(
    state: State<'_, AppState>,
    project: String,
) -> Result<Vec<query::LabelRow>, ApiError> {
    let ws = state.inner().workspace.lock().unwrap().clone_handle();
    task::spawn_blocking(move || list_labels_inner(&ws, &project))
        .await
        .map_err(|e| ApiError::Internal { message: e.to_string() })?
}
```

NOTE: `Workspace::clone_handle()` may not exist in core. If `Workspace` is `!Clone`, the simplest pattern is to drop the `spawn_blocking` for read commands (they're fast and the runtime tolerates a brief block) **OR** to use a connection pool. For the smallest first step, **use the `block_in_place` variant**:

```rust
#[tauri::command]
#[specta::specta]
pub async fn list_projects(state: State<'_, AppState>) -> Result<Vec<query::ProjectRow>, ApiError> {
    tokio::task::block_in_place(|| {
        let ws = state.workspace.lock().unwrap();
        list_projects_inner(&ws)
    })
}
```

`block_in_place` is safe inside the multi-thread runtime and avoids needing to clone the Workspace. Substitute this pattern for every command that fails to compile with `clone_handle`.

- [ ] **Step 5: Verify `query::*` symbols actually exist in core**

Run: `grep -nE "pub fn (list_projects|list_issues|get_issue|list_statuses|list_labels|get_project)" crates/kanban-core/src/query.rs`

If any function names differ (e.g., `list_projects` vs `all_projects`), update the imports in `commands.rs` to match. Do NOT add new query functions in this task — if a needed read isn't available, file a follow-up task and skip the corresponding command for now.

- [ ] **Step 6: Run tests**

Run: `cargo test -p kanban-tauri --test commands_smoke`
Expected: PASS, 2 tests.

Run: `cargo check -p kanban-tauri`
Expected: success.

- [ ] **Step 7: Commit**

```bash
git add crates/kanban-tauri/Cargo.toml crates/kanban-tauri/src crates/kanban-tauri/tests/commands_smoke.rs
git commit -m "feat(tauri): add read commands (list_projects/issues/statuses/labels, get_*) over kanban-core"
```

---

### Task 8: `apply` mutator command + undo/redo

**Files:**
- Modify: `crates/kanban-tauri/src/commands.rs`
- Test: `crates/kanban-tauri/tests/apply_smoke.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/kanban-tauri/tests/apply_smoke.rs`:

```rust
use kanban_core::{operation::{CreateProject, Operation}, Workspace};
use kanban_tauri::commands::apply_inner;
use uuid::Uuid;

#[test]
fn apply_create_project_then_list_returns_one() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(dir.path().join("data.db")).unwrap();

    let id = Uuid::now_v7();
    let op = Operation::CreateProject(CreateProject {
        id,
        name: "Auth Service".into(),
        prefix: "AUTH".into(),
        // remaining required fields — match the Spec #1 CreateProject struct
        // (run `awk '/^pub struct CreateProject/,/^}/' crates/kanban-core/src/operation.rs`
        // to get the exact set; pad with sensible defaults for description/colour/etc.)
        ..CreateProject::default_for_test()
    });
    apply_inner(&ws, op).unwrap();

    let projects = kanban_tauri::commands::list_projects_inner(&ws).unwrap();
    assert_eq!(projects.len(), 1);
}
```

If `CreateProject::default_for_test()` doesn't exist in core, replace `..CreateProject::default_for_test()` with explicit field initialisers — list every field the struct requires.

- [ ] **Step 2: Run the test (it should fail)**

Run: `cargo test -p kanban-tauri --test apply_smoke`
Expected: FAIL — `apply_inner` not found.

- [ ] **Step 3: Implement `apply_inner` and the Tauri command**

Append to `crates/kanban-tauri/src/commands.rs`:

```rust
use kanban_core::operation::Operation;

#[derive(Debug, Clone, Serialize, Type)]
pub struct ApplyResult {
    pub op_id: uuid::Uuid,
}

pub fn apply_inner(ws: &Workspace, op: Operation) -> Result<ApplyResult, ApiError> {
    let op_id = ws.apply(op)?;
    Ok(ApplyResult { op_id })
}

pub fn undo_inner(ws: &Workspace) -> Result<Option<Operation>, ApiError> {
    Ok(ws.undo()?)
}

pub fn redo_inner(ws: &Workspace) -> Result<Option<Operation>, ApiError> {
    Ok(ws.redo()?)
}

#[tauri::command]
#[specta::specta]
pub async fn apply(state: State<'_, AppState>, op: Operation) -> Result<ApplyResult, ApiError> {
    tokio::task::block_in_place(|| {
        let ws = state.workspace.lock().unwrap();
        apply_inner(&ws, op)
    })
}

#[tauri::command]
#[specta::specta]
pub async fn undo(state: State<'_, AppState>) -> Result<Option<Operation>, ApiError> {
    tokio::task::block_in_place(|| {
        let ws = state.workspace.lock().unwrap();
        undo_inner(&ws)
    })
}

#[tauri::command]
#[specta::specta]
pub async fn redo(state: State<'_, AppState>) -> Result<Option<Operation>, ApiError> {
    tokio::task::block_in_place(|| {
        let ws = state.workspace.lock().unwrap();
        redo_inner(&ws)
    })
}
```

The exact `Workspace::apply` return type may be `Result<Uuid>` or `Result<Operation>` — adjust `apply_inner`'s body to map whatever core returns into `ApplyResult { op_id }`. Run `grep -n "pub fn apply" crates/kanban-core/src/workspace.rs` to confirm.

- [ ] **Step 4: Run tests**

Run: `cargo test -p kanban-tauri`
Expected: PASS — both `commands_smoke.rs` and `apply_smoke.rs` green.

- [ ] **Step 5: Commit**

```bash
git add crates/kanban-tauri/src/commands.rs crates/kanban-tauri/tests/apply_smoke.rs
git commit -m "feat(tauri): add apply/undo/redo commands wrapping Workspace"
```

---

### Task 9: Settings command + `Settings`/`ThemeChoice` types

**Files:**
- Modify: `crates/kanban-tauri/src/settings.rs`
- Modify: `crates/kanban-tauri/src/commands.rs`
- Test: `crates/kanban-tauri/tests/settings_command.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/kanban-tauri/tests/settings_command.rs`:

```rust
use kanban_core::Workspace;
use kanban_tauri::commands::{get_settings_inner, update_settings_inner};
use kanban_tauri::settings::ThemeChoice;

#[test]
fn get_settings_defaults_to_system() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(dir.path().join("data.db")).unwrap();
    let settings = get_settings_inner(&ws).unwrap();
    assert_eq!(settings.theme, ThemeChoice::System);
}

#[test]
fn update_settings_persists_theme_choice() {
    let dir = tempfile::tempdir().unwrap();
    let ws = Workspace::open(dir.path().join("data.db")).unwrap();
    let after = update_settings_inner(&ws, ThemeChoice::Dark).unwrap();
    assert_eq!(after.theme, ThemeChoice::Dark);
    let again = get_settings_inner(&ws).unwrap();
    assert_eq!(again.theme, ThemeChoice::Dark);
}
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `cargo test -p kanban-tauri --test settings_command`
Expected: FAIL — `kanban_tauri::settings::ThemeChoice` and helpers not defined.

- [ ] **Step 3: Define `Settings` and `ThemeChoice`**

Replace `crates/kanban-tauri/src/settings.rs`:

```rust
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum ThemeChoice {
    System,
    Light,
    Dark,
}

impl ThemeChoice {
    pub fn as_str(self) -> &'static str {
        match self {
            ThemeChoice::System => "system",
            ThemeChoice::Light => "light",
            ThemeChoice::Dark => "dark",
        }
    }

    pub fn parse(s: &str) -> Option<ThemeChoice> {
        match s {
            "system" => Some(ThemeChoice::System),
            "light" => Some(ThemeChoice::Light),
            "dark" => Some(ThemeChoice::Dark),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub struct Settings {
    pub theme: ThemeChoice,
}
```

- [ ] **Step 4: Add settings commands to `commands.rs`**

Append:

```rust
use crate::settings::{Settings, ThemeChoice};

pub fn get_settings_inner(ws: &Workspace) -> Result<Settings, ApiError> {
    let raw = ws.get_setting("theme")?.unwrap_or_else(|| "system".into());
    let theme = ThemeChoice::parse(&raw).unwrap_or(ThemeChoice::System);
    Ok(Settings { theme })
}

pub fn update_settings_inner(ws: &Workspace, theme: ThemeChoice) -> Result<Settings, ApiError> {
    ws.set_setting("theme", theme.as_str())?;
    Ok(Settings { theme })
}

#[tauri::command]
#[specta::specta]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, ApiError> {
    tokio::task::block_in_place(|| {
        let ws = state.workspace.lock().unwrap();
        get_settings_inner(&ws)
    })
}

#[tauri::command]
#[specta::specta]
pub async fn update_settings(
    state: State<'_, AppState>,
    theme: ThemeChoice,
) -> Result<Settings, ApiError> {
    tokio::task::block_in_place(|| {
        let ws = state.workspace.lock().unwrap();
        update_settings_inner(&ws, theme)
    })
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p kanban-tauri`
Expected: PASS — all four test files green.

- [ ] **Step 6: Commit**

```bash
git add crates/kanban-tauri/src/settings.rs crates/kanban-tauri/src/commands.rs crates/kanban-tauri/tests/settings_command.rs
git commit -m "feat(tauri): add Settings + ThemeChoice + get/update_settings commands"
```

---

### Task 10: `tauri-specta` export → `ui/src/data/bindings.ts`

**Files:**
- Modify: `crates/kanban-tauri/src/main.rs`
- Modify: `crates/kanban-tauri/build.rs`
- Create: `ui/src/data/.gitkeep`
- (Generated, git-tracked) `ui/src/data/bindings.ts`

- [ ] **Step 1: Wire commands + types into the specta builder**

Replace `crates/kanban-tauri/src/main.rs`:

```rust
mod commands;
mod error;
mod settings;
mod state;

use specta_typescript::Typescript;
use state::AppState;
use tauri_specta::{collect_commands, collect_events, Builder};

fn main() {
    let app_state = AppState::from_default()
        .expect("failed to open kanban workspace");

    let specta_builder = Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            commands::list_projects,
            commands::get_project,
            commands::list_issues,
            commands::get_issue,
            commands::list_statuses,
            commands::list_labels,
            commands::get_settings,
            commands::apply,
            commands::undo,
            commands::redo,
            commands::update_settings,
        ])
        .events(collect_events![]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(Typescript::default(), "../../ui/src/data/bindings.ts")
        .expect("failed to export typescript bindings");

    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: Stub the data dir so `bindings.ts` has somewhere to go**

```bash
mkdir -p ui/src/data
touch ui/src/data/.gitkeep
```

- [ ] **Step 3: Build the binary in debug mode to trigger the export**

Run: `cargo build -p kanban-tauri`
Expected: success. The binary writes `ui/src/data/bindings.ts` on first run.

If build complains about `dyn FnOnce` or signature mismatches, re-check the `tauri-specta` version pinned in workspace deps (we use `2.0.0-rc.21`); minor RC bumps shuffle the API.

Run: `head -50 ui/src/data/bindings.ts`
Expected: TS file with `export type Operation`, `export type Project`, etc., and a typed `commands` object.

- [ ] **Step 4: Run typecheck on the UI**

Run: `pnpm --dir ui typecheck`
Expected: PASS — `bindings.ts` is valid TS, no consumers yet.

- [ ] **Step 5: Verify generated file is committed (git-tracked, per spec)**

Run: `git status ui/src/data/bindings.ts`
Expected: shown as untracked. Add it to staging.

- [ ] **Step 6: Commit**

```bash
git add crates/kanban-tauri/src/main.rs ui/src/data
git commit -m "feat(tauri): export tauri-specta TS bindings to ui/src/data/bindings.ts"
```

---

## Phase 3 — React app shell

### Task 11: Theme provider + `useSystemTheme` hook (test-first)

**Files:**
- Create: `ui/src/theme/useSystemTheme.ts`
- Create: `ui/src/theme/ThemeProvider.tsx`
- Create: `ui/tests/theme/useSystemTheme.test.ts`
- Create: `ui/tests/theme/ThemeProvider.test.tsx`

- [ ] **Step 1: Write the failing test for `useSystemTheme`**

Create `ui/tests/theme/useSystemTheme.test.ts`:

```ts
import { renderHook, act } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useSystemTheme } from "@/theme/useSystemTheme";

function mockMatchMedia(initial: boolean) {
  const listeners: Array<(e: MediaQueryListEvent) => void> = [];
  const mql = {
    matches: initial,
    addEventListener: (_evt: string, l: (e: MediaQueryListEvent) => void) => listeners.push(l),
    removeEventListener: (_evt: string, l: (e: MediaQueryListEvent) => void) => {
      const i = listeners.indexOf(l);
      if (i >= 0) listeners.splice(i, 1);
    },
    media: "(prefers-color-scheme: dark)",
    onchange: null,
    dispatchEvent: () => true,
  } as unknown as MediaQueryList;
  vi.stubGlobal("matchMedia", () => mql);
  return {
    mql,
    fire: (matches: boolean) => listeners.forEach((l) => l({ matches } as MediaQueryListEvent)),
  };
}

describe("useSystemTheme", () => {
  it("returns 'dark' when matchMedia matches", () => {
    mockMatchMedia(true);
    const { result } = renderHook(() => useSystemTheme());
    expect(result.current).toBe("dark");
  });

  it("returns 'light' when matchMedia does not match", () => {
    mockMatchMedia(false);
    const { result } = renderHook(() => useSystemTheme());
    expect(result.current).toBe("light");
  });

  it("re-renders when system preference changes", () => {
    const { fire } = mockMatchMedia(false);
    const { result } = renderHook(() => useSystemTheme());
    expect(result.current).toBe("light");
    act(() => fire(true));
    expect(result.current).toBe("dark");
  });
});
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `pnpm --dir ui test:unit`
Expected: FAIL — `useSystemTheme` not defined.

- [ ] **Step 3: Implement `useSystemTheme`**

Create `ui/src/theme/useSystemTheme.ts`:

```ts
import { useEffect, useState } from "react";

export type SystemTheme = "light" | "dark";

export function useSystemTheme(): SystemTheme {
  const [systemTheme, setSystemTheme] = useState<SystemTheme>(() =>
    typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light",
  );

  useEffect(() => {
    const mql = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => setSystemTheme(e.matches ? "dark" : "light");
    mql.addEventListener("change", handler);
    return () => mql.removeEventListener("change", handler);
  }, []);

  return systemTheme;
}
```

- [ ] **Step 4: Write the failing test for `ThemeProvider`**

Create `ui/tests/theme/ThemeProvider.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { ThemeProvider } from "@/theme/ThemeProvider";

vi.mock("@/data/queries", () => ({
  useSettings: vi.fn(),
  useUpdateSettings: vi.fn(),
}));

import { useSettings } from "@/data/queries";

beforeEach(() => {
  document.documentElement.classList.remove("dark");
  vi.stubGlobal("matchMedia", () => ({
    matches: false,
    addEventListener: () => {},
    removeEventListener: () => {},
    media: "",
    onchange: null,
    dispatchEvent: () => true,
  }));
});

describe("ThemeProvider", () => {
  it("adds dark class when explicit theme is dark", () => {
    (useSettings as unknown as ReturnType<typeof vi.fn>).mockReturnValue({ data: { theme: "dark" } });
    render(<ThemeProvider><span data-testid="child">x</span></ThemeProvider>);
    expect(screen.getByTestId("child")).toBeInTheDocument();
    expect(document.documentElement.classList.contains("dark")).toBe(true);
  });

  it("does not add dark class when explicit theme is light", () => {
    (useSettings as unknown as ReturnType<typeof vi.fn>).mockReturnValue({ data: { theme: "light" } });
    render(<ThemeProvider><span /></ThemeProvider>);
    expect(document.documentElement.classList.contains("dark")).toBe(false);
  });
});
```

- [ ] **Step 5: Implement `ThemeProvider`**

Create `ui/src/theme/ThemeProvider.tsx`:

```tsx
import { useEffect, type ReactNode } from "react";
import { useSettings } from "@/data/queries";
import { useSystemTheme } from "./useSystemTheme";

export function ThemeProvider({ children }: { children: ReactNode }) {
  const { data: settings } = useSettings();
  const systemTheme = useSystemTheme();

  useEffect(() => {
    const choice = settings?.theme ?? "system";
    const effective = choice === "system" ? systemTheme : choice;
    document.documentElement.classList.toggle("dark", effective === "dark");
  }, [settings?.theme, systemTheme]);

  return <>{children}</>;
}
```

The `useSettings` hook lands in Task 16 — for now, this test mocks it. When Task 16 ships, the test still passes.

- [ ] **Step 6: Run tests**

Run: `pnpm --dir ui test:unit`
Expected: PASS — useSystemTheme + ThemeProvider tests green.

- [ ] **Step 7: Commit**

```bash
git add ui/src/theme ui/tests/theme
git commit -m "feat(ui): add useSystemTheme hook and ThemeProvider with test coverage"
```

---

### Task 12: Tauri client wrapper (`ui/src/data/client.ts`) + `ApiError` handling

**Files:**
- Create: `ui/src/data/client.ts`
- Create: `ui/tests/data/client.test.ts`

- [ ] **Step 1: Write the failing test**

Create `ui/tests/data/client.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { isApiError, asApiError } from "@/data/client";

describe("client error helpers", () => {
  it("identifies a tagged ApiError", () => {
    const e = { kind: "NotFound", resource: "issue", key: "AUTH-12" };
    expect(isApiError(e)).toBe(true);
  });

  it("wraps an unknown error as Internal", () => {
    const e = new Error("boom");
    const api = asApiError(e);
    expect(api.kind).toBe("Internal");
    if (api.kind === "Internal") expect(api.message).toContain("boom");
  });

  it("passes through an existing ApiError", () => {
    const original = { kind: "Validation", field: "prefix", message: "bad" } as const;
    const api = asApiError(original);
    expect(api).toBe(original);
  });
});
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `pnpm --dir ui test:unit -- client`
Expected: FAIL — `@/data/client` not found.

- [ ] **Step 3: Implement the client wrapper**

Create `ui/src/data/client.ts`:

```ts
import type { ApiError } from "./bindings";

export function isApiError(value: unknown): value is ApiError {
  return (
    typeof value === "object" &&
    value !== null &&
    "kind" in value &&
    typeof (value as { kind: unknown }).kind === "string" &&
    ["NotFound", "Validation", "Conflict", "Internal"].includes((value as { kind: string }).kind)
  );
}

export function asApiError(value: unknown): ApiError {
  if (isApiError(value)) return value;
  const message = value instanceof Error ? value.message : String(value);
  return { kind: "Internal", message };
}
```

- [ ] **Step 4: Run tests**

Run: `pnpm --dir ui test:unit`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add ui/src/data/client.ts ui/tests/data/client.test.ts
git commit -m "feat(ui): add ApiError helpers in data/client"
```

---

### Task 13: Router + providers + empty state (`/`)

**Files:**
- Modify: `ui/src/App.tsx`
- Create: `ui/src/routes/_layout.tsx`
- Create: `ui/src/routes/index.tsx`

- [ ] **Step 1: Replace `App.tsx` with the provider tree + router**

```tsx
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider, createBrowserRouter } from "react-router";
import { ThemeProvider } from "@/theme/ThemeProvider";
import Layout from "./routes/_layout";
import IndexRoute from "./routes/index";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: true,
      staleTime: 5_000,
      retry: 1,
    },
  },
});

const router = createBrowserRouter([
  {
    element: <Layout />,
    children: [
      { path: "/", element: <IndexRoute /> },
      // /p/:prefix and /p/:prefix/i/:key wired in later tasks
    ],
  },
]);

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        <RouterProvider router={router} />
      </ThemeProvider>
    </QueryClientProvider>
  );
}
```

- [ ] **Step 2: Create `routes/_layout.tsx`** (sidebar slot left empty for Task 14)

```tsx
import { Outlet } from "react-router";

export default function Layout() {
  return (
    <div className="grid h-screen grid-cols-[200px_1fr] bg-white text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <aside className="border-r border-black/10 p-3 text-sm dark:border-white/10">
        {/* Sidebar lands in Task 14 */}
      </aside>
      <main className="overflow-hidden">
        <Outlet />
      </main>
    </div>
  );
}
```

- [ ] **Step 3: Create `routes/index.tsx`**

```tsx
export default function IndexRoute() {
  return (
    <div className="flex h-full items-center justify-center">
      <p className="text-sm text-neutral-500">Select or create a project to begin.</p>
    </div>
  );
}
```

- [ ] **Step 4: Manual sanity check**

Run: `pnpm --dir ui dev` and load `http://localhost:1420`. Expect the empty state. Ctrl-C.

Run: `pnpm --dir ui typecheck`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add ui/src/App.tsx ui/src/routes
git commit -m "feat(ui): add router, providers, layout, and empty-state route"
```

---

### Task 14: Sidebar with project list + new-project dialog (test-first)

**Files:**
- Create: `ui/src/components/sidebar/ProjectList.tsx`
- Create: `ui/src/components/sidebar/NewProjectDialog.tsx`
- Create: `ui/tests/components/sidebar/ProjectList.test.tsx`
- Create: `ui/tests/components/sidebar/NewProjectDialog.test.tsx`
- Modify: `ui/src/routes/_layout.tsx`

- [ ] **Step 1: Write the failing test for `ProjectList`**

Create `ui/tests/components/sidebar/ProjectList.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { describe, expect, it, vi } from "vitest";
import { ProjectList } from "@/components/sidebar/ProjectList";

vi.mock("@/data/queries", () => ({
  useProjects: () => ({
    data: [
      { id: "p1", prefix: "AUTH", name: "Auth", issue_count: 12 },
      { id: "p2", prefix: "PAY", name: "Payments", issue_count: 4 },
    ],
    isLoading: false,
    isError: false,
  }),
}));

describe("ProjectList", () => {
  it("renders each project with prefix and count", () => {
    render(<MemoryRouter><ProjectList /></MemoryRouter>);
    expect(screen.getByText(/AUTH · 12/)).toBeInTheDocument();
    expect(screen.getByText(/PAY · 4/)).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `pnpm --dir ui test:unit -- ProjectList`
Expected: FAIL — component not found.

- [ ] **Step 3: Implement `ProjectList`**

Create `ui/src/components/sidebar/ProjectList.tsx`:

```tsx
import { NavLink } from "react-router";
import { useProjects } from "@/data/queries";

export function ProjectList() {
  const { data, isLoading, isError } = useProjects();
  if (isLoading) return <p className="px-2 text-xs text-neutral-500">loading…</p>;
  if (isError) return <p className="px-2 text-xs text-red-500">couldn't load projects</p>;
  if (!data?.length) return <p className="px-2 text-xs text-neutral-500">no projects yet</p>;

  return (
    <nav className="flex flex-col gap-1">
      {data.map((p) => (
        <NavLink
          key={p.id}
          to={`/p/${p.prefix}`}
          className={({ isActive }) =>
            `rounded-md px-3 py-1.5 text-sm ${
              isActive
                ? "bg-neutral-900 text-white dark:bg-white dark:text-neutral-900"
                : "text-neutral-700 hover:bg-black/5 dark:text-neutral-300 dark:hover:bg-white/10"
            }`
          }
        >
          {p.prefix} · {p.issue_count ?? 0}
        </NavLink>
      ))}
    </nav>
  );
}
```

- [ ] **Step 4: Write the failing test for `NewProjectDialog`**

Create `ui/tests/components/sidebar/NewProjectDialog.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { NewProjectDialog } from "@/components/sidebar/NewProjectDialog";

const apply = vi.fn();
vi.mock("@/data/mutations", () => ({
  useApply: () => ({ mutate: apply, mutateAsync: apply, isPending: false }),
}));

describe("NewProjectDialog", () => {
  it("validates prefix client-side and surfaces error", async () => {
    render(<NewProjectDialog open onOpenChange={() => {}} />);
    await userEvent.type(screen.getByLabelText(/name/i), "Auth Service");
    await userEvent.type(screen.getByLabelText(/prefix/i), "lower");
    await userEvent.click(screen.getByRole("button", { name: /create/i }));
    expect(await screen.findByText(/2-8 uppercase letters/i)).toBeInTheDocument();
    expect(apply).not.toHaveBeenCalled();
  });

  it("submits a CreateProject Operation when valid", async () => {
    render(<NewProjectDialog open onOpenChange={() => {}} />);
    await userEvent.type(screen.getByLabelText(/name/i), "Auth Service");
    await userEvent.type(screen.getByLabelText(/prefix/i), "AUTH");
    await userEvent.click(screen.getByRole("button", { name: /create/i }));
    expect(apply).toHaveBeenCalledTimes(1);
    const op = apply.mock.calls[0][0];
    expect(op.op).toBe("CreateProject");
    expect(op.args.prefix).toBe("AUTH");
  });
});
```

- [ ] **Step 5: Implement `NewProjectDialog`**

Create `ui/src/components/sidebar/NewProjectDialog.tsx`:

```tsx
import { useForm } from "react-hook-form";
import { z } from "zod";
import { v7 as uuidv7 } from "uuid";
import { useApply } from "@/data/mutations";
import { ops } from "@/data/ops";

const schema = z.object({
  name: z.string().min(1, "name is required").max(80),
  prefix: z.string().regex(/^[A-Z]{2,8}$/, "2-8 uppercase letters"),
});

type FormValues = z.infer<typeof schema>;

export function NewProjectDialog({
  open,
  onOpenChange,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}) {
  const apply = useApply();
  const { register, handleSubmit, formState } = useForm<FormValues>({
    mode: "onSubmit",
    resolver: async (values) => {
      const r = schema.safeParse(values);
      if (r.success) return { values: r.data, errors: {} };
      const errors: Record<string, { type: string; message: string }> = {};
      for (const e of r.error.errors) errors[e.path[0] as string] = { type: "z", message: e.message };
      return { values: {}, errors };
    },
  });

  if (!open) return null;

  const onSubmit = (values: FormValues) => {
    apply.mutate(ops.createProject({ id: uuidv7(), name: values.name, prefix: values.prefix }));
    onOpenChange(false);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <form
        onSubmit={handleSubmit(onSubmit)}
        className="w-[360px] rounded-lg bg-white p-5 shadow-lg dark:bg-neutral-900"
      >
        <h3 className="mb-3 text-base font-semibold">New project</h3>
        <label className="mb-3 block text-sm">
          <span>Name</span>
          <input
            {...register("name")}
            className="mt-1 w-full rounded-md border border-black/15 bg-transparent px-2 py-1.5 text-sm outline-none focus:border-blue-500 dark:border-white/15"
          />
          {formState.errors.name && (
            <span className="text-xs text-red-500">{formState.errors.name.message}</span>
          )}
        </label>
        <label className="mb-4 block text-sm">
          <span>Prefix</span>
          <input
            {...register("prefix")}
            placeholder="AUTH"
            className="mt-1 w-full rounded-md border border-black/15 bg-transparent px-2 py-1.5 font-mono text-sm uppercase outline-none focus:border-blue-500 dark:border-white/15"
          />
          {formState.errors.prefix && (
            <span className="text-xs text-red-500">{formState.errors.prefix.message}</span>
          )}
        </label>
        <div className="flex justify-end gap-2 text-sm">
          <button type="button" onClick={() => onOpenChange(false)} className="px-3 py-1.5">
            Cancel
          </button>
          <button
            type="submit"
            disabled={apply.isPending}
            className="rounded-md bg-blue-600 px-3 py-1.5 text-white hover:bg-blue-700 disabled:opacity-60"
          >
            {apply.isPending ? "Creating…" : "Create"}
          </button>
        </div>
      </form>
    </div>
  );
}
```

NOTE: The `ops` and `useApply` modules don't exist yet — they land in Tasks 15–17. The component imports them now; tests pass because they are mocked. Implementation order is intentional: write the consumers, then build the data layer to satisfy them.

- [ ] **Step 6: Mount the sidebar in the layout**

Edit `ui/src/routes/_layout.tsx`:

```tsx
import { Outlet } from "react-router";
import { useState } from "react";
import { ProjectList } from "@/components/sidebar/ProjectList";
import { NewProjectDialog } from "@/components/sidebar/NewProjectDialog";

export default function Layout() {
  const [newProjectOpen, setNewProjectOpen] = useState(false);
  return (
    <div className="grid h-screen grid-cols-[200px_1fr] bg-white text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <aside className="flex flex-col border-r border-black/10 p-3 text-sm dark:border-white/10">
        <div className="mb-2 px-2 text-[11px] font-semibold uppercase tracking-wider text-neutral-500">
          Projects
        </div>
        <ProjectList />
        <button
          onClick={() => setNewProjectOpen(true)}
          className="mt-3 rounded-md border border-dashed border-black/15 px-3 py-1.5 text-xs hover:bg-black/5 dark:border-white/15 dark:hover:bg-white/10"
        >
          + New project
        </button>
        <NewProjectDialog open={newProjectOpen} onOpenChange={setNewProjectOpen} />
      </aside>
      <main className="overflow-hidden">
        <Outlet />
      </main>
    </div>
  );
}
```

- [ ] **Step 7: Run tests**

Run: `pnpm --dir ui test:unit`
Expected: PASS — both new tests green; existing tests still pass.

- [ ] **Step 8: Commit**

```bash
git add ui/src/components/sidebar ui/tests/components/sidebar ui/src/routes/_layout.tsx
git commit -m "feat(ui): add sidebar ProjectList and NewProjectDialog with TDD"
```

---

## Phase 4 — Data layer (TS)

### Task 15: Operation builders (`data/ops.ts`) — test-first

**Files:**
- Create: `ui/src/data/ops.ts`
- Create: `ui/tests/data/ops.test.ts`

- [ ] **Step 1: Write the failing test**

Create `ui/tests/data/ops.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { ops, change } from "@/data/ops";

describe("ops builders", () => {
  it("createProject wraps args under {op,args}", () => {
    const o = ops.createProject({ id: "p1", name: "Auth", prefix: "AUTH" } as any);
    expect(o).toEqual({ op: "CreateProject", args: { id: "p1", name: "Auth", prefix: "AUTH" } });
  });

  it("updateIssueField uses id + change shape", () => {
    const o = ops.updateIssueField({ id: "u1", change: change.title("hi") });
    expect(o.op).toBe("UpdateIssueField");
    expect(o.args.id).toBe("u1");
    expect(o.args.change).toEqual({ field: "Title", value: "hi" });
  });

  it("change.priority emits PascalCase Priority value", () => {
    expect(change.priority("high")).toEqual({ field: "Priority", value: "high" });
  });

  it("reorderIssue carries id and new_sort_key", () => {
    const o = ops.reorderIssue({ id: "u2", new_sort_key: 1.5 });
    expect(o).toEqual({ op: "ReorderIssue", args: { id: "u2", new_sort_key: 1.5 } });
  });
});
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `pnpm --dir ui test:unit -- ops`
Expected: FAIL — module missing.

- [ ] **Step 3: Implement `data/ops.ts`**

```ts
import type {
  Operation,
  IssueFieldChange,
  CreateProject,
  UpdateProject,
  ArchiveProject,
  DeleteProject,
  CreateIssue,
  UpdateIssueField,
  ReorderIssue,
  DeleteIssue,
  CreateLabel,
  UpdateLabel,
  DeleteLabel,
  AttachLabel,
  DetachLabel,
  ImportSnapshot,
} from "./bindings";

export const ops = {
  createProject:    (args: CreateProject)    => ({ op: "CreateProject",    args }) as unknown as Operation,
  updateProject:    (args: UpdateProject)    => ({ op: "UpdateProject",    args }) as unknown as Operation,
  archiveProject:   (args: ArchiveProject)   => ({ op: "ArchiveProject",   args }) as unknown as Operation,
  deleteProject:    (args: DeleteProject)    => ({ op: "DeleteProject",    args }) as unknown as Operation,
  createIssue:      (args: CreateIssue)      => ({ op: "CreateIssue",      args }) as unknown as Operation,
  updateIssueField: (args: UpdateIssueField) => ({ op: "UpdateIssueField", args }) as unknown as Operation,
  reorderIssue:     (args: ReorderIssue)     => ({ op: "ReorderIssue",     args }) as unknown as Operation,
  deleteIssue:      (args: DeleteIssue)      => ({ op: "DeleteIssue",      args }) as unknown as Operation,
  createLabel:      (args: CreateLabel)      => ({ op: "CreateLabel",      args }) as unknown as Operation,
  updateLabel:      (args: UpdateLabel)      => ({ op: "UpdateLabel",      args }) as unknown as Operation,
  deleteLabel:      (args: DeleteLabel)      => ({ op: "DeleteLabel",      args }) as unknown as Operation,
  attachLabel:      (args: AttachLabel)      => ({ op: "AttachLabel",      args }) as unknown as Operation,
  detachLabel:      (args: DetachLabel)      => ({ op: "DetachLabel",      args }) as unknown as Operation,
  importSnapshot:   (args: ImportSnapshot)   => ({ op: "ImportSnapshot",   args }) as unknown as Operation,
} as const;

export const change = {
  title:       (v: string)         => ({ field: "Title",       value: v }) as IssueFieldChange,
  description: (v: string | null)  => ({ field: "Description", value: v }) as IssueFieldChange,
  status:      (v: string)         => ({ field: "Status",      value: v }) as IssueFieldChange,
  priority:    (v: "none" | "low" | "medium" | "high" | "urgent") =>
    ({ field: "Priority", value: v }) as IssueFieldChange,
  dueDate:     (v: string | null)  => ({ field: "DueDate",     value: v }) as IssueFieldChange,
} as const;
```

The `as unknown as Operation` casts are necessary because tauri-specta's generated `Operation` is a discriminated union; TS can't narrow the wrapping shape from the field itself. The tests assert the runtime shape.

- [ ] **Step 4: Run tests**

Run: `pnpm --dir ui test:unit -- ops`
Expected: PASS, 4 tests.

- [ ] **Step 5: Commit**

```bash
git add ui/src/data/ops.ts ui/tests/data/ops.test.ts
git commit -m "feat(ui): add Operation + IssueFieldChange builders with tests"
```

---

### Task 16: Query hooks (`data/queries.ts`)

**Files:**
- Create: `ui/src/data/queries.ts`
- Create: `ui/tests/data/queries.test.tsx`

- [ ] **Step 1: Write the failing test**

Create `ui/tests/data/queries.test.tsx`:

```tsx
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";

vi.mock("@/data/bindings", () => ({
  commands: {
    listProjects: vi.fn().mockResolvedValue({ status: "ok", data: [{ id: "p1", prefix: "AUTH", name: "Auth", issue_count: 0 }] }),
    listIssues: vi.fn().mockResolvedValue({ status: "ok", data: [] }),
    getIssue: vi.fn(),
    listStatuses: vi.fn(),
    listLabels: vi.fn(),
    getSettings: vi.fn(),
  },
}));

import { useProjects } from "@/data/queries";

const wrapper = ({ children }: { children: React.ReactNode }) => {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
};

describe("useProjects", () => {
  it("returns the project list when commands.listProjects resolves", async () => {
    const { result } = renderHook(() => useProjects(), { wrapper });
    await waitFor(() => expect(result.current.isSuccess).toBe(true));
    expect(result.current.data?.[0].prefix).toBe("AUTH");
  });
});
```

The mocked `commands.listProjects` returns `{ status: "ok", data: ... }` which mirrors `tauri-specta`'s generated `Result<T, E>` shape. The query unwraps it.

- [ ] **Step 2: Run the test (it should fail)**

Run: `pnpm --dir ui test:unit -- queries`
Expected: FAIL — module missing.

- [ ] **Step 3: Implement `data/queries.ts`**

```ts
import { useQuery } from "@tanstack/react-query";
import { commands } from "./bindings";
import { asApiError } from "./client";
import type { Project, Issue, IssueDetail, Status, Label, Settings } from "./bindings";

export const qk = {
  projects: ()                  => ["projects"] as const,
  project:  (prefix: string)    => ["projects", prefix] as const,
  issues:   (prefix: string)    => ["projects", prefix, "issues"] as const,
  issue:    (key: string)       => ["issues", key] as const,
  statuses: (prefix: string)    => ["projects", prefix, "statuses"] as const,
  labels:   (prefix: string)    => ["projects", prefix, "labels"] as const,
  settings: ()                  => ["settings"] as const,
} as const;

async function unwrap<T>(p: Promise<{ status: "ok"; data: T } | { status: "error"; error: unknown }>): Promise<T> {
  const r = await p;
  if (r.status === "ok") return r.data;
  throw asApiError(r.error);
}

export const useProjects = () =>
  useQuery({ queryKey: qk.projects(), queryFn: () => unwrap<Project[]>(commands.listProjects()) });

export const useIssues = (prefix: string) =>
  useQuery({
    queryKey: qk.issues(prefix),
    queryFn: () => unwrap<Issue[]>(commands.listIssues(prefix, { status: null, label: null, search: null })),
    enabled: !!prefix,
  });

export const useIssue = (key: string) =>
  useQuery({
    queryKey: qk.issue(key),
    queryFn: () => unwrap<IssueDetail>(commands.getIssue(key)),
    enabled: !!key,
  });

export const useStatuses = (prefix: string) =>
  useQuery({
    queryKey: qk.statuses(prefix),
    queryFn: () => unwrap<Status[]>(commands.listStatuses(prefix)),
    enabled: !!prefix,
  });

export const useLabels = (prefix: string) =>
  useQuery({
    queryKey: qk.labels(prefix),
    queryFn: () => unwrap<Label[]>(commands.listLabels(prefix)),
    enabled: !!prefix,
  });

export const useSettings = () =>
  useQuery({ queryKey: qk.settings(), queryFn: () => unwrap<Settings>(commands.getSettings()) });
```

(If tauri-specta's actual generated shape differs from `{ status, data | error }`, adjust `unwrap` to match. Some specta versions just throw — in that case `queryFn` is `() => commands.X()` directly and `unwrap` is unnecessary.)

- [ ] **Step 4: Run tests**

Run: `pnpm --dir ui test:unit`
Expected: PASS — all earlier tests still green; new query test green.

- [ ] **Step 5: Commit**

```bash
git add ui/src/data/queries.ts ui/tests/data/queries.test.tsx
git commit -m "feat(ui): add TanStack Query hooks for reads (projects/issues/statuses/labels/settings)"
```

---

### Task 17: Mutations + optimistic dispatcher (`data/mutations.ts`)

**Files:**
- Create: `ui/src/data/mutations.ts`
- Create: `ui/tests/data/mutations.test.tsx`

- [ ] **Step 1: Write the failing test**

Create `ui/tests/data/mutations.test.tsx`:

```tsx
import { renderHook, act, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";

const applyMock = vi.fn().mockResolvedValue({ status: "ok", data: { op_id: "42" } });
vi.mock("@/data/bindings", () => ({
  commands: { apply: applyMock, undo: vi.fn(), redo: vi.fn(), updateSettings: vi.fn() },
}));

import { useApply } from "@/data/mutations";
import { ops, change } from "@/data/ops";
import { qk } from "@/data/queries";

const wrapper = (client: QueryClient) =>
  function W({ children }: { children: React.ReactNode }) {
    return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
  };

describe("useApply optimistic dispatcher", () => {
  it("optimistically removes a deleted issue from the cache and rolls back on error", async () => {
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    client.setQueryData(qk.issues("AUTH"), [{ id: "u1", key: "AUTH-1" }, { id: "u2", key: "AUTH-2" }]);

    applyMock.mockRejectedValueOnce({ kind: "Conflict", message: "no" });

    const { result } = renderHook(() => useApply("AUTH"), { wrapper: wrapper(client) });
    await act(async () => {
      result.current.mutate(ops.deleteIssue({ id: "u1" } as any));
      await waitFor(() => expect(result.current.isError).toBe(true));
    });
    const cache = client.getQueryData<any[]>(qk.issues("AUTH"));
    expect(cache?.map((i) => i.id)).toEqual(["u1", "u2"]); // rolled back
  });

  it("optimistically adds a created project to the cache on success", async () => {
    applyMock.mockResolvedValueOnce({ status: "ok", data: { op_id: "x" } });
    const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
    client.setQueryData(qk.projects(), [{ id: "p1", prefix: "AUTH", name: "Auth", issue_count: 0 }]);

    const { result } = renderHook(() => useApply(), { wrapper: wrapper(client) });
    await act(async () => {
      result.current.mutate(ops.createProject({ id: "p2", prefix: "PAY", name: "Pay" } as any));
      await waitFor(() => expect(result.current.isSuccess).toBe(true));
    });
    const cache = client.getQueryData<any[]>(qk.projects());
    expect(cache?.map((p) => p.prefix)).toContain("PAY");
  });
});
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `pnpm --dir ui test:unit -- mutations`
Expected: FAIL — module missing.

- [ ] **Step 3: Implement `data/mutations.ts`**

```ts
import { useMutation, useQueryClient, type QueryClient } from "@tanstack/react-query";
import { commands, type Operation, type Settings, type ThemeChoice } from "./bindings";
import { asApiError } from "./client";
import { qk } from "./queries";

type Ctx = { rollback?: () => void };

function optimisticPatch(qc: QueryClient, op: Operation, prefix?: string): Ctx {
  switch ((op as { op: string }).op) {
    case "CreateProject": {
      const prev = qc.getQueryData<unknown[]>(qk.projects()) ?? [];
      qc.setQueryData(qk.projects(), [...prev, (op as any).args]);
      return { rollback: () => qc.setQueryData(qk.projects(), prev) };
    }
    case "DeleteProject": {
      const prev = qc.getQueryData<any[]>(qk.projects()) ?? [];
      qc.setQueryData(
        qk.projects(),
        prev.filter((p) => p.id !== (op as any).args.id),
      );
      return { rollback: () => qc.setQueryData(qk.projects(), prev) };
    }
    case "CreateIssue": {
      if (!prefix) return {};
      const prev = qc.getQueryData<any[]>(qk.issues(prefix)) ?? [];
      qc.setQueryData(qk.issues(prefix), [...prev, (op as any).args]);
      return { rollback: () => qc.setQueryData(qk.issues(prefix), prev) };
    }
    case "DeleteIssue": {
      if (!prefix) return {};
      const prev = qc.getQueryData<any[]>(qk.issues(prefix)) ?? [];
      qc.setQueryData(
        qk.issues(prefix),
        prev.filter((i) => i.id !== (op as any).args.id),
      );
      return { rollback: () => qc.setQueryData(qk.issues(prefix), prev) };
    }
    case "UpdateIssueField":
    case "ReorderIssue": {
      // For these we just re-fetch on settle — granular cache patches per
      // IssueFieldChange variant land in Spec #3 polish.
      return {};
    }
    default:
      return {};
  }
}

function invalidateFor(qc: QueryClient, op: Operation, prefix?: string) {
  const tag = (op as { op: string }).op;
  if (tag === "CreateProject" || tag === "UpdateProject" || tag === "ArchiveProject" || tag === "DeleteProject") {
    qc.invalidateQueries({ queryKey: qk.projects() });
  }
  if (tag === "CreateIssue" || tag === "UpdateIssueField" || tag === "ReorderIssue" || tag === "DeleteIssue") {
    if (prefix) qc.invalidateQueries({ queryKey: qk.issues(prefix) });
    if (tag === "UpdateIssueField") {
      const id = (op as any).args?.id;
      if (id) qc.invalidateQueries({ queryKey: qk.issue(id) });
    }
  }
  if (tag === "AttachLabel" || tag === "DetachLabel" || tag === "CreateLabel" || tag === "UpdateLabel" || tag === "DeleteLabel") {
    if (prefix) qc.invalidateQueries({ queryKey: qk.labels(prefix) });
    if (prefix) qc.invalidateQueries({ queryKey: qk.issues(prefix) });
  }
  if (tag === "ImportSnapshot") {
    qc.invalidateQueries();
  }
}

export function useApply(prefixForCache?: string) {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (op: Operation) => {
      const r = await commands.apply(op);
      if ((r as any).status !== "ok") throw asApiError((r as any).error);
      return (r as any).data;
    },
    onMutate: (op) => optimisticPatch(qc, op, prefixForCache),
    onError: (_err, _op, ctx?: Ctx) => ctx?.rollback?.(),
    onSettled: (_data, _err, op) => invalidateFor(qc, op, prefixForCache),
  });
}

export function useUpdateSettings() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (theme: ThemeChoice): Promise<Settings> => {
      const r = await commands.updateSettings(theme);
      if ((r as any).status !== "ok") throw asApiError((r as any).error);
      return (r as any).data;
    },
    onSuccess: (settings) => qc.setQueryData(qk.settings(), settings),
  });
}
```

- [ ] **Step 4: Run tests**

Run: `pnpm --dir ui test:unit`
Expected: PASS — all tests green.

- [ ] **Step 5: Commit**

```bash
git add ui/src/data/mutations.ts ui/tests/data/mutations.test.tsx
git commit -m "feat(ui): add useApply with optimistic dispatch + useUpdateSettings"
```

---

## Phase 5 — Board

### Task 18: `lib/sortKey.ts` midpoint helper (test-first)

**Files:**
- Create: `ui/src/lib/sortKey.ts`
- Create: `ui/tests/lib/sortKey.test.ts`

- [ ] **Step 1: Write the failing test**

Create `ui/tests/lib/sortKey.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { midpoint } from "@/lib/sortKey";

describe("midpoint", () => {
  it("returns 1024 for an empty column", () => {
    expect(midpoint(undefined, undefined)).toBe(1024);
  });
  it("subtracts 1024 when dropped at top", () => {
    expect(midpoint(undefined, 5000)).toBe(5000 - 1024);
  });
  it("adds 1024 when dropped at bottom", () => {
    expect(midpoint(5000, undefined)).toBe(5000 + 1024);
  });
  it("returns the average between two neighbors", () => {
    expect(midpoint(1, 2)).toBe(1.5);
    expect(midpoint(0, 10)).toBe(5);
  });
});
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `pnpm --dir ui test:unit -- sortKey`
Expected: FAIL — module missing.

- [ ] **Step 3: Implement `lib/sortKey.ts`**

```ts
// f64 precision affords ~50 reorders before adjacent sort_keys collide.
// A `RebalanceColumn` operation lands in Spec #3.
export function midpoint(before?: number, after?: number): number {
  if (before == null && after == null) return 1024;
  if (before == null) return after! - 1024;
  if (after == null) return before + 1024;
  return (before + after) / 2;
}
```

- [ ] **Step 4: Run tests**

Run: `pnpm --dir ui test:unit -- sortKey`
Expected: PASS, 4 tests.

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/sortKey.ts ui/tests/lib/sortKey.test.ts
git commit -m "feat(ui): add sort-key midpoint helper"
```

---

### Task 19: `IssueCard` (presentational, test-first) + `BoardColumn`

**Files:**
- Create: `ui/src/components/board/IssueCard.tsx`
- Create: `ui/src/components/board/BoardColumn.tsx`
- Create: `ui/tests/components/board/IssueCard.test.tsx`

- [ ] **Step 1: Write the failing test for `IssueCard`**

Create `ui/tests/components/board/IssueCard.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { describe, expect, it } from "vitest";
import { IssueCard } from "@/components/board/IssueCard";

describe("IssueCard", () => {
  it("renders the key and title", () => {
    render(
      <MemoryRouter>
        <IssueCard
          projectPrefix="AUTH"
          issue={{ id: "u1", key: "AUTH-12", title: "Add OAuth", priority: "high", sort_key: 1 } as any}
        />
      </MemoryRouter>,
    );
    expect(screen.getByText("AUTH-12")).toBeInTheDocument();
    expect(screen.getByText("Add OAuth")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `pnpm --dir ui test:unit -- IssueCard`
Expected: FAIL — component missing.

- [ ] **Step 3: Implement `IssueCard`**

Create `ui/src/components/board/IssueCard.tsx`:

```tsx
import { useNavigate } from "react-router";
import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import type { Issue } from "@/data/bindings";

export function IssueCard({ projectPrefix, issue }: { projectPrefix: string; issue: Issue }) {
  const navigate = useNavigate();
  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: issue.id,
  });
  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    opacity: isDragging ? 0.4 : 1,
  };
  return (
    <div
      ref={setNodeRef}
      style={style}
      {...attributes}
      {...listeners}
      onClick={() => navigate(`/p/${projectPrefix}/i/${issue.key}`)}
      className="cursor-grab rounded-md border border-black/10 bg-white p-2.5 shadow-sm hover:border-black/20 dark:border-white/10 dark:bg-neutral-900"
    >
      <div className="font-mono text-[11px] text-neutral-500">{issue.key}</div>
      <div className="mt-1 text-[13px] font-medium leading-snug">{issue.title}</div>
      {issue.priority && issue.priority !== "none" && (
        <span className="mt-1 inline-block rounded bg-red-100 px-1.5 py-0.5 text-[10px] text-red-700 dark:bg-red-900/40 dark:text-red-300">
          {issue.priority}
        </span>
      )}
    </div>
  );
}
```

- [ ] **Step 4: Implement `BoardColumn`**

Create `ui/src/components/board/BoardColumn.tsx`:

```tsx
import { SortableContext, verticalListSortingStrategy } from "@dnd-kit/sortable";
import type { Issue, Status } from "@/data/bindings";
import { IssueCard } from "./IssueCard";

export function BoardColumn({
  status,
  issues,
  projectPrefix,
  onAddIssue,
}: {
  status: Status;
  issues: Issue[];
  projectPrefix: string;
  onAddIssue: (statusId: string) => void;
}) {
  return (
    <div className="flex w-[260px] shrink-0 flex-col gap-2 rounded-lg bg-black/[0.03] p-2.5 dark:bg-white/[0.04]">
      <div className="flex items-center justify-between px-1">
        <span className="text-sm font-semibold">{status.name}</span>
        <span className="text-xs text-neutral-500">{issues.length}</span>
      </div>
      <SortableContext items={issues.map((i) => i.id)} strategy={verticalListSortingStrategy}>
        {issues.map((issue) => (
          <IssueCard key={issue.id} projectPrefix={projectPrefix} issue={issue} />
        ))}
      </SortableContext>
      <button
        onClick={() => onAddIssue(status.id)}
        className="mt-auto rounded-md border border-dashed border-black/15 px-2 py-1 text-xs text-neutral-600 hover:bg-black/5 dark:border-white/15 dark:text-neutral-300 dark:hover:bg-white/10"
      >
        + Add issue
      </button>
    </div>
  );
}
```

- [ ] **Step 5: Run tests**

Run: `pnpm --dir ui test:unit -- IssueCard`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add ui/src/components/board ui/tests/components/board
git commit -m "feat(ui): add IssueCard + BoardColumn presentational components"
```

---

### Task 20: `Board` view with dnd-kit + reorder/cross-column drag

**Files:**
- Create: `ui/src/components/board/Board.tsx`
- Create: `ui/src/routes/projects.$prefix.tsx`
- Create: `ui/tests/components/board/Board.test.tsx`
- Modify: `ui/src/App.tsx` (route added)

- [ ] **Step 1: Write the failing test**

Create `ui/tests/components/board/Board.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";

vi.mock("@/data/queries", () => ({
  useIssues: () => ({
    data: [
      { id: "u1", key: "AUTH-1", title: "First",  status_id: "s1", sort_key: 1 },
      { id: "u2", key: "AUTH-2", title: "Second", status_id: "s2", sort_key: 1 },
    ],
    isLoading: false, isError: false,
  }),
  useStatuses: () => ({
    data: [
      { id: "s1", name: "To Do",       sort_order: 1 },
      { id: "s2", name: "In Progress", sort_order: 2 },
    ],
    isLoading: false, isError: false,
  }),
  useLabels: () => ({ data: [], isLoading: false, isError: false }),
  qk: { issues: () => ["x"], statuses: () => ["s"], labels: () => ["l"] },
}));

vi.mock("@/data/mutations", () => ({ useApply: () => ({ mutate: vi.fn(), isPending: false }) }));

import { Board } from "@/components/board/Board";

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={new QueryClient()}><MemoryRouter>{children}</MemoryRouter></QueryClientProvider>
);

describe("Board", () => {
  it("renders one column per status with the right issues", () => {
    render(<Board projectPrefix="AUTH" projectId="p1" />, { wrapper });
    expect(screen.getByText("To Do")).toBeInTheDocument();
    expect(screen.getByText("In Progress")).toBeInTheDocument();
    expect(screen.getByText("First")).toBeInTheDocument();
    expect(screen.getByText("Second")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `pnpm --dir ui test:unit -- Board`
Expected: FAIL — component missing.

- [ ] **Step 3: Implement `Board`**

Create `ui/src/components/board/Board.tsx`:

```tsx
import { useMemo, useState } from "react";
import { DndContext, type DragEndEvent, PointerSensor, useSensor, useSensors } from "@dnd-kit/core";
import { useIssues, useStatuses } from "@/data/queries";
import { useApply } from "@/data/mutations";
import { ops, change } from "@/data/ops";
import { midpoint } from "@/lib/sortKey";
import { BoardColumn } from "./BoardColumn";
import { NewIssueInline } from "./NewIssueInline";
import type { Issue } from "@/data/bindings";

export function Board({ projectPrefix, projectId }: { projectPrefix: string; projectId: string }) {
  const { data: issues = [] } = useIssues(projectPrefix);
  const { data: statuses = [] } = useStatuses(projectPrefix);
  const apply = useApply(projectPrefix);
  const sensors = useSensors(useSensor(PointerSensor, { activationConstraint: { distance: 4 } }));
  const [addingStatus, setAddingStatus] = useState<string | null>(null);

  const columns = useMemo(() => {
    return statuses
      .slice()
      .sort((a, b) => a.sort_order - b.sort_order)
      .map((s) => ({
        status: s,
        issues: issues.filter((i) => i.status_id === s.id).sort((a, b) => a.sort_key - b.sort_key),
      }));
  }, [statuses, issues]);

  function handleDragEnd(event: DragEndEvent) {
    const draggedId = event.active.id as string;
    const targetId = event.over?.id as string | undefined;
    if (!targetId || draggedId === targetId) return;

    const dragged = issues.find((i) => i.id === draggedId);
    if (!dragged) return;

    // Resolve target column + neighbours
    const target = issues.find((i) => i.id === targetId);
    const targetStatusId = target?.status_id ?? statuses.find((s) => s.id === targetId)?.id;
    if (!targetStatusId) return;

    const colIssues = issues
      .filter((i) => i.status_id === targetStatusId && i.id !== draggedId)
      .sort((a, b) => a.sort_key - b.sort_key);
    const idx = target ? colIssues.findIndex((i) => i.id === target.id) : colIssues.length;
    const before = idx > 0 ? colIssues[idx - 1]?.sort_key : undefined;
    const after  = colIssues[idx]?.sort_key;

    if (dragged.status_id !== targetStatusId) {
      apply.mutate(ops.updateIssueField({ id: dragged.id, change: change.status(targetStatusId) }));
    }
    apply.mutate(ops.reorderIssue({ id: dragged.id, new_sort_key: midpoint(before, after) }));
  }

  return (
    <div className="flex h-full flex-col">
      <DndContext sensors={sensors} onDragEnd={handleDragEnd}>
        <div className="flex h-full gap-3 overflow-x-auto p-4">
          {columns.map(({ status, issues }) => (
            <BoardColumn
              key={status.id}
              status={status}
              issues={issues as Issue[]}
              projectPrefix={projectPrefix}
              onAddIssue={() => setAddingStatus(status.id)}
            />
          ))}
        </div>
      </DndContext>
      {addingStatus && (
        <NewIssueInline
          projectId={projectId}
          projectPrefix={projectPrefix}
          statusId={addingStatus}
          onDone={() => setAddingStatus(null)}
        />
      )}
    </div>
  );
}
```

- [ ] **Step 4: Add the project route**

Create `ui/src/routes/projects.$prefix.tsx`:

```tsx
import { useParams } from "react-router";
import { Board } from "@/components/board/Board";
import { useProjects } from "@/data/queries";

export default function ProjectRoute() {
  const { prefix = "" } = useParams();
  const { data: projects } = useProjects();
  const project = projects?.find((p) => p.prefix === prefix);
  if (!project) return <div className="p-6 text-sm text-neutral-500">project not found</div>;
  return <Board projectPrefix={prefix} projectId={project.id} />;
}
```

- [ ] **Step 5: Wire the route in `App.tsx`**

Replace the router definition in `ui/src/App.tsx`:

```tsx
import ProjectRoute from "./routes/projects.$prefix";

const router = createBrowserRouter([
  {
    element: <Layout />,
    children: [
      { path: "/", element: <IndexRoute /> },
      { path: "/p/:prefix", element: <ProjectRoute /> },
    ],
  },
]);
```

- [ ] **Step 6: Run tests**

Run: `pnpm --dir ui test:unit`
Expected: PASS — all green. (`NewIssueInline` is referenced but not yet defined; tests don't render it because `addingStatus` starts as null. Typecheck will fail until Task 21.)

Run: `pnpm --dir ui typecheck`
Expected: FAIL on missing `NewIssueInline` — proceed to Task 21 immediately to satisfy the import.

- [ ] **Step 7: Commit**

```bash
git add ui/src/components/board/Board.tsx ui/src/routes/projects.$prefix.tsx ui/src/App.tsx ui/tests/components/board/Board.test.tsx
git commit -m "feat(ui): add Board with dnd-kit drag + cross-column status update"
```

---

### Task 21: `NewIssueInline` form + `CreateIssue` flow

**Files:**
- Create: `ui/src/components/board/NewIssueInline.tsx`
- Create: `ui/src/lib/uuid.ts`
- Create: `ui/tests/components/board/NewIssueInline.test.tsx`

- [ ] **Step 1: Write the failing test**

Create `ui/tests/components/board/NewIssueInline.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

const mutate = vi.fn();
vi.mock("@/data/mutations", () => ({ useApply: () => ({ mutate, isPending: false }) }));

import { NewIssueInline } from "@/components/board/NewIssueInline";

describe("NewIssueInline", () => {
  it("submits a CreateIssue Operation on Enter", async () => {
    render(
      <NewIssueInline projectId="p1" projectPrefix="AUTH" statusId="s1" onDone={() => {}} />,
    );
    await userEvent.type(screen.getByPlaceholderText(/new issue/i), "Add login{enter}");
    expect(mutate).toHaveBeenCalledTimes(1);
    const op = mutate.mock.calls[0][0];
    expect(op.op).toBe("CreateIssue");
    expect(op.args.title).toBe("Add login");
    expect(op.args.project_id).toBe("p1");
    expect(op.args.status_id).toBe("s1");
  });
});
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `pnpm --dir ui test:unit -- NewIssueInline`
Expected: FAIL — module missing.

- [ ] **Step 3: Add a UUID v7 helper**

Create `ui/src/lib/uuid.ts`:

```ts
import { v7 } from "uuid";
export const uuidv7 = (): string => v7();
```

- [ ] **Step 4: Implement `NewIssueInline`**

Create `ui/src/components/board/NewIssueInline.tsx`:

```tsx
import { useState } from "react";
import { useApply } from "@/data/mutations";
import { ops } from "@/data/ops";
import { uuidv7 } from "@/lib/uuid";

export function NewIssueInline({
  projectId,
  projectPrefix,
  statusId,
  onDone,
}: {
  projectId: string;
  projectPrefix: string;
  statusId: string;
  onDone: () => void;
}) {
  const [title, setTitle] = useState("");
  const apply = useApply(projectPrefix);
  return (
    <div className="border-t border-black/10 p-3 dark:border-white/10">
      <input
        autoFocus
        placeholder="New issue title…"
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Escape") onDone();
          if (e.key === "Enter" && title.trim()) {
            apply.mutate(
              ops.createIssue({
                id: uuidv7(),
                project_id: projectId,
                title: title.trim(),
                description: null,
                status_id: statusId,
                priority: "medium",
                due_date: null,
                label_ids: [],
              } as any),
            );
            setTitle("");
            onDone();
          }
        }}
        className="w-full rounded-md border border-black/15 bg-white px-2 py-1.5 text-sm dark:border-white/15 dark:bg-neutral-900"
      />
    </div>
  );
}
```

- [ ] **Step 5: Run tests + typecheck**

Run: `pnpm --dir ui test:unit`
Expected: PASS.

Run: `pnpm --dir ui typecheck`
Expected: PASS — Board's import of `NewIssueInline` now resolves.

- [ ] **Step 6: Commit**

```bash
git add ui/src/components/board/NewIssueInline.tsx ui/src/lib/uuid.ts ui/tests/components/board/NewIssueInline.test.tsx
git commit -m "feat(ui): add NewIssueInline form for inline create-issue"
```

---

## Phase 6 — Detail panel

### Task 22: `MarkdownView` (test-first) + system-browser link handling

**Files:**
- Create: `ui/src/components/detail/MarkdownView.tsx`
- Create: `ui/tests/components/detail/MarkdownView.test.tsx`

- [ ] **Step 1: Write the failing test**

Create `ui/tests/components/detail/MarkdownView.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const openMock = vi.fn();
vi.mock("@tauri-apps/plugin-shell", () => ({ open: openMock }));

import { MarkdownView } from "@/components/detail/MarkdownView";

describe("MarkdownView", () => {
  it("renders headings, lists, and code", () => {
    render(<MarkdownView source={"# Hello\n\n- one\n- two\n\n`code`"} />);
    expect(screen.getByRole("heading", { name: "Hello" })).toBeInTheDocument();
    expect(screen.getAllByRole("listitem")).toHaveLength(2);
    expect(screen.getByText("code")).toBeInTheDocument();
  });

  it("opens links via tauri shell instead of in-WebView nav", async () => {
    render(<MarkdownView source={"[link](https://example.com)"} />);
    const a = screen.getByRole("link", { name: "link" });
    await a.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
    expect(openMock).toHaveBeenCalledWith("https://example.com");
  });
});
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `pnpm --dir ui test:unit -- MarkdownView`
Expected: FAIL — module missing.

- [ ] **Step 3: Implement `MarkdownView`**

Create `ui/src/components/detail/MarkdownView.tsx`:

```tsx
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import { open } from "@tauri-apps/plugin-shell";

export function MarkdownView({ source }: { source: string }) {
  return (
    <ReactMarkdown
      remarkPlugins={[remarkGfm]}
      components={{
        a({ href, children, ...rest }) {
          return (
            <a
              {...rest}
              href={href}
              onClick={(e) => {
                if (href) {
                  e.preventDefault();
                  void open(href);
                }
              }}
              className="underline decoration-dotted underline-offset-2"
            >
              {children}
            </a>
          );
        },
      }}
    >
      {source}
    </ReactMarkdown>
  );
}
```

- [ ] **Step 4: Run tests**

Run: `pnpm --dir ui test:unit -- MarkdownView`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add ui/src/components/detail/MarkdownView.tsx ui/tests/components/detail/MarkdownView.test.tsx
git commit -m "feat(ui): add MarkdownView with system-browser link handling"
```

---

### Task 23: `EditableField` + `EditableDescription`

**Files:**
- Create: `ui/src/components/detail/EditableField.tsx`
- Create: `ui/src/components/detail/EditableDescription.tsx`
- Create: `ui/tests/components/detail/EditableField.test.tsx`

- [ ] **Step 1: Write the failing test for `EditableField`**

Create `ui/tests/components/detail/EditableField.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { EditableField } from "@/components/detail/EditableField";

describe("EditableField", () => {
  it("commits new value on blur", async () => {
    const onCommit = vi.fn();
    render(<EditableField value="old" onCommit={onCommit} />);
    await userEvent.click(screen.getByText("old"));
    const input = screen.getByDisplayValue("old");
    await userEvent.clear(input);
    await userEvent.type(input, "new");
    input.blur();
    expect(onCommit).toHaveBeenCalledWith("new");
  });

  it("reverts on Escape", async () => {
    const onCommit = vi.fn();
    render(<EditableField value="old" onCommit={onCommit} />);
    await userEvent.click(screen.getByText("old"));
    const input = screen.getByDisplayValue("old");
    await userEvent.type(input, "{Escape}");
    expect(onCommit).not.toHaveBeenCalled();
    expect(screen.getByText("old")).toBeInTheDocument();
  });
});
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `pnpm --dir ui test:unit -- EditableField`
Expected: FAIL.

- [ ] **Step 3: Implement `EditableField`**

Create `ui/src/components/detail/EditableField.tsx`:

```tsx
import { useState } from "react";

export function EditableField({
  value,
  onCommit,
  className,
}: {
  value: string;
  onCommit: (next: string) => void;
  className?: string;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);

  if (!editing) {
    return (
      <h3
        className={`cursor-text text-lg font-semibold leading-tight ${className ?? ""}`}
        onClick={() => {
          setDraft(value);
          setEditing(true);
        }}
      >
        {value}
      </h3>
    );
  }

  return (
    <input
      autoFocus
      value={draft}
      onChange={(e) => setDraft(e.target.value)}
      onBlur={() => {
        if (draft.trim() && draft !== value) onCommit(draft.trim());
        setEditing(false);
      }}
      onKeyDown={(e) => {
        if (e.key === "Enter") (e.target as HTMLInputElement).blur();
        if (e.key === "Escape") {
          setDraft(value);
          setEditing(false);
        }
      }}
      className={`w-full rounded-md border border-black/15 bg-transparent px-2 py-1 text-lg font-semibold outline-none focus:border-blue-500 dark:border-white/15 ${className ?? ""}`}
    />
  );
}
```

- [ ] **Step 4: Implement `EditableDescription`**

Create `ui/src/components/detail/EditableDescription.tsx`:

```tsx
import { useState } from "react";
import { MarkdownView } from "./MarkdownView";

export function EditableDescription({
  value,
  onCommit,
}: {
  value: string;
  onCommit: (next: string | null) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [draft, setDraft] = useState(value);

  if (!editing) {
    return (
      <div>
        <div className="mb-2 flex items-center justify-between">
          <span className="text-[11px] font-semibold uppercase tracking-wider text-neutral-500">Description</span>
          <button
            onClick={() => {
              setDraft(value);
              setEditing(true);
            }}
            className="rounded-md border border-black/15 px-2 py-0.5 text-[11px] hover:bg-black/5 dark:border-white/15 dark:hover:bg-white/10"
          >
            Edit
          </button>
        </div>
        <MarkdownView source={value || "_no description_"} />
      </div>
    );
  }

  return (
    <div>
      <textarea
        autoFocus
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        rows={10}
        className="w-full rounded-md border border-black/15 bg-transparent p-2 font-mono text-sm dark:border-white/15"
      />
      <div className="mt-2 flex justify-end gap-2 text-sm">
        <button onClick={() => setEditing(false)} className="px-3 py-1">Cancel</button>
        <button
          onClick={() => {
            onCommit(draft.length ? draft : null);
            setEditing(false);
          }}
          className="rounded-md bg-blue-600 px-3 py-1 text-white hover:bg-blue-700"
        >
          Save
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 5: Run tests**

Run: `pnpm --dir ui test:unit -- Editable`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add ui/src/components/detail/EditableField.tsx ui/src/components/detail/EditableDescription.tsx ui/tests/components/detail/EditableField.test.tsx
git commit -m "feat(ui): add EditableField and EditableDescription for detail panel"
```

---

### Task 24: `IssuePanel` + nested route + Esc/click-out close

**Files:**
- Create: `ui/src/components/detail/IssuePanel.tsx`
- Create: `ui/src/routes/projects.$prefix.issues.$key.tsx`
- Create: `ui/tests/components/detail/IssuePanel.test.tsx`
- Modify: `ui/src/App.tsx`

- [ ] **Step 1: Write the failing test**

Create `ui/tests/components/detail/IssuePanel.test.tsx`:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";

const mutate = vi.fn();
vi.mock("@/data/mutations", () => ({ useApply: () => ({ mutate, isPending: false }) }));
vi.mock("@/data/queries", () => ({
  useIssue: () => ({
    data: { id: "u1", key: "AUTH-12", title: "Hello", description: "# body", status_id: "s1", priority: "high", labels: [] },
    isLoading: false, isError: false,
  }),
  useStatuses: () => ({ data: [{ id: "s1", name: "To Do", sort_order: 1 }, { id: "s2", name: "Done", sort_order: 2 }] }),
}));
vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn() }));

import { IssuePanel } from "@/components/detail/IssuePanel";

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={new QueryClient()}>
    <MemoryRouter initialEntries={["/p/AUTH/i/AUTH-12"]}>
      <Routes>
        <Route path="/p/:prefix/i/:key" element={children} />
      </Routes>
    </MemoryRouter>
  </QueryClientProvider>
);

describe("IssuePanel", () => {
  it("renders issue title and key", () => {
    render(<IssuePanel />, { wrapper });
    expect(screen.getByText("AUTH-12")).toBeInTheDocument();
    expect(screen.getByText("Hello")).toBeInTheDocument();
  });

  it("dispatches updateIssueField on status change", async () => {
    render(<IssuePanel />, { wrapper });
    await userEvent.selectOptions(screen.getByRole("combobox"), "s2");
    expect(mutate).toHaveBeenCalled();
    const op = mutate.mock.calls.at(-1)![0];
    expect(op.op).toBe("UpdateIssueField");
    expect(op.args.change).toEqual({ field: "Status", value: "s2" });
  });
});
```

- [ ] **Step 2: Run the test (it should fail)**

Run: `pnpm --dir ui test:unit -- IssuePanel`
Expected: FAIL.

- [ ] **Step 3: Implement `IssuePanel`**

Create `ui/src/components/detail/IssuePanel.tsx`:

```tsx
import { useEffect } from "react";
import { useNavigate, useParams } from "react-router";
import { useIssue, useStatuses } from "@/data/queries";
import { useApply } from "@/data/mutations";
import { ops, change } from "@/data/ops";
import { EditableField } from "./EditableField";
import { EditableDescription } from "./EditableDescription";

export function IssuePanel() {
  const { prefix = "", key = "" } = useParams();
  const navigate = useNavigate();
  const close = () => navigate(`/p/${prefix}`);
  const { data: issue } = useIssue(key);
  const { data: statuses = [] } = useStatuses(prefix);
  const apply = useApply(prefix);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") close(); };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  if (!issue) return null;

  return (
    <>
      <div onClick={close} className="absolute inset-0 z-40 bg-black/40 backdrop-blur-[1px]" />
      <aside className="absolute right-0 top-0 z-50 flex h-full w-[520px] flex-col border-l border-black/10 bg-white shadow-2xl dark:border-white/10 dark:bg-neutral-950">
        <header className="flex items-center justify-between border-b border-black/10 px-4 py-3 text-sm dark:border-white/10">
          <div className="flex items-center gap-3">
            <span className="font-mono text-xs text-neutral-500">{issue.key}</span>
            <select
              value={issue.status_id}
              onChange={(e) =>
                apply.mutate(ops.updateIssueField({ id: issue.id, change: change.status(e.target.value) }))
              }
              className="rounded border border-black/15 bg-transparent px-1.5 py-0.5 text-xs dark:border-white/15"
            >
              {statuses.map((s) => (
                <option key={s.id} value={s.id}>{s.name}</option>
              ))}
            </select>
          </div>
          <button onClick={close} className="rounded p-1 text-neutral-500 hover:bg-black/5 dark:hover:bg-white/10">✕</button>
        </header>

        <section className="px-5 pt-4">
          <EditableField
            value={issue.title}
            onCommit={(next) => apply.mutate(ops.updateIssueField({ id: issue.id, change: change.title(next) }))}
          />
        </section>

        <section className="flex-1 overflow-y-auto px-5 py-3">
          <EditableDescription
            value={issue.description ?? ""}
            onCommit={(next) =>
              apply.mutate(ops.updateIssueField({ id: issue.id, change: change.description(next) }))
            }
          />
        </section>
      </aside>
    </>
  );
}
```

- [ ] **Step 4: Add the nested route**

Create `ui/src/routes/projects.$prefix.issues.$key.tsx`:

```tsx
import { useParams } from "react-router";
import { useProjects } from "@/data/queries";
import { Board } from "@/components/board/Board";
import { IssuePanel } from "@/components/detail/IssuePanel";

export default function IssueRoute() {
  const { prefix = "" } = useParams();
  const { data: projects } = useProjects();
  const project = projects?.find((p) => p.prefix === prefix);
  if (!project) return null;
  return (
    <div className="relative h-full">
      <Board projectPrefix={prefix} projectId={project.id} />
      <IssuePanel />
    </div>
  );
}
```

- [ ] **Step 5: Wire the route in `App.tsx`**

```tsx
import IssueRoute from "./routes/projects.$prefix.issues.$key";

// inside createBrowserRouter children:
{ path: "/p/:prefix/i/:key", element: <IssueRoute /> },
```

- [ ] **Step 6: Run tests + typecheck**

Run: `pnpm --dir ui test:unit`
Expected: PASS.

Run: `pnpm --dir ui typecheck`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add ui/src/components/detail/IssuePanel.tsx ui/src/routes/projects.$prefix.issues.$key.tsx ui/src/App.tsx ui/tests/components/detail/IssuePanel.test.tsx
git commit -m "feat(ui): add IssuePanel detail overlay with click-to-edit fields"
```

---

### Task 25: Title bar with theme toggle

**Files:**
- Create: `ui/src/components/chrome/TitleBar.tsx`
- Create: `ui/src/components/chrome/ThemeToggle.tsx`
- Modify: `ui/src/routes/_layout.tsx`

- [ ] **Step 1: Implement `ThemeToggle`**

Create `ui/src/components/chrome/ThemeToggle.tsx`:

```tsx
import { Moon, Sun, Monitor } from "lucide-react";
import { useSettings } from "@/data/queries";
import { useUpdateSettings } from "@/data/mutations";
import type { ThemeChoice } from "@/data/bindings";

const ORDER: ThemeChoice[] = ["system", "light", "dark"];
const ICON: Record<ThemeChoice, JSX.Element> = {
  system: <Monitor className="h-4 w-4" />,
  light: <Sun className="h-4 w-4" />,
  dark: <Moon className="h-4 w-4" />,
};

export function ThemeToggle() {
  const { data: settings } = useSettings();
  const update = useUpdateSettings();
  const current = settings?.theme ?? "system";
  const next = () => {
    const i = ORDER.indexOf(current);
    update.mutate(ORDER[(i + 1) % ORDER.length]);
  };
  return (
    <button
      onClick={next}
      title={`theme: ${current}`}
      className="rounded p-1.5 text-neutral-500 hover:bg-black/5 dark:hover:bg-white/10"
    >
      {ICON[current]}
    </button>
  );
}
```

- [ ] **Step 2: Implement `TitleBar`**

Create `ui/src/components/chrome/TitleBar.tsx`:

```tsx
import { ThemeToggle } from "./ThemeToggle";

export function TitleBar() {
  return (
    <header
      data-tauri-drag-region
      className="flex h-11 shrink-0 items-center justify-between border-b border-black/10 pl-[76px] pr-3 dark:border-white/10"
    >
      <span className="text-xs font-medium text-neutral-600 dark:text-neutral-300">kanban</span>
      <ThemeToggle />
    </header>
  );
}
```

- [ ] **Step 3: Mount the title bar in the layout**

Edit `ui/src/routes/_layout.tsx` — wrap with TitleBar above the existing grid:

```tsx
import { Outlet } from "react-router";
import { useState } from "react";
import { TitleBar } from "@/components/chrome/TitleBar";
import { ProjectList } from "@/components/sidebar/ProjectList";
import { NewProjectDialog } from "@/components/sidebar/NewProjectDialog";

export default function Layout() {
  const [newProjectOpen, setNewProjectOpen] = useState(false);
  return (
    <div className="flex h-screen flex-col bg-white text-neutral-900 dark:bg-neutral-950 dark:text-neutral-100">
      <TitleBar />
      <div className="grid flex-1 grid-cols-[200px_1fr] overflow-hidden">
        <aside className="flex flex-col border-r border-black/10 p-3 text-sm dark:border-white/10">
          <div className="mb-2 px-2 text-[11px] font-semibold uppercase tracking-wider text-neutral-500">Projects</div>
          <ProjectList />
          <button
            onClick={() => setNewProjectOpen(true)}
            className="mt-3 rounded-md border border-dashed border-black/15 px-3 py-1.5 text-xs hover:bg-black/5 dark:border-white/15 dark:hover:bg-white/10"
          >
            + New project
          </button>
          <NewProjectDialog open={newProjectOpen} onOpenChange={setNewProjectOpen} />
        </aside>
        <main className="overflow-hidden">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Verify**

Run: `pnpm --dir ui typecheck`
Expected: PASS.

Run: `pnpm --dir ui test:unit`
Expected: PASS.

Run: `pnpm --dir ui dev` and visit http://localhost:1420 — confirm the title bar shows the theme toggle, clicking cycles `Monitor → Sun → Moon`. Ctrl-C.

- [ ] **Step 5: Commit**

```bash
git add ui/src/components/chrome ui/src/routes/_layout.tsx
git commit -m "feat(ui): add TitleBar with ThemeToggle"
```

---

## Phase 7 — E2E harness

### Task 26: `tauri-driver` smoke harness (with documented fallback)

**Files:**
- Modify: `ui/playwright.config.ts`
- Create: `ui/e2e/launch.ts`
- Create: `ui/e2e/smoke-stub.spec.ts`

This task is **time-boxed**. If you spend more than 2 hours fighting `tauri-driver` setup on macOS, **abort this task**, delete what you've started, and file a Spec #3 prerequisite issue ("E2E via tauri-driver: harness setup"). The fall-back is to ship Spec #2 with Vitest+RTL coverage only and add E2E in Spec #3. The acceptance gate (Task 32) accepts either path.

- [ ] **Step 1: Install `tauri-driver`**

Run:
```bash
cargo install tauri-driver
which tauri-driver
```
Expected: binary path printed.

- [ ] **Step 2: Build the Tauri app in debug mode**

Run: `cargo build -p kanban-tauri`
Expected: success. The binary lives at `target/debug/kanban-tauri`.

- [ ] **Step 3: Add a Playwright launcher helper**

Create `ui/e2e/launch.ts`:

```ts
import { spawn, type ChildProcess } from "node:child_process";
import { test as base, expect } from "@playwright/test";

let driver: ChildProcess | null = null;

export const test = base.extend<{ tauriApp: void }>({
  tauriApp: [
    async ({}, use) => {
      driver = spawn("tauri-driver", ["--port", "4444"], { stdio: "inherit" });
      // give the driver a moment to bind
      await new Promise((r) => setTimeout(r, 500));
      await use();
      driver?.kill("SIGTERM");
    },
    { auto: true },
  ],
});

export { expect };
```

If `tauri-driver` requires more elaborate spawn args (binary path, devtools port), update accordingly per your installed version's docs.

- [ ] **Step 4: Add a stub smoke test**

Create `ui/e2e/smoke-stub.spec.ts`:

```ts
import { test, expect } from "./launch";

test("driver spawns and the app boots", async ({ page }) => {
  await page.goto("about:blank");
  expect(page).toBeTruthy();
});
```

This is a placeholder. Real flows ship in Tasks 27–28. The point of this task is verifying the driver harness comes up at all.

- [ ] **Step 5: Run the stub**

Run: `pnpm --dir ui exec playwright test e2e/smoke-stub.spec.ts`
Expected: PASS.

If FAIL: bail out per the time-box. Delete `ui/e2e/smoke-stub.spec.ts`, `ui/e2e/launch.ts`, drop `test:e2e` from the CI workflow in Task 29, and file Spec #3 prerequisite "E2E via tauri-driver: harness setup."

- [ ] **Step 6: Commit (only if step 5 passes)**

```bash
git add ui/e2e ui/playwright.config.ts
git commit -m "test(ui): bring up tauri-driver E2E harness with smoke stub"
```

---

## Phase 8 — Real E2E

### Task 27: Happy-path smoke E2E

**Files:**
- Create: `ui/e2e/happy-path.spec.ts`

If Task 26 was aborted, **skip this task** and go to Task 29.

- [ ] **Step 1: Implement the happy-path E2E**

Create `ui/e2e/happy-path.spec.ts`:

```ts
import { test, expect } from "./launch";

test("create project → create issue → drag to next status → open detail → edit title", async ({ page }) => {
  await page.goto("http://localhost:1420");

  await page.getByRole("button", { name: "+ New project" }).click();
  await page.getByLabel(/name/i).fill("Auth Service");
  await page.getByLabel(/prefix/i).fill("AUTH");
  await page.getByRole("button", { name: "Create" }).click();

  await page.getByRole("link", { name: /AUTH/ }).click();

  // First column should now be visible — find its add button
  const firstColumnAdd = page.getByRole("button", { name: /\+ Add issue/ }).first();
  await firstColumnAdd.click();
  await page.getByPlaceholder(/new issue/i).fill("Add OAuth login");
  await page.keyboard.press("Enter");

  await expect(page.getByText("Add OAuth login")).toBeVisible();

  // Click the card to open the detail panel
  await page.getByText("Add OAuth login").click();
  await expect(page.getByText(/AUTH-1$/)).toBeVisible();

  // Edit title
  await page.getByText("Add OAuth login").click();
  await page.getByDisplayValue("Add OAuth login").fill("Add OAuth (PKCE)");
  await page.keyboard.press("Enter");
  await expect(page.getByText("Add OAuth (PKCE)")).toBeVisible();
});
```

- [ ] **Step 2: Run**

Pre-req — clear the test DB so the run is hermetic:
```bash
rm -f ~/.kanban/data.db
```

Run: `pnpm --dir ui exec playwright test e2e/happy-path.spec.ts`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add ui/e2e/happy-path.spec.ts
git commit -m "test(ui): add happy-path E2E covering create→issue→edit"
```

---

### Task 28: Theme persistence E2E

**Files:**
- Create: `ui/e2e/theme-persistence.spec.ts`

If Task 26 was aborted, **skip this task**.

- [ ] **Step 1: Implement**

Create `ui/e2e/theme-persistence.spec.ts`:

```ts
import { test, expect } from "./launch";

test("toggling theme to dark survives a relaunch", async ({ page }) => {
  await page.goto("http://localhost:1420");

  // Cycle: system → light → dark
  await page.getByTitle(/theme: system/).click();
  await page.getByTitle(/theme: light/).click();
  await expect(page.locator("html")).toHaveClass(/dark/);

  // Reload simulates a relaunch (the workspace_settings row persists)
  await page.reload();
  await expect(page.locator("html")).toHaveClass(/dark/);
});
```

- [ ] **Step 2: Run**

Run: `pnpm --dir ui exec playwright test e2e/theme-persistence.spec.ts`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add ui/e2e/theme-persistence.spec.ts
git commit -m "test(ui): add theme persistence E2E"
```

---

## Phase 9 — CI / release / docs

### Task 29: Extend CI workflow

**Files:**
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Read current CI**

Run: `cat .github/workflows/ci.yml`
Note the existing job structure (Spec #1 added: fmt-check, clippy, cargo test).

- [ ] **Step 2: Replace with the extended workflow**

Write `.github/workflows/ci.yml` (preserving the existing Rust gate jobs, adding the UI jobs):

```yaml
name: CI

on:
  push:
    branches: [main, dev]
  pull_request:

jobs:
  rust:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: rustfmt, clippy }
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace
      - run: cargo build -p kanban-tauri

  ui:
    runs-on: macos-latest
    needs: rust
    defaults: { run: { working-directory: ui } }
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with: { version: 9 }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: pnpm, cache-dependency-path: ui/pnpm-lock.yaml }
      - run: pnpm install --frozen-lockfile
      - run: pnpm typecheck
      - run: pnpm lint
      - run: pnpm test:unit

  tauri-build:
    runs-on: macos-latest
    needs: [rust, ui]
    defaults: { run: { working-directory: ui } }
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: pnpm/action-setup@v4
        with: { version: 9 }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: pnpm, cache-dependency-path: ui/pnpm-lock.yaml }
      - run: pnpm install --frozen-lockfile
      - run: pnpm tauri build --debug --bundles app

  e2e:
    if: github.event_name == 'pull_request'
    runs-on: macos-latest
    needs: tauri-build
    defaults: { run: { working-directory: ui } }
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: pnpm/action-setup@v4
        with: { version: 9 }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: pnpm, cache-dependency-path: ui/pnpm-lock.yaml }
      - run: pnpm install --frozen-lockfile
      - run: cargo install tauri-driver
      - run: pnpm exec playwright install --with-deps chromium
      - run: pnpm test:e2e
```

If Task 26 was aborted (no E2E), delete the `e2e:` job entirely.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: extend workflow with pnpm typecheck/lint/Vitest, tauri build, and Playwright E2E"
```

---

### Task 30: Release workflow (macOS)

**Files:**
- Create: `.github/workflows/release.yml`

- [ ] **Step 1: Write the workflow**

```yaml
name: Release

on:
  push:
    tags: ["v2.*"]

jobs:
  bundle-macos:
    runs-on: macos-latest
    defaults: { run: { working-directory: ui } }
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: aarch64-apple-darwin,x86_64-apple-darwin }
      - uses: pnpm/action-setup@v4
        with: { version: 9 }
      - uses: actions/setup-node@v4
        with: { node-version: 20, cache: pnpm, cache-dependency-path: ui/pnpm-lock.yaml }
      - run: pnpm install --frozen-lockfile
      - run: pnpm tauri build --target universal-apple-darwin --bundles dmg,app
      - name: Upload DMG to release
        uses: softprops/action-gh-release@v2
        with:
          files: |
            ../target/universal-apple-darwin/release/bundle/dmg/*.dmg
            ../target/universal-apple-darwin/release/bundle/macos/*.app.tar.gz
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add release workflow producing macOS universal DMG + .app.tar.gz"
```

---

### Task 31: README + DEVELOPMENT.md + CLAUDE.md updates

**Files:**
- Modify: `README.md`
- Modify: `DEVELOPMENT.md`
- Modify: `CLAUDE.md`

- [ ] **Step 1: Append a "Desktop app" section to `README.md`**

Add after the existing CLI quick-start:

```markdown
## Desktop app

Download the latest macOS DMG from [GitHub Releases](https://github.com/akassharjun/kanban/releases). The first release is **unsigned**; on first launch you may need:

```sh
xattr -dr com.apple.quarantine /Applications/kanban.app
```

The desktop app is a Tauri shell over the same `kanban-core` library — projects and issues created from the CLI show up after the GUI window receives focus.

## Development (GUI)

See `DEVELOPMENT.md`.
```

- [ ] **Step 2: Add new sections to `DEVELOPMENT.md`**

Append:

```markdown
## Tauri GUI

```sh
pnpm --dir ui install
pnpm --dir ui dev               # Vite dev server
pnpm --dir ui tauri dev         # Tauri shell pointed at Vite
pnpm --dir ui tauri build       # Production .app + .dmg
```

### Tests

```sh
pnpm --dir ui test:unit         # Vitest + RTL
pnpm --dir ui test:e2e          # Playwright + tauri-driver (requires `cargo install tauri-driver`)
```

### Unsigned builds

The first ship is unsigned. On macOS, a downloaded `.app` may be quarantined:

```sh
xattr -dr com.apple.quarantine /Applications/kanban.app
```

Apple Developer signing lands in Spec #3.
```

- [ ] **Step 3: Add a "Tauri layer" note to `CLAUDE.md`**

After the existing "Architecture invariants" section, append:

```markdown
### Tauri layer (Spec #2 onward)

- The GUI lives in `crates/kanban-tauri` (Rust shell) + `ui/` (Vite/React/Tailwind v4).
- Tauri command handlers wrap `Workspace` calls in `tokio::task::block_in_place` (or `spawn_blocking`) so the sync core never blocks the runtime.
- `apply(Operation)` is the single mutation command. The 14 Operation variants stay the only path to issue/project/label/status writes.
- `workspace_settings` table (migration `0002`) stores app-level prefs (theme); accessed via `Workspace::get_setting`/`set_setting` — a documented exception to the single-mutator invariant. Settings have no undo semantics.
- `tauri-specta` generates `ui/src/data/bindings.ts` at build time. The file is git-tracked.
- The GUI does not auto-refresh on external CLI mutations; refresh on focus is sufficient. SQLite `update_hook` watcher is deferred to Spec #3.
```

- [ ] **Step 4: Commit**

```bash
git add README.md DEVELOPMENT.md CLAUDE.md
git commit -m "docs: README/DEVELOPMENT/CLAUDE updates for Spec #2 GUI shell"
```

---

## Phase 10 — Acceptance gate

### Task 32: Acceptance smoke + tag candidate

- [ ] **Step 1: Wipe local data so the test is hermetic**

```bash
rm -rf ~/.kanban
```

- [ ] **Step 2: Acceptance criterion 1 — bootstrap on empty DB**

Run: `pnpm --dir ui tauri dev`

Expected:
- Window opens.
- Empty state visible: "Select or create a project to begin."
- `~/.kanban/data.db` exists.
- `sqlite3 ~/.kanban/data.db ".schema workspace_settings"` shows the new table.

Ctrl-C the dev process.

- [ ] **Step 3: Acceptance criterion 2 — full CRUD flow**

Re-run `pnpm --dir ui tauri dev` and manually verify:
- "+ New project" creates a project with prefix validation (`lowercase` is rejected, `AUTH` succeeds).
- Clicking the project shows three default status columns.
- "+ Add issue" inline form creates an issue.
- Drag a card to the next column — status updates and card appears in the new column.
- Drag within a column — order persists.
- Click a card — detail panel slides in.
- Click the title — input appears, edit, Enter — title updates.
- Status dropdown — change status, panel closes if status changes (or stays — current spec keeps panel open).
- Edit description — Save persists; refresh shows new content.
- Esc closes panel.

- [ ] **Step 4: Acceptance criterion 3 — theme persistence**

In the same window:
- Cycle theme: system → light → dark. `<html>` flips to `class="dark"`.
- Quit the app fully (`⌘Q`).
- Relaunch with `pnpm --dir ui tauri dev`.
- Window opens with `<html class="dark">` already applied.

- [ ] **Step 5: Acceptance criterion 4 — CLI cross-process**

In another terminal:
```bash
cargo run -p kanban-cli -- project create FOO --name "Foo Service"
```

Switch focus back to the GUI window. The sidebar should now list `FOO` (after a brief refetch).

- [ ] **Step 6: Acceptance criterion 5 — production bundle**

Run: `pnpm --dir ui tauri build --bundles dmg,app`
Expected: `target/release/bundle/dmg/kanban_*.dmg` and `target/release/bundle/macos/kanban.app` produced. Open `kanban.app` (after `xattr -dr com.apple.quarantine`) and verify the empty state on a clean DB.

- [ ] **Step 7: Acceptance criterion 6 — CI green**

Push the branch and open a PR against `dev`. Confirm:
- `rust` job passes (fmt + clippy + tests + cargo build kanban-tauri).
- `ui` job passes (pnpm typecheck + lint + Vitest).
- `tauri-build` job passes.
- `e2e` job passes (or absent if Task 26 was aborted).

- [ ] **Step 8: Tag the alpha**

After merge to `dev`:

```bash
git checkout dev
git pull --ff-only
git tag v2.0.0-alpha.1
git push origin v2.0.0-alpha.1
```

This triggers the release workflow (Task 30) and produces the first downloadable DMG.

---

## Self-Review Notes

**Spec coverage check:** every section of the design doc maps to tasks above:

- "Workspace layout" → Task 1, 2 (Cargo + ui scaffold), 3 (test config)
- "Schema additions" → Task 4
- "Tauri command surface" → Tasks 5–10
- "tauri-specta wiring" → Task 10
- "React app structure / Routing tree / Component map" → Tasks 11, 13, 14, 19, 20, 24, 25
- "Data layer / Query keys / useApply / TS Operation builders" → Tasks 12, 15, 16, 17
- "Board mechanics / sort_key" → Tasks 18, 20, 21
- "Detail panel mechanics / markdown / edit-in-place" → Tasks 22, 23, 24
- "Theme system" → Tasks 9, 11, 25
- "Tauri window + title bar" → Task 1 (config), Task 25 (UI)
- "Multi-process behavior (no watcher)" → covered by Task 16's `refetchOnWindowFocus`
- "Error handling" → Tasks 5 (ApiError), 12 (TS helpers), 17 (mutation toast wiring)
- "Testing" → Tasks 11, 12, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24 (unit/component); 26–28 (E2E)
- "CI workflow" → Task 29
- "Bundling and release" → Task 30
- "Documentation updates" → Task 31
- "Acceptance criteria" → Task 32

**No placeholders:** every code block contains compile-able content. Where the underlying core API surface is uncertain (e.g., exact `query::*` function names), the plan instructs the implementer to `grep` and adjust — not to leave a TBD.

**Type consistency:** `Operation` shape is `{ op, args }` everywhere. `IssueFieldChange` shape is `{ field, value }` with PascalCase variants everywhere. UUIDs are `string` in TS. `useApply(prefix?)` signature is consistent across producers and consumers.
