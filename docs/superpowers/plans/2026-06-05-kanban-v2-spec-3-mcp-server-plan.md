# Kanban v2 — Spec #3 MCP Server Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a `kanban-mcp` stdio MCP server that exposes `kanban-core` to MCP clients (Claude Desktop, Claude Code) as semantic tools — read the board, create/update/move issues, undo/redo — every write going through `Workspace::apply`.

**Architecture:** A new `crates/kanban-mcp` binary built on `rmcp` 1.7 (official Rust MCP SDK, stdio transport). It holds an `Arc<Mutex<Workspace>>` opened via `Workspace::open_default()` (shares `~/.kanban/data.db` with CLI/GUI). Each async `#[tool]` runs the sync core inside `tokio::task::spawn_blocking` (the proven `kanban-tauri` pattern); the mutex guard never crosses `.await`. Tools speak human identifiers (project `prefix`, issue `key`, status/label by name) and resolve them to core UUIDs internally.

**Tech Stack:** Rust 2024, `rmcp` 1.7 (`server,macros,transport-io`), `schemars` 1.x, `tokio`, `serde`/`serde_json`, `tracing`. Tests: `cargo test` (per-tool handler tests on an in-memory `Workspace`; one in-process `tokio::io::duplex` integration test driving `initialize`/`tools/list`/`tools/call`).

**Spec:** `docs/superpowers/specs/2026-06-05-kanban-v2-spec-3-mcp-server-design.md`

---

## Conventions for every task

- Run cargo via `~/.cargo/bin/cargo` (the `cargo` alias may be absent).
- Gates that must stay green: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`.
- The workspace sets `clippy::unwrap_used`/`expect_used`/`panic` to **warn**, and CI runs `-D warnings`. In production code use `?`/`map_err`; in test modules add the same allow header the existing tests use (copy from an existing `crates/kanban-core/tests/*.rs`, e.g. `#![allow(clippy::unwrap_used)]`, and `#[allow(clippy::panic)]` on a test mod that uses `panic!`).
- **No `Co-Authored-By` / AI-attribution lines in commits, ever.** Use the exact commit messages given.

## File structure

```
crates/kanban-mcp/
├── Cargo.toml
└── src/
    ├── main.rs        # tracing→stderr, build KanbanServer, serve(stdio()).waiting()
    ├── server.rs      # KanbanServer: Arc<Mutex<Workspace>>, ToolRouter, blocking() helper, ServerHandler, all #[tool]s
    ├── convert.rs     # output DTOs (ProjectOut/IssueOut/StatusOut/LabelOut) + From<core type>, status-name maps
    ├── inputs.rs      # tool input structs (serde::Deserialize + schemars::JsonSchema)
    └── error.rs       # kanban_core::Error -> rmcp::ErrorData mapping + resolution-miss helpers
crates/kanban-mcp/tests/
    └── stdio_smoke.rs # in-process duplex integration test
crates/kanban-core/src/workspace.rs   # +query_project_by_prefix, +query_issue_by_identifier
crates/kanban-tauri/src/commands.rs   # get_issue_inner switches to the new core helper (debt retirement)
```

---

## Phase 0 — Crate scaffold

### Task 1: Add the `kanban-mcp` crate to the workspace

**Files:**
- Modify: `Cargo.toml` (workspace members + workspace deps)
- Create: `crates/kanban-mcp/Cargo.toml`
- Create: `crates/kanban-mcp/src/main.rs`

- [ ] **Step 1: Add the crate to workspace members + add the new workspace deps**

Edit the root `Cargo.toml`. Add `crates/kanban-mcp` to `members`. Add these to `[workspace.dependencies]` (keep all existing entries unchanged):

```toml
# new for kanban-mcp
rmcp = { version = "1.7", features = ["server", "macros", "transport-io"] }
schemars = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }
```

(`tokio`, `serde`, `serde_json`, `anyhow`, `thiserror`, `chrono`, `uuid` already exist in `[workspace.dependencies]` from Spec #2 — reuse them.)

- [ ] **Step 2: Create `crates/kanban-mcp/Cargo.toml`**

```toml
[package]
name = "kanban-mcp"
version = "0.0.1"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "Stdio MCP server exposing kanban-core to AI assistants."

[lints]
workspace = true

[[bin]]
name = "kanban-mcp"
path = "src/main.rs"

[dependencies]
kanban-core = { path = "../kanban-core" }
rmcp = { workspace = true }
schemars = { workspace = true }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }

[dev-dependencies]
# the `client` feature gives an in-process client for the integration test
rmcp = { version = "1.7", features = ["server", "macros", "transport-io", "client"] }
tempfile = { workspace = true }
```

Note: `tokio` in `[workspace.dependencies]` was added in Spec #2 as `{ version = "1", features = ["rt-multi-thread", "macros", "sync"] }`. `rmcp` stdio also needs `tokio` feature `io-std`. Add `io-std` (and `io-util` for the duplex test) to the workspace `tokio` features: change it to `features = ["rt-multi-thread", "macros", "sync", "io-std", "io-util", "signal"]`. Verify `kanban-tauri` still builds after this (only adds features, so it will).

- [ ] **Step 3: Create a minimal `crates/kanban-mcp/src/main.rs`** (compiles; real server lands in Task 4)

```rust
//! Stdio MCP server for kanban-core. Real implementation lands in later tasks.
fn main() {
    eprintln!("kanban-mcp: not yet implemented");
}
```

- [ ] **Step 4: Verify the workspace resolves and builds**

Run: `~/.cargo/bin/cargo check --workspace`
Expected: success — `rmcp` and friends resolve. If `rmcp = "1.7"` does not resolve to a `1.7.x`, pick the latest published `1.x` and note it (the macro/API in this plan targets `rmcp` 1.x — do NOT use a `0.x` version, the macro shape differs).

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock crates/kanban-mcp
git commit -m "chore(mcp): scaffold kanban-mcp crate in workspace"
```

---

## Phase 1 — Core resolution helpers

### Task 2: `query_project_by_prefix` + `query_issue_by_identifier` (test-first) + retire Tauri scan debt

**Files:**
- Modify: `crates/kanban-core/src/workspace.rs` (add two methods)
- Create: `crates/kanban-core/tests/resolution.rs`
- Modify: `crates/kanban-tauri/src/commands.rs` (`get_issue_inner` uses the new helper)

Context: `projects.prefix` has `idx_projects_prefix` (UNIQUE) and `issues.identifier` has a `UNIQUE` constraint — both lookups are indexed, no migration needed. The existing store layer is `crate::store::read::{projects,issues}`; the existing `Workspace::query_project_by_id` calls `crate::store::read::projects::by_id(&self.conn, id)`. Mirror that style.

- [ ] **Step 1: Write the failing test** `crates/kanban-core/tests/resolution.rs`

Copy the clippy-allow header from an existing file in `crates/kanban-core/tests/` (e.g. `#![allow(clippy::unwrap_used)]`). Use the real `Operation`/`CreateProject`/`CreateIssue` shapes (5-field CreateProject; CreateIssue needs id, project_id, title, description, status_id, priority, due_date, label_ids). Seed via `Workspace::apply`.

```rust
#![allow(clippy::unwrap_used)]
use kanban_core::operation::{CreateProject, Operation};
use kanban_core::Workspace;
use uuid::Uuid;

#[test]
fn project_by_prefix_hit_and_miss() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let id = Uuid::now_v7();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name: "Auth Service".into(),
        prefix: "AUTH".into(),
        description: None,
        icon: None,
    }))
    .unwrap();

    let hit = ws.query_project_by_prefix("AUTH").unwrap();
    assert_eq!(hit.map(|p| p.id), Some(id));
    assert!(ws.query_project_by_prefix("NOPE").unwrap().is_none());
}
```

For the issue test, after creating the project, create an issue and assert `query_issue_by_identifier` finds it by its `identifier` (the issue's `identifier` is assigned by core, e.g. `AUTH-1`). To get a valid `status_id`, read the project's statuses with `ws.query_statuses_for_project(project_id)` and use the first. Add:

```rust
#[test]
fn issue_by_identifier_hit_and_miss() {
    use kanban_core::operation::CreateIssue;
    use kanban_core::types::Priority;

    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = Uuid::now_v7();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid,
        name: "Auth".into(),
        prefix: "AUTH".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let status_id = ws.query_statuses_for_project(pid).unwrap()[0].id;

    let iid = Uuid::now_v7();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id: iid,
        project_id: pid,
        title: "Add OAuth".into(),
        description: None,
        status_id,
        priority: Priority::Medium,
        due_date: None,
        label_ids: vec![],
    }))
    .unwrap();

    let issue = ws.query_issue_by_identifier("AUTH-1").unwrap();
    assert_eq!(issue.map(|i| i.id), Some(iid));
    assert!(ws.query_issue_by_identifier("AUTH-999").unwrap().is_none());
}
```

- [ ] **Step 2: Run the test, confirm it FAILS to compile** (`no method named query_project_by_prefix`)

Run: `~/.cargo/bin/cargo test -p kanban-core --test resolution`
Expected: compile failure.

- [ ] **Step 3: Add the two read methods to `impl Workspace`** in `crates/kanban-core/src/workspace.rs`

Add `pub(crate)` store functions if you prefer to match the layering, but the smallest correct change is two inline indexed queries (the settings accessors from Spec #2 set the precedent for inline queries on `Workspace`). Use `rusqlite::OptionalExtension`.

```rust
/// Look up a project by its unique `prefix`. Returns `Ok(None)` if absent.
///
/// # Errors
/// Returns a database error if the read fails.
pub fn query_project_by_prefix(
    &self,
    prefix: &str,
) -> crate::error::Result<Option<crate::types::Project>> {
    use rusqlite::OptionalExtension;
    let id: Option<uuid::Uuid> = self
        .conn
        .query_row(
            "SELECT id FROM projects WHERE prefix = ?1",
            [prefix],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .map(|s| uuid::Uuid::parse_str(&s))
        .transpose()
        .map_err(|e| crate::error::Error::InvalidSnapshot(format!("bad project uuid: {e}")))?;
    match id {
        Some(id) => Ok(Some(self.query_project_by_id(id)?)),
        None => Ok(None),
    }
}

/// Look up an issue by its unique `identifier` (e.g. `AUTH-12`). Returns `Ok(None)` if absent.
///
/// # Errors
/// Returns a database error if the read fails.
pub fn query_issue_by_identifier(
    &self,
    identifier: &str,
) -> crate::error::Result<Option<crate::types::Issue>> {
    use rusqlite::OptionalExtension;
    let id: Option<uuid::Uuid> = self
        .conn
        .query_row(
            "SELECT id FROM issues WHERE identifier = ?1",
            [identifier],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .map(|s| uuid::Uuid::parse_str(&s))
        .transpose()
        .map_err(|e| crate::error::Error::InvalidSnapshot(format!("bad issue uuid: {e}")))?;
    match id {
        Some(id) => Ok(Some(self.query_issue_by_id(id)?)),
        None => Ok(None),
    }
}
```

(If the `projects.id`/`issues.id` columns store UUIDs as TEXT, the parse above is correct. Confirm by checking how `query_project_by_id` reads ids in `crate::store::read::projects`; match its uuid handling. If the store already exposes a `by_prefix`/`by_identifier`, call that instead and delete the inline SQL.)

- [ ] **Step 4: Run the test, confirm PASS**

Run: `~/.cargo/bin/cargo test -p kanban-core --test resolution`
Expected: 2 pass.

- [ ] **Step 5: Retire the Tauri scan debt** — in `crates/kanban-tauri/src/commands.rs`, change `get_issue_inner` to use the new helper:

```rust
pub fn get_issue_inner(ws: &Workspace, key: &str) -> Result<IssueDto, ApiError> {
    match ws.query_issue_by_identifier(key)? {
        Some(issue) => Ok(IssueDto::from(issue)),
        None => Err(ApiError::NotFound {
            resource: "issue".into(),
            key: key.into(),
        }),
    }
}
```

Remove the now-stale `FOLLOWUP(perf)` comment above it. (The `?` works because `From<kanban_core::Error> for ApiError` exists.)

- [ ] **Step 6: Verify nothing regressed**

Run: `~/.cargo/bin/cargo test --workspace` → all pass (kanban-tauri's existing `get_issue` tests still green).
Run: `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` → clean.
Run: `~/.cargo/bin/cargo fmt --all -- --check` → clean (`cargo fmt --all` first if needed).

- [ ] **Step 7: Commit**

```bash
git add crates/kanban-core/src/workspace.rs crates/kanban-core/tests/resolution.rs crates/kanban-tauri/src/commands.rs
git commit -m "feat(core): add prefix/identifier resolution helpers; retire tauri get_issue scan"
```

---

## Phase 2 — Error mapping + server skeleton

### Task 3: `kanban_core::Error` → `rmcp::ErrorData` mapping (test-first)

**Files:**
- Create: `crates/kanban-mcp/src/error.rs`
- Modify: `crates/kanban-mcp/src/main.rs` (declare `mod error;` — temporary until Task 4 restructures)

- [ ] **Step 1: Write `crates/kanban-mcp/src/error.rs`** with the mapping + helpers + inline tests

```rust
//! Map kanban-core errors (and resolution misses) to MCP tool errors.
use rmcp::ErrorData as McpError;

/// Convert a `kanban_core::Error` into an MCP tool error with an LLM-actionable message.
pub fn to_mcp(err: kanban_core::Error) -> McpError {
    use kanban_core::Error;
    match err {
        Error::NotFound { kind, id } => {
            McpError::resource_not_found(format!("{kind} not found: {id}"), None)
        }
        Error::Validation(v) => {
            McpError::invalid_params(format!("{}: {}", v.field, v.reason), None)
        }
        Error::Conflict(msg) => McpError::invalid_request(msg, None),
        Error::Db(e) => McpError::internal_error(format!("db: {e}"), None),
        Error::Io(e) => McpError::internal_error(format!("io: {e}"), None),
        Error::Serde(e) => McpError::internal_error(format!("serde: {e}"), None),
        Error::InvalidSnapshot(s) => McpError::invalid_params(format!("snapshot: {s}"), None),
    }
}

/// A "not found by human identifier" error (project prefix / issue key).
pub fn not_found(resource: &str, key: &str) -> McpError {
    McpError::resource_not_found(format!("{resource} not found: {key}"), None)
}

/// A "name not found within a project" error that lists the valid options.
pub fn unknown_name(kind: &str, name: &str, project: &str, available: &[String]) -> McpError {
    McpError::invalid_params(
        format!(
            "{kind} '{name}' not found in project {project}; available: {}",
            available.join(", ")
        ),
        None,
    )
}

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn validation_maps_to_invalid_params_with_field() {
        let err = kanban_core::Error::Validation(kanban_core::ValidationError {
            field: "prefix".into(),
            reason: "must be 2-8 uppercase letters".into(),
        });
        let mcp = to_mcp(err);
        assert!(mcp.message.contains("prefix"));
        assert!(mcp.message.contains("uppercase"));
    }

    #[test]
    fn unknown_name_lists_available() {
        let mcp = unknown_name("status", "Doing", "AUTH", &["To Do".into(), "Done".into()]);
        assert!(mcp.message.contains("Doing"));
        assert!(mcp.message.contains("To Do, Done"));
    }
}
```

(Confirm `McpError`/`ErrorData` exposes a public `message` field for the asserts; rmcp 1.7's `ErrorData` has `code`, `message: Cow<'static, str>`, `data`. If `message` is not directly accessible, assert via `format!("{mcp:?}")` containing the substrings instead.)

- [ ] **Step 2: Wire `mod error;` into `main.rs`** (replace the Task 1 stub body — keep `fn main` minimal):

```rust
mod error;

fn main() {
    eprintln!("kanban-mcp: not yet implemented");
}
```

(`mod error` is unused by `main` yet; if clippy flags `dead_code` on the helpers, that's fine — they're used from Task 4 onward in the same crate. If `-D warnings` fails on dead_code now, add `#![allow(dead_code)]` at the top of `error.rs` with a comment that Task 4 consumes it, and remove it in Task 4.)

- [ ] **Step 3: Test + gates**

Run: `~/.cargo/bin/cargo test -p kanban-mcp` → 2 pass.
Run: `~/.cargo/bin/cargo clippy -p kanban-mcp --all-targets -- -D warnings` → clean.

- [ ] **Step 4: Commit**

```bash
git add crates/kanban-mcp/src/error.rs crates/kanban-mcp/src/main.rs
git commit -m "feat(mcp): add kanban-core error to MCP error mapping"
```

---

### Task 4: Server skeleton + `list_projects` tool + stdio integration test

This task stands the whole pipeline up end-to-end with one read tool, proven by an in-process integration test.

**Files:**
- Create: `crates/kanban-mcp/src/server.rs`
- Create: `crates/kanban-mcp/src/convert.rs`
- Create: `crates/kanban-mcp/src/inputs.rs` (empty-ish for now; `list_projects` has no params)
- Rewrite: `crates/kanban-mcp/src/main.rs`
- Create: `crates/kanban-mcp/tests/stdio_smoke.rs`

- [ ] **Step 1: Create `crates/kanban-mcp/src/convert.rs`** — output DTOs (clean, human-facing; no UUIDs, no `sort_key`)

```rust
//! Output shapes returned to the MCP client. Human-facing: prefixes/keys/names,
//! never UUIDs or sort keys.
use kanban_core::types::{Issue, Label, Project, Status};
use serde::Serialize;

#[derive(Serialize)]
pub struct ProjectOut {
    pub prefix: String,
    pub name: String,
    pub description: Option<String>,
}
impl From<Project> for ProjectOut {
    fn from(p: Project) -> Self {
        Self { prefix: p.prefix, name: p.name, description: p.description }
    }
}

#[derive(Serialize)]
pub struct StatusOut {
    pub name: String,
    pub category: String,
}
impl From<Status> for StatusOut {
    fn from(s: Status) -> Self {
        Self { name: s.name, category: s.category.as_str().to_owned() }
    }
}

#[derive(Serialize)]
pub struct LabelOut {
    pub name: String,
    pub color: String,
}
impl From<Label> for LabelOut {
    fn from(l: Label) -> Self {
        Self { name: l.name, color: l.color }
    }
}

#[derive(Serialize)]
pub struct IssueOut {
    pub key: String,
    pub title: String,
    pub status: String, // status NAME
    pub priority: String,
    pub due_date: Option<String>,
    pub description: Option<String>,
}

impl IssueOut {
    /// Build from a core `Issue` plus a status-id→name map for the issue's project.
    pub fn from_issue(i: Issue, status_name: &str) -> Self {
        Self {
            key: i.identifier,
            title: i.title,
            status: status_name.to_owned(),
            priority: i.priority.as_str().to_owned(),
            due_date: i.due_date.map(|d| d.to_string()),
            description: i.description,
        }
    }
}
```

- [ ] **Step 2: Create `crates/kanban-mcp/src/inputs.rs`** (input structs land here; start with the ones used so far — none for `list_projects`, so just a module doc for now)

```rust
//! Tool input parameter structs (serde::Deserialize + schemars::JsonSchema).
//! Populated as tools are added.
```

- [ ] **Step 3: Create `crates/kanban-mcp/src/server.rs`** — the server, the `blocking()` helper, `ServerHandler`, and `list_projects`

```rust
use std::sync::{Arc, Mutex};

use rmcp::{
    handler::server::router::tool::ToolRouter,
    model::{CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler,
};

use kanban_core::Workspace;

use crate::convert::ProjectOut;
use crate::error::to_mcp;

#[derive(Clone)]
pub struct KanbanServer {
    workspace: Arc<Mutex<Workspace>>,
    tool_router: ToolRouter<KanbanServer>,
}

impl KanbanServer {
    /// Open the default workspace (`~/.kanban/data.db` or `$KANBAN_DB`).
    ///
    /// # Errors
    /// Returns an error if the workspace cannot be opened.
    pub fn from_default() -> anyhow::Result<Self> {
        let ws = Workspace::open_default()?;
        Ok(Self::with_workspace(ws))
    }

    /// Build a server around an already-open workspace (used by tests).
    #[must_use]
    pub fn with_workspace(ws: Workspace) -> Self {
        Self {
            workspace: Arc::new(Mutex::new(ws)),
            tool_router: Self::tool_router(),
        }
    }

    /// Run a sync closure against the workspace on a blocking thread, never
    /// holding the mutex guard across `.await`. Maps core errors to MCP errors.
    async fn blocking<T, F>(&self, f: F) -> Result<T, McpError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Workspace) -> Result<T, kanban_core::Error> + Send + 'static,
    {
        let ws = Arc::clone(&self.workspace);
        tokio::task::spawn_blocking(move || {
            let mut guard = ws
                .lock()
                .map_err(|_| kanban_core::Error::Conflict("workspace mutex poisoned".into()))?;
            f(&mut guard)
        })
        .await
        .map_err(|e| McpError::internal_error(format!("task join error: {e}"), None))?
        .map_err(to_mcp)
    }
}

/// Serialize a value to a pretty JSON text content block.
pub(crate) fn json_content<T: serde::Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| McpError::internal_error(format!("serialize: {e}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

#[tool_router]
impl KanbanServer {
    #[tool(description = "List all projects (prefix, name, description).")]
    async fn list_projects(&self) -> Result<CallToolResult, McpError> {
        let projects = self.blocking(|ws| ws.query_projects()).await?;
        let out: Vec<ProjectOut> = projects.into_iter().map(ProjectOut::from).collect();
        json_content(&out)
    }
}

#[tool_handler]
impl ServerHandler for KanbanServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::from_build_env())
            .with_protocol_version(ProtocolVersion::V_2024_11_05)
            .with_instructions(
                "Read and manage a local kanban board. Address projects by prefix (e.g. AUTH) \
                 and issues by key (e.g. AUTH-12). Statuses and labels are referenced by name."
                    .to_string(),
            )
    }
}
```

NOTE on the `blocking` signature: `query_projects` takes `&self`, but `apply`/`undo` take `&mut self`; using `&mut Workspace` in the closure covers both. For read-only tools the `&mut` is harmless.

NOTE on rmcp API surface: the imports above (`rmcp::model::{...}`, `rmcp::{tool, tool_handler, tool_router, ErrorData, ServerHandler}`, `rmcp::handler::server::router::tool::ToolRouter`) match rmcp 1.7. If a path differs in the resolved version, fix the import to the real path (the macro and type names — `tool_router`, `tool_handler`, `ToolRouter`, `Parameters`, `CallToolResult`, `Content`, `ServerInfo`, `ServerCapabilities`, `Implementation`, `ProtocolVersion`, `ErrorData`, `ServerHandler`, `ServiceExt`, `transport::stdio` — are stable in 1.x).

- [ ] **Step 4: Rewrite `crates/kanban-mcp/src/main.rs`**

```rust
mod convert;
mod error;
mod inputs;
mod server;

use anyhow::Result;
use rmcp::{transport::stdio, ServiceExt};
use tracing_subscriber::EnvFilter;

use crate::server::KanbanServer;

#[tokio::main]
async fn main() -> Result<()> {
    // stdout is the JSON-RPC channel — all logging goes to stderr.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let server = KanbanServer::from_default()?;
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
```

(`main` now uses `.serve()`/`.waiting()` — real entry point; no `expect`. If clippy still wants an allow somewhere, prefer `?` over `expect`.)

- [ ] **Step 5: Create the in-process integration test** `crates/kanban-mcp/tests/stdio_smoke.rs`

This drives the server with a real in-process client over `tokio::io::duplex`, exercising `initialize` (done by `client.serve`), `tools/list`, and a `list_projects` call against a temp-DB workspace.

```rust
#![allow(clippy::unwrap_used)]
use kanban_core::operation::{CreateProject, Operation};
use kanban_core::Workspace;
use kanban_mcp::server::KanbanServer; // requires a lib target — see note below
use rmcp::{model::CallToolRequestParam, ClientHandler, ServiceExt};
use uuid::Uuid;

#[derive(Default, Clone)]
struct TestClient;
impl ClientHandler for TestClient {}

#[tokio::test]
async fn lists_tools_and_calls_list_projects() {
    // Seed a temp workspace with one project.
    let dir = tempfile::tempdir().unwrap();
    let mut ws = Workspace::open(&dir.path().join("data.db")).unwrap();
    ws.apply(Operation::CreateProject(CreateProject {
        id: Uuid::now_v7(),
        name: "Auth".into(),
        prefix: "AUTH".into(),
        description: None,
        icon: None,
    }))
    .unwrap();

    let server = KanbanServer::with_workspace(ws);
    let (server_t, client_t) = tokio::io::duplex(8192);

    let server_handle = tokio::spawn(async move {
        let svc = server.serve(server_t).await?;
        svc.waiting().await?;
        anyhow::Ok(())
    });

    let client = TestClient.serve(client_t).await.unwrap();

    let tools = client.list_all_tools().await.unwrap();
    assert!(tools.iter().any(|t| t.name == "list_projects"));

    let result = client
        .call_tool(CallToolRequestParam { name: "list_projects".into(), arguments: None })
        .await
        .unwrap();
    let text = format!("{:?}", result.content);
    assert!(text.contains("AUTH"));

    client.cancel().await.unwrap();
    let _ = server_handle.await;
}
```

**Note — lib target:** the integration test imports `kanban_mcp::server::KanbanServer`, which requires `kanban-mcp` to expose a library, not just a binary. Add a `src/lib.rs` exposing the modules and make `main.rs` use the lib:

- Create `crates/kanban-mcp/src/lib.rs`:
  ```rust
  pub mod convert;
  pub mod error;
  pub mod inputs;
  pub mod server;
  ```
- Add to `Cargo.toml`: `[lib]\nname = "kanban_mcp"\npath = "src/lib.rs"` (keep the `[[bin]]`).
- Change `main.rs` to drop the `mod` declarations and use `kanban_mcp::server::KanbanServer` (mirrors how `kanban-tauri` is lib+bin).

- [ ] **Step 6: Run the integration test + gates**

Run: `~/.cargo/bin/cargo test -p kanban-mcp` → the error tests + `lists_tools_and_calls_list_projects` pass.
Run: `~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings` → clean.
Run: `~/.cargo/bin/cargo fmt --all -- --check` → clean.

If `client.serve`/`list_all_tools`/`call_tool`/`CallToolRequestParam`/`result.content` differ in the resolved rmcp version, adjust to the real client API (the in-process duplex approach itself is stable). If the handshake needs an explicit protocol version, pass it; the defaults should suffice.

- [ ] **Step 7: Commit**

```bash
git add crates/kanban-mcp Cargo.toml Cargo.lock
git commit -m "feat(mcp): stdio server skeleton with list_projects and in-process smoke test"
```

---

## Phase 3 — Read tools

For every tool below: add the `#[tool]` method to the `#[tool_router] impl KanbanServer` block in `server.rs`, add its input struct (if any) to `inputs.rs`, extend the integration test (or add a focused `#[tokio::test]`) to call it, run gates, commit. Each tool resolves human identifiers via the Task 2 helpers and returns `json_content(&out)`.

### Task 5: `list_statuses` + `list_labels`

**Files:** Modify `crates/kanban-mcp/src/server.rs`, `crates/kanban-mcp/src/inputs.rs`, `crates/kanban-mcp/tests/stdio_smoke.rs`.

- [ ] **Step 1: Add the input struct** to `inputs.rs`

```rust
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ProjectRef {
    /// Project prefix, e.g. "AUTH".
    pub project: String,
}
```

- [ ] **Step 2: Add the tools** to the `#[tool_router] impl KanbanServer` block

```rust
#[tool(description = "List the status columns of a project, in board order. `project` is the prefix, e.g. AUTH.")]
async fn list_statuses(
    &self,
    rmcp::handler::server::wrapper::Parameters(args): rmcp::handler::server::wrapper::Parameters<crate::inputs::ProjectRef>,
) -> Result<CallToolResult, McpError> {
    let prefix = args.project.clone();
    let statuses = self
        .blocking(move |ws| {
            let Some(p) = ws.query_project_by_prefix(&prefix)? else {
                return Ok(None);
            };
            Ok(Some(ws.query_statuses_for_project(p.id)?))
        })
        .await?;
    let statuses = statuses.ok_or_else(|| crate::error::not_found("project", &args.project))?;
    let out: Vec<crate::convert::StatusOut> =
        statuses.into_iter().map(crate::convert::StatusOut::from).collect();
    crate::server::json_content(&out)
}

#[tool(description = "List the labels of a project. `project` is the prefix, e.g. AUTH.")]
async fn list_labels(
    &self,
    rmcp::handler::server::wrapper::Parameters(args): rmcp::handler::server::wrapper::Parameters<crate::inputs::ProjectRef>,
) -> Result<CallToolResult, McpError> {
    let prefix = args.project.clone();
    let labels = self
        .blocking(move |ws| {
            let Some(p) = ws.query_project_by_prefix(&prefix)? else {
                return Ok(None);
            };
            Ok(Some(ws.query_labels_for_project(p.id)?))
        })
        .await?;
    let labels = labels.ok_or_else(|| crate::error::not_found("project", &args.project))?;
    let out: Vec<crate::convert::LabelOut> =
        labels.into_iter().map(crate::convert::LabelOut::from).collect();
    crate::server::json_content(&out)
}
```

(Tidy the imports — add `use rmcp::handler::server::wrapper::Parameters;` at the top of `server.rs` and use `Parameters(args)` directly instead of the fully-qualified form. Make `json_content` callable — it's already `pub(crate)` in `server.rs`, so call it as `json_content(&out)` from within the same module.)

- [ ] **Step 3: Add a test** (in `stdio_smoke.rs` or a new `#[tokio::test]`): seed the project (a fresh project has 3 default statuses), call `list_statuses` with `{"project":"AUTH"}`, assert the default status names appear. Use `arguments: Some(rmcp::object!({"project":"AUTH"}))`.

- [ ] **Step 4: Gates** (`cargo test -p kanban-mcp`, clippy `--workspace --all-targets -D warnings`, fmt) → green.

- [ ] **Step 5: Commit**
```bash
git add crates/kanban-mcp
git commit -m "feat(mcp): add list_statuses and list_labels tools"
```

### Task 6: `list_issues` + `get_issue` + `search_issues`

**Files:** Modify `server.rs`, `inputs.rs`, `convert.rs` (a status-name map helper), test file.

- [ ] **Step 1: Add a status-name helper** to `convert.rs`

```rust
use std::collections::HashMap;
use uuid::Uuid;

/// Map status id -> status name for a project's statuses.
pub fn status_name_map(statuses: &[Status]) -> HashMap<Uuid, String> {
    statuses.iter().map(|s| (s.id, s.name.clone())).collect()
}
```

- [ ] **Step 2: Add input structs** to `inputs.rs`

```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListIssues {
    /// Project prefix, e.g. "AUTH".
    pub project: String,
    /// Optional status name to filter by (e.g. "In Progress").
    #[serde(default)]
    pub status: Option<String>,
    /// Optional priority filter: one of none, low, medium, high, urgent.
    #[serde(default)]
    pub priority: Option<String>,
    /// Optional full-text search within the project.
    #[serde(default)]
    pub search: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct IssueKey {
    /// Issue key, e.g. "AUTH-12".
    pub key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchIssues {
    /// Full-text query.
    pub query: String,
    /// Optional project prefix to scope the search.
    #[serde(default)]
    pub project: Option<String>,
}
```

- [ ] **Step 3: Implement the tools** in `server.rs`. Use `kanban_core::query::IssueFilter` (has `for_project(uuid)`, `status_ids: Vec<Uuid>`, `priorities: Vec<Priority>`, `search_text: Option<String>`) and `ws.query_issues(filter)` / `ws.search(query, filter)`. For each issue, resolve its `status_id` to a name via the project's `status_name_map`.

`list_issues` — resolve project→id, build a filter (status name → status_id via the project's statuses; `priority` string → `Priority` via `parse`), call `query_issues`, map to `IssueOut` using the status map. On unknown status name return `crate::error::unknown_name(...)`. Concretely:

```rust
#[tool(description = "List issues in a project. Optionally filter by status name, priority, or a search term.")]
async fn list_issues(
    &self,
    Parameters(args): Parameters<crate::inputs::ListIssues>,
) -> Result<CallToolResult, McpError> {
    use kanban_core::query::IssueFilter;
    use kanban_core::types::Priority;

    let project = args.project.clone();
    let result = self
        .blocking(move |ws| {
            let Some(p) = ws.query_project_by_prefix(&project)? else {
                return Ok(Err("no_project"));
            };
            let statuses = ws.query_statuses_for_project(p.id)?;
            let mut filter = IssueFilter::for_project(p.id);
            if let Some(name) = &args.status {
                match statuses.iter().find(|s| s.name == *name) {
                    Some(s) => filter.status_ids = vec![s.id],
                    None => {
                        let names: Vec<String> = statuses.iter().map(|s| s.name.clone()).collect();
                        return Ok(Err_unknown("status", name, &project, names));
                    }
                }
            }
            if let Some(pr) = &args.priority {
                let parsed: Priority = pr
                    .parse()
                    .map_err(|_| kanban_core::Error::Validation(kanban_core::ValidationError {
                        field: "priority".into(),
                        reason: "must be one of none, low, medium, high, urgent".into(),
                    }))?;
                filter.priorities = vec![parsed];
            }
            filter.search_text = args.search.clone();
            let issues = ws.query_issues(filter)?;
            let map = crate::convert::status_name_map(&statuses);
            Ok(Ok((issues, map)))
        })
        .await?;
    // `result` is Result<Ok((issues,map)) | Err(unknown-name/no-project)>; flatten:
    let (issues, map) = match result {
        Ok(pair) => pair,
        Err(e) => return Err(e),
    };
    let out: Vec<crate::convert::IssueOut> = issues
        .into_iter()
        .map(|i| {
            let name = map.get(&i.status_id).cloned().unwrap_or_default();
            crate::convert::IssueOut::from_issue(i, &name)
        })
        .collect();
    json_content(&out)
}
```

The inner closure cannot easily return an `McpError` (it returns `kanban_core::Error`). Resolve this cleanly by having the closure return a domain-level enum instead of smuggling `McpError`. Simplest approach: **do resolution in two `blocking` calls** — first resolve the project + statuses, mapping misses to `McpError` outside the closure; then build the filter and query. Rewrite `list_issues` as:

```rust
#[tool(description = "List issues in a project. Optionally filter by status name, priority, or a search term.")]
async fn list_issues(
    &self,
    Parameters(args): Parameters<crate::inputs::ListIssues>,
) -> Result<CallToolResult, McpError> {
    use kanban_core::query::IssueFilter;
    use kanban_core::types::Priority;

    // 1) resolve project + its statuses
    let prefix = args.project.clone();
    let resolved = self
        .blocking(move |ws| {
            let Some(p) = ws.query_project_by_prefix(&prefix)? else { return Ok(None) };
            let statuses = ws.query_statuses_for_project(p.id)?;
            Ok(Some((p.id, statuses)))
        })
        .await?;
    let (project_id, statuses) =
        resolved.ok_or_else(|| crate::error::not_found("project", &args.project))?;

    // 2) build filter (name/priority resolution -> McpError here)
    let mut filter = IssueFilter::for_project(project_id);
    if let Some(name) = &args.status {
        let s = statuses.iter().find(|s| &s.name == name).ok_or_else(|| {
            crate::error::unknown_name(
                "status",
                name,
                &args.project,
                &statuses.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
            )
        })?;
        filter.status_ids = vec![s.id];
    }
    if let Some(pr) = &args.priority {
        let parsed: Priority = pr.parse().map_err(|_| {
            McpError::invalid_params("priority must be one of none, low, medium, high, urgent", None)
        })?;
        filter.priorities = vec![parsed];
    }
    filter.search_text = args.search.clone();

    // 3) query
    let issues = self.blocking(move |ws| ws.query_issues(filter)).await?;
    let map = crate::convert::status_name_map(&statuses);
    let out: Vec<crate::convert::IssueOut> = issues
        .into_iter()
        .map(|i| {
            let name = map.get(&i.status_id).cloned().unwrap_or_default();
            crate::convert::IssueOut::from_issue(i, &name)
        })
        .collect();
    json_content(&out)
}
```

(Use ONLY this second version — delete the first sketch. The two-phase pattern keeps `McpError` construction out of the `blocking` closure, which can only yield `kanban_core::Error`.)

`get_issue` — resolve the issue by key, then fetch its project's statuses to name the status:

```rust
#[tool(description = "Get one issue by key (e.g. AUTH-12), including its markdown description.")]
async fn get_issue(
    &self,
    Parameters(args): Parameters<crate::inputs::IssueKey>,
) -> Result<CallToolResult, McpError> {
    let key = args.key.clone();
    let resolved = self
        .blocking(move |ws| {
            let Some(issue) = ws.query_issue_by_identifier(&key)? else { return Ok(None) };
            let statuses = ws.query_statuses_for_project(issue.project_id)?;
            Ok(Some((issue, statuses)))
        })
        .await?;
    let (issue, statuses) = resolved.ok_or_else(|| crate::error::not_found("issue", &args.key))?;
    let name = statuses
        .iter()
        .find(|s| s.id == issue.status_id)
        .map(|s| s.name.clone())
        .unwrap_or_default();
    json_content(&crate::convert::IssueOut::from_issue(issue, &name))
}
```

`search_issues` — optional project scope; use `ws.search(query, filter)`:

```rust
#[tool(description = "Full-text search issues by title/description. Optionally scope to a project prefix.")]
async fn search_issues(
    &self,
    Parameters(args): Parameters<crate::inputs::SearchIssues>,
) -> Result<CallToolResult, McpError> {
    use kanban_core::query::IssueFilter;

    let project = args.project.clone();
    let resolved = self
        .blocking(move |ws| {
            let pid = match &project {
                Some(prefix) => match ws.query_project_by_prefix(prefix)? {
                    Some(p) => Some(p.id),
                    None => return Ok(None), // signal unknown project
                },
                None => None,
            };
            // collect every project's statuses so we can name statuses across projects
            let projects = ws.query_projects()?;
            let mut statuses = Vec::new();
            for p in &projects {
                statuses.extend(ws.query_statuses_for_project(p.id)?);
            }
            let filter = match pid {
                Some(id) => IssueFilter::for_project(id),
                None => IssueFilter::default(),
            };
            let issues = ws.search(&args.query, filter)?;
            Ok(Some((issues, statuses)))
        })
        .await?;
    let (issues, statuses) = match resolved {
        Some(v) => v,
        None => return Err(crate::error::not_found("project", args.project.as_deref().unwrap_or(""))),
    };
    let map = crate::convert::status_name_map(&statuses);
    let out: Vec<crate::convert::IssueOut> = issues
        .into_iter()
        .map(|i| {
            let name = map.get(&i.status_id).cloned().unwrap_or_default();
            crate::convert::IssueOut::from_issue(i, &name)
        })
        .collect();
    json_content(&out)
}
```

- [ ] **Step 4: Tests** — add `#[tokio::test]`s (or extend the smoke test): seed a project + an issue (status default), then: `list_issues {project:AUTH}` returns the issue; `get_issue {key:"AUTH-1"}` returns title; `search_issues {query:"<word from title>"}` returns it; unknown status name returns an error mentioning the available statuses.

- [ ] **Step 5: Gates** → green. **Commit**
```bash
git add crates/kanban-mcp
git commit -m "feat(mcp): add list_issues, get_issue, search_issues tools"
```

---

## Phase 4 — Write tools

Every write tool builds an `Operation` and calls `ws.apply(op)` inside `blocking`. Generate UUIDs with `uuid::Uuid::now_v7()`. Resolve names/keys with the Task 2 helpers + status lookups (two-phase, as in Task 6, to keep `McpError` out of the closure).

### Task 7: `create_project`

**Files:** `inputs.rs`, `server.rs`, test.

- [ ] **Step 1: Input**
```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateProjectInput {
    /// Display name, e.g. "Auth Service".
    pub name: String,
    /// Unique prefix: 2-8 uppercase letters, e.g. "AUTH".
    pub prefix: String,
    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
}
```

- [ ] **Step 2: Tool** (in `server.rs`)
```rust
#[tool(description = "Create a new project. prefix must be 2-8 uppercase letters.")]
async fn create_project(
    &self,
    Parameters(args): Parameters<crate::inputs::CreateProjectInput>,
) -> Result<CallToolResult, McpError> {
    use kanban_core::operation::{CreateProject, Operation};
    let id = uuid::Uuid::now_v7();
    let prefix = args.prefix.clone();
    self.blocking(move |ws| {
        ws.apply(Operation::CreateProject(CreateProject {
            id,
            name: args.name,
            prefix: args.prefix,
            description: args.description,
            icon: None,
        }))
        .map(|_| ())
    })
    .await?;
    json_content(&serde_json::json!({ "created": prefix }))
}
```
(Core validates the prefix; an invalid one surfaces as `Validation` → `invalid_params` via `to_mcp`.)

- [ ] **Step 3: Test** — call `create_project {name,prefix:"PAY"}`, then `list_projects` includes PAY; an invalid prefix (`"pay"`) returns an `invalid_params` error mentioning "uppercase".
- [ ] **Step 4: Gates + commit** `feat(mcp): add create_project tool`

### Task 8: `create_issue`

- [ ] **Step 1: Input**
```rust
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateIssueInput {
    /// Project prefix, e.g. "AUTH".
    pub project: String,
    /// Issue title.
    pub title: String,
    /// Optional markdown description.
    #[serde(default)]
    pub description: Option<String>,
    /// Optional status name; defaults to the project's first column.
    #[serde(default)]
    pub status: Option<String>,
    /// Optional priority: none, low, medium, high, urgent. Defaults to medium.
    #[serde(default)]
    pub priority: Option<String>,
    /// Optional due date, YYYY-MM-DD.
    #[serde(default)]
    pub due_date: Option<String>,
}
```

- [ ] **Step 2: Tool** — two-phase: resolve project + statuses (→ McpError on miss), pick the status (named or first), parse priority (default medium), parse due_date (`chrono::NaiveDate::parse_from_str(.., "%Y-%m-%d")` → invalid_params on error), then apply `CreateIssue`. Return the new issue's key (re-read by identifier is unnecessary; instead return `{ "created": "<title>" }`, or query the project's issues to find the new id's identifier — simpler to return a success note and let the caller `list_issues`). Concretely return `json_content(&serde_json::json!({"created": args.title}))` after apply. (Resolving the assigned key would require reading back; out of scope — the caller can `list_issues`.)

Show the full handler:
```rust
#[tool(description = "Create an issue in a project. status defaults to the first column, priority to medium.")]
async fn create_issue(
    &self,
    Parameters(args): Parameters<crate::inputs::CreateIssueInput>,
) -> Result<CallToolResult, McpError> {
    use kanban_core::operation::{CreateIssue, Operation};
    use kanban_core::types::Priority;

    // phase 1: resolve project + statuses
    let prefix = args.project.clone();
    let resolved = self
        .blocking(move |ws| {
            let Some(p) = ws.query_project_by_prefix(&prefix)? else { return Ok(None) };
            let statuses = ws.query_statuses_for_project(p.id)?;
            Ok(Some((p.id, statuses)))
        })
        .await?;
    let (project_id, statuses) =
        resolved.ok_or_else(|| crate::error::not_found("project", &args.project))?;

    // status: named or first
    let status_id = match &args.status {
        Some(name) => {
            statuses
                .iter()
                .find(|s| &s.name == name)
                .ok_or_else(|| {
                    crate::error::unknown_name(
                        "status",
                        name,
                        &args.project,
                        &statuses.iter().map(|s| s.name.clone()).collect::<Vec<_>>(),
                    )
                })?
                .id
        }
        None => statuses.first().map(|s| s.id).ok_or_else(|| {
            McpError::internal_error("project has no statuses", None)
        })?,
    };

    let priority: Priority = match &args.priority {
        Some(p) => p.parse().map_err(|_| {
            McpError::invalid_params("priority must be one of none, low, medium, high, urgent", None)
        })?,
        None => Priority::Medium,
    };
    let due_date = match &args.due_date {
        Some(d) => Some(chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|_| {
            McpError::invalid_params("due_date must be YYYY-MM-DD", None)
        })?),
        None => None,
    };

    let title = args.title.clone();
    self.blocking(move |ws| {
        ws.apply(Operation::CreateIssue(CreateIssue {
            id: uuid::Uuid::now_v7(),
            project_id,
            title: args.title,
            description: args.description,
            status_id,
            priority,
            due_date,
            label_ids: vec![],
        }))
        .map(|_| ())
    })
    .await?;
    json_content(&serde_json::json!({ "created": title }))
}
```

- [ ] **Step 3: Test** — create_issue in AUTH, then list_issues shows it with the default status; bad priority → invalid_params.
- [ ] **Step 4: Gates + commit** `feat(mcp): add create_issue tool`

### Task 9: `update_issue`

- [ ] **Step 1: Input** — `key` + optional `title`/`description`/`priority`/`status`/`due_date` (all `#[serde(default)] Option<...>`).
- [ ] **Step 2: Tool** — resolve the issue by key (→ NotFound) and its project's statuses. For each provided field build an `UpdateIssueField` op with the matching `IssueFieldChange` variant and apply it (multiple ops, one per field):
  - title → `IssueFieldChange::Title(String)`
  - description → `IssueFieldChange::Description(Option<String>)` (allow clearing with an explicit empty/`null`)
  - priority → parse to `Priority`, `IssueFieldChange::Priority(Priority)`
  - status → resolve name → status_id, `IssueFieldChange::Status(Uuid)`
  - due_date → parse `NaiveDate`, `IssueFieldChange::DueDate(Option<NaiveDate>)`
  Apply them in one `blocking` closure (resolve status_id/priority/date BEFORE the closure to keep McpError out). If no fields are provided, return `invalid_params("provide at least one field to update")`.
- [ ] **Step 3: Test** — update title and priority, then get_issue reflects both; updating an unknown key → NotFound.
- [ ] **Step 4: Gates + commit** `feat(mcp): add update_issue tool`

### Task 10: `move_issue`

- [ ] **Step 1: Input** — `key`, optional `status` (name), optional `before` (sibling key), optional `after` (sibling key).
- [ ] **Step 2: Tool** — resolve the issue + project statuses. Determine the target status_id (named, else keep current). If it changes, apply `UpdateIssueField{Status}`. Then compute a new `sort_key` and apply `ReorderIssue`:
  - Read the target column's issues (`query_issues` filtered to the target status_id), excluding the dragged issue, sorted by `sort_key`.
  - Find neighbours: if `before`/`after` given, locate that sibling by identifier and take the keys on either side; else append (neighbour-before = last, neighbour-after = none).
  - `new_sort_key = midpoint(before_key, after_key)` using the same convention as the GUI: both present → average; only-before → before+1024; only-after → after-1024; neither → 1024. Implement a private `fn midpoint(before: Option<f64>, after: Option<f64>) -> f64` in `server.rs`.
  - Apply `ReorderIssue { id, new_sort_key }`. (Core stores `sort_key` losslessly; you pass the plain `f64` — `ReorderIssue.new_sort_key` is a real `f64` in Rust; the `serde_f64::bits` encoding only matters at the JSON boundary, which does not apply here because we call `apply` in-process with a typed `Operation`.)
- [ ] **Step 3: Test** — create two issues in AUTH; `move_issue {key:"AUTH-2", before:"AUTH-1"}` then `list_issues` shows AUTH-2 before AUTH-1; `move_issue {key:"AUTH-1", status:"Done"}` then get_issue shows status Done.
- [ ] **Step 4: Gates + commit** `feat(mcp): add move_issue tool (status change + reorder)`

### Task 11: `undo` + `redo`

- [ ] **Step 1: Tools** (no params)
```rust
#[tool(description = "Undo the most recent change.")]
async fn undo(&self) -> Result<CallToolResult, McpError> {
    self.blocking(|ws| ws.undo().map(|_| ())).await?;
    json_content(&serde_json::json!({ "undone": true }))
}

#[tool(description = "Redo the most recently undone change.")]
async fn redo(&self) -> Result<CallToolResult, McpError> {
    self.blocking(|ws| ws.redo().map(|_| ())).await?;
    json_content(&serde_json::json!({ "redone": true }))
}
```
(`Workspace::undo`/`redo` return `OperationOutcome`; map to `()`. "Nothing to undo" surfaces as `Conflict` → `invalid_request`.)

- [ ] **Step 2: Test** — create_project, undo → list_projects empty, redo → it's back. Undo on empty history → error.
- [ ] **Step 3: Gates + commit** `feat(mcp): add undo and redo tools`

---

## Phase 5 — Docs + acceptance

### Task 12: Docs — README + DEVELOPMENT + CLAUDE

**Files:** Modify `README.md`, `DEVELOPMENT.md`, `CLAUDE.md`.

- [ ] **Step 1: README** — add a short "Use from an AI assistant (MCP)" section: build with `cargo build -p kanban-mcp --release` (binary at `target/release/kanban-mcp`); it serves over stdio and shares `~/.kanban/data.db`.

- [ ] **Step 2: DEVELOPMENT.md** — add an "MCP server (Spec #3)" section with the registration snippets:

Claude Desktop (`~/Library/Application Support/Claude/claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "kanban": {
      "command": "/absolute/path/to/target/release/kanban-mcp",
      "env": { "KANBAN_DB": "/Users/you/.kanban/data.db" }
    }
  }
}
```
Claude Code: `claude mcp add kanban -- /absolute/path/to/target/release/kanban-mcp` (or the equivalent `.mcp.json` entry). Note `KANBAN_DB` is optional (defaults to `~/.kanban/data.db`).

- [ ] **Step 3: CLAUDE.md** — add a short "MCP layer (Spec #3)" note under the Tauri layer section: stdio server in `crates/kanban-mcp` on `rmcp`; shares the workspace DB; reads via `query_*`, writes through `Workspace::apply`; semantic tools resolve prefix/key/status-name to ids; agent orchestration deferred to Spec #4.

- [ ] **Step 4: Commit**
```bash
git add README.md DEVELOPMENT.md CLAUDE.md
git commit -m "docs: document the kanban-mcp server and client registration"
```

### Task 13: Acceptance gate

- [ ] **Step 1: Full gates**
```bash
~/.cargo/bin/cargo fmt --all -- --check
~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings
~/.cargo/bin/cargo test --workspace
```
All green.

- [ ] **Step 2: End-to-end tool sequence test** — add one `#[tokio::test]` in `stdio_smoke.rs` (or a new `tests/e2e_flow.rs`) that drives, over the in-process client, the full loop on a temp DB: `create_project` → `create_issue` → `move_issue` (status change) → `get_issue` (assert new status) → `undo` → `get_issue` (assert reverted). This is the executable form of design acceptance criteria 2–3.

- [ ] **Step 3: Cross-process sanity (manual, documented)** — build the binary, register it with a client per Task 12, ask the assistant to "create a project FOO and an issue in it", then confirm with `target/release/kanban-cli project list` (or `cargo run -p kanban-cli -- project list`) that FOO exists. Record the result in the PR description. (This is manual; not a CI test.)

- [ ] **Step 4: Commit any test additions**
```bash
git add crates/kanban-mcp
git commit -m "test(mcp): add end-to-end create/move/undo flow over stdio"
```

- [ ] **Step 5:** Use **superpowers:finishing-a-development-branch** to open a PR against `dev`.

---

## Self-review notes

**Spec coverage:** every design section maps to a task — crate/stack (Task 1), core resolution helpers + debt retirement (Task 2), error mapping (Task 3), server skeleton + stdio + integration test (Task 4), read tools (Tasks 5–6), write tools create/update/move/undo/redo (Tasks 7–11), docs/config (Task 12), acceptance incl. the create→move→undo flow and cross-process check (Task 13). Non-goals (delete tools, HTTP, orchestration, resources/prompts, new migration) are not present in any task.

**Identifier handling is consistent:** projects by `prefix`, issues by `key`/`identifier`, statuses/labels by `name`; resolution always via the Task 2 helpers + status lookups, with misses mapped to `not_found`/`unknown_name`.

**`McpError` is kept out of `blocking` closures** (which can only yield `kanban_core::Error`): name/priority/date resolution happens in a first `blocking` (project+statuses) then in plain async code before a second `blocking` that applies the op. Tasks 6, 8, 9, 10 all follow this two-phase shape.

**rmcp version caveat:** all macro/type/import paths target `rmcp` 1.x (1.7 verified). If the resolved version differs, the implementer adjusts imports to the real paths — the names (`tool`/`tool_router`/`tool_handler`/`ToolRouter`/`Parameters`/`CallToolResult`/`Content`/`ServerInfo`/`ServerCapabilities`/`ServiceExt`/`transport::stdio`/`ErrorData`) and the duplex test transport are stable across 1.x.
