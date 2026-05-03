# Kanban v2 — Core + CLI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust `kanban-core` library and `kanban-cli` binary that implement projects, issues, labels, default statuses, search, sort, undo/redo, and JSON import/export — every behaviour test-first.

**Architecture:** Two-crate Cargo workspace. `kanban-core` is sync, uses `rusqlite`, hand-rolled migrations, FTS5 for search, and a single public `Workspace::apply(Operation)` mutator enforced by module visibility. `kanban-cli` is the only consumer in v1 — a clap CLI with `insta` snapshot tests.

**Tech Stack:** Rust 2024 edition, `rusqlite` (with `bundled` + `serde_json` features), `serde` + `serde_json`, `uuid` (v7), `chrono`, `thiserror`, `clap` (derive), `assert_cmd`, `insta`, `proptest`, `tempfile`.

**Spec:** `docs/superpowers/specs/2026-05-03-kanban-v2-core-cli-design.md`

---

## Phase Map

| Phase | Tasks | Topic |
|-------|-------|-------|
| 0 | 1 | Workspace bootstrap |
| 1 | 2–5 | Errors, IDs, validators, domain types |
| 2 | 6–7 | SQLite connection + migration runner |
| 3 | 8 | `Workspace::open` |
| 4 | 9 | `operation_log` and `activity_log` writers |
| 5 | 10–15 | Project Operations vertical slice |
| 6 | 16 | `Workspace::undo` / `redo` |
| 7 | 17 | Undo/redo property test |
| 8 | 18–23 | Issue Operations |
| 9 | 24–26 | Label Operations |
| 10 | 27 | Search (FTS5 query) |
| 11 | 28–31 | Snapshot export/import |
| 12 | 32–37 | CLI surface |
| 13 | 38–41 | Smoke, perf, CI, README |

---

## Phase 0 — Workspace bootstrap

### Task 1: Cargo workspace skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `crates/kanban-core/Cargo.toml`
- Create: `crates/kanban-core/src/lib.rs`
- Create: `crates/kanban-cli/Cargo.toml`
- Create: `crates/kanban-cli/src/main.rs`
- Create: `.gitignore`
- Create: `rustfmt.toml`
- Create: `clippy.toml`

- [ ] **Step 1: Write `Cargo.toml` (workspace manifest)**

```toml
[workspace]
resolver = "2"
members = ["crates/kanban-core", "crates/kanban-cli"]

[workspace.package]
edition = "2024"
rust-version = "1.85"
license = "MIT"
authors = ["akassharjun"]

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }
# explicit deny set; broaden later
unwrap_used = "warn"
expect_used = "warn"
panic = "warn"
todo = "deny"
unimplemented = "deny"

[workspace.dependencies]
rusqlite = { version = "0.31", features = ["bundled", "serde_json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v7", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "1"
clap = { version = "4", features = ["derive", "env"] }
anyhow = "1"

# Test-only
assert_cmd = "2"
insta = { version = "1", features = ["yaml", "filters"] }
proptest = "1"
tempfile = "3"
```

- [ ] **Step 2: Write `crates/kanban-core/Cargo.toml`**

```toml
[package]
name = "kanban-core"
version = "0.0.1"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "Core library for the Kanban project management tool."

[lints]
workspace = true

[dependencies]
rusqlite = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
proptest = { workspace = true }
tempfile = { workspace = true }
```

- [ ] **Step 3: Write `crates/kanban-core/src/lib.rs`**

```rust
//! kanban-core: data model, applier, queries, migrations.
```

- [ ] **Step 4: Write `crates/kanban-cli/Cargo.toml`**

```toml
[package]
name = "kanban-cli"
version = "0.0.1"
edition.workspace = true
rust-version.workspace = true
license.workspace = true
authors.workspace = true
description = "Command-line interface for kanban-core."

[[bin]]
name = "kanban"
path = "src/main.rs"

[lints]
workspace = true

[dependencies]
kanban-core = { path = "../kanban-core" }
clap = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }

[dev-dependencies]
assert_cmd = { workspace = true }
insta = { workspace = true }
tempfile = { workspace = true }
```

- [ ] **Step 5: Write `crates/kanban-cli/src/main.rs`**

```rust
fn main() {
    eprintln!("kanban-cli: not yet implemented");
    std::process::exit(1);
}
```

- [ ] **Step 6: Write `.gitignore`**

```
/target
**/*.rs.bk
*.pdb
.DS_Store
.idea/
.vscode/
.kanban-test/
```

- [ ] **Step 7: Write `rustfmt.toml`**

```
edition = "2024"
max_width = 100
use_field_init_shorthand = true
```

- [ ] **Step 8: Write `clippy.toml`**

```
msrv = "1.85"
```

- [ ] **Step 9: Verify workspace compiles**

Run: `cargo build`
Expected: success, two crates compile.

Run: `cargo test`
Expected: success, 0 tests, 0 failures.

Run: `cargo clippy --workspace --all-targets -- -D warnings`
Expected: success, no warnings.

Run: `cargo fmt --all -- --check`
Expected: success, no diffs.

- [ ] **Step 10: Commit**

```bash
git add Cargo.toml Cargo.lock crates/ .gitignore rustfmt.toml clippy.toml
git commit -m "chore: bootstrap cargo workspace with kanban-core and kanban-cli"
```

---

## Phase 1 — Primitives

### Task 2: Error type and Result alias

**Files:**
- Create: `crates/kanban-core/src/error.rs`
- Modify: `crates/kanban-core/src/lib.rs`
- Test: `crates/kanban-core/src/error.rs` (inline `#[cfg(test)]` module)

- [ ] **Step 1: Write the failing test**

Append to `crates/kanban-core/src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_error_displays_field_and_reason() {
        let err = Error::Validation(ValidationError {
            field: "prefix".to_string(),
            reason: "must be 3+ uppercase letters".to_string(),
        });
        let msg = err.to_string();
        assert!(msg.contains("prefix"), "got: {msg}");
        assert!(msg.contains("must be 3+ uppercase letters"), "got: {msg}");
    }

    #[test]
    fn not_found_displays_kind_and_id() {
        let err = Error::NotFound {
            kind: EntityKind::Issue,
            id: "KAN-42".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Issue"), "got: {msg}");
        assert!(msg.contains("KAN-42"), "got: {msg}");
    }
}
```

- [ ] **Step 2: Run the test (it should fail to compile)**

Run: `cargo test -p kanban-core`
Expected: FAIL — `Error`, `ValidationError`, `EntityKind` not defined.

- [ ] **Step 3: Implement `Error`, `ValidationError`, `EntityKind`, `Result`**

Replace the contents of `crates/kanban-core/src/error.rs` (above the `#[cfg(test)]` block) with:

```rust
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("validation failed: {0}")]
    Validation(ValidationError),

    #[error("{kind} not found: {id}")]
    NotFound { kind: EntityKind, id: String },

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid snapshot: {0}")]
    InvalidSnapshot(String),

    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: String,
    pub reason: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.reason)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Project,
    Issue,
    Label,
    Status,
}

impl std::fmt::Display for EntityKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityKind::Project => f.write_str("Project"),
            EntityKind::Issue => f.write_str("Issue"),
            EntityKind::Label => f.write_str("Label"),
            EntityKind::Status => f.write_str("Status"),
        }
    }
}
```

- [ ] **Step 4: Wire the module into `lib.rs`**

Replace `crates/kanban-core/src/lib.rs` with:

```rust
//! kanban-core: data model, applier, queries, migrations.

pub mod error;

pub use error::{EntityKind, Error, Result, ValidationError};
```

- [ ] **Step 5: Run the tests**

Run: `cargo test -p kanban-core`
Expected: PASS, 2 tests.

- [ ] **Step 6: Commit**

```bash
git add crates/kanban-core/src/error.rs crates/kanban-core/src/lib.rs
git commit -m "feat(core): add Error, Result, EntityKind, ValidationError"
```

---

### Task 3: ID generation and clock abstraction

**Files:**
- Create: `crates/kanban-core/src/ids.rs`
- Create: `crates/kanban-core/src/time.rs`
- Modify: `crates/kanban-core/src/lib.rs`

- [ ] **Step 1: Write the failing test for `ids`**

Create `crates/kanban-core/src/ids.rs`:

```rust
use uuid::Uuid;

#[must_use]
pub fn new_id() -> Uuid {
    Uuid::now_v7()
}

#[must_use]
pub fn format_identifier(prefix: &str, seq: i64) -> String {
    format!("{prefix}-{seq}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_id_returns_v7_uuids_that_sort_chronologically() {
        let a = new_id();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = new_id();
        assert_eq!(a.get_version_num(), 7);
        assert_eq!(b.get_version_num(), 7);
        assert!(a < b, "v7 ids should sort by creation time: {a} < {b}");
    }

    #[test]
    fn format_identifier_joins_prefix_and_seq() {
        assert_eq!(format_identifier("KAN", 42), "KAN-42");
        assert_eq!(format_identifier("AUTH", 1), "AUTH-1");
    }
}
```

- [ ] **Step 2: Write the failing test for `time`**

Create `crates/kanban-core/src/time.rs`:

```rust
use chrono::{DateTime, Utc};
use std::sync::Arc;

/// Source of "now" — pluggable for deterministic tests.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[derive(Debug)]
pub struct FixedClock {
    instant: DateTime<Utc>,
}

impl FixedClock {
    #[must_use]
    pub fn new(instant: DateTime<Utc>) -> Self {
        Self { instant }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        self.instant
    }
}

#[must_use]
pub fn system_clock() -> Arc<dyn Clock> {
    Arc::new(SystemClock)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn fixed_clock_returns_the_same_instant() {
        let t = Utc.with_ymd_and_hms(2026, 5, 3, 12, 0, 0).unwrap();
        let clock = FixedClock::new(t);
        assert_eq!(clock.now(), t);
        assert_eq!(clock.now(), t);
    }

    #[test]
    fn system_clock_returns_recent_instant() {
        let clock = SystemClock;
        let now = clock.now();
        let delta = (Utc::now() - now).num_seconds().abs();
        assert!(delta < 5, "system clock should be near Utc::now: {delta}s drift");
    }
}
```

- [ ] **Step 3: Wire modules into `lib.rs`**

Replace `crates/kanban-core/src/lib.rs`:

```rust
//! kanban-core: data model, applier, queries, migrations.

pub mod error;
pub mod ids;
pub mod time;

pub use error::{EntityKind, Error, Result, ValidationError};
pub use ids::{format_identifier, new_id};
pub use time::{Clock, FixedClock, SystemClock, system_clock};
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kanban-core`
Expected: PASS, 6 tests total.

- [ ] **Step 5: Commit**

```bash
git add crates/kanban-core/src/ids.rs crates/kanban-core/src/time.rs crates/kanban-core/src/lib.rs
git commit -m "feat(core): add UUIDv7 id generation, identifier formatting, Clock abstraction"
```

---

### Task 4: Validators

**Files:**
- Create: `crates/kanban-core/src/validate.rs`
- Modify: `crates/kanban-core/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

Create `crates/kanban-core/src/validate.rs`:

```rust
use crate::error::{Error, Result, ValidationError};

/// Project prefix: 3–6 uppercase ASCII letters, e.g., "KAN", "AUTH".
pub fn project_prefix(value: &str) -> Result<()> {
    if value.is_empty() {
        return Err(invalid("prefix", "must not be empty"));
    }
    if !(3..=6).contains(&value.len()) {
        return Err(invalid("prefix", "must be 3–6 characters"));
    }
    if !value.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(invalid("prefix", "must be uppercase ASCII letters only"));
    }
    Ok(())
}

/// Non-empty trimmed string field. Returns the trimmed value on success.
pub fn nonempty_field<'a>(field: &str, value: &'a str) -> Result<&'a str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(invalid(field, "must not be empty"));
    }
    Ok(trimmed)
}

/// Hex color: `#RRGGBB`.
pub fn hex_color(value: &str) -> Result<()> {
    if value.len() != 7 || !value.starts_with('#') {
        return Err(invalid("color", "must be a 7-char hex string starting with #"));
    }
    if !value[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(invalid("color", "must contain only hex digits"));
    }
    Ok(())
}

fn invalid(field: &str, reason: &str) -> Error {
    Error::Validation(ValidationError {
        field: field.to_string(),
        reason: reason.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_prefix_accepts_valid() {
        assert!(project_prefix("KAN").is_ok());
        assert!(project_prefix("AUTH").is_ok());
        assert!(project_prefix("ABCDEF").is_ok());
    }

    #[test]
    fn project_prefix_rejects_invalid() {
        assert!(project_prefix("").is_err());
        assert!(project_prefix("ab").is_err());
        assert!(project_prefix("kan").is_err());
        assert!(project_prefix("KAN1").is_err());
        assert!(project_prefix("TOOOLONG").is_err());
    }

    #[test]
    fn nonempty_field_trims() {
        assert_eq!(nonempty_field("name", "  hi  ").unwrap(), "hi");
    }

    #[test]
    fn nonempty_field_rejects_empty_or_whitespace() {
        assert!(nonempty_field("name", "").is_err());
        assert!(nonempty_field("name", "   ").is_err());
    }

    #[test]
    fn hex_color_accepts_valid() {
        assert!(hex_color("#000000").is_ok());
        assert!(hex_color("#FFFFFF").is_ok());
        assert!(hex_color("#3b82f6").is_ok());
    }

    #[test]
    fn hex_color_rejects_invalid() {
        assert!(hex_color("000000").is_err());
        assert!(hex_color("#00").is_err());
        assert!(hex_color("#GGGGGG").is_err());
    }
}
```

- [ ] **Step 2: Wire into `lib.rs`**

Add to `crates/kanban-core/src/lib.rs`:

```rust
pub mod validate;
```

(Keep all existing `pub mod` and `pub use` lines.)

- [ ] **Step 3: Run the tests**

Run: `cargo test -p kanban-core`
Expected: PASS, 12 tests total.

- [ ] **Step 4: Commit**

```bash
git add crates/kanban-core/src/validate.rs crates/kanban-core/src/lib.rs
git commit -m "feat(core): add validators for prefix, non-empty fields, hex color"
```

---

### Task 5: Domain types

**Files:**
- Create: `crates/kanban-core/src/types.rs`
- Modify: `crates/kanban-core/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

Create `crates/kanban-core/src/types.rs`:

```rust
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    None,
    Urgent,
    High,
    Medium,
    Low,
}

impl Priority {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Priority::None => "none",
            Priority::Urgent => "urgent",
            Priority::High => "high",
            Priority::Medium => "medium",
            Priority::Low => "low",
        }
    }

    /// Sort key (lower = higher priority). `None` sorts last.
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Priority::Urgent => 0,
            Priority::High => 1,
            Priority::Medium => 2,
            Priority::Low => 3,
            Priority::None => 4,
        }
    }
}

impl std::str::FromStr for Priority {
    type Err = crate::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Priority::None),
            "urgent" => Ok(Priority::Urgent),
            "high" => Ok(Priority::High),
            "medium" => Ok(Priority::Medium),
            "low" => Ok(Priority::Low),
            other => Err(crate::error::Error::Validation(crate::error::ValidationError {
                field: "priority".into(),
                reason: format!("unknown value '{other}'"),
            })),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Paused,
    Completed,
    Archived,
}

impl ProjectStatus {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ProjectStatus::Active => "active",
            ProjectStatus::Paused => "paused",
            ProjectStatus::Completed => "completed",
            ProjectStatus::Archived => "archived",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatusCategory {
    Unstarted,
    Started,
    Blocked,
    Completed,
    Discarded,
}

impl StatusCategory {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            StatusCategory::Unstarted => "unstarted",
            StatusCategory::Started => "started",
            StatusCategory::Blocked => "blocked",
            StatusCategory::Completed => "completed",
            StatusCategory::Discarded => "discarded",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub status: ProjectStatus,
    pub next_seq: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub category: StatusCategory,
    pub color: String,
    pub position: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    pub id: Uuid,
    pub project_id: Uuid,
    pub seq: i64,
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub status_id: Uuid,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
    pub sort_key: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub color: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn priority_round_trips_via_str() {
        for p in [
            Priority::None,
            Priority::Urgent,
            Priority::High,
            Priority::Medium,
            Priority::Low,
        ] {
            let s = p.as_str();
            let parsed: Priority = s.parse().unwrap();
            assert_eq!(parsed, p);
        }
    }

    #[test]
    fn priority_rank_orders_correctly() {
        assert!(Priority::Urgent.rank() < Priority::High.rank());
        assert!(Priority::High.rank() < Priority::Medium.rank());
        assert!(Priority::Medium.rank() < Priority::Low.rank());
        assert!(Priority::Low.rank() < Priority::None.rank());
    }

    #[test]
    fn priority_from_unknown_str_errors() {
        let err: Result<Priority, _> = "wat".parse();
        assert!(err.is_err());
    }

    #[test]
    fn project_status_as_str_round_trips_via_serde() {
        let s = serde_json::to_string(&ProjectStatus::Active).unwrap();
        assert_eq!(s, "\"active\"");
        let p: ProjectStatus = serde_json::from_str("\"paused\"").unwrap();
        assert_eq!(p, ProjectStatus::Paused);
    }
}
```

- [ ] **Step 2: Wire into `lib.rs`**

Add to `crates/kanban-core/src/lib.rs`:

```rust
pub mod types;

pub use types::{Issue, Label, Priority, Project, ProjectStatus, Status, StatusCategory};
```

- [ ] **Step 3: Run the tests**

Run: `cargo test -p kanban-core`
Expected: PASS, 16 tests total.

- [ ] **Step 4: Commit**

```bash
git add crates/kanban-core/src/types.rs crates/kanban-core/src/lib.rs
git commit -m "feat(core): add Priority, ProjectStatus, StatusCategory and entity types"
```

---

## Phase 2 — Storage foundations

### Task 6: SQLite connection and pragmas

**Files:**
- Create: `crates/kanban-core/src/store/mod.rs`
- Create: `crates/kanban-core/src/store/connection.rs`
- Modify: `crates/kanban-core/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

Create `crates/kanban-core/src/store/connection.rs`:

```rust
use crate::error::Result;
use rusqlite::Connection;
use std::path::Path;

pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    apply_pragmas(&conn)?;
    Ok(conn)
}

pub fn open_file(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    apply_pragmas(&conn)?;
    Ok(conn)
}

fn apply_pragmas(conn: &Connection) -> Result<()> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "temp_store", "MEMORY")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pragma_value(conn: &Connection, name: &str) -> String {
        conn.query_row(&format!("PRAGMA {name}"), [], |r| r.get::<_, String>(0))
            .unwrap_or_else(|_| {
                conn.query_row(&format!("PRAGMA {name}"), [], |r| {
                    Ok(r.get::<_, i64>(0)?.to_string())
                })
                .unwrap()
            })
    }

    #[test]
    fn in_memory_applies_required_pragmas() {
        let conn = open_in_memory().unwrap();
        assert_eq!(pragma_value(&conn, "synchronous"), "1"); // NORMAL
        assert_eq!(pragma_value(&conn, "foreign_keys"), "1");
    }

    #[test]
    fn file_applies_wal_mode() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.db");
        let conn = open_file(&path).unwrap();
        let mode = pragma_value(&conn, "journal_mode");
        assert_eq!(mode.to_lowercase(), "wal");
    }

    #[test]
    fn open_file_creates_parent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/inner/data.db");
        let _conn = open_file(&path).unwrap();
        assert!(path.exists());
    }
}
```

- [ ] **Step 2: Create `crates/kanban-core/src/store/mod.rs`**

```rust
pub(crate) mod connection;
```

- [ ] **Step 3: Wire into `lib.rs`**

Add (keep existing entries):

```rust
mod store;
```

Note: `store` is **private** to `kanban-core`. It is NOT `pub mod`. This is the architecture-rot fix from the spec — only `Workspace::apply` is the public mutator, so the entire `store` module is hidden.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kanban-core`
Expected: PASS, 19 tests total.

- [ ] **Step 5: Commit**

```bash
git add crates/kanban-core/src/store/ crates/kanban-core/src/lib.rs
git commit -m "feat(core): add SQLite connection helpers with WAL pragmas"
```

---

### Task 7: Migration runner and `0001_init.sql`

**Files:**
- Create: `crates/kanban-core/migrations/0001_init.sql`
- Create: `crates/kanban-core/src/store/migrations.rs`
- Modify: `crates/kanban-core/src/store/mod.rs`

- [ ] **Step 1: Write `0001_init.sql`**

Create `crates/kanban-core/migrations/0001_init.sql`:

```sql
-- v1 initial schema. After release this file is FROZEN. Schema changes go in 0002+.

CREATE TABLE schema_migrations (
  version    INTEGER PRIMARY KEY,
  applied_at TEXT    NOT NULL
) STRICT;

CREATE TABLE projects (
  id          TEXT PRIMARY KEY,
  name        TEXT NOT NULL,
  prefix      TEXT NOT NULL,
  description TEXT,
  icon        TEXT,
  status      TEXT NOT NULL CHECK (status IN ('active','paused','completed','archived')),
  next_seq    INTEGER NOT NULL DEFAULT 1,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
) STRICT;
CREATE UNIQUE INDEX idx_projects_prefix ON projects(prefix);

CREATE TABLE statuses (
  id         TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name       TEXT NOT NULL,
  category   TEXT NOT NULL CHECK (category IN ('unstarted','started','blocked','completed','discarded')),
  color      TEXT NOT NULL,
  position   INTEGER NOT NULL,
  UNIQUE (project_id, name)
) STRICT;
CREATE INDEX idx_statuses_project ON statuses(project_id, position);

CREATE TABLE issues (
  id           TEXT PRIMARY KEY,
  project_id   TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  seq          INTEGER NOT NULL,
  identifier   TEXT NOT NULL,
  title        TEXT NOT NULL,
  description  TEXT,
  status_id    TEXT NOT NULL REFERENCES statuses(id),
  priority     TEXT NOT NULL DEFAULT 'none'
                 CHECK (priority IN ('none','urgent','high','medium','low')),
  due_date     TEXT,
  sort_key     REAL NOT NULL,
  created_at   TEXT NOT NULL,
  updated_at   TEXT NOT NULL,
  UNIQUE (project_id, seq),
  UNIQUE (identifier)
) STRICT;
CREATE INDEX idx_issues_project ON issues(project_id);
CREATE INDEX idx_issues_status ON issues(status_id);
CREATE INDEX idx_issues_sort ON issues(project_id, status_id, sort_key);

CREATE TABLE labels (
  id         TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name       TEXT NOT NULL,
  color      TEXT NOT NULL,
  UNIQUE (project_id, name)
) STRICT;

CREATE TABLE issue_labels (
  issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
  label_id TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
  PRIMARY KEY (issue_id, label_id)
) STRICT;

CREATE TABLE operation_log (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  op_type         TEXT NOT NULL,
  payload         TEXT NOT NULL,
  inverse_payload TEXT NOT NULL,
  applied_at      TEXT NOT NULL,
  undone          INTEGER NOT NULL DEFAULT 0 CHECK (undone IN (0,1))
) STRICT;
CREATE INDEX idx_operation_log_undone ON operation_log(undone, id);

CREATE TABLE activity_log (
  id        INTEGER PRIMARY KEY AUTOINCREMENT,
  op_id     INTEGER NOT NULL REFERENCES operation_log(id),
  issue_id  TEXT REFERENCES issues(id) ON DELETE SET NULL,
  field     TEXT NOT NULL,
  old_value TEXT,
  new_value TEXT,
  at        TEXT NOT NULL
) STRICT;
CREATE INDEX idx_activity_issue ON activity_log(issue_id);

CREATE VIRTUAL TABLE issue_search USING fts5 (
  title,
  description,
  content='issues',
  content_rowid='rowid',
  tokenize = 'porter unicode61'
);

-- Keep FTS in sync with issues
CREATE TRIGGER issues_ai AFTER INSERT ON issues BEGIN
  INSERT INTO issue_search(rowid, title, description) VALUES (new.rowid, new.title, new.description);
END;
CREATE TRIGGER issues_ad AFTER DELETE ON issues BEGIN
  INSERT INTO issue_search(issue_search, rowid, title, description) VALUES('delete', old.rowid, old.title, old.description);
END;
CREATE TRIGGER issues_au AFTER UPDATE ON issues BEGIN
  INSERT INTO issue_search(issue_search, rowid, title, description) VALUES('delete', old.rowid, old.title, old.description);
  INSERT INTO issue_search(rowid, title, description) VALUES (new.rowid, new.title, new.description);
END;
```

- [ ] **Step 2: Write the failing tests**

Create `crates/kanban-core/src/store/migrations.rs`:

```rust
use crate::error::Result;
use chrono::Utc;
use rusqlite::{Connection, params};

const MIGRATIONS: &[(i64, &str, &str)] = &[
    (1, "init", include_str!("../../migrations/0001_init.sql")),
];

pub fn run(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
           version    INTEGER PRIMARY KEY,
           applied_at TEXT    NOT NULL
         ) STRICT",
    )?;

    for (version, _name, sql) in MIGRATIONS {
        let already: bool = conn.query_row(
            "SELECT 1 FROM schema_migrations WHERE version = ?1",
            params![version],
            |_| Ok(true),
        ).optional()?.unwrap_or(false);

        if already {
            continue;
        }

        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            params![version, Utc::now().to_rfc3339()],
        )?;
        tx.commit()?;
    }

    Ok(())
}

trait OptionalRow<T> {
    fn optional(self) -> rusqlite::Result<Option<T>>;
}

impl<T> OptionalRow<T> for rusqlite::Result<T> {
    fn optional(self) -> rusqlite::Result<Option<T>> {
        match self {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::connection::open_in_memory;

    fn table_names(conn: &Connection) -> Vec<String> {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type IN ('table','view') ORDER BY name")
            .unwrap();
        stmt.query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    }

    #[test]
    fn run_creates_all_v1_tables() {
        let mut conn = open_in_memory().unwrap();
        run(&mut conn).unwrap();
        let tables = table_names(&conn);
        for expected in [
            "activity_log",
            "issue_labels",
            "issue_search",
            "issues",
            "labels",
            "operation_log",
            "projects",
            "schema_migrations",
            "statuses",
        ] {
            assert!(tables.contains(&expected.to_string()), "missing {expected} in {tables:?}");
        }
    }

    #[test]
    fn run_is_idempotent() {
        let mut conn = open_in_memory().unwrap();
        run(&mut conn).unwrap();
        run(&mut conn).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn fts_triggers_keep_search_in_sync() {
        let mut conn = open_in_memory().unwrap();
        run(&mut conn).unwrap();
        // Manually insert minimum data to test triggers — full apply tested elsewhere.
        conn.execute(
            "INSERT INTO projects(id,name,prefix,status,next_seq,created_at,updated_at)
             VALUES('p','P','PPP','active',1,'2026-01-01T00:00:00Z','2026-01-01T00:00:00Z')",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO statuses(id,project_id,name,category,color,position)
             VALUES('s','p','Todo','unstarted','#000000',0)",
            [],
        ).unwrap();
        conn.execute(
            "INSERT INTO issues(id,project_id,seq,identifier,title,description,status_id,priority,sort_key,created_at,updated_at)
             VALUES('i','p',1,'PPP-1','add login','user can login','s','high',1.0,'2026-01-01T00:00:00Z','2026-01-01T00:00:00Z')",
            [],
        ).unwrap();

        let hits: i64 = conn.query_row(
            "SELECT count(*) FROM issue_search WHERE issue_search MATCH 'login'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(hits, 1);
    }
}
```

- [ ] **Step 3: Update `crates/kanban-core/src/store/mod.rs`**

```rust
pub(crate) mod connection;
pub(crate) mod migrations;
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kanban-core`
Expected: PASS, 22 tests total.

- [ ] **Step 5: Commit**

```bash
git add crates/kanban-core/migrations/ crates/kanban-core/src/store/
git commit -m "feat(core): add 0001_init migration and idempotent runner with FTS5 triggers"
```

---

## Phase 3 — Workspace handle

### Task 8: `Workspace::open`

**Files:**
- Create: `crates/kanban-core/src/workspace.rs`
- Modify: `crates/kanban-core/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

Create `crates/kanban-core/src/workspace.rs`:

```rust
use crate::error::Result;
use crate::store::{connection, migrations};
use crate::time::{Clock, system_clock};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Opaque handle to a kanban workspace.
///
/// Holds an open SQLite connection and a clock. NOT `Sync` — multi-threaded
/// callers must own one `Workspace` per thread (or use a pool).
pub struct Workspace {
    pub(crate) conn: Connection,
    pub(crate) clock: Arc<dyn Clock>,
    pub(crate) path: WorkspacePath,
}

#[derive(Debug, Clone)]
pub enum WorkspacePath {
    InMemory,
    File(PathBuf),
}

impl Workspace {
    /// Open the workspace at `~/.kanban/data.db` (or `$KANBAN_DB`).
    pub fn open_default() -> Result<Self> {
        let path = default_db_path()?;
        Self::open(&path)
    }

    /// Open a workspace at the given file path.
    pub fn open(path: &Path) -> Result<Self> {
        let mut conn = connection::open_file(path)?;
        migrations::run(&mut conn)?;
        Ok(Self {
            conn,
            clock: system_clock(),
            path: WorkspacePath::File(path.to_path_buf()),
        })
    }

    /// Open an in-memory workspace (for tests).
    pub fn open_in_memory() -> Result<Self> {
        let mut conn = connection::open_in_memory()?;
        migrations::run(&mut conn)?;
        Ok(Self {
            conn,
            clock: system_clock(),
            path: WorkspacePath::InMemory,
        })
    }

    /// Inject a clock for deterministic tests.
    #[must_use]
    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }
}

fn default_db_path() -> Result<PathBuf> {
    if let Ok(env) = std::env::var("KANBAN_DB") {
        return Ok(PathBuf::from(env));
    }
    let home = std::env::var("HOME").map_err(|_| {
        crate::error::Error::Validation(crate::error::ValidationError {
            field: "HOME".into(),
            reason: "must be set to derive default db path".into(),
        })
    })?;
    Ok(PathBuf::from(home).join(".kanban").join("data.db"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory_runs_migrations() {
        let ws = Workspace::open_in_memory().unwrap();
        let count: i64 = ws
            .conn
            .query_row("SELECT COUNT(*) FROM schema_migrations", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn open_file_creates_db() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.db");
        let _ws = Workspace::open(&path).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn open_default_uses_kanban_db_env() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("override.db");
        // Safety: serial test would be ideal; KANBAN_DB only read in this fn.
        // This test is intentionally simple and may flake under parallel test runs;
        // we accept that for v1 — see DEVELOPMENT.md.
        unsafe { std::env::set_var("KANBAN_DB", &path); }
        let _ws = Workspace::open_default().unwrap();
        assert!(path.exists());
        unsafe { std::env::remove_var("KANBAN_DB"); }
    }
}
```

- [ ] **Step 2: Wire into `lib.rs`**

Add (keep existing entries):

```rust
pub mod workspace;
pub use workspace::{Workspace, WorkspacePath};
```

- [ ] **Step 3: Run the tests**

Run: `cargo test -p kanban-core`
Expected: PASS, 25 tests total.

- [ ] **Step 4: Commit**

```bash
git add crates/kanban-core/src/workspace.rs crates/kanban-core/src/lib.rs
git commit -m "feat(core): add Workspace handle with open, open_in_memory, default path"
```

---

## Phase 4 — Operation log writers

### Task 9: `operation_log` and `activity_log` low-level writers

**Files:**
- Create: `crates/kanban-core/src/store/write/mod.rs`
- Create: `crates/kanban-core/src/store/write/operation_log.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/kanban-core/src/store/write/operation_log.rs`:

```rust
use crate::error::Result;
use chrono::{DateTime, Utc};
use rusqlite::{Transaction, params};

/// Insert one row into `operation_log`. Returns the new `op_id`.
pub(crate) fn insert_operation(
    tx: &Transaction<'_>,
    op_type: &str,
    payload: &str,
    inverse_payload: &str,
    applied_at: DateTime<Utc>,
) -> Result<i64> {
    tx.execute(
        "INSERT INTO operation_log(op_type, payload, inverse_payload, applied_at, undone)
         VALUES (?1, ?2, ?3, ?4, 0)",
        params![op_type, payload, inverse_payload, applied_at.to_rfc3339()],
    )?;
    Ok(tx.last_insert_rowid())
}

/// Insert one row into `activity_log` linked to a previously-inserted op_id.
pub(crate) fn insert_activity(
    tx: &Transaction<'_>,
    op_id: i64,
    issue_id: Option<&str>,
    field: &str,
    old_value: Option<&str>,
    new_value: Option<&str>,
    at: DateTime<Utc>,
) -> Result<()> {
    tx.execute(
        "INSERT INTO activity_log(op_id, issue_id, field, old_value, new_value, at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![op_id, issue_id, field, old_value, new_value, at.to_rfc3339()],
    )?;
    Ok(())
}

/// Mark `op_id` as undone (1) or redone (0).
pub(crate) fn set_op_undone(tx: &Transaction<'_>, op_id: i64, undone: bool) -> Result<()> {
    tx.execute(
        "UPDATE operation_log SET undone = ?1 WHERE id = ?2",
        params![i64::from(undone), op_id],
    )?;
    Ok(())
}

/// Discard the redo branch (any rows where undone=1). Called when a new forward op lands
/// while a redo branch existed.
pub(crate) fn truncate_redo_branch(tx: &Transaction<'_>) -> Result<()> {
    tx.execute("DELETE FROM operation_log WHERE undone = 1", [])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{connection::open_in_memory, migrations};

    fn fresh_tx_owner() -> rusqlite::Connection {
        let mut c = open_in_memory().unwrap();
        migrations::run(&mut c).unwrap();
        c
    }

    #[test]
    fn insert_operation_returns_monotonic_ids() {
        let mut c = fresh_tx_owner();
        let now = Utc::now();
        let tx = c.transaction().unwrap();
        let a = insert_operation(&tx, "Test", "{}", "{}", now).unwrap();
        let b = insert_operation(&tx, "Test", "{}", "{}", now).unwrap();
        tx.commit().unwrap();
        assert!(b > a);
    }

    #[test]
    fn insert_activity_links_to_op_id() {
        let mut c = fresh_tx_owner();
        let now = Utc::now();
        let tx = c.transaction().unwrap();
        let op_id = insert_operation(&tx, "Test", "{}", "{}", now).unwrap();
        insert_activity(&tx, op_id, None, "title", Some("a"), Some("b"), now).unwrap();
        tx.commit().unwrap();
        let count: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM activity_log WHERE op_id = ?1",
                params![op_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn set_op_undone_toggles_flag() {
        let mut c = fresh_tx_owner();
        let now = Utc::now();
        let tx = c.transaction().unwrap();
        let op_id = insert_operation(&tx, "Test", "{}", "{}", now).unwrap();
        set_op_undone(&tx, op_id, true).unwrap();
        tx.commit().unwrap();
        let undone: i64 = c
            .query_row(
                "SELECT undone FROM operation_log WHERE id = ?1",
                params![op_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(undone, 1);
    }

    #[test]
    fn truncate_redo_branch_only_deletes_undone_rows() {
        let mut c = fresh_tx_owner();
        let now = Utc::now();
        let tx = c.transaction().unwrap();
        let live = insert_operation(&tx, "L", "{}", "{}", now).unwrap();
        let dead = insert_operation(&tx, "D", "{}", "{}", now).unwrap();
        set_op_undone(&tx, dead, true).unwrap();
        truncate_redo_branch(&tx).unwrap();
        tx.commit().unwrap();
        let live_exists: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM operation_log WHERE id = ?1",
                params![live],
                |r| r.get(0),
            )
            .unwrap();
        let dead_exists: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM operation_log WHERE id = ?1",
                params![dead],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(live_exists, 1);
        assert_eq!(dead_exists, 0);
    }
}
```

- [ ] **Step 2: Create `crates/kanban-core/src/store/write/mod.rs`**

```rust
pub(crate) mod operation_log;
```

- [ ] **Step 3: Update `crates/kanban-core/src/store/mod.rs`**

```rust
pub(crate) mod connection;
pub(crate) mod migrations;
pub(crate) mod write;
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p kanban-core`
Expected: PASS, 29 tests total.

- [ ] **Step 5: Commit**

```bash
git add crates/kanban-core/src/store/
git commit -m "feat(core): add operation_log and activity_log writers"
```

---

## Phase 5 — Project Operations (vertical slice)

This phase lands the full pattern that every later operation follows: define the operation, write a failing test, implement applier + writer, verify, commit. Inverses are computed in Task 15.

### Task 10: `Operation` enum scaffold and `CreateProject`

**Files:**
- Create: `crates/kanban-core/src/operation.rs`
- Create: `crates/kanban-core/src/apply/mod.rs`
- Create: `crates/kanban-core/src/apply/projects.rs`
- Create: `crates/kanban-core/src/store/write/projects.rs`
- Create: `crates/kanban-core/src/store/write/statuses.rs`
- Create: `crates/kanban-core/tests/apply_projects.rs`
- Modify: `crates/kanban-core/src/lib.rs`
- Modify: `crates/kanban-core/src/workspace.rs`
- Modify: `crates/kanban-core/src/store/write/mod.rs`

- [ ] **Step 1: Define the `Operation` enum (initially with `CreateProject` only)**

Create `crates/kanban-core/src/operation.rs`:

```rust
use crate::types::{Priority, ProjectStatus};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", content = "args")]
pub enum Operation {
    CreateProject(CreateProject),
    UpdateProject(UpdateProject),
    ArchiveProject(ArchiveProject),
    DeleteProject(DeleteProject),

    CreateIssue(CreateIssue),
    UpdateIssueField(UpdateIssueField),
    ReorderIssue(ReorderIssue),
    DeleteIssue(DeleteIssue),

    CreateLabel(CreateLabel),
    UpdateLabel(UpdateLabel),
    DeleteLabel(DeleteLabel),
    AttachLabel(AttachLabel),
    DetachLabel(DetachLabel),
}

// ----- Project ops -----

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateProject {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateProject {
    pub id: Uuid,
    pub patch: ProjectPatch,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ProjectPatch {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub icon: Option<Option<String>>,
    pub status: Option<ProjectStatus>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchiveProject {
    pub id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteProject {
    pub id: Uuid,
}

// ----- Issue ops (placeholders to keep enum cohesive; implemented in Phase 8) -----

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateIssue {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status_id: Uuid,
    pub priority: Priority,
    pub due_date: Option<NaiveDate>,
    pub label_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateIssueField {
    pub id: Uuid,
    pub change: IssueFieldChange,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "field", content = "value")]
pub enum IssueFieldChange {
    Title(String),
    Description(Option<String>),
    Status(Uuid),
    Priority(Priority),
    DueDate(Option<NaiveDate>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReorderIssue {
    pub id: Uuid,
    pub new_sort_key: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteIssue {
    pub id: Uuid,
}

// ----- Label ops (placeholders for Phase 9) -----

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateLabel {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateLabel {
    pub id: Uuid,
    pub patch: LabelPatch,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LabelPatch {
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteLabel {
    pub id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttachLabel {
    pub issue_id: Uuid,
    pub label_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetachLabel {
    pub issue_id: Uuid,
    pub label_id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationOutcome {
    pub op_id: i64,
}
```

- [ ] **Step 2: Write the failing integration test**

Create `crates/kanban-core/tests/apply_projects.rs`:

```rust
use kanban_core::operation::{CreateProject, Operation};
use kanban_core::{Workspace, new_id};

#[test]
fn create_project_inserts_row_and_seeds_default_statuses() {
    let ws = Workspace::open_in_memory().unwrap();

    let id = new_id();
    let outcome = ws
        .apply(Operation::CreateProject(CreateProject {
            id,
            name: "Auth Service".into(),
            prefix: "AUTH".into(),
            description: Some("oauth flows".into()),
            icon: None,
        }))
        .unwrap();
    assert!(outcome.op_id > 0);

    let project = ws.query_project_by_id(id).unwrap();
    assert_eq!(project.name, "Auth Service");
    assert_eq!(project.prefix, "AUTH");
    assert_eq!(project.next_seq, 1);

    let statuses = ws.query_statuses_for_project(id).unwrap();
    let names: Vec<_> = statuses.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(
        names,
        vec!["Todo", "Backlog", "In Progress", "In Review", "Blocked", "Discarded", "Done"]
    );
}

#[test]
fn create_project_rejects_duplicate_prefix() {
    let ws = Workspace::open_in_memory().unwrap();
    ws.apply(Operation::CreateProject(CreateProject {
        id: new_id(),
        name: "A".into(),
        prefix: "DUP".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let err = ws
        .apply(Operation::CreateProject(CreateProject {
            id: new_id(),
            name: "B".into(),
            prefix: "DUP".into(),
            description: None,
            icon: None,
        }))
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.to_lowercase().contains("conflict"), "got: {msg}");
}

#[test]
fn create_project_rejects_invalid_prefix() {
    let ws = Workspace::open_in_memory().unwrap();
    let err = ws
        .apply(Operation::CreateProject(CreateProject {
            id: new_id(),
            name: "A".into(),
            prefix: "lower".into(),
            description: None,
            icon: None,
        }))
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("uppercase"), "got: {msg}");
}
```

- [ ] **Step 3: Run the test to confirm it fails**

Run: `cargo test -p kanban-core --test apply_projects`
Expected: FAIL — `Workspace::apply`, `query_project_by_id`, `query_statuses_for_project` not found.

- [ ] **Step 4: Implement project + status writers**

Create `crates/kanban-core/src/store/write/projects.rs`:

```rust
use crate::error::Result;
use crate::types::{Project, ProjectStatus};
use chrono::{DateTime, Utc};
use rusqlite::{Transaction, params};
use uuid::Uuid;

pub(crate) fn insert(
    tx: &Transaction<'_>,
    id: Uuid,
    name: &str,
    prefix: &str,
    description: Option<&str>,
    icon: Option<&str>,
    now: DateTime<Utc>,
) -> Result<Project> {
    let now_s = now.to_rfc3339();
    tx.execute(
        "INSERT INTO projects(id,name,prefix,description,icon,status,next_seq,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,'active',1,?6,?6)",
        params![id.to_string(), name, prefix, description, icon, now_s],
    )?;
    Ok(Project {
        id,
        name: name.to_string(),
        prefix: prefix.to_string(),
        description: description.map(str::to_string),
        icon: icon.map(str::to_string),
        status: ProjectStatus::Active,
        next_seq: 1,
        created_at: now,
        updated_at: now,
    })
}

pub(crate) fn delete(tx: &Transaction<'_>, id: Uuid) -> Result<()> {
    tx.execute("DELETE FROM projects WHERE id = ?1", params![id.to_string()])?;
    Ok(())
}

pub(crate) fn set_status(
    tx: &Transaction<'_>,
    id: Uuid,
    status: ProjectStatus,
    now: DateTime<Utc>,
) -> Result<()> {
    tx.execute(
        "UPDATE projects SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status.as_str(), now.to_rfc3339(), id.to_string()],
    )?;
    Ok(())
}

pub(crate) fn update_fields(
    tx: &Transaction<'_>,
    id: Uuid,
    name: Option<&str>,
    description: Option<Option<&str>>,
    icon: Option<Option<&str>>,
    status: Option<ProjectStatus>,
    now: DateTime<Utc>,
) -> Result<()> {
    if let Some(v) = name {
        tx.execute(
            "UPDATE projects SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![v, now.to_rfc3339(), id.to_string()],
        )?;
    }
    if let Some(v) = description {
        tx.execute(
            "UPDATE projects SET description = ?1, updated_at = ?2 WHERE id = ?3",
            params![v, now.to_rfc3339(), id.to_string()],
        )?;
    }
    if let Some(v) = icon {
        tx.execute(
            "UPDATE projects SET icon = ?1, updated_at = ?2 WHERE id = ?3",
            params![v, now.to_rfc3339(), id.to_string()],
        )?;
    }
    if let Some(v) = status {
        set_status(tx, id, v, now)?;
    }
    Ok(())
}
```

Create `crates/kanban-core/src/store/write/statuses.rs`:

```rust
use crate::error::Result;
use crate::ids::new_id;
use crate::types::StatusCategory;
use rusqlite::{Transaction, params};
use uuid::Uuid;

const DEFAULTS: &[(&str, StatusCategory, &str, i64)] = &[
    ("Todo",        StatusCategory::Unstarted, "#94a3b8", 0),
    ("Backlog",     StatusCategory::Unstarted, "#64748b", 1),
    ("In Progress", StatusCategory::Started,   "#3b82f6", 2),
    ("In Review",   StatusCategory::Started,   "#a855f7", 3),
    ("Blocked",     StatusCategory::Blocked,   "#ef4444", 4),
    ("Discarded",   StatusCategory::Discarded, "#6b7280", 5),
    ("Done",        StatusCategory::Completed, "#22c55e", 6),
];

pub(crate) fn seed_defaults(tx: &Transaction<'_>, project_id: Uuid) -> Result<Vec<Uuid>> {
    let mut ids = Vec::with_capacity(DEFAULTS.len());
    for (name, category, color, position) in DEFAULTS {
        let id = new_id();
        tx.execute(
            "INSERT INTO statuses(id,project_id,name,category,color,position)
             VALUES (?1,?2,?3,?4,?5,?6)",
            params![id.to_string(), project_id.to_string(), name, category.as_str(), color, position],
        )?;
        ids.push(id);
    }
    Ok(ids)
}
```

- [ ] **Step 5: Implement `apply` for `CreateProject`**

Create `crates/kanban-core/src/apply/mod.rs`:

```rust
use crate::error::{Error, Result};
use crate::operation::{Operation, OperationOutcome};
use crate::store::write::operation_log;
use crate::workspace::Workspace;
use chrono::Utc;

pub(crate) mod projects;

impl Workspace {
    /// The single public mutator. Validates, executes, and records `op` in one transaction.
    pub fn apply(&mut self, op: Operation) -> Result<OperationOutcome> {
        let now = self.clock.now();
        let payload = serde_json::to_string(&op)?;
        let inverse = compute_inverse(&op)?;
        let inverse_payload = serde_json::to_string(&inverse)?;

        let tx = self.conn.transaction()?;
        // Discard redo branch when a new forward op lands.
        operation_log::truncate_redo_branch(&tx)?;

        match &op {
            Operation::CreateProject(args) => projects::create(&tx, args, now)?,
            Operation::UpdateProject(args) => projects::update(&tx, args, now)?,
            Operation::ArchiveProject(args) => projects::archive(&tx, args, now)?,
            Operation::DeleteProject(args) => projects::delete(&tx, args)?,
            // Issue/label arms land in Phase 8/9 — until then return InvalidSnapshot.
            other => return Err(Error::InvalidSnapshot(format!("unsupported op: {other:?}"))),
        }

        let op_id = operation_log::insert_operation(
            &tx,
            op_type_name(&op),
            &payload,
            &inverse_payload,
            now,
        )?;
        tx.commit()?;
        Ok(OperationOutcome { op_id })
    }
}

fn op_type_name(op: &Operation) -> &'static str {
    match op {
        Operation::CreateProject(_) => "CreateProject",
        Operation::UpdateProject(_) => "UpdateProject",
        Operation::ArchiveProject(_) => "ArchiveProject",
        Operation::DeleteProject(_) => "DeleteProject",
        Operation::CreateIssue(_) => "CreateIssue",
        Operation::UpdateIssueField(_) => "UpdateIssueField",
        Operation::ReorderIssue(_) => "ReorderIssue",
        Operation::DeleteIssue(_) => "DeleteIssue",
        Operation::CreateLabel(_) => "CreateLabel",
        Operation::UpdateLabel(_) => "UpdateLabel",
        Operation::DeleteLabel(_) => "DeleteLabel",
        Operation::AttachLabel(_) => "AttachLabel",
        Operation::DetachLabel(_) => "DetachLabel",
    }
}

/// Inverse computation. Implemented incrementally (Task 15, 23, 26).
/// Until the inverse for a variant is implemented, we return a placeholder operation
/// that, when applied, would error — undo against an unsupported op simply fails loud.
pub(crate) fn compute_inverse(op: &Operation) -> Result<Operation> {
    match op {
        // Project inverses land in Task 15.
        Operation::CreateProject(args) => Ok(Operation::DeleteProject(crate::operation::DeleteProject { id: args.id })),
        Operation::DeleteProject(args) => Ok(Operation::CreateProject(crate::operation::CreateProject {
            id: args.id,
            name: "<undo placeholder>".into(),
            prefix: "UNDO".into(),
            description: None,
            icon: None,
        })),
        Operation::UpdateProject(args) => Ok(Operation::UpdateProject(args.clone())),
        Operation::ArchiveProject(args) => Ok(Operation::ArchiveProject(args.clone())),
        // Issue/label inverses land in later phases.
        other => Err(Error::InvalidSnapshot(format!("inverse not yet implemented for {other:?}"))),
    }
}
```

> Note: `compute_inverse` is intentionally weak right now. Task 15 replaces it with a proper implementation that captures pre-state for project ops. The current placeholders exist only so `apply` compiles and the basic CreateProject test passes.

Create `crates/kanban-core/src/apply/projects.rs`:

```rust
use crate::error::{Error, Result, ValidationError};
use crate::operation::{ArchiveProject, CreateProject, DeleteProject, UpdateProject};
use crate::store::write::{projects as wp, statuses as ws};
use crate::types::ProjectStatus;
use crate::validate;
use chrono::{DateTime, Utc};
use rusqlite::{Transaction, params};

pub(crate) fn create(tx: &Transaction<'_>, args: &CreateProject, now: DateTime<Utc>) -> Result<()> {
    let name = validate::nonempty_field("name", &args.name)?.to_string();
    validate::project_prefix(&args.prefix)?;

    let exists: Option<i64> = tx
        .query_row(
            "SELECT 1 FROM projects WHERE prefix = ?1",
            params![&args.prefix],
            |r| r.get(0),
        )
        .ok();
    if exists.is_some() {
        return Err(Error::Conflict(format!("project prefix '{}' is already in use", args.prefix)));
    }

    wp::insert(
        tx,
        args.id,
        &name,
        &args.prefix,
        args.description.as_deref(),
        args.icon.as_deref(),
        now,
    )?;
    ws::seed_defaults(tx, args.id)?;
    Ok(())
}

pub(crate) fn update(tx: &Transaction<'_>, args: &UpdateProject, now: DateTime<Utc>) -> Result<()> {
    if !exists(tx, args.id)? {
        return Err(Error::NotFound { kind: crate::EntityKind::Project, id: args.id.to_string() });
    }
    if let Some(name) = &args.patch.name {
        validate::nonempty_field("name", name)?;
    }
    wp::update_fields(
        tx,
        args.id,
        args.patch.name.as_deref(),
        args.patch.description.as_ref().map(|o| o.as_deref()),
        args.patch.icon.as_ref().map(|o| o.as_deref()),
        args.patch.status,
        now,
    )?;
    Ok(())
}

pub(crate) fn archive(tx: &Transaction<'_>, args: &ArchiveProject, now: DateTime<Utc>) -> Result<()> {
    if !exists(tx, args.id)? {
        return Err(Error::NotFound { kind: crate::EntityKind::Project, id: args.id.to_string() });
    }
    wp::set_status(tx, args.id, ProjectStatus::Archived, now)?;
    Ok(())
}

pub(crate) fn delete(tx: &Transaction<'_>, args: &DeleteProject) -> Result<()> {
    if !exists(tx, args.id)? {
        return Err(Error::NotFound { kind: crate::EntityKind::Project, id: args.id.to_string() });
    }
    wp::delete(tx, args.id)?;
    Ok(())
}

fn exists(tx: &Transaction<'_>, id: uuid::Uuid) -> Result<bool> {
    let n: i64 = tx
        .query_row("SELECT COUNT(*) FROM projects WHERE id = ?1", params![id.to_string()], |r| r.get(0))?;
    Ok(n > 0)
}
```

- [ ] **Step 6: Add the read API used by the test**

Create `crates/kanban-core/src/store/read/mod.rs`:

```rust
pub(crate) mod projects;
pub(crate) mod statuses;
```

Create `crates/kanban-core/src/store/read/projects.rs`:

```rust
use crate::error::{Error, Result};
use crate::types::{Project, ProjectStatus};
use chrono::DateTime;
use rusqlite::{Connection, params};
use std::str::FromStr;
use uuid::Uuid;

pub(crate) fn by_id(conn: &Connection, id: Uuid) -> Result<Project> {
    conn.query_row(
        "SELECT id,name,prefix,description,icon,status,next_seq,created_at,updated_at
         FROM projects WHERE id = ?1",
        params![id.to_string()],
        row_to_project,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Error::NotFound {
            kind: crate::EntityKind::Project,
            id: id.to_string(),
        },
        other => other.into(),
    })
}

pub(crate) fn list_all(conn: &Connection) -> Result<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id,name,prefix,description,icon,status,next_seq,created_at,updated_at
         FROM projects ORDER BY created_at ASC",
    )?;
    let rows = stmt.query_map([], row_to_project)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

fn row_to_project(r: &rusqlite::Row<'_>) -> rusqlite::Result<Project> {
    let id_s: String = r.get(0)?;
    let status_s: String = r.get(5)?;
    let created_s: String = r.get(7)?;
    let updated_s: String = r.get(8)?;
    Ok(Project {
        id: Uuid::from_str(&id_s).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?,
        name: r.get(1)?,
        prefix: r.get(2)?,
        description: r.get(3)?,
        icon: r.get(4)?,
        status: parse_status(&status_s)?,
        next_seq: r.get(6)?,
        created_at: DateTime::parse_from_rfc3339(&created_s)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
            .with_timezone(&chrono::Utc),
        updated_at: DateTime::parse_from_rfc3339(&updated_s)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?
            .with_timezone(&chrono::Utc),
    })
}

fn parse_status(s: &str) -> rusqlite::Result<ProjectStatus> {
    match s {
        "active" => Ok(ProjectStatus::Active),
        "paused" => Ok(ProjectStatus::Paused),
        "completed" => Ok(ProjectStatus::Completed),
        "archived" => Ok(ProjectStatus::Archived),
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            0, rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("unknown project status '{other}'"))),
        )),
    }
}
```

Create `crates/kanban-core/src/store/read/statuses.rs`:

```rust
use crate::error::Result;
use crate::types::{Status, StatusCategory};
use rusqlite::{Connection, params};
use std::str::FromStr;
use uuid::Uuid;

pub(crate) fn for_project(conn: &Connection, project_id: Uuid) -> Result<Vec<Status>> {
    let mut stmt = conn.prepare(
        "SELECT id,project_id,name,category,color,position FROM statuses
         WHERE project_id = ?1 ORDER BY position ASC",
    )?;
    let rows = stmt.query_map(params![project_id.to_string()], row_to_status)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

fn row_to_status(r: &rusqlite::Row<'_>) -> rusqlite::Result<Status> {
    let id: String = r.get(0)?;
    let pid: String = r.get(1)?;
    let category_s: String = r.get(3)?;
    Ok(Status {
        id: Uuid::from_str(&id).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?,
        project_id: Uuid::from_str(&pid).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?,
        name: r.get(2)?,
        category: parse_category(&category_s)?,
        color: r.get(4)?,
        position: r.get(5)?,
    })
}

fn parse_category(s: &str) -> rusqlite::Result<StatusCategory> {
    match s {
        "unstarted" => Ok(StatusCategory::Unstarted),
        "started" => Ok(StatusCategory::Started),
        "blocked" => Ok(StatusCategory::Blocked),
        "completed" => Ok(StatusCategory::Completed),
        "discarded" => Ok(StatusCategory::Discarded),
        other => Err(rusqlite::Error::FromSqlConversionFailure(
            0, rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("unknown status category '{other}'"))),
        )),
    }
}
```

- [ ] **Step 7: Add `query_*` methods on `Workspace`**

Append to `crates/kanban-core/src/workspace.rs`:

```rust
use crate::store::read;
use crate::types::{Project, Status};
use uuid::Uuid;

impl Workspace {
    pub fn query_project_by_id(&self, id: Uuid) -> crate::error::Result<Project> {
        read::projects::by_id(&self.conn, id)
    }

    pub fn query_projects(&self) -> crate::error::Result<Vec<Project>> {
        read::projects::list_all(&self.conn)
    }

    pub fn query_statuses_for_project(&self, project_id: Uuid) -> crate::error::Result<Vec<Status>> {
        read::statuses::for_project(&self.conn, project_id)
    }
}
```

- [ ] **Step 8: Update `lib.rs` and `store/mod.rs`**

Add to `crates/kanban-core/src/lib.rs`:

```rust
pub mod operation;
pub mod apply;

pub use operation::{Operation, OperationOutcome};
```

Update `crates/kanban-core/src/store/mod.rs`:

```rust
pub(crate) mod connection;
pub(crate) mod migrations;
pub(crate) mod read;
pub(crate) mod write;
```

Update `crates/kanban-core/src/store/write/mod.rs`:

```rust
pub(crate) mod operation_log;
pub(crate) mod projects;
pub(crate) mod statuses;
```

- [ ] **Step 9: Run the tests**

Run: `cargo test -p kanban-core`
Expected: PASS, all integration + unit tests green.

- [ ] **Step 10: Commit**

```bash
git add crates/kanban-core/src crates/kanban-core/tests
git commit -m "feat(core): add Operation enum + CreateProject apply path with default-status seeding"
```

---

### Task 11: Project read coverage tests

**Files:**
- Modify: `crates/kanban-core/tests/apply_projects.rs`

- [ ] **Step 1: Append the failing tests**

```rust
use kanban_core::types::ProjectStatus;

#[test]
fn query_projects_returns_all_in_creation_order() {
    let ws = Workspace::open_in_memory().unwrap();
    for prefix in ["AAA", "BBB", "CCC"] {
        ws.apply(Operation::CreateProject(CreateProject {
            id: new_id(),
            name: prefix.into(),
            prefix: prefix.into(),
            description: None,
            icon: None,
        }))
        .unwrap();
    }
    let projects = ws.query_projects().unwrap();
    let prefixes: Vec<_> = projects.iter().map(|p| p.prefix.clone()).collect();
    assert_eq!(prefixes, vec!["AAA", "BBB", "CCC"]);
}

#[test]
fn query_project_by_id_returns_not_found_for_missing() {
    let ws = Workspace::open_in_memory().unwrap();
    let err = ws.query_project_by_id(new_id()).unwrap_err();
    assert!(err.to_string().contains("Project not found"), "{err}");
}

#[test]
fn newly_created_project_has_active_status() {
    let ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id,
        name: "X".into(),
        prefix: "XYZ".into(),
        description: None,
        icon: None,
    }))
    .unwrap();
    let p = ws.query_project_by_id(id).unwrap();
    assert_eq!(p.status, ProjectStatus::Active);
}
```

- [ ] **Step 2: Run**

Run: `cargo test -p kanban-core --test apply_projects`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/kanban-core/tests/apply_projects.rs
git commit -m "test(core): cover query_projects, query_project_by_id, default status"
```

---

### Task 12: `UpdateProject`

**Files:**
- Modify: `crates/kanban-core/tests/apply_projects.rs`

- [ ] **Step 1: Append the failing tests**

```rust
use kanban_core::operation::{ProjectPatch, UpdateProject};

#[test]
fn update_project_changes_name_and_updates_timestamp() {
    let ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id, name: "Old".into(), prefix: "UPD".into(), description: None, icon: None,
    })).unwrap();
    let before = ws.query_project_by_id(id).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(10));
    ws.apply(Operation::UpdateProject(UpdateProject {
        id,
        patch: ProjectPatch { name: Some("New".into()), ..Default::default() },
    })).unwrap();

    let after = ws.query_project_by_id(id).unwrap();
    assert_eq!(after.name, "New");
    assert!(after.updated_at >= before.updated_at);
}

#[test]
fn update_project_clears_description_with_some_none() {
    let ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id, name: "X".into(), prefix: "DSC".into(), description: Some("hi".into()), icon: None,
    })).unwrap();
    ws.apply(Operation::UpdateProject(UpdateProject {
        id,
        patch: ProjectPatch { description: Some(None), ..Default::default() },
    })).unwrap();
    let p = ws.query_project_by_id(id).unwrap();
    assert!(p.description.is_none());
}

#[test]
fn update_project_unknown_id_returns_not_found() {
    let ws = Workspace::open_in_memory().unwrap();
    let err = ws.apply(Operation::UpdateProject(UpdateProject {
        id: new_id(),
        patch: ProjectPatch { name: Some("nope".into()), ..Default::default() },
    })).unwrap_err();
    assert!(err.to_string().contains("not found"), "{err}");
}
```

- [ ] **Step 2: Run, observe pass**

Run: `cargo test -p kanban-core --test apply_projects`
Expected: PASS — `update` was already wired in Task 10.

- [ ] **Step 3: Commit**

```bash
git add crates/kanban-core/tests/apply_projects.rs
git commit -m "test(core): cover UpdateProject across name, description, missing id"
```

---

### Task 13: `ArchiveProject` test coverage

**Files:**
- Modify: `crates/kanban-core/tests/apply_projects.rs`

- [ ] **Step 1: Append the failing tests**

```rust
use kanban_core::operation::ArchiveProject;

#[test]
fn archive_project_sets_status_archived() {
    let ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id, name: "X".into(), prefix: "ARC".into(), description: None, icon: None,
    })).unwrap();
    ws.apply(Operation::ArchiveProject(ArchiveProject { id })).unwrap();
    let p = ws.query_project_by_id(id).unwrap();
    assert_eq!(p.status, ProjectStatus::Archived);
}

#[test]
fn archive_project_unknown_id_errors() {
    let ws = Workspace::open_in_memory().unwrap();
    let err = ws.apply(Operation::ArchiveProject(ArchiveProject { id: new_id() })).unwrap_err();
    assert!(err.to_string().contains("not found"), "{err}");
}
```

- [ ] **Step 2: Run**

Run: `cargo test -p kanban-core --test apply_projects`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/kanban-core/tests/apply_projects.rs
git commit -m "test(core): cover ArchiveProject"
```

---

### Task 14: `DeleteProject`

**Files:**
- Modify: `crates/kanban-core/tests/apply_projects.rs`

- [ ] **Step 1: Append the failing tests**

```rust
use kanban_core::operation::DeleteProject;

#[test]
fn delete_project_removes_row_and_cascades_statuses() {
    let ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id, name: "X".into(), prefix: "DEL".into(), description: None, icon: None,
    })).unwrap();
    ws.apply(Operation::DeleteProject(DeleteProject { id })).unwrap();

    assert!(ws.query_project_by_id(id).is_err());
    let s = ws.query_statuses_for_project(id).unwrap();
    assert!(s.is_empty(), "statuses should cascade: got {s:?}");
}

#[test]
fn delete_project_unknown_id_errors() {
    let ws = Workspace::open_in_memory().unwrap();
    let err = ws.apply(Operation::DeleteProject(DeleteProject { id: new_id() })).unwrap_err();
    assert!(err.to_string().contains("not found"), "{err}");
}
```

- [ ] **Step 2: Run**

Run: `cargo test -p kanban-core --test apply_projects`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/kanban-core/tests/apply_projects.rs
git commit -m "test(core): cover DeleteProject and CASCADE on statuses"
```

---

### Task 15: Proper inverse computation for project ops

The placeholder `compute_inverse` in `apply/mod.rs` is incorrect for `DeleteProject` (it loses pre-state) and for `UpdateProject` (returns the same op, not the rollback). This task captures pre-state at apply time and fixes inverses.

**Files:**
- Modify: `crates/kanban-core/src/apply/mod.rs`
- Modify: `crates/kanban-core/src/apply/projects.rs`

- [ ] **Step 1: Add a failing test**

Append to `crates/kanban-core/tests/apply_projects.rs`:

```rust
#[test]
fn inverse_of_create_project_is_delete_with_same_id() {
    let ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    let op = Operation::CreateProject(CreateProject {
        id, name: "X".into(), prefix: "INV".into(), description: None, icon: None,
    });
    ws.apply(op.clone()).unwrap();
    let inv: Operation = ws.last_inverse().unwrap();
    match inv {
        Operation::DeleteProject(d) => assert_eq!(d.id, id),
        other => panic!("expected DeleteProject, got {other:?}"),
    }
}

#[test]
fn inverse_of_delete_project_restores_full_record() {
    let ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id, name: "Restore Me".into(), prefix: "RES".into(),
        description: Some("d".into()), icon: Some("🚀".into()),
    })).unwrap();
    ws.apply(Operation::DeleteProject(DeleteProject { id })).unwrap();
    let inv = ws.last_inverse().unwrap();
    match inv {
        Operation::CreateProject(c) => {
            assert_eq!(c.id, id);
            assert_eq!(c.name, "Restore Me");
            assert_eq!(c.prefix, "RES");
            assert_eq!(c.description.as_deref(), Some("d"));
        }
        other => panic!("expected CreateProject, got {other:?}"),
    }
}
```

- [ ] **Step 2: Add `Workspace::last_inverse` (test helper, public for now)**

Append to `crates/kanban-core/src/workspace.rs`:

```rust
impl Workspace {
    /// Read the most-recent operation's `inverse_payload`. Used by tests; will be reused by `undo`.
    pub fn last_inverse(&self) -> crate::error::Result<crate::operation::Operation> {
        let payload: String = self.conn.query_row(
            "SELECT inverse_payload FROM operation_log ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )?;
        Ok(serde_json::from_str(&payload)?)
    }
}
```

- [ ] **Step 3: Capture pre-state for delete and rewrite inverses**

In `crates/kanban-core/src/apply/mod.rs`, replace `compute_inverse` and the `apply` `match` with a version that consults the DB before the mutation runs. Refactor `apply` to a two-phase shape (capture-pre then mutate):

```rust
use crate::error::{Error, Result};
use crate::operation::*;
use crate::store::{read, write::operation_log};
use crate::workspace::Workspace;
use chrono::Utc;

pub(crate) mod projects;

impl Workspace {
    pub fn apply(&mut self, op: Operation) -> Result<OperationOutcome> {
        let now = self.clock.now();
        let payload = serde_json::to_string(&op)?;
        let tx = self.conn.transaction()?;
        operation_log::truncate_redo_branch(&tx)?;

        // Capture pre-state needed to invert this op.
        let inverse = capture_inverse(&tx, &op)?;
        let inverse_payload = serde_json::to_string(&inverse)?;

        match &op {
            Operation::CreateProject(args) => projects::create(&tx, args, now)?,
            Operation::UpdateProject(args) => projects::update(&tx, args, now)?,
            Operation::ArchiveProject(args) => projects::archive(&tx, args, now)?,
            Operation::DeleteProject(args) => projects::delete(&tx, args)?,
            other => return Err(Error::InvalidSnapshot(format!("unsupported op: {other:?}"))),
        }

        let op_id = operation_log::insert_operation(
            &tx,
            op_type_name(&op),
            &payload,
            &inverse_payload,
            now,
        )?;
        tx.commit()?;
        Ok(OperationOutcome { op_id })
    }
}

fn op_type_name(op: &Operation) -> &'static str {
    match op {
        Operation::CreateProject(_) => "CreateProject",
        Operation::UpdateProject(_) => "UpdateProject",
        Operation::ArchiveProject(_) => "ArchiveProject",
        Operation::DeleteProject(_) => "DeleteProject",
        Operation::CreateIssue(_) => "CreateIssue",
        Operation::UpdateIssueField(_) => "UpdateIssueField",
        Operation::ReorderIssue(_) => "ReorderIssue",
        Operation::DeleteIssue(_) => "DeleteIssue",
        Operation::CreateLabel(_) => "CreateLabel",
        Operation::UpdateLabel(_) => "UpdateLabel",
        Operation::DeleteLabel(_) => "DeleteLabel",
        Operation::AttachLabel(_) => "AttachLabel",
        Operation::DetachLabel(_) => "DetachLabel",
    }
}

fn capture_inverse(tx: &rusqlite::Transaction<'_>, op: &Operation) -> Result<Operation> {
    match op {
        Operation::CreateProject(args) => Ok(Operation::DeleteProject(DeleteProject { id: args.id })),
        Operation::DeleteProject(args) => projects::inverse_of_delete(tx, args),
        Operation::UpdateProject(args) => projects::inverse_of_update(tx, args),
        Operation::ArchiveProject(args) => projects::inverse_of_archive(tx, args),
        // Issue/label inverses come in later phases.
        other => Err(Error::InvalidSnapshot(format!("inverse not yet implemented for {other:?}"))),
    }
}
```

Append to `crates/kanban-core/src/apply/projects.rs`:

```rust
pub(crate) fn inverse_of_delete(tx: &Transaction<'_>, args: &DeleteProject) -> Result<Operation> {
    let p = crate::store::read::projects::by_id_via_tx(tx, args.id)?;
    Ok(Operation::CreateProject(CreateProject {
        id: p.id,
        name: p.name,
        prefix: p.prefix,
        description: p.description,
        icon: p.icon,
    }))
}

pub(crate) fn inverse_of_update(tx: &Transaction<'_>, args: &UpdateProject) -> Result<Operation> {
    let p = crate::store::read::projects::by_id_via_tx(tx, args.id)?;
    Ok(Operation::UpdateProject(UpdateProject {
        id: p.id,
        patch: ProjectPatch {
            name: args.patch.name.as_ref().map(|_| p.name.clone()),
            description: args.patch.description.as_ref().map(|_| p.description.clone()),
            icon: args.patch.icon.as_ref().map(|_| p.icon.clone()),
            status: args.patch.status.as_ref().map(|_| p.status),
        },
    }))
}

pub(crate) fn inverse_of_archive(tx: &Transaction<'_>, args: &ArchiveProject) -> Result<Operation> {
    let p = crate::store::read::projects::by_id_via_tx(tx, args.id)?;
    Ok(Operation::UpdateProject(UpdateProject {
        id: p.id,
        patch: ProjectPatch { status: Some(p.status), ..Default::default() },
    }))
}
```

Add to `crates/kanban-core/src/store/read/projects.rs` (a transaction-scoped read so capture_inverse can run before the mutation commits):

```rust
pub(crate) fn by_id_via_tx(tx: &rusqlite::Transaction<'_>, id: Uuid) -> Result<Project> {
    tx.query_row(
        "SELECT id,name,prefix,description,icon,status,next_seq,created_at,updated_at
         FROM projects WHERE id = ?1",
        params![id.to_string()],
        row_to_project,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Error::NotFound {
            kind: crate::EntityKind::Project,
            id: id.to_string(),
        },
        other => other.into(),
    })
}
```

Update the imports at the top of `crates/kanban-core/src/apply/projects.rs` to also import `Operation` and the patch types:

```rust
use crate::error::{Error, Result, ValidationError};
use crate::operation::{ArchiveProject, CreateProject, DeleteProject, Operation, ProjectPatch, UpdateProject};
use crate::store::write::{projects as wp, statuses as ws};
use crate::types::ProjectStatus;
use crate::validate;
use chrono::{DateTime, Utc};
use rusqlite::{Transaction, params};
```

- [ ] **Step 4: Run**

Run: `cargo test -p kanban-core --test apply_projects`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/kanban-core/src crates/kanban-core/tests/apply_projects.rs
git commit -m "feat(core): capture pre-state for project op inverses"
```

---

## Phase 6 — Undo / redo

### Task 16: `Workspace::undo` and `Workspace::redo`

**Files:**
- Create: `crates/kanban-core/src/undo.rs`
- Create: `crates/kanban-core/tests/undo_redo.rs`
- Modify: `crates/kanban-core/src/lib.rs`
- Modify: `crates/kanban-core/src/workspace.rs`

- [ ] **Step 1: Write the failing test**

Create `crates/kanban-core/tests/undo_redo.rs`:

```rust
use kanban_core::operation::{CreateProject, DeleteProject, Operation, ProjectPatch, UpdateProject};
use kanban_core::{Workspace, new_id};

#[test]
fn undo_create_project_removes_it() {
    let ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id, name: "Tmp".into(), prefix: "TMP".into(), description: None, icon: None,
    })).unwrap();
    ws.undo().unwrap();
    assert!(ws.query_project_by_id(id).is_err());
}

#[test]
fn undo_then_redo_restores_state() {
    let ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id, name: "X".into(), prefix: "RDO".into(), description: None, icon: None,
    })).unwrap();
    ws.undo().unwrap();
    ws.redo().unwrap();
    let p = ws.query_project_by_id(id).unwrap();
    assert_eq!(p.prefix, "RDO");
}

#[test]
fn forward_op_truncates_redo_branch() {
    let ws = Workspace::open_in_memory().unwrap();
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id, name: "A".into(), prefix: "AAA".into(), description: None, icon: None,
    })).unwrap();
    ws.undo().unwrap();
    let id2 = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: id2, name: "B".into(), prefix: "BBB".into(), description: None, icon: None,
    })).unwrap();
    // Redo should now error: nothing to redo.
    let err = ws.redo().unwrap_err();
    assert!(err.to_string().contains("nothing to redo"), "{err}");
}

#[test]
fn undo_with_empty_log_errors() {
    let ws = Workspace::open_in_memory().unwrap();
    let err = ws.undo().unwrap_err();
    assert!(err.to_string().contains("nothing to undo"), "{err}");
}

#[test]
fn undo_persists_across_workspace_open() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.db");
    let id = new_id();
    {
        let ws = Workspace::open(&path).unwrap();
        ws.apply(Operation::CreateProject(CreateProject {
            id, name: "P".into(), prefix: "PER".into(), description: None, icon: None,
        })).unwrap();
        ws.undo().unwrap();
    }
    {
        let ws = Workspace::open(&path).unwrap();
        // After reopen, undo branch survives — redo should still work.
        ws.redo().unwrap();
        let p = ws.query_project_by_id(id).unwrap();
        assert_eq!(p.prefix, "PER");
    }
}
```

- [ ] **Step 2: Run, confirm failure**

Run: `cargo test -p kanban-core --test undo_redo`
Expected: FAIL — `undo`, `redo` not defined.

- [ ] **Step 3: Implement `undo` and `redo`**

Create `crates/kanban-core/src/undo.rs`:

```rust
use crate::apply::capture_inverse;
use crate::error::{Error, Result};
use crate::operation::{Operation, OperationOutcome};
use crate::store::write::operation_log;
use crate::workspace::Workspace;

impl Workspace {
    pub fn undo(&mut self) -> Result<OperationOutcome> {
        let row = peek_undoable(&self.conn)?;
        let op: Operation = serde_json::from_str(&row.inverse_payload)?;

        let now = self.clock.now();
        let tx = self.conn.transaction()?;

        // Apply the inverse mutation directly without writing a new operation_log row.
        crate::apply::dispatch(&tx, &op, now)?;

        operation_log::set_op_undone(&tx, row.id, true)?;
        tx.commit()?;
        Ok(OperationOutcome { op_id: row.id })
    }

    pub fn redo(&mut self) -> Result<OperationOutcome> {
        let row = peek_redoable(&self.conn)?;
        let op: Operation = serde_json::from_str(&row.payload)?;

        let now = self.clock.now();
        let tx = self.conn.transaction()?;

        crate::apply::dispatch(&tx, &op, now)?;

        operation_log::set_op_undone(&tx, row.id, false)?;
        tx.commit()?;
        Ok(OperationOutcome { op_id: row.id })
    }
}

struct OpRow {
    id: i64,
    payload: String,
    inverse_payload: String,
}

fn peek_undoable(conn: &rusqlite::Connection) -> Result<OpRow> {
    conn.query_row(
        "SELECT id, payload, inverse_payload FROM operation_log
         WHERE undone = 0 ORDER BY id DESC LIMIT 1",
        [],
        |r| Ok(OpRow { id: r.get(0)?, payload: r.get(1)?, inverse_payload: r.get(2)? }),
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Error::Conflict("nothing to undo".into()),
        other => other.into(),
    })
}

fn peek_redoable(conn: &rusqlite::Connection) -> Result<OpRow> {
    conn.query_row(
        "SELECT id, payload, inverse_payload FROM operation_log
         WHERE undone = 1 ORDER BY id ASC LIMIT 1",
        [],
        |r| Ok(OpRow { id: r.get(0)?, payload: r.get(1)?, inverse_payload: r.get(2)? }),
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Error::Conflict("nothing to redo".into()),
        other => other.into(),
    })
}
```

- [ ] **Step 4: Extract a `dispatch` helper from `apply`**

In `crates/kanban-core/src/apply/mod.rs`, factor out a transaction-scoped dispatch fn so `undo`/`redo` can re-use the per-op apply logic without touching the operation_log:

```rust
pub(crate) fn dispatch(
    tx: &rusqlite::Transaction<'_>,
    op: &Operation,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    match op {
        Operation::CreateProject(args) => projects::create(tx, args, now)?,
        Operation::UpdateProject(args) => projects::update(tx, args, now)?,
        Operation::ArchiveProject(args) => projects::archive(tx, args, now)?,
        Operation::DeleteProject(args) => projects::delete(tx, args)?,
        other => return Err(Error::InvalidSnapshot(format!("unsupported op: {other:?}"))),
    }
    Ok(())
}
```

Then update `Workspace::apply` to call `dispatch`:

```rust
match &op {
    _ => dispatch(&tx, &op, now)?,
}
```

(remove the now-duplicate match arms in `apply`).

- [ ] **Step 5: Wire `undo` module into `lib.rs`**

Add to `crates/kanban-core/src/lib.rs`:

```rust
pub mod undo;
```

- [ ] **Step 6: Run**

Run: `cargo test -p kanban-core`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add crates/kanban-core/src/undo.rs crates/kanban-core/src/apply/mod.rs crates/kanban-core/src/lib.rs crates/kanban-core/tests/undo_redo.rs
git commit -m "feat(core): add Workspace::undo and redo backed by operation_log"
```

---

## Phase 7 — Property test

### Task 17: Property test for project ops

**Files:**
- Create: `crates/kanban-core/tests/undo_redo_property.rs`

- [ ] **Step 1: Write the property test**

```rust
use kanban_core::operation::{CreateProject, DeleteProject, Operation, ProjectPatch, UpdateProject};
use kanban_core::types::ProjectStatus;
use kanban_core::{Workspace, new_id};
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum ProjectStep {
    Create { name: String, prefix: String },
    Update { idx: usize, new_name: String },
    Archive { idx: usize },
    Delete { idx: usize },
}

fn step_strategy() -> impl Strategy<Value = ProjectStep> {
    prop_oneof![
        ("[A-Z]{3,5}", "[a-zA-Z ]{1,12}").prop_map(|(prefix, name)| ProjectStep::Create { name, prefix }),
        (0usize..4, "[a-zA-Z]{1,8}").prop_map(|(idx, new_name)| ProjectStep::Update { idx, new_name }),
        (0usize..4).prop_map(|idx| ProjectStep::Archive { idx }),
        (0usize..4).prop_map(|idx| ProjectStep::Delete { idx }),
    ]
}

fn snapshot(ws: &Workspace) -> Vec<(String, String, ProjectStatus)> {
    let mut v: Vec<_> = ws.query_projects().unwrap().into_iter()
        .map(|p| (p.prefix.clone(), p.name.clone(), p.status))
        .collect();
    v.sort();
    v
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn undo_redo_round_trip_for_project_ops(steps in proptest::collection::vec(step_strategy(), 1..12)) {
        let mut ws = Workspace::open_in_memory().unwrap();
        let mut snapshots: Vec<Vec<(String, String, ProjectStatus)>> = vec![snapshot(&ws)];
        let mut applied = Vec::new();
        let mut ids: Vec<uuid::Uuid> = Vec::new();
        let mut used_prefixes = std::collections::HashSet::new();

        for s in &steps {
            let res = match s {
                ProjectStep::Create { name, prefix } => {
                    if used_prefixes.contains(prefix) { continue; }
                    used_prefixes.insert(prefix.clone());
                    let id = new_id();
                    ids.push(id);
                    ws.apply(Operation::CreateProject(CreateProject {
                        id, name: name.clone(), prefix: prefix.clone(),
                        description: None, icon: None,
                    }))
                }
                ProjectStep::Update { idx, new_name } => {
                    let Some(&id) = ids.get(*idx) else { continue };
                    ws.apply(Operation::UpdateProject(UpdateProject {
                        id, patch: ProjectPatch { name: Some(new_name.clone()), ..Default::default() },
                    }))
                }
                ProjectStep::Archive { idx } => {
                    let Some(&id) = ids.get(*idx) else { continue };
                    ws.apply(Operation::ArchiveProject(kanban_core::operation::ArchiveProject { id }))
                }
                ProjectStep::Delete { idx } => {
                    let Some(&id) = ids.get(*idx) else { continue };
                    ws.apply(Operation::DeleteProject(DeleteProject { id }))
                }
            };
            if res.is_ok() {
                applied.push(s.clone());
                snapshots.push(snapshot(&ws));
            }
        }

        // Undo all the way back.
        for k in (1..snapshots.len()).rev() {
            ws.undo().unwrap();
            prop_assert_eq!(snapshot(&ws), snapshots[k - 1].clone(), "after undo at step {}", k);
        }
        // Redo all the way forward.
        for k in 1..snapshots.len() {
            ws.redo().unwrap();
            prop_assert_eq!(snapshot(&ws), snapshots[k].clone(), "after redo at step {}", k);
        }
    }
}
```

- [ ] **Step 2: Run**

Run: `cargo test -p kanban-core --test undo_redo_property`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/kanban-core/tests/undo_redo_property.rs
git commit -m "test(core): property test for project op undo/redo invariants"
```

---

## Phase 8 — Issue Operations

### Task 18: `CreateIssue` with race-safe `next_seq`

**Files:**
- Create: `crates/kanban-core/src/store/write/issues.rs`
- Create: `crates/kanban-core/src/apply/issues.rs`
- Create: `crates/kanban-core/src/store/read/issues.rs`
- Create: `crates/kanban-core/tests/apply_issues.rs`
- Modify: `crates/kanban-core/src/store/write/mod.rs`
- Modify: `crates/kanban-core/src/store/read/mod.rs`
- Modify: `crates/kanban-core/src/apply/mod.rs`
- Modify: `crates/kanban-core/src/workspace.rs`

- [ ] **Step 1: Failing test**

Create `crates/kanban-core/tests/apply_issues.rs`:

```rust
use kanban_core::operation::{CreateIssue, CreateProject, Operation};
use kanban_core::types::Priority;
use kanban_core::{Workspace, new_id};
use std::sync::Arc;
use std::sync::Mutex;
use uuid::Uuid;

fn fresh_with_project() -> (Workspace, Uuid, Uuid) {
    let mut ws = Workspace::open_in_memory().unwrap();
    let project_id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: project_id, name: "T".into(), prefix: "TST".into(),
        description: None, icon: None,
    })).unwrap();
    let todo_id = ws.query_statuses_for_project(project_id).unwrap()[0].id;
    (ws, project_id, todo_id)
}

#[test]
fn create_issue_assigns_seq_1_and_identifier_with_prefix() {
    let (mut ws, pid, sid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id, project_id: pid, title: "first".into(), description: None,
        status_id: sid, priority: Priority::None, due_date: None, label_ids: vec![],
    })).unwrap();
    let issue = ws.query_issue_by_id(id).unwrap();
    assert_eq!(issue.seq, 1);
    assert_eq!(issue.identifier, "TST-1");
}

#[test]
fn create_issue_increments_seq_per_project() {
    let (mut ws, pid, sid) = fresh_with_project();
    for _ in 0..3 {
        ws.apply(Operation::CreateIssue(CreateIssue {
            id: new_id(), project_id: pid, title: "x".into(), description: None,
            status_id: sid, priority: Priority::None, due_date: None, label_ids: vec![],
        })).unwrap();
    }
    let p = ws.query_project_by_id(pid).unwrap();
    assert_eq!(p.next_seq, 4);
    let issues = ws.query_issues(kanban_core::query::IssueFilter::for_project(pid)).unwrap();
    let identifiers: Vec<_> = issues.iter().map(|i| i.identifier.clone()).collect();
    assert_eq!(identifiers, vec!["TST-1".to_string(), "TST-2".into(), "TST-3".into()]);
}

#[test]
fn create_issue_attaches_labels_in_one_op() {
    let (mut ws, pid, sid) = fresh_with_project();
    // Insert a label directly via apply once labels land — but for now we seed via the writer.
    // This test is enabled in Task 25 once AttachLabel is wired through apply.
    let _ = (ws, pid, sid);
}

#[test]
fn create_issue_validates_title_nonempty() {
    let (mut ws, pid, sid) = fresh_with_project();
    let err = ws.apply(Operation::CreateIssue(CreateIssue {
        id: new_id(), project_id: pid, title: "   ".into(), description: None,
        status_id: sid, priority: Priority::None, due_date: None, label_ids: vec![],
    })).unwrap_err();
    assert!(err.to_string().contains("title"), "{err}");
}

#[test]
fn create_issue_unknown_project_errors() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let err = ws.apply(Operation::CreateIssue(CreateIssue {
        id: new_id(), project_id: new_id(), title: "x".into(), description: None,
        status_id: new_id(), priority: Priority::None, due_date: None, label_ids: vec![],
    })).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("not found"), "{err}");
}

#[test]
fn concurrent_creates_do_not_collide() {
    // Two workspaces against the same file simulate concurrent CLI invocations.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.db");

    let project_id = new_id();
    let mut bootstrap = Workspace::open(&path).unwrap();
    bootstrap.apply(Operation::CreateProject(CreateProject {
        id: project_id, name: "C".into(), prefix: "CON".into(),
        description: None, icon: None,
    })).unwrap();
    let sid = bootstrap.query_statuses_for_project(project_id).unwrap()[0].id;
    drop(bootstrap);

    let path = Arc::new(path);
    let sid = Arc::new(Mutex::new(sid));
    let mut handles = vec![];
    for _ in 0..8 {
        let path = Arc::clone(&path);
        let sid = sid.lock().unwrap().clone();
        handles.push(std::thread::spawn(move || {
            let mut ws = Workspace::open(&path).unwrap();
            ws.apply(Operation::CreateIssue(CreateIssue {
                id: new_id(), project_id, title: "race".into(), description: None,
                status_id: sid, priority: Priority::None, due_date: None, label_ids: vec![],
            })).unwrap();
        }));
    }
    for h in handles { h.join().unwrap(); }

    let ws = Workspace::open(&path).unwrap();
    let issues = ws.query_issues(kanban_core::query::IssueFilter::for_project(project_id)).unwrap();
    assert_eq!(issues.len(), 8);
    let mut seqs: Vec<_> = issues.iter().map(|i| i.seq).collect();
    seqs.sort();
    assert_eq!(seqs, (1..=8).collect::<Vec<_>>());
}
```

- [ ] **Step 2: Run, confirm failure**

Run: `cargo test -p kanban-core --test apply_issues`
Expected: FAIL — `CreateIssue` not wired in apply, `query_issue_by_id` and `query_issues` not present.

- [ ] **Step 3: Implement issue writer**

Create `crates/kanban-core/src/store/write/issues.rs`:

```rust
use crate::error::{Error, Result};
use crate::types::{Issue, Priority};
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{Transaction, params};
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub(crate) fn insert(
    tx: &Transaction<'_>,
    id: Uuid,
    project_id: Uuid,
    title: &str,
    description: Option<&str>,
    status_id: Uuid,
    priority: Priority,
    due_date: Option<NaiveDate>,
    sort_key: f64,
    now: DateTime<Utc>,
) -> Result<Issue> {
    let now_s = now.to_rfc3339();
    // Race-safe per-project sequence allocation.
    let seq: i64 = tx.query_row(
        "UPDATE projects SET next_seq = next_seq + 1 WHERE id = ?1
         RETURNING next_seq - 1",
        params![project_id.to_string()],
        |r| r.get(0),
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Error::NotFound {
            kind: crate::EntityKind::Project,
            id: project_id.to_string(),
        },
        other => other.into(),
    })?;

    let prefix: String = tx.query_row(
        "SELECT prefix FROM projects WHERE id = ?1",
        params![project_id.to_string()],
        |r| r.get(0),
    )?;
    let identifier = format!("{prefix}-{seq}");

    tx.execute(
        "INSERT INTO issues(id,project_id,seq,identifier,title,description,status_id,
                            priority,due_date,sort_key,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?11)",
        params![
            id.to_string(),
            project_id.to_string(),
            seq,
            identifier,
            title,
            description,
            status_id.to_string(),
            priority.as_str(),
            due_date.map(|d| d.to_string()),
            sort_key,
            now_s,
        ],
    )?;
    Ok(Issue {
        id,
        project_id,
        seq,
        identifier,
        title: title.to_string(),
        description: description.map(str::to_string),
        status_id,
        priority,
        due_date,
        sort_key,
        created_at: now,
        updated_at: now,
    })
}

pub(crate) fn delete(tx: &Transaction<'_>, id: Uuid) -> Result<()> {
    tx.execute("DELETE FROM issues WHERE id = ?1", params![id.to_string()])?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn update_field(
    tx: &Transaction<'_>,
    id: Uuid,
    column: &str,
    sql_value: rusqlite::types::Value,
    now: DateTime<Utc>,
) -> Result<()> {
    let sql = format!("UPDATE issues SET {column} = ?1, updated_at = ?2 WHERE id = ?3");
    tx.execute(&sql, params![sql_value, now.to_rfc3339(), id.to_string()])?;
    Ok(())
}

pub(crate) fn set_sort_key(tx: &Transaction<'_>, id: Uuid, key: f64, now: DateTime<Utc>) -> Result<()> {
    tx.execute(
        "UPDATE issues SET sort_key = ?1, updated_at = ?2 WHERE id = ?3",
        params![key, now.to_rfc3339(), id.to_string()],
    )?;
    Ok(())
}
```

- [ ] **Step 4: Implement issue read**

Create `crates/kanban-core/src/store/read/issues.rs`:

```rust
use crate::error::{Error, Result};
use crate::types::{Issue, Priority};
use chrono::{DateTime, NaiveDate};
use rusqlite::{Connection, ToSql, params, types::Value};
use std::str::FromStr;
use uuid::Uuid;

pub(crate) fn by_id(conn: &Connection, id: Uuid) -> Result<Issue> {
    conn.query_row(
        ISSUE_SELECT,
        params![id.to_string()],
        row_to_issue,
    )
    .map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Error::NotFound {
            kind: crate::EntityKind::Issue,
            id: id.to_string(),
        },
        other => other.into(),
    })
}

pub(crate) fn by_id_via_tx(tx: &rusqlite::Transaction<'_>, id: Uuid) -> Result<Issue> {
    tx.query_row(ISSUE_SELECT, params![id.to_string()], row_to_issue)
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound {
                kind: crate::EntityKind::Issue,
                id: id.to_string(),
            },
            other => other.into(),
        })
}

pub(crate) fn list(conn: &Connection, filter: &crate::query::IssueFilter) -> Result<Vec<Issue>> {
    let (sql, params) = filter.build_sql(ISSUE_LIST_BASE);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), row_to_issue)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

const ISSUE_SELECT: &str = "
SELECT id,project_id,seq,identifier,title,description,status_id,priority,due_date,sort_key,created_at,updated_at
FROM issues WHERE id = ?1";

pub(crate) const ISSUE_LIST_BASE: &str = "
SELECT id,project_id,seq,identifier,title,description,status_id,priority,due_date,sort_key,created_at,updated_at
FROM issues";

fn row_to_issue(r: &rusqlite::Row<'_>) -> rusqlite::Result<Issue> {
    let id: String = r.get(0)?;
    let pid: String = r.get(1)?;
    let sid: String = r.get(6)?;
    let priority_s: String = r.get(7)?;
    let due_s: Option<String> = r.get(8)?;
    let created_s: String = r.get(10)?;
    let updated_s: String = r.get(11)?;
    Ok(Issue {
        id: Uuid::from_str(&id).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?,
        project_id: Uuid::from_str(&pid).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?,
        seq: r.get(2)?,
        identifier: r.get(3)?,
        title: r.get(4)?,
        description: r.get(5)?,
        status_id: Uuid::from_str(&sid).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?,
        priority: priority_s.parse().map_err(|e: crate::error::Error| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))))?,
        due_date: due_s.map(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))).transpose()?,
        sort_key: r.get(9)?,
        created_at: DateTime::parse_from_rfc3339(&created_s).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?.with_timezone(&chrono::Utc),
        updated_at: DateTime::parse_from_rfc3339(&updated_s).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?.with_timezone(&chrono::Utc),
    })
}
```

- [ ] **Step 5: Add `query` module with `IssueFilter`**

Create `crates/kanban-core/src/query.rs`:

```rust
use crate::types::Priority;
use chrono::NaiveDate;
use rusqlite::types::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Default)]
pub struct IssueFilter {
    pub project_id: Option<Uuid>,
    pub status_ids: Vec<Uuid>,
    pub priorities: Vec<Priority>,
    pub label_ids: Vec<Uuid>,
    pub due_before: Option<NaiveDate>,
    pub due_after: Option<NaiveDate>,
    pub search_text: Option<String>,
    pub sort: SortBy,
    pub reverse: bool,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Manual,
    Priority,
    Created,
    Updated,
    Due,
}

impl Default for SortBy {
    fn default() -> Self { SortBy::Manual }
}

impl IssueFilter {
    pub fn for_project(project_id: Uuid) -> Self {
        Self { project_id: Some(project_id), ..Self::default() }
    }

    pub fn build_sql(&self, base: &str) -> (String, Vec<Value>) {
        let mut sql = String::from(base);
        let mut params: Vec<Value> = Vec::new();
        let mut first = true;
        let mut and = |sql: &mut String, first: &mut bool| {
            if *first { sql.push_str(" WHERE "); *first = false; }
            else { sql.push_str(" AND "); }
        };

        if let Some(pid) = self.project_id {
            and(&mut sql, &mut first);
            sql.push_str("project_id = ?");
            params.push(Value::Text(pid.to_string()));
        }
        if !self.status_ids.is_empty() {
            and(&mut sql, &mut first);
            sql.push_str(&format!("status_id IN ({})", placeholders(self.status_ids.len())));
            for s in &self.status_ids { params.push(Value::Text(s.to_string())); }
        }
        if !self.priorities.is_empty() {
            and(&mut sql, &mut first);
            sql.push_str(&format!("priority IN ({})", placeholders(self.priorities.len())));
            for p in &self.priorities { params.push(Value::Text(p.as_str().to_string())); }
        }
        if let Some(d) = self.due_before {
            and(&mut sql, &mut first);
            sql.push_str("due_date IS NOT NULL AND due_date < ?");
            params.push(Value::Text(d.to_string()));
        }
        if let Some(d) = self.due_after {
            and(&mut sql, &mut first);
            sql.push_str("due_date IS NOT NULL AND due_date > ?");
            params.push(Value::Text(d.to_string()));
        }
        if !self.label_ids.is_empty() {
            and(&mut sql, &mut first);
            sql.push_str(&format!(
                "id IN (SELECT issue_id FROM issue_labels WHERE label_id IN ({}))",
                placeholders(self.label_ids.len())
            ));
            for l in &self.label_ids { params.push(Value::Text(l.to_string())); }
        }

        let dir = if self.reverse { "DESC" } else { "ASC" };
        let order = match self.sort {
            SortBy::Manual => format!("ORDER BY sort_key {dir}"),
            SortBy::Priority => format!(
                "ORDER BY CASE priority \
                 WHEN 'urgent' THEN 0 WHEN 'high' THEN 1 WHEN 'medium' THEN 2 \
                 WHEN 'low' THEN 3 ELSE 4 END {dir}, created_at {dir}"
            ),
            SortBy::Created => format!("ORDER BY created_at {dir}"),
            SortBy::Updated => format!("ORDER BY updated_at {dir}"),
            SortBy::Due => format!("ORDER BY due_date IS NULL, due_date {dir}"),
        };
        sql.push(' ');
        sql.push_str(&order);
        if let Some(n) = self.limit {
            sql.push_str(" LIMIT ?");
            params.push(Value::Integer(n));
        }
        (sql, params)
    }
}

fn placeholders(n: usize) -> String {
    std::iter::repeat("?").take(n).collect::<Vec<_>>().join(",")
}
```

Wire into `lib.rs`:

```rust
pub mod query;
```

- [ ] **Step 6: Implement `apply` for `CreateIssue`**

Create `crates/kanban-core/src/apply/issues.rs`:

```rust
use crate::error::{Error, Result};
use crate::operation::{CreateIssue, DeleteIssue, Operation};
use crate::store::write::issues as wi;
use crate::validate;
use chrono::{DateTime, Utc};
use rusqlite::{Transaction, params};
use uuid::Uuid;

pub(crate) fn create(tx: &Transaction<'_>, args: &CreateIssue, now: DateTime<Utc>) -> Result<()> {
    validate::nonempty_field("title", &args.title)?;

    // Project + status FKs validated implicitly by INSERT, but produce a clearer error.
    let project_exists: bool = tx.query_row(
        "SELECT COUNT(*) FROM projects WHERE id = ?1",
        params![args.project_id.to_string()],
        |r| r.get::<_, i64>(0).map(|n| n > 0),
    )?;
    if !project_exists {
        return Err(Error::NotFound {
            kind: crate::EntityKind::Project,
            id: args.project_id.to_string(),
        });
    }

    let status_exists: bool = tx.query_row(
        "SELECT COUNT(*) FROM statuses WHERE id = ?1 AND project_id = ?2",
        params![args.status_id.to_string(), args.project_id.to_string()],
        |r| r.get::<_, i64>(0).map(|n| n > 0),
    )?;
    if !status_exists {
        return Err(Error::NotFound {
            kind: crate::EntityKind::Status,
            id: args.status_id.to_string(),
        });
    }

    // Sort key: place at end. Compute MAX(sort_key)+1.0.
    let max_sort: f64 = tx.query_row(
        "SELECT COALESCE(MAX(sort_key), 0.0) FROM issues
         WHERE project_id = ?1 AND status_id = ?2",
        params![args.project_id.to_string(), args.status_id.to_string()],
        |r| r.get(0),
    )?;

    wi::insert(
        tx,
        args.id,
        args.project_id,
        validate::nonempty_field("title", &args.title)?,
        args.description.as_deref(),
        args.status_id,
        args.priority,
        args.due_date,
        max_sort + 1.0,
        now,
    )?;

    for label_id in &args.label_ids {
        tx.execute(
            "INSERT INTO issue_labels(issue_id, label_id) VALUES (?1, ?2)",
            params![args.id.to_string(), label_id.to_string()],
        )?;
    }

    Ok(())
}

pub(crate) fn delete(tx: &Transaction<'_>, args: &DeleteIssue) -> Result<()> {
    let exists: bool = tx.query_row(
        "SELECT COUNT(*) FROM issues WHERE id = ?1",
        params![args.id.to_string()],
        |r| r.get::<_, i64>(0).map(|n| n > 0),
    )?;
    if !exists {
        return Err(Error::NotFound { kind: crate::EntityKind::Issue, id: args.id.to_string() });
    }
    wi::delete(tx, args.id)?;
    Ok(())
}

pub(crate) fn inverse_of_create(args: &CreateIssue) -> Operation {
    Operation::DeleteIssue(DeleteIssue { id: args.id })
}

pub(crate) fn inverse_of_delete(tx: &Transaction<'_>, args: &DeleteIssue) -> Result<Operation> {
    let issue = crate::store::read::issues::by_id_via_tx(tx, args.id)?;
    let mut label_ids = Vec::new();
    let mut stmt = tx.prepare("SELECT label_id FROM issue_labels WHERE issue_id = ?1")?;
    let rows = stmt.query_map(params![args.id.to_string()], |r| r.get::<_, String>(0))?;
    for r in rows {
        let id_s = r?;
        label_ids.push(Uuid::parse_str(&id_s).map_err(|e| Error::InvalidSnapshot(e.to_string()))?);
    }
    Ok(Operation::CreateIssue(CreateIssue {
        id: issue.id,
        project_id: issue.project_id,
        title: issue.title,
        description: issue.description,
        status_id: issue.status_id,
        priority: issue.priority,
        due_date: issue.due_date,
        label_ids,
    }))
}
```

- [ ] **Step 7: Wire issue dispatch in `apply/mod.rs`**

Add to the `dispatch` and `capture_inverse` matches:

```rust
pub(crate) mod issues;

pub(crate) fn dispatch(tx: &rusqlite::Transaction<'_>, op: &Operation, now: chrono::DateTime<chrono::Utc>) -> Result<()> {
    match op {
        Operation::CreateProject(args) => projects::create(tx, args, now)?,
        Operation::UpdateProject(args) => projects::update(tx, args, now)?,
        Operation::ArchiveProject(args) => projects::archive(tx, args, now)?,
        Operation::DeleteProject(args) => projects::delete(tx, args)?,
        Operation::CreateIssue(args) => issues::create(tx, args, now)?,
        Operation::DeleteIssue(args) => issues::delete(tx, args)?,
        other => return Err(Error::InvalidSnapshot(format!("unsupported op: {other:?}"))),
    }
    Ok(())
}

fn capture_inverse(tx: &rusqlite::Transaction<'_>, op: &Operation) -> Result<Operation> {
    match op {
        Operation::CreateProject(args) => Ok(Operation::DeleteProject(DeleteProject { id: args.id })),
        Operation::DeleteProject(args) => projects::inverse_of_delete(tx, args),
        Operation::UpdateProject(args) => projects::inverse_of_update(tx, args),
        Operation::ArchiveProject(args) => projects::inverse_of_archive(tx, args),
        Operation::CreateIssue(args) => Ok(issues::inverse_of_create(args)),
        Operation::DeleteIssue(args) => issues::inverse_of_delete(tx, args),
        other => Err(Error::InvalidSnapshot(format!("inverse not yet implemented for {other:?}"))),
    }
}
```

- [ ] **Step 8: Add `query_issue_by_id` and `query_issues`**

Append to `crates/kanban-core/src/workspace.rs`:

```rust
use crate::query::IssueFilter;
use crate::types::Issue;

impl Workspace {
    pub fn query_issue_by_id(&self, id: Uuid) -> crate::error::Result<Issue> {
        crate::store::read::issues::by_id(&self.conn, id)
    }
    pub fn query_issues(&self, filter: IssueFilter) -> crate::error::Result<Vec<Issue>> {
        crate::store::read::issues::list(&self.conn, &filter)
    }
}
```

Update `crates/kanban-core/src/store/read/mod.rs`:

```rust
pub(crate) mod issues;
pub(crate) mod projects;
pub(crate) mod statuses;
```

Update `crates/kanban-core/src/store/write/mod.rs`:

```rust
pub(crate) mod issues;
pub(crate) mod operation_log;
pub(crate) mod projects;
pub(crate) mod statuses;
```

- [ ] **Step 9: Run**

Run: `cargo test -p kanban-core --test apply_issues`
Expected: PASS (excluding the placeholder label-attachment test, which should still compile because it does nothing).

- [ ] **Step 10: Commit**

```bash
git add crates/kanban-core/
git commit -m "feat(core): add CreateIssue/DeleteIssue with race-safe next_seq allocation"
```

---

### Task 19: `UpdateIssueField`

**Files:**
- Modify: `crates/kanban-core/src/apply/issues.rs`
- Modify: `crates/kanban-core/src/apply/mod.rs`
- Modify: `crates/kanban-core/tests/apply_issues.rs`

- [ ] **Step 1: Failing tests**

Append to `crates/kanban-core/tests/apply_issues.rs`:

```rust
use kanban_core::operation::{IssueFieldChange, UpdateIssueField};

#[test]
fn update_issue_title() {
    let (mut ws, pid, sid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id, project_id: pid, title: "old".into(), description: None,
        status_id: sid, priority: Priority::None, due_date: None, label_ids: vec![],
    })).unwrap();
    ws.apply(Operation::UpdateIssueField(UpdateIssueField {
        id, change: IssueFieldChange::Title("new".into()),
    })).unwrap();
    assert_eq!(ws.query_issue_by_id(id).unwrap().title, "new");
}

#[test]
fn update_issue_priority_and_undo_restores_it() {
    let (mut ws, pid, sid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id, project_id: pid, title: "p".into(), description: None,
        status_id: sid, priority: Priority::High, due_date: None, label_ids: vec![],
    })).unwrap();
    ws.apply(Operation::UpdateIssueField(UpdateIssueField {
        id, change: IssueFieldChange::Priority(Priority::Low),
    })).unwrap();
    assert_eq!(ws.query_issue_by_id(id).unwrap().priority, Priority::Low);
    ws.undo().unwrap();
    assert_eq!(ws.query_issue_by_id(id).unwrap().priority, Priority::High);
}

#[test]
fn update_issue_status_to_status_in_other_project_errors() {
    let (mut ws, pid_a, sid_a) = fresh_with_project();
    // create a second project
    let pid_b = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid_b, name: "B".into(), prefix: "BTW".into(), description: None, icon: None,
    })).unwrap();
    let sid_b = ws.query_statuses_for_project(pid_b).unwrap()[0].id;

    let id = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id, project_id: pid_a, title: "x".into(), description: None,
        status_id: sid_a, priority: Priority::None, due_date: None, label_ids: vec![],
    })).unwrap();

    let err = ws.apply(Operation::UpdateIssueField(UpdateIssueField {
        id, change: IssueFieldChange::Status(sid_b),
    })).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("status"), "{err}");
}
```

- [ ] **Step 2: Implement `update_field`**

Append to `crates/kanban-core/src/apply/issues.rs`:

```rust
use crate::operation::{IssueFieldChange, UpdateIssueField};
use rusqlite::types::Value;

pub(crate) fn update_field(tx: &Transaction<'_>, args: &UpdateIssueField, now: DateTime<Utc>) -> Result<()> {
    let issue = crate::store::read::issues::by_id_via_tx(tx, args.id)?;
    match &args.change {
        IssueFieldChange::Title(new) => {
            crate::validate::nonempty_field("title", new)?;
            crate::store::write::issues::update_field(tx, args.id, "title", Value::Text(new.clone()), now)?;
        }
        IssueFieldChange::Description(new) => {
            let v = match new { Some(s) => Value::Text(s.clone()), None => Value::Null };
            crate::store::write::issues::update_field(tx, args.id, "description", v, now)?;
        }
        IssueFieldChange::Status(new) => {
            // Status must belong to the same project.
            let same_project: bool = tx.query_row(
                "SELECT COUNT(*) FROM statuses WHERE id = ?1 AND project_id = ?2",
                params![new.to_string(), issue.project_id.to_string()],
                |r| r.get::<_, i64>(0).map(|n| n > 0),
            )?;
            if !same_project {
                return Err(Error::Validation(crate::error::ValidationError {
                    field: "status".into(),
                    reason: "must belong to the same project".into(),
                }));
            }
            crate::store::write::issues::update_field(tx, args.id, "status_id", Value::Text(new.to_string()), now)?;
        }
        IssueFieldChange::Priority(new) => {
            crate::store::write::issues::update_field(tx, args.id, "priority", Value::Text(new.as_str().to_string()), now)?;
        }
        IssueFieldChange::DueDate(new) => {
            let v = match new { Some(d) => Value::Text(d.to_string()), None => Value::Null };
            crate::store::write::issues::update_field(tx, args.id, "due_date", v, now)?;
        }
    }
    Ok(())
}

pub(crate) fn inverse_of_update_field(tx: &Transaction<'_>, args: &UpdateIssueField) -> Result<Operation> {
    let issue = crate::store::read::issues::by_id_via_tx(tx, args.id)?;
    let inverse_change = match &args.change {
        IssueFieldChange::Title(_) => IssueFieldChange::Title(issue.title),
        IssueFieldChange::Description(_) => IssueFieldChange::Description(issue.description),
        IssueFieldChange::Status(_) => IssueFieldChange::Status(issue.status_id),
        IssueFieldChange::Priority(_) => IssueFieldChange::Priority(issue.priority),
        IssueFieldChange::DueDate(_) => IssueFieldChange::DueDate(issue.due_date),
    };
    Ok(Operation::UpdateIssueField(UpdateIssueField {
        id: args.id, change: inverse_change,
    }))
}
```

- [ ] **Step 3: Wire into dispatch and capture_inverse**

In `crates/kanban-core/src/apply/mod.rs`, add to the `dispatch` match:

```rust
Operation::UpdateIssueField(args) => issues::update_field(tx, args, now)?,
```

And in `capture_inverse`:

```rust
Operation::UpdateIssueField(args) => issues::inverse_of_update_field(tx, args),
```

- [ ] **Step 4: Run**

Run: `cargo test -p kanban-core --test apply_issues`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/kanban-core/
git commit -m "feat(core): add UpdateIssueField across all 5 fields with field-level inverse"
```

---

### Task 20: `ReorderIssue` + sort key math

**Files:**
- Modify: `crates/kanban-core/src/apply/issues.rs`
- Modify: `crates/kanban-core/src/apply/mod.rs`
- Modify: `crates/kanban-core/tests/apply_issues.rs`

- [ ] **Step 1: Failing tests**

Append:

```rust
use kanban_core::operation::ReorderIssue;
use kanban_core::query::IssueFilter;

#[test]
fn reorder_changes_sort_key_and_listing_order() {
    let (mut ws, pid, sid) = fresh_with_project();
    let mut ids = Vec::new();
    for _ in 0..3 {
        let id = new_id();
        ids.push(id);
        ws.apply(Operation::CreateIssue(CreateIssue {
            id, project_id: pid, title: "x".into(), description: None,
            status_id: sid, priority: Priority::None, due_date: None, label_ids: vec![],
        })).unwrap();
    }
    // Move third before first.
    let issues = ws.query_issues(IssueFilter::for_project(pid)).unwrap();
    let first_key = issues[0].sort_key;
    ws.apply(Operation::ReorderIssue(ReorderIssue {
        id: ids[2], new_sort_key: first_key - 1.0,
    })).unwrap();
    let issues = ws.query_issues(IssueFilter::for_project(pid)).unwrap();
    assert_eq!(issues[0].id, ids[2]);
}
```

- [ ] **Step 2: Implement**

Append to `crates/kanban-core/src/apply/issues.rs`:

```rust
use crate::operation::ReorderIssue;

pub(crate) fn reorder(tx: &Transaction<'_>, args: &ReorderIssue, now: DateTime<Utc>) -> Result<()> {
    let _ = crate::store::read::issues::by_id_via_tx(tx, args.id)?;
    crate::store::write::issues::set_sort_key(tx, args.id, args.new_sort_key, now)?;
    Ok(())
}

pub(crate) fn inverse_of_reorder(tx: &Transaction<'_>, args: &ReorderIssue) -> Result<Operation> {
    let issue = crate::store::read::issues::by_id_via_tx(tx, args.id)?;
    Ok(Operation::ReorderIssue(ReorderIssue {
        id: args.id, new_sort_key: issue.sort_key,
    }))
}
```

Add to dispatch and capture_inverse:

```rust
Operation::ReorderIssue(args) => issues::reorder(tx, args, now)?,
// ...
Operation::ReorderIssue(args) => issues::inverse_of_reorder(tx, args),
```

- [ ] **Step 3: Run + commit**

Run: `cargo test -p kanban-core --test apply_issues`
Expected: PASS.

```bash
git add crates/kanban-core/
git commit -m "feat(core): add ReorderIssue with sort_key math"
```

---

### Task 21: Issue list filters integration tests

**Files:**
- Create: `crates/kanban-core/tests/query_issues.rs`

- [ ] **Step 1: Write the tests**

```rust
use chrono::NaiveDate;
use kanban_core::operation::{CreateIssue, CreateProject, Operation};
use kanban_core::query::{IssueFilter, SortBy};
use kanban_core::types::Priority;
use kanban_core::{Workspace, new_id};

fn seeded() -> (Workspace, uuid::Uuid) {
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid, name: "Q".into(), prefix: "QRY".into(), description: None, icon: None,
    })).unwrap();
    let statuses = ws.query_statuses_for_project(pid).unwrap();
    let todo = statuses[0].id;
    let inprog = statuses[2].id;
    for (title, st, pr, due) in [
        ("a-todo-high", todo, Priority::High, Some("2026-06-01")),
        ("b-todo-low", todo, Priority::Low, None),
        ("c-prog-medium", inprog, Priority::Medium, Some("2026-05-15")),
    ] {
        ws.apply(Operation::CreateIssue(CreateIssue {
            id: new_id(), project_id: pid, title: title.into(), description: None,
            status_id: st, priority: pr,
            due_date: due.map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap()),
            label_ids: vec![],
        })).unwrap();
    }
    (ws, pid)
}

#[test]
fn filter_by_status() {
    let (ws, pid) = seeded();
    let todo = ws.query_statuses_for_project(pid).unwrap()[0].id;
    let issues = ws.query_issues(IssueFilter {
        project_id: Some(pid), status_ids: vec![todo], ..Default::default()
    }).unwrap();
    assert_eq!(issues.len(), 2);
}

#[test]
fn filter_by_priority() {
    let (ws, pid) = seeded();
    let issues = ws.query_issues(IssueFilter {
        project_id: Some(pid), priorities: vec![Priority::High, Priority::Medium], ..Default::default()
    }).unwrap();
    assert_eq!(issues.len(), 2);
}

#[test]
fn filter_due_before() {
    let (ws, pid) = seeded();
    let cutoff = NaiveDate::parse_from_str("2026-05-20", "%Y-%m-%d").unwrap();
    let issues = ws.query_issues(IssueFilter {
        project_id: Some(pid), due_before: Some(cutoff), ..Default::default()
    }).unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].title, "c-prog-medium");
}

#[test]
fn sort_by_priority() {
    let (ws, pid) = seeded();
    let issues = ws.query_issues(IssueFilter {
        project_id: Some(pid), sort: SortBy::Priority, ..Default::default()
    }).unwrap();
    let titles: Vec<_> = issues.iter().map(|i| i.title.clone()).collect();
    assert_eq!(titles, vec![
        "a-todo-high".to_string(),
        "c-prog-medium".into(),
        "b-todo-low".into(),
    ]);
}

#[test]
fn limit_caps_results() {
    let (ws, pid) = seeded();
    let issues = ws.query_issues(IssueFilter {
        project_id: Some(pid), limit: Some(2), ..Default::default()
    }).unwrap();
    assert_eq!(issues.len(), 2);
}
```

- [ ] **Step 2: Run + commit**

Run: `cargo test -p kanban-core --test query_issues`
Expected: PASS.

```bash
git add crates/kanban-core/tests/query_issues.rs
git commit -m "test(core): cover IssueFilter combinators (status, priority, due, sort, limit)"
```

---

### Task 22: Extend property test to include issue ops

**Files:**
- Modify: `crates/kanban-core/tests/undo_redo_property.rs`

- [ ] **Step 1: Append a new strategy and proptest**

```rust
use kanban_core::operation::{
    CreateIssue, DeleteIssue, IssueFieldChange, ReorderIssue, UpdateIssueField,
};
use kanban_core::query::IssueFilter;
use kanban_core::types::Priority;

#[derive(Debug, Clone)]
enum IssueStep {
    Create { title: String },
    UpdateTitle { idx: usize, title: String },
    UpdatePriority { idx: usize, priority: Priority },
    Reorder { idx: usize, key: f64 },
    Delete { idx: usize },
}

fn issue_step() -> impl Strategy<Value = IssueStep> {
    let priorities = prop_oneof![
        Just(Priority::None), Just(Priority::Urgent), Just(Priority::High),
        Just(Priority::Medium), Just(Priority::Low),
    ];
    prop_oneof![
        "[a-zA-Z ]{1,12}".prop_map(|t| IssueStep::Create { title: t }),
        (0usize..6, "[a-zA-Z]{1,8}").prop_map(|(idx, title)| IssueStep::UpdateTitle { idx, title }),
        (0usize..6, priorities.clone()).prop_map(|(idx, priority)| IssueStep::UpdatePriority { idx, priority }),
        (0usize..6, -10.0f64..10.0f64).prop_map(|(idx, key)| IssueStep::Reorder { idx, key }),
        (0usize..6).prop_map(|idx| IssueStep::Delete { idx }),
    ]
}

fn issue_snapshot(ws: &Workspace, pid: uuid::Uuid) -> Vec<(String, Priority, f64)> {
    let mut v: Vec<_> = ws.query_issues(IssueFilter::for_project(pid)).unwrap()
        .into_iter().map(|i| (i.title, i.priority, i.sort_key)).collect();
    v.sort_by(|a, b| a.0.cmp(&b.0).then(a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal)));
    v
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn undo_redo_round_trip_for_issue_ops(steps in proptest::collection::vec(issue_step(), 1..10)) {
        let mut ws = Workspace::open_in_memory().unwrap();
        let pid = new_id();
        ws.apply(Operation::CreateProject(CreateProject {
            id: pid, name: "P".into(), prefix: "PROP".into(),
            description: None, icon: None,
        })).unwrap();
        let sid = ws.query_statuses_for_project(pid).unwrap()[0].id;

        let mut snapshots = vec![issue_snapshot(&ws, pid)];
        let mut ids: Vec<uuid::Uuid> = Vec::new();
        for s in &steps {
            let res = match s {
                IssueStep::Create { title } => {
                    let id = new_id();
                    let r = ws.apply(Operation::CreateIssue(CreateIssue {
                        id, project_id: pid, title: title.clone(), description: None,
                        status_id: sid, priority: Priority::None, due_date: None, label_ids: vec![],
                    }));
                    if r.is_ok() { ids.push(id); }
                    r
                }
                IssueStep::UpdateTitle { idx, title } => {
                    let Some(&id) = ids.get(*idx) else { continue };
                    ws.apply(Operation::UpdateIssueField(UpdateIssueField {
                        id, change: IssueFieldChange::Title(title.clone()),
                    }))
                }
                IssueStep::UpdatePriority { idx, priority } => {
                    let Some(&id) = ids.get(*idx) else { continue };
                    ws.apply(Operation::UpdateIssueField(UpdateIssueField {
                        id, change: IssueFieldChange::Priority(*priority),
                    }))
                }
                IssueStep::Reorder { idx, key } => {
                    let Some(&id) = ids.get(*idx) else { continue };
                    ws.apply(Operation::ReorderIssue(ReorderIssue { id, new_sort_key: *key }))
                }
                IssueStep::Delete { idx } => {
                    let Some(&id) = ids.get(*idx) else { continue };
                    ws.apply(Operation::DeleteIssue(DeleteIssue { id }))
                }
            };
            if res.is_ok() { snapshots.push(issue_snapshot(&ws, pid)); }
        }

        for k in (1..snapshots.len()).rev() {
            ws.undo().unwrap();
            prop_assert_eq!(issue_snapshot(&ws, pid), snapshots[k - 1].clone());
        }
        for k in 1..snapshots.len() {
            ws.redo().unwrap();
            prop_assert_eq!(issue_snapshot(&ws, pid), snapshots[k].clone());
        }
    }
}
```

- [ ] **Step 2: Run**

Run: `cargo test -p kanban-core --test undo_redo_property`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/kanban-core/tests/undo_redo_property.rs
git commit -m "test(core): extend undo/redo property test to issue ops"
```

---

### Task 23: Issue activity log writes

The activity_log was added to the schema but the issue ops don't yet emit rows. This task wires it for `UpdateIssueField` (the only op that produces field-level history rows in v1).

**Files:**
- Modify: `crates/kanban-core/src/apply/issues.rs`
- Modify: `crates/kanban-core/src/apply/mod.rs`
- Create: `crates/kanban-core/src/store/read/log.rs`
- Modify: `crates/kanban-core/src/workspace.rs`
- Modify: `crates/kanban-core/src/store/read/mod.rs`
- Create: `crates/kanban-core/tests/activity_log.rs`

- [ ] **Step 1: Failing test**

Create `crates/kanban-core/tests/activity_log.rs`:

```rust
use kanban_core::operation::{CreateIssue, CreateProject, IssueFieldChange, Operation, UpdateIssueField};
use kanban_core::types::Priority;
use kanban_core::{Workspace, new_id};

#[test]
fn activity_log_records_priority_change() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid, name: "X".into(), prefix: "ACT".into(), description: None, icon: None,
    })).unwrap();
    let sid = ws.query_statuses_for_project(pid).unwrap()[0].id;
    let id = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id, project_id: pid, title: "t".into(), description: None,
        status_id: sid, priority: Priority::Low, due_date: None, label_ids: vec![],
    })).unwrap();
    ws.apply(Operation::UpdateIssueField(UpdateIssueField {
        id, change: IssueFieldChange::Priority(Priority::High),
    })).unwrap();

    let entries = ws.query_issue_history(id).unwrap();
    let priority_changes: Vec<_> = entries.iter().filter(|e| e.field == "priority").collect();
    assert_eq!(priority_changes.len(), 1);
    assert_eq!(priority_changes[0].old_value.as_deref(), Some("low"));
    assert_eq!(priority_changes[0].new_value.as_deref(), Some("high"));
}
```

- [ ] **Step 2: Add the `ActivityEntry` type and reader**

Append to `crates/kanban-core/src/types.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityEntry {
    pub id: i64,
    pub op_id: i64,
    pub issue_id: Option<Uuid>,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub at: DateTime<Utc>,
}
```

Re-export from `lib.rs`: append `ActivityEntry` to the `pub use types::{...}` line.

Create `crates/kanban-core/src/store/read/log.rs`:

```rust
use crate::error::Result;
use crate::types::ActivityEntry;
use chrono::DateTime;
use rusqlite::{Connection, params};
use std::str::FromStr;
use uuid::Uuid;

pub(crate) fn for_issue(conn: &Connection, issue_id: Uuid) -> Result<Vec<ActivityEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, op_id, issue_id, field, old_value, new_value, at
         FROM activity_log WHERE issue_id = ?1 ORDER BY id ASC",
    )?;
    let rows = stmt.query_map(params![issue_id.to_string()], row_to_entry)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

fn row_to_entry(r: &rusqlite::Row<'_>) -> rusqlite::Result<ActivityEntry> {
    let issue_id: Option<String> = r.get(2)?;
    let at_s: String = r.get(6)?;
    Ok(ActivityEntry {
        id: r.get(0)?,
        op_id: r.get(1)?,
        issue_id: issue_id.map(|s| Uuid::from_str(&s).unwrap()),
        field: r.get(3)?,
        old_value: r.get(4)?,
        new_value: r.get(5)?,
        at: DateTime::parse_from_rfc3339(&at_s).unwrap().with_timezone(&chrono::Utc),
    })
}
```

Update `crates/kanban-core/src/store/read/mod.rs`:

```rust
pub(crate) mod issues;
pub(crate) mod log;
pub(crate) mod projects;
pub(crate) mod statuses;
```

Append to `crates/kanban-core/src/workspace.rs`:

```rust
use crate::types::ActivityEntry;

impl Workspace {
    pub fn query_issue_history(&self, issue_id: Uuid) -> crate::error::Result<Vec<ActivityEntry>> {
        crate::store::read::log::for_issue(&self.conn, issue_id)
    }
}
```

- [ ] **Step 3: Emit activity_log rows from `update_field`**

`apply` is responsible for inserting into `operation_log` and getting back the `op_id`, then passing it to per-op activity emitters. Refactor `apply` so it returns the `op_id` early and the per-op functions can write activity rows.

In `crates/kanban-core/src/apply/mod.rs`, restructure:

```rust
impl Workspace {
    pub fn apply(&mut self, op: Operation) -> Result<OperationOutcome> {
        let now = self.clock.now();
        let payload = serde_json::to_string(&op)?;
        let tx = self.conn.transaction()?;
        operation_log::truncate_redo_branch(&tx)?;

        let inverse = capture_inverse(&tx, &op)?;
        let inverse_payload = serde_json::to_string(&inverse)?;

        let op_id = operation_log::insert_operation(
            &tx, op_type_name(&op), &payload, &inverse_payload, now,
        )?;

        // Capture pre-state for activity log emission, then mutate, then emit.
        let pre = capture_activity_pre(&tx, &op)?;
        dispatch(&tx, &op, now)?;
        emit_activity(&tx, op_id, &op, pre, now)?;

        tx.commit()?;
        Ok(OperationOutcome { op_id })
    }
}

#[derive(Default)]
pub(crate) struct ActivityPre {
    pub(crate) issue_pre: Option<crate::types::Issue>,
}

fn capture_activity_pre(tx: &rusqlite::Transaction<'_>, op: &Operation) -> Result<ActivityPre> {
    let mut pre = ActivityPre::default();
    if let Operation::UpdateIssueField(args) = op {
        pre.issue_pre = Some(crate::store::read::issues::by_id_via_tx(tx, args.id)?);
    }
    Ok(pre)
}

fn emit_activity(
    tx: &rusqlite::Transaction<'_>,
    op_id: i64,
    op: &Operation,
    pre: ActivityPre,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
    if let Operation::UpdateIssueField(args) = op {
        let pre_issue = pre.issue_pre.expect("issue_pre captured for UpdateIssueField");
        let (field, old, new) = match &args.change {
            crate::operation::IssueFieldChange::Title(v) =>
                ("title", Some(pre_issue.title), Some(v.clone())),
            crate::operation::IssueFieldChange::Description(v) =>
                ("description", pre_issue.description, v.clone()),
            crate::operation::IssueFieldChange::Status(v) =>
                ("status", Some(pre_issue.status_id.to_string()), Some(v.to_string())),
            crate::operation::IssueFieldChange::Priority(v) =>
                ("priority", Some(pre_issue.priority.as_str().to_string()), Some(v.as_str().to_string())),
            crate::operation::IssueFieldChange::DueDate(v) =>
                ("due_date", pre_issue.due_date.map(|d| d.to_string()), v.map(|d| d.to_string())),
        };
        let issue_id_s = args.id.to_string();
        crate::store::write::operation_log::insert_activity(
            tx,
            op_id,
            Some(issue_id_s.as_str()),
            field,
            old.as_deref(),
            new.as_deref(),
            now,
        )?;
    }
    Ok(())
}
```

Note: this adds activity_log emission only for `UpdateIssueField` — the spec calls activity_log "purely derivative" for the timeline view (§6 notes the timeline as `kanban issue history`). Other ops can be added later without changing the schema.

- [ ] **Step 4: Run**

Run: `cargo test -p kanban-core --test activity_log`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/kanban-core/
git commit -m "feat(core): emit activity_log rows for UpdateIssueField; expose query_issue_history"
```

---

## Phase 9 — Label Operations

### Task 24: `CreateLabel`, `UpdateLabel`, `DeleteLabel`

**Files:**
- Create: `crates/kanban-core/src/store/write/labels.rs`
- Create: `crates/kanban-core/src/apply/labels.rs`
- Create: `crates/kanban-core/src/store/read/labels.rs`
- Create: `crates/kanban-core/tests/apply_labels.rs`
- Modify: `crates/kanban-core/src/apply/mod.rs`, `store/{read,write}/mod.rs`, `workspace.rs`

- [ ] **Step 1: Failing tests**

```rust
// tests/apply_labels.rs
use kanban_core::operation::{CreateLabel, CreateProject, DeleteLabel, LabelPatch, Operation, UpdateLabel};
use kanban_core::{Workspace, new_id};

fn fresh_with_project() -> (Workspace, uuid::Uuid) {
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid, name: "L".into(), prefix: "LBL".into(), description: None, icon: None,
    })).unwrap();
    (ws, pid)
}

#[test]
fn create_label_inserts() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateLabel(CreateLabel {
        id, project_id: pid, name: "backend".into(), color: "#3b82f6".into(),
    })).unwrap();
    let labels = ws.query_labels_for_project(pid).unwrap();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].name, "backend");
}

#[test]
fn create_label_rejects_duplicate_name_per_project() {
    let (mut ws, pid) = fresh_with_project();
    ws.apply(Operation::CreateLabel(CreateLabel {
        id: new_id(), project_id: pid, name: "dup".into(), color: "#000000".into(),
    })).unwrap();
    let err = ws.apply(Operation::CreateLabel(CreateLabel {
        id: new_id(), project_id: pid, name: "dup".into(), color: "#000000".into(),
    })).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("conflict"), "{err}");
}

#[test]
fn create_label_validates_color() {
    let (mut ws, pid) = fresh_with_project();
    let err = ws.apply(Operation::CreateLabel(CreateLabel {
        id: new_id(), project_id: pid, name: "x".into(), color: "blue".into(),
    })).unwrap_err();
    assert!(err.to_string().contains("color"), "{err}");
}

#[test]
fn update_label_renames() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateLabel(CreateLabel {
        id, project_id: pid, name: "old".into(), color: "#000000".into(),
    })).unwrap();
    ws.apply(Operation::UpdateLabel(UpdateLabel {
        id, patch: LabelPatch { name: Some("new".into()), ..Default::default() },
    })).unwrap();
    let labels = ws.query_labels_for_project(pid).unwrap();
    assert_eq!(labels[0].name, "new");
}

#[test]
fn delete_label_undo_restores_it() {
    let (mut ws, pid) = fresh_with_project();
    let id = new_id();
    ws.apply(Operation::CreateLabel(CreateLabel {
        id, project_id: pid, name: "kept".into(), color: "#000000".into(),
    })).unwrap();
    ws.apply(Operation::DeleteLabel(DeleteLabel { id })).unwrap();
    assert!(ws.query_labels_for_project(pid).unwrap().is_empty());
    ws.undo().unwrap();
    assert_eq!(ws.query_labels_for_project(pid).unwrap().len(), 1);
}
```

- [ ] **Step 2: Implement writer**

Create `crates/kanban-core/src/store/write/labels.rs`:

```rust
use crate::error::Result;
use rusqlite::{Transaction, params};
use uuid::Uuid;

pub(crate) fn insert(tx: &Transaction<'_>, id: Uuid, project_id: Uuid, name: &str, color: &str) -> Result<()> {
    tx.execute(
        "INSERT INTO labels(id,project_id,name,color) VALUES (?1,?2,?3,?4)",
        params![id.to_string(), project_id.to_string(), name, color],
    )?;
    Ok(())
}

pub(crate) fn delete(tx: &Transaction<'_>, id: Uuid) -> Result<()> {
    tx.execute("DELETE FROM labels WHERE id = ?1", params![id.to_string()])?;
    Ok(())
}

pub(crate) fn update_fields(
    tx: &Transaction<'_>, id: Uuid, name: Option<&str>, color: Option<&str>,
) -> Result<()> {
    if let Some(v) = name {
        tx.execute("UPDATE labels SET name = ?1 WHERE id = ?2", params![v, id.to_string()])?;
    }
    if let Some(v) = color {
        tx.execute("UPDATE labels SET color = ?1 WHERE id = ?2", params![v, id.to_string()])?;
    }
    Ok(())
}

pub(crate) fn attach(tx: &Transaction<'_>, issue_id: Uuid, label_id: Uuid) -> Result<()> {
    tx.execute(
        "INSERT OR IGNORE INTO issue_labels(issue_id, label_id) VALUES (?1, ?2)",
        params![issue_id.to_string(), label_id.to_string()],
    )?;
    Ok(())
}

pub(crate) fn detach(tx: &Transaction<'_>, issue_id: Uuid, label_id: Uuid) -> Result<()> {
    tx.execute(
        "DELETE FROM issue_labels WHERE issue_id = ?1 AND label_id = ?2",
        params![issue_id.to_string(), label_id.to_string()],
    )?;
    Ok(())
}
```

- [ ] **Step 3: Implement read**

Create `crates/kanban-core/src/store/read/labels.rs`:

```rust
use crate::error::{Error, Result};
use crate::types::Label;
use rusqlite::{Connection, params};
use std::str::FromStr;
use uuid::Uuid;

pub(crate) fn for_project(conn: &Connection, project_id: Uuid) -> Result<Vec<Label>> {
    let mut stmt = conn.prepare("SELECT id,project_id,name,color FROM labels WHERE project_id = ?1 ORDER BY name")?;
    let rows = stmt.query_map(params![project_id.to_string()], row_to_label)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub(crate) fn by_id_via_tx(tx: &rusqlite::Transaction<'_>, id: Uuid) -> Result<Label> {
    tx.query_row(
        "SELECT id,project_id,name,color FROM labels WHERE id = ?1",
        params![id.to_string()],
        row_to_label,
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => Error::NotFound { kind: crate::EntityKind::Label, id: id.to_string() },
        other => other.into(),
    })
}

fn row_to_label(r: &rusqlite::Row<'_>) -> rusqlite::Result<Label> {
    let id: String = r.get(0)?;
    let pid: String = r.get(1)?;
    Ok(Label {
        id: Uuid::from_str(&id).unwrap(),
        project_id: Uuid::from_str(&pid).unwrap(),
        name: r.get(2)?,
        color: r.get(3)?,
    })
}
```

- [ ] **Step 4: Implement applier**

Create `crates/kanban-core/src/apply/labels.rs`:

```rust
use crate::error::{Error, Result};
use crate::operation::*;
use crate::store::write::labels as wl;
use crate::validate;
use rusqlite::{Transaction, params};
use uuid::Uuid;

pub(crate) fn create(tx: &Transaction<'_>, args: &CreateLabel) -> Result<()> {
    validate::nonempty_field("name", &args.name)?;
    validate::hex_color(&args.color)?;
    let exists: bool = tx.query_row(
        "SELECT COUNT(*) FROM labels WHERE project_id = ?1 AND name = ?2",
        params![args.project_id.to_string(), &args.name],
        |r| r.get::<_, i64>(0).map(|n| n > 0),
    )?;
    if exists {
        return Err(Error::Conflict(format!("label '{}' already exists in project", args.name)));
    }
    wl::insert(tx, args.id, args.project_id, &args.name, &args.color)?;
    Ok(())
}

pub(crate) fn update(tx: &Transaction<'_>, args: &UpdateLabel) -> Result<()> {
    if let Some(c) = &args.patch.color { validate::hex_color(c)?; }
    if let Some(n) = &args.patch.name { validate::nonempty_field("name", n)?; }
    wl::update_fields(tx, args.id, args.patch.name.as_deref(), args.patch.color.as_deref())?;
    Ok(())
}

pub(crate) fn delete(tx: &Transaction<'_>, args: &DeleteLabel) -> Result<()> {
    wl::delete(tx, args.id)?;
    Ok(())
}

pub(crate) fn attach(tx: &Transaction<'_>, args: &AttachLabel) -> Result<()> {
    wl::attach(tx, args.issue_id, args.label_id)?;
    Ok(())
}

pub(crate) fn detach(tx: &Transaction<'_>, args: &DetachLabel) -> Result<()> {
    wl::detach(tx, args.issue_id, args.label_id)?;
    Ok(())
}

pub(crate) fn inverse_of_create(args: &CreateLabel) -> Operation {
    Operation::DeleteLabel(DeleteLabel { id: args.id })
}

pub(crate) fn inverse_of_delete(tx: &Transaction<'_>, args: &DeleteLabel) -> Result<Operation> {
    let l = crate::store::read::labels::by_id_via_tx(tx, args.id)?;
    Ok(Operation::CreateLabel(CreateLabel {
        id: l.id, project_id: l.project_id, name: l.name, color: l.color,
    }))
}

pub(crate) fn inverse_of_update(tx: &Transaction<'_>, args: &UpdateLabel) -> Result<Operation> {
    let l = crate::store::read::labels::by_id_via_tx(tx, args.id)?;
    Ok(Operation::UpdateLabel(UpdateLabel {
        id: l.id,
        patch: LabelPatch {
            name: args.patch.name.as_ref().map(|_| l.name),
            color: args.patch.color.as_ref().map(|_| l.color),
        },
    }))
}

pub(crate) fn inverse_of_attach(args: &AttachLabel) -> Operation {
    Operation::DetachLabel(DetachLabel { issue_id: args.issue_id, label_id: args.label_id })
}

pub(crate) fn inverse_of_detach(args: &DetachLabel) -> Operation {
    Operation::AttachLabel(AttachLabel { issue_id: args.issue_id, label_id: args.label_id })
}
```

- [ ] **Step 5: Wire dispatch + capture_inverse**

In `apply/mod.rs`, add the missing arms:

```rust
pub(crate) mod labels;

// in dispatch:
Operation::CreateLabel(args) => labels::create(tx, args)?,
Operation::UpdateLabel(args) => labels::update(tx, args)?,
Operation::DeleteLabel(args) => labels::delete(tx, args)?,
Operation::AttachLabel(args) => labels::attach(tx, args)?,
Operation::DetachLabel(args) => labels::detach(tx, args)?,

// in capture_inverse:
Operation::CreateLabel(args) => Ok(labels::inverse_of_create(args)),
Operation::DeleteLabel(args) => labels::inverse_of_delete(tx, args),
Operation::UpdateLabel(args) => labels::inverse_of_update(tx, args),
Operation::AttachLabel(args) => Ok(labels::inverse_of_attach(args)),
Operation::DetachLabel(args) => Ok(labels::inverse_of_detach(args)),
```

Add to `Workspace`:

```rust
impl Workspace {
    pub fn query_labels_for_project(&self, project_id: uuid::Uuid) -> crate::error::Result<Vec<crate::types::Label>> {
        crate::store::read::labels::for_project(&self.conn, project_id)
    }
}
```

Update mod.rs entries (`labels` in both `read/` and `write/`).

- [ ] **Step 6: Run + commit**

Run: `cargo test -p kanban-core`
Expected: PASS.

```bash
git add crates/kanban-core/
git commit -m "feat(core): add Create/Update/Delete/Attach/Detach Label with inverses"
```

---

### Task 25: AttachLabel through `CreateIssue` integration test

**Files:**
- Modify: `crates/kanban-core/tests/apply_issues.rs`

- [ ] **Step 1: Replace the placeholder test**

Replace the `create_issue_attaches_labels_in_one_op` placeholder with:

```rust
use kanban_core::operation::{CreateLabel};

#[test]
fn create_issue_attaches_labels_in_one_op() {
    let (mut ws, pid, sid) = fresh_with_project();
    let label_id = new_id();
    ws.apply(Operation::CreateLabel(CreateLabel {
        id: label_id, project_id: pid, name: "feat".into(), color: "#3b82f6".into(),
    })).unwrap();
    let issue_id = new_id();
    ws.apply(Operation::CreateIssue(CreateIssue {
        id: issue_id, project_id: pid, title: "x".into(), description: None,
        status_id: sid, priority: Priority::None, due_date: None,
        label_ids: vec![label_id],
    })).unwrap();

    let count: i64 = ws.conn_for_test().query_row(
        "SELECT COUNT(*) FROM issue_labels WHERE issue_id = ?1 AND label_id = ?2",
        rusqlite::params![issue_id.to_string(), label_id.to_string()], |r| r.get(0),
    ).unwrap();
    assert_eq!(count, 1);
}
```

> Note: this test reaches into the workspace's `Connection` for direct verification. Rather than gating with a feature, expose a `cfg(test)`-only accessor that's compiled out of release builds:

In `crates/kanban-core/src/workspace.rs`:

```rust
impl Workspace {
    /// Test-only accessor. Compiled out of release builds.
    #[cfg(test)]
    pub(crate) fn conn_for_test(&self) -> &rusqlite::Connection { &self.conn }

    /// Doc-hidden accessor for integration tests in this crate's `tests/` folder.
    /// Stable for the duration of v1; do NOT rely on this from external crates.
    #[doc(hidden)]
    pub fn _conn_for_integration_tests(&self) -> &rusqlite::Connection { &self.conn }
}
```

Use the doc-hidden accessor in the integration test (`tests/` is its own compilation unit, so plain `cfg(test)` doesn't apply):

```rust
let count: i64 = ws._conn_for_integration_tests().query_row(
    "SELECT COUNT(*) FROM issue_labels WHERE issue_id = ?1 AND label_id = ?2",
    rusqlite::params![issue_id.to_string(), label_id.to_string()],
    |r| r.get(0),
).unwrap();
assert_eq!(count, 1);
```

No feature flags, no `[features]` block, no `[patch.crates-io]`. The `_conn_for_integration_tests` name + `#[doc(hidden)]` is sufficient to keep external consumers from depending on it.

- [ ] **Step 2: Run**

Run: `cargo test -p kanban-core --test apply_issues`
Expected: PASS.

- [ ] **Step 3: Commit**

```bash
git add crates/kanban-core/
git commit -m "test(core): cover label attachment via CreateIssue"
```

---

### Task 26: Extend property test to labels

**Files:**
- Modify: `crates/kanban-core/tests/undo_redo_property.rs`

Same pattern as Task 22 — add label-creation/attach/detach steps and a `label_snapshot` snapshot. The structure mirrors Task 22 exactly, so:

- [ ] **Step 1: Add a `LabelStep` enum and proptest similar to `IssueStep`**

The strategies generate `CreateLabel { name, color }`, `AttachLabel { issue_idx, label_idx }`, `DetachLabel { issue_idx, label_idx }`, `DeleteLabel { idx }`. The snapshot function captures `(label_name, attached_issue_titles)` tuples sorted.

- [ ] **Step 2: Run + commit**

```bash
cargo test -p kanban-core --test undo_redo_property
git add crates/kanban-core/tests/undo_redo_property.rs
git commit -m "test(core): extend undo/redo property test to label ops"
```

---

## Phase 10 — Search

### Task 27: `search()` query

**Files:**
- Create: `crates/kanban-core/src/store/read/search.rs`
- Modify: `crates/kanban-core/src/workspace.rs`
- Modify: `crates/kanban-core/src/store/read/mod.rs`
- Create: `crates/kanban-core/tests/search.rs`

- [ ] **Step 1: Failing tests**

Create `crates/kanban-core/tests/search.rs`:

```rust
use kanban_core::operation::{CreateIssue, CreateProject, Operation};
use kanban_core::query::IssueFilter;
use kanban_core::types::Priority;
use kanban_core::{Workspace, new_id};

fn seeded() -> (Workspace, uuid::Uuid) {
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid, name: "S".into(), prefix: "SRC".into(), description: None, icon: None,
    })).unwrap();
    let sid = ws.query_statuses_for_project(pid).unwrap()[0].id;
    for (t, d) in [
        ("Add OAuth login", Some("user can authenticate via OAuth")),
        ("Fix board crash", Some("crash when dragging cards")),
        ("Document API", None),
    ] {
        ws.apply(Operation::CreateIssue(CreateIssue {
            id: new_id(), project_id: pid, title: t.into(),
            description: d.map(str::to_string),
            status_id: sid, priority: Priority::None, due_date: None, label_ids: vec![],
        })).unwrap();
    }
    (ws, pid)
}

#[test]
fn search_by_title_returns_match() {
    let (ws, pid) = seeded();
    let hits = ws.search("OAuth", IssueFilter::for_project(pid)).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].title, "Add OAuth login");
}

#[test]
fn search_by_description() {
    let (ws, pid) = seeded();
    let hits = ws.search("dragging", IssueFilter::for_project(pid)).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].title, "Fix board crash");
}

#[test]
fn search_combines_with_filter() {
    let (ws, pid) = seeded();
    let hits = ws.search("API", IssueFilter {
        project_id: Some(pid), priorities: vec![Priority::Urgent], ..Default::default()
    }).unwrap();
    assert!(hits.is_empty(), "no urgent issues match 'API'");
}

#[test]
fn search_after_delete_drops_match() {
    let (mut ws, pid) = seeded();
    let issues = ws.query_issues(IssueFilter::for_project(pid)).unwrap();
    let oauth = issues.iter().find(|i| i.title.contains("OAuth")).unwrap();
    ws.apply(Operation::DeleteIssue(kanban_core::operation::DeleteIssue { id: oauth.id })).unwrap();
    let hits = ws.search("OAuth", IssueFilter::for_project(pid)).unwrap();
    assert!(hits.is_empty());
}
```

- [ ] **Step 2: Implement search**

Create `crates/kanban-core/src/store/read/search.rs`:

```rust
use crate::error::Result;
use crate::query::IssueFilter;
use crate::store::read::issues::ISSUE_LIST_BASE;
use crate::types::Issue;
use rusqlite::Connection;
use rusqlite::types::Value;

pub(crate) fn search(conn: &Connection, query: &str, mut filter: IssueFilter) -> Result<Vec<Issue>> {
    filter.search_text = Some(query.to_string());
    let (mut sql, mut params) = filter.build_sql_with_search(ISSUE_LIST_BASE);
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), super::issues::row_to_issue)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}
```

> Note on `row_to_issue` visibility: change `fn row_to_issue` in `store/read/issues.rs` from `fn` to `pub(super)` so `search` module can import it.

Add to `IssueFilter` in `crates/kanban-core/src/query.rs`:

```rust
impl IssueFilter {
    pub fn build_sql_with_search(&self, base: &str) -> (String, Vec<Value>) {
        if self.search_text.is_none() {
            return self.build_sql(base);
        }
        let q = self.search_text.as_ref().unwrap();
        let mut sql = format!(
            "{base} JOIN issue_search s ON s.rowid = issues.rowid WHERE issue_search MATCH ?"
        );
        // We've already opened the WHERE clause; add filters with AND.
        let mut params: Vec<Value> = vec![Value::Text(q.clone())];

        if let Some(pid) = self.project_id {
            sql.push_str(" AND project_id = ?");
            params.push(Value::Text(pid.to_string()));
        }
        if !self.status_ids.is_empty() {
            sql.push_str(&format!(" AND status_id IN ({})", placeholders(self.status_ids.len())));
            for s in &self.status_ids { params.push(Value::Text(s.to_string())); }
        }
        if !self.priorities.is_empty() {
            sql.push_str(&format!(" AND priority IN ({})", placeholders(self.priorities.len())));
            for p in &self.priorities { params.push(Value::Text(p.as_str().to_string())); }
        }
        if let Some(d) = self.due_before {
            sql.push_str(" AND due_date IS NOT NULL AND due_date < ?");
            params.push(Value::Text(d.to_string()));
        }
        if let Some(d) = self.due_after {
            sql.push_str(" AND due_date IS NOT NULL AND due_date > ?");
            params.push(Value::Text(d.to_string()));
        }
        if !self.label_ids.is_empty() {
            sql.push_str(&format!(
                " AND id IN (SELECT issue_id FROM issue_labels WHERE label_id IN ({}))",
                placeholders(self.label_ids.len())
            ));
            for l in &self.label_ids { params.push(Value::Text(l.to_string())); }
        }
        sql.push_str(" ORDER BY rank");
        if let Some(n) = self.limit {
            sql.push_str(" LIMIT ?");
            params.push(Value::Integer(n));
        }
        (sql, params)
    }
}
```

Add to `Workspace`:

```rust
impl Workspace {
    pub fn search(&self, query: &str, filter: crate::query::IssueFilter) -> crate::error::Result<Vec<crate::types::Issue>> {
        crate::store::read::search::search(&self.conn, query, filter)
    }
}
```

Update `crates/kanban-core/src/store/read/mod.rs` to add `pub(crate) mod search;`.

- [ ] **Step 3: Run + commit**

```bash
cargo test -p kanban-core --test search
git add crates/kanban-core/
git commit -m "feat(core): add FTS5-backed search() composing with IssueFilter"
```

---

## Phase 11 — Snapshot, export, import

### Task 28: `WorkspaceSnapshot` type

**Files:**
- Create: `crates/kanban-core/src/snapshot.rs`
- Modify: `crates/kanban-core/src/lib.rs`

- [ ] **Step 1: Define and test the type**

```rust
// crates/kanban-core/src/snapshot.rs
use crate::types::{Issue, Label, Project, Status};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const SNAPSHOT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub schema_version: u32,
    pub exported_at: DateTime<Utc>,
    pub projects: Vec<Project>,
    pub statuses: Vec<Status>,
    pub issues: Vec<Issue>,
    pub labels: Vec<Label>,
    pub issue_labels: Vec<IssueLabelLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IssueLabelLink {
    pub issue_id: Uuid,
    pub label_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_round_trips_via_json() {
        let snap = WorkspaceSnapshot {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            exported_at: Utc::now(),
            projects: vec![],
            statuses: vec![],
            issues: vec![],
            labels: vec![],
            issue_labels: vec![],
        };
        let s = serde_json::to_string(&snap).unwrap();
        let back: WorkspaceSnapshot = serde_json::from_str(&s).unwrap();
        assert_eq!(back.schema_version, 1);
    }
}
```

Wire into lib.rs: `pub mod snapshot; pub use snapshot::{WorkspaceSnapshot, IssueLabelLink, SNAPSHOT_SCHEMA_VERSION};`

- [ ] **Step 2: Run + commit**

```bash
cargo test -p kanban-core
git add crates/kanban-core/src
git commit -m "feat(core): add WorkspaceSnapshot type for export/import"
```

---

### Task 29: `export_snapshot`

**Files:**
- Modify: `crates/kanban-core/src/workspace.rs`
- Create: `crates/kanban-core/tests/snapshot_export.rs`

- [ ] **Step 1: Failing test**

```rust
use kanban_core::operation::{CreateIssue, CreateProject, Operation};
use kanban_core::types::Priority;
use kanban_core::{Workspace, new_id};

#[test]
fn export_snapshot_contains_all_entities() {
    let mut ws = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id: pid, name: "X".into(), prefix: "EXP".into(), description: None, icon: None,
    })).unwrap();
    let sid = ws.query_statuses_for_project(pid).unwrap()[0].id;
    ws.apply(Operation::CreateIssue(CreateIssue {
        id: new_id(), project_id: pid, title: "t".into(), description: None,
        status_id: sid, priority: Priority::None, due_date: None, label_ids: vec![],
    })).unwrap();
    let snap = ws.export_snapshot().unwrap();
    assert_eq!(snap.schema_version, 1);
    assert_eq!(snap.projects.len(), 1);
    assert_eq!(snap.statuses.len(), 7);
    assert_eq!(snap.issues.len(), 1);
}
```

- [ ] **Step 2: Implement**

Append to `crates/kanban-core/src/workspace.rs`:

```rust
use crate::snapshot::{IssueLabelLink, WorkspaceSnapshot, SNAPSHOT_SCHEMA_VERSION};

impl Workspace {
    pub fn export_snapshot(&self) -> crate::error::Result<WorkspaceSnapshot> {
        let projects = crate::store::read::projects::list_all(&self.conn)?;

        let mut statuses = Vec::new();
        let mut labels = Vec::new();
        for p in &projects {
            statuses.extend(crate::store::read::statuses::for_project(&self.conn, p.id)?);
            labels.extend(crate::store::read::labels::for_project(&self.conn, p.id)?);
        }

        let issues = crate::store::read::issues::list(&self.conn, &crate::query::IssueFilter::default())?;

        let mut issue_labels = Vec::new();
        let mut stmt = self.conn.prepare("SELECT issue_id, label_id FROM issue_labels")?;
        let rows = stmt.query_map([], |r| {
            Ok(IssueLabelLink {
                issue_id: uuid::Uuid::parse_str(&r.get::<_, String>(0)?).unwrap(),
                label_id: uuid::Uuid::parse_str(&r.get::<_, String>(1)?).unwrap(),
            })
        })?;
        for r in rows { issue_labels.push(r?); }

        Ok(WorkspaceSnapshot {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            exported_at: chrono::Utc::now(),
            projects, statuses, issues, labels, issue_labels,
        })
    }
}
```

- [ ] **Step 3: Run + commit**

```bash
cargo test -p kanban-core --test snapshot_export
git add crates/kanban-core/
git commit -m "feat(core): add Workspace::export_snapshot"
```

---

### Task 30: `ImportSnapshot` operation

**Files:**
- Create: `crates/kanban-core/src/apply/snapshot.rs`
- Modify: `crates/kanban-core/src/operation.rs`
- Modify: `crates/kanban-core/src/apply/mod.rs`
- Create: `crates/kanban-core/tests/snapshot_import.rs`

- [ ] **Step 1: Add `ImportSnapshot` to `Operation`**

In `crates/kanban-core/src/operation.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConflictPolicy { Skip, Overwrite, Fail }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportSnapshot {
    pub snapshot: crate::snapshot::WorkspaceSnapshot,
    pub policy: ConflictPolicy,
}

// add to Operation enum:
//     ImportSnapshot(ImportSnapshot),
```

- [ ] **Step 2: Failing test**

```rust
// crates/kanban-core/tests/snapshot_import.rs
use kanban_core::operation::{ConflictPolicy, ImportSnapshot, Operation};
use kanban_core::Workspace;

#[test]
fn import_into_empty_db_writes_all_entities() {
    let mut donor = Workspace::open_in_memory().unwrap();
    // seed donor with one project + one issue
    let pid = kanban_core::new_id();
    donor.apply(Operation::CreateProject(kanban_core::operation::CreateProject {
        id: pid, name: "P".into(), prefix: "IMP".into(), description: None, icon: None,
    })).unwrap();
    let sid = donor.query_statuses_for_project(pid).unwrap()[0].id;
    let iid = kanban_core::new_id();
    donor.apply(Operation::CreateIssue(kanban_core::operation::CreateIssue {
        id: iid, project_id: pid, title: "x".into(), description: None,
        status_id: sid, priority: kanban_core::types::Priority::None,
        due_date: None, label_ids: vec![],
    })).unwrap();
    let snap = donor.export_snapshot().unwrap();

    let mut target = Workspace::open_in_memory().unwrap();
    target.apply(Operation::ImportSnapshot(ImportSnapshot {
        snapshot: snap, policy: ConflictPolicy::Fail,
    })).unwrap();

    assert_eq!(target.query_projects().unwrap().len(), 1);
    assert_eq!(target.query_issue_by_id(iid).unwrap().title, "x");
}

#[test]
fn import_with_id_collision_fails_under_fail_policy() {
    // create same project in target first, then try to import a snapshot containing it
    let mut donor = Workspace::open_in_memory().unwrap();
    let pid = kanban_core::new_id();
    donor.apply(Operation::CreateProject(kanban_core::operation::CreateProject {
        id: pid, name: "Donor".into(), prefix: "DDD".into(), description: None, icon: None,
    })).unwrap();
    let snap = donor.export_snapshot().unwrap();

    let mut target = Workspace::open_in_memory().unwrap();
    target.apply(Operation::CreateProject(kanban_core::operation::CreateProject {
        id: pid, name: "Same id different name".into(), prefix: "TGT".into(),
        description: None, icon: None,
    })).unwrap();
    let err = target.apply(Operation::ImportSnapshot(ImportSnapshot {
        snapshot: snap, policy: ConflictPolicy::Fail,
    })).unwrap_err();
    assert!(err.to_string().to_lowercase().contains("conflict"), "{err}");
}

#[test]
fn import_skip_policy_keeps_existing_rows() {
    let mut donor = Workspace::open_in_memory().unwrap();
    let pid = kanban_core::new_id();
    donor.apply(Operation::CreateProject(kanban_core::operation::CreateProject {
        id: pid, name: "From Donor".into(), prefix: "DON".into(), description: None, icon: None,
    })).unwrap();
    let snap = donor.export_snapshot().unwrap();

    let mut target = Workspace::open_in_memory().unwrap();
    target.apply(Operation::CreateProject(kanban_core::operation::CreateProject {
        id: pid, name: "Original".into(), prefix: "ORG".into(), description: None, icon: None,
    })).unwrap();
    target.apply(Operation::ImportSnapshot(ImportSnapshot {
        snapshot: snap, policy: ConflictPolicy::Skip,
    })).unwrap();
    let p = target.query_project_by_id(pid).unwrap();
    assert_eq!(p.name, "Original", "skip should keep existing row");
}
```

- [ ] **Step 3: Implement**

Create `crates/kanban-core/src/apply/snapshot.rs`:

```rust
use crate::error::{Error, Result};
use crate::operation::{ConflictPolicy, ImportSnapshot, Operation};
use crate::snapshot::WorkspaceSnapshot;
use crate::types::ProjectStatus;
use rusqlite::{Transaction, params};

pub(crate) fn import(tx: &Transaction<'_>, args: &ImportSnapshot) -> Result<()> {
    if args.snapshot.schema_version != crate::snapshot::SNAPSHOT_SCHEMA_VERSION {
        return Err(Error::InvalidSnapshot(format!(
            "schema {} not supported (expected {})",
            args.snapshot.schema_version, crate::snapshot::SNAPSHOT_SCHEMA_VERSION
        )));
    }

    for p in &args.snapshot.projects {
        upsert_project(tx, p, args.policy)?;
    }
    for s in &args.snapshot.statuses {
        upsert_status(tx, s, args.policy)?;
    }
    for l in &args.snapshot.labels {
        upsert_label(tx, l, args.policy)?;
    }
    for i in &args.snapshot.issues {
        upsert_issue(tx, i, args.policy)?;
    }
    for link in &args.snapshot.issue_labels {
        tx.execute(
            "INSERT OR IGNORE INTO issue_labels(issue_id, label_id) VALUES (?1, ?2)",
            params![link.issue_id.to_string(), link.label_id.to_string()],
        )?;
    }
    Ok(())
}

fn exists<S: AsRef<str>>(tx: &Transaction<'_>, table: &str, id: S) -> Result<bool> {
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE id = ?1");
    let n: i64 = tx.query_row(&sql, params![id.as_ref()], |r| r.get(0))?;
    Ok(n > 0)
}

fn handle_conflict(policy: ConflictPolicy, kind: crate::error::EntityKind, id: &str) -> Result<bool> {
    match policy {
        ConflictPolicy::Skip => Ok(true),                // caller skips the row
        ConflictPolicy::Overwrite => Ok(false),         // caller overwrites
        ConflictPolicy::Fail => Err(Error::Conflict(format!("{kind:?} {id} already exists"))),
    }
}

fn upsert_project(tx: &Transaction<'_>, p: &crate::types::Project, policy: ConflictPolicy) -> Result<()> {
    let id = p.id.to_string();
    if exists(tx, "projects", &id)? {
        if handle_conflict(policy, crate::EntityKind::Project, &id)? { return Ok(()); }
        tx.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
    }
    tx.execute(
        "INSERT INTO projects(id,name,prefix,description,icon,status,next_seq,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
        params![
            id, p.name, p.prefix, p.description, p.icon,
            p.status.as_str(), p.next_seq,
            p.created_at.to_rfc3339(), p.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

fn upsert_status(tx: &Transaction<'_>, s: &crate::types::Status, policy: ConflictPolicy) -> Result<()> {
    let id = s.id.to_string();
    if exists(tx, "statuses", &id)? {
        if handle_conflict(policy, crate::EntityKind::Status, &id)? { return Ok(()); }
        tx.execute("DELETE FROM statuses WHERE id = ?1", params![id])?;
    }
    tx.execute(
        "INSERT INTO statuses(id,project_id,name,category,color,position) VALUES (?1,?2,?3,?4,?5,?6)",
        params![id, s.project_id.to_string(), s.name, s.category.as_str(), s.color, s.position],
    )?;
    Ok(())
}

fn upsert_label(tx: &Transaction<'_>, l: &crate::types::Label, policy: ConflictPolicy) -> Result<()> {
    let id = l.id.to_string();
    if exists(tx, "labels", &id)? {
        if handle_conflict(policy, crate::EntityKind::Label, &id)? { return Ok(()); }
        tx.execute("DELETE FROM labels WHERE id = ?1", params![id])?;
    }
    tx.execute(
        "INSERT INTO labels(id,project_id,name,color) VALUES (?1,?2,?3,?4)",
        params![id, l.project_id.to_string(), l.name, l.color],
    )?;
    Ok(())
}

fn upsert_issue(tx: &Transaction<'_>, i: &crate::types::Issue, policy: ConflictPolicy) -> Result<()> {
    let id = i.id.to_string();
    if exists(tx, "issues", &id)? {
        if handle_conflict(policy, crate::EntityKind::Issue, &id)? { return Ok(()); }
        tx.execute("DELETE FROM issues WHERE id = ?1", params![id])?;
    }
    tx.execute(
        "INSERT INTO issues(id,project_id,seq,identifier,title,description,status_id,priority,
                            due_date,sort_key,created_at,updated_at)
         VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
        params![
            id, i.project_id.to_string(), i.seq, i.identifier, i.title, i.description,
            i.status_id.to_string(), i.priority.as_str(),
            i.due_date.map(|d| d.to_string()), i.sort_key,
            i.created_at.to_rfc3339(), i.updated_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}

pub(crate) fn inverse_of_import(_tx: &Transaction<'_>, _args: &ImportSnapshot) -> Result<Operation> {
    // Inverting a full snapshot import isn't supported in v1: the spec calls out that
    // `kanban undo` after import "rolls the whole thing back" — implemented as truncating
    // every table touched by the import and reinstating the pre-import snapshot. That
    // requires capturing a pre-import snapshot, which the applier does separately (see
    // snapshot_pre).
    Err(Error::InvalidSnapshot(
        "ImportSnapshot inverse computed via pre-snapshot capture, not in capture_inverse".into(),
    ))
}
```

In `crates/kanban-core/src/apply/mod.rs`, add ImportSnapshot handling. Because the inverse is a pre-snapshot, capture it specially:

```rust
pub(crate) mod snapshot;

// ...

impl Workspace {
    pub fn apply(&mut self, op: Operation) -> Result<OperationOutcome> {
        let now = self.clock.now();
        let payload = serde_json::to_string(&op)?;
        let tx = self.conn.transaction()?;
        operation_log::truncate_redo_branch(&tx)?;

        let inverse = match &op {
            Operation::ImportSnapshot(_) => {
                // Pre-snapshot of current workspace IS the inverse: re-importing it overwrites.
                let pre = export_snapshot_via_tx(&tx)?;
                Operation::ImportSnapshot(crate::operation::ImportSnapshot {
                    snapshot: pre,
                    policy: crate::operation::ConflictPolicy::Overwrite,
                })
            }
            _ => capture_inverse(&tx, &op)?,
        };
        let inverse_payload = serde_json::to_string(&inverse)?;

        let op_id = operation_log::insert_operation(
            &tx, op_type_name(&op), &payload, &inverse_payload, now,
        )?;
        let pre = capture_activity_pre(&tx, &op)?;
        dispatch(&tx, &op, now)?;
        emit_activity(&tx, op_id, &op, pre, now)?;
        tx.commit()?;
        Ok(OperationOutcome { op_id })
    }
}

fn export_snapshot_via_tx(tx: &rusqlite::Transaction<'_>) -> Result<crate::snapshot::WorkspaceSnapshot> {
    // Equivalent to Workspace::export_snapshot but via a transaction (read-only).
    use crate::snapshot::{IssueLabelLink, WorkspaceSnapshot, SNAPSHOT_SCHEMA_VERSION};
    let projects = {
        let mut stmt = tx.prepare("SELECT id,name,prefix,description,icon,status,next_seq,created_at,updated_at FROM projects ORDER BY created_at ASC")?;
        stmt.query_map([], crate::store::read::projects::row_to_project_pub)?.collect::<rusqlite::Result<Vec<_>>>()?
    };
    let mut statuses = Vec::new();
    let mut labels = Vec::new();
    for p in &projects {
        statuses.extend(crate::store::read::statuses::for_project_via_tx(tx, p.id)?);
        labels.extend(crate::store::read::labels::for_project_via_tx(tx, p.id)?);
    }
    let mut issues = Vec::new();
    let mut stmt = tx.prepare(crate::store::read::issues::ISSUE_LIST_BASE)?;
    let rows = stmt.query_map([], crate::store::read::issues::row_to_issue)?;
    for r in rows { issues.push(r?); }
    let mut issue_labels = Vec::new();
    let mut stmt = tx.prepare("SELECT issue_id, label_id FROM issue_labels")?;
    let rows = stmt.query_map([], |r| Ok(IssueLabelLink {
        issue_id: uuid::Uuid::parse_str(&r.get::<_, String>(0)?).unwrap(),
        label_id: uuid::Uuid::parse_str(&r.get::<_, String>(1)?).unwrap(),
    }))?;
    for r in rows { issue_labels.push(r?); }

    Ok(WorkspaceSnapshot {
        schema_version: SNAPSHOT_SCHEMA_VERSION,
        exported_at: chrono::Utc::now(),
        projects, statuses, issues, labels, issue_labels,
    })
}
```

Expose the `pub(crate)` accessors referenced above. Add to `crates/kanban-core/src/store/read/projects.rs`:

```rust
// Expose the existing private helper so the apply layer can re-use it.
pub(crate) fn row_to_project_pub(r: &rusqlite::Row<'_>) -> rusqlite::Result<crate::types::Project> {
    row_to_project(r)
}
```

Add to `crates/kanban-core/src/store/read/statuses.rs`:

```rust
pub(crate) fn for_project_via_tx(
    tx: &rusqlite::Transaction<'_>,
    project_id: uuid::Uuid,
) -> Result<Vec<crate::types::Status>> {
    let mut stmt = tx.prepare(
        "SELECT id,project_id,name,category,color,position FROM statuses
         WHERE project_id = ?1 ORDER BY position ASC",
    )?;
    let rows = stmt.query_map(rusqlite::params![project_id.to_string()], row_to_status)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}
```

Add to `crates/kanban-core/src/store/read/labels.rs`:

```rust
pub(crate) fn for_project_via_tx(
    tx: &rusqlite::Transaction<'_>,
    project_id: uuid::Uuid,
) -> Result<Vec<crate::types::Label>> {
    let mut stmt = tx.prepare("SELECT id,project_id,name,color FROM labels WHERE project_id = ?1 ORDER BY name")?;
    let rows = stmt.query_map(rusqlite::params![project_id.to_string()], row_to_label)?;
    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}
```

In `crates/kanban-core/src/store/read/issues.rs`, change visibility:

```rust
// was: const ISSUE_LIST_BASE: &str = "...";
pub(crate) const ISSUE_LIST_BASE: &str = "...";

// was: fn row_to_issue(r: &rusqlite::Row<'_>) -> ...
pub(crate) fn row_to_issue(r: &rusqlite::Row<'_>) -> rusqlite::Result<crate::types::Issue> { /* unchanged body */ }
```

Also add to dispatch:

```rust
Operation::ImportSnapshot(args) => snapshot::import(tx, args)?,
```

- [ ] **Step 4: Run + commit**

```bash
cargo test -p kanban-core --test snapshot_import
git add crates/kanban-core/
git commit -m "feat(core): add ImportSnapshot op with conflict policies and undo support"
```

---

### Task 31: Round-trip test

**Files:**
- Create: `crates/kanban-core/tests/snapshot_roundtrip.rs`

- [ ] **Step 1: Test**

```rust
use kanban_core::operation::{
    ConflictPolicy, CreateIssue, CreateLabel, CreateProject, ImportSnapshot, Operation,
};
use kanban_core::types::Priority;
use kanban_core::{Workspace, new_id};

#[test]
fn export_then_import_into_empty_yields_equivalent_state() {
    let mut a = Workspace::open_in_memory().unwrap();
    let pid = new_id();
    a.apply(Operation::CreateProject(CreateProject {
        id: pid, name: "RT".into(), prefix: "RTR".into(), description: Some("d".into()), icon: None,
    })).unwrap();
    let sid = a.query_statuses_for_project(pid).unwrap()[0].id;
    let label_id = new_id();
    a.apply(Operation::CreateLabel(CreateLabel {
        id: label_id, project_id: pid, name: "feat".into(), color: "#3b82f6".into(),
    })).unwrap();
    a.apply(Operation::CreateIssue(CreateIssue {
        id: new_id(), project_id: pid, title: "round-trip me".into(), description: Some("body".into()),
        status_id: sid, priority: Priority::High, due_date: None, label_ids: vec![label_id],
    })).unwrap();

    let snap = a.export_snapshot().unwrap();

    let mut b = Workspace::open_in_memory().unwrap();
    b.apply(Operation::ImportSnapshot(ImportSnapshot {
        snapshot: snap, policy: ConflictPolicy::Fail,
    })).unwrap();

    let snap_b = b.export_snapshot().unwrap();
    assert_eq!(snap_b.projects.len(), 1);
    assert_eq!(snap_b.projects[0].name, "RT");
    assert_eq!(snap_b.statuses.len(), 7);
    assert_eq!(snap_b.issues.len(), 1);
    assert_eq!(snap_b.labels.len(), 1);
    assert_eq!(snap_b.issue_labels.len(), 1);
}
```

- [ ] **Step 2: Run + commit**

```bash
cargo test -p kanban-core --test snapshot_roundtrip
git add crates/kanban-core/tests/snapshot_roundtrip.rs
git commit -m "test(core): export → import round-trip is lossless"
```

---

## Phase 12 — CLI surface

### Task 32: clap scaffold + global flags + error → exit code mapping

**Files:**
- Create: `crates/kanban-cli/src/cli.rs`
- Create: `crates/kanban-cli/src/exit.rs`
- Create: `crates/kanban-cli/src/output.rs`
- Modify: `crates/kanban-cli/src/main.rs`
- Create: `crates/kanban-cli/tests/help.rs`

- [ ] **Step 1: clap definitions**

Create `crates/kanban-cli/src/cli.rs`:

```rust
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "kanban",
    version,
    about = "Local-first kanban for projects, issues, labels, undo/redo, and JSON I/O."
)]
pub struct Cli {
    /// Override database path (also $KANBAN_DB).
    #[arg(long, env = "KANBAN_DB", global = true)]
    pub db: Option<PathBuf>,

    /// Emit machine-readable JSON instead of human-readable output.
    #[arg(long, global = true)]
    pub json: bool,

    /// Suppress non-error output.
    #[arg(long, global = true, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Verbose diagnostic output on stderr.
    #[arg(long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub cmd: Cmd,
}

#[derive(Debug, Subcommand)]
pub enum Cmd {
    Project(crate::cmd::project::ProjectCmd),
    Issue(crate::cmd::issue::IssueCmd),
    Label(crate::cmd::label::LabelCmd),
    Status(crate::cmd::status::StatusCmd),
    Search(crate::cmd::search::SearchArgs),
    Export(crate::cmd::export::ExportArgs),
    Import(crate::cmd::import::ImportArgs),
    Undo,
    Redo,
    Batch(crate::cmd::batch::BatchArgs),
}
```

- [ ] **Step 2: Exit code mapping**

Create `crates/kanban-cli/src/exit.rs`:

```rust
use kanban_core::Error;

pub const EXIT_OK: i32 = 0;
pub const EXIT_USER: i32 = 1;
pub const EXIT_NOT_FOUND: i32 = 2;
pub const EXIT_VALIDATION: i32 = 3;
pub const EXIT_INTERNAL: i32 = 4;

pub fn code_for(err: &Error) -> i32 {
    match err {
        Error::Validation(_) => EXIT_VALIDATION,
        Error::NotFound { .. } => EXIT_NOT_FOUND,
        Error::Conflict(_) => EXIT_VALIDATION,
        Error::Db(_) | Error::Io(_) | Error::Serde(_) | Error::InvalidSnapshot(_) => EXIT_INTERNAL,
    }
}
```

- [ ] **Step 3: Output helpers (human and JSON)**

Create `crates/kanban-cli/src/output.rs`:

```rust
use serde::Serialize;
use std::io::{Write, stdout};

pub struct Out {
    pub json: bool,
}

impl Out {
    pub fn print<T: Serialize + std::fmt::Display>(&self, value: &T) -> std::io::Result<()> {
        if self.json {
            let s = serde_json::to_string_pretty(value).unwrap();
            writeln!(stdout(), "{s}")
        } else {
            writeln!(stdout(), "{value}")
        }
    }

    pub fn print_json<T: Serialize>(&self, value: &T) -> std::io::Result<()> {
        let s = serde_json::to_string_pretty(value).unwrap();
        writeln!(stdout(), "{s}")
    }

    pub fn print_human(&self, s: &str) -> std::io::Result<()> {
        if !self.json { writeln!(stdout(), "{s}")?; }
        Ok(())
    }
}
```

- [ ] **Step 4: Wire `main.rs`**

Replace `crates/kanban-cli/src/main.rs`:

```rust
mod cli;
mod cmd;
mod exit;
mod output;

use clap::Parser;
use cli::{Cli, Cmd};
use kanban_core::Workspace;

fn main() {
    let args = Cli::parse();
    let result = run(args);
    match result {
        Ok(()) => std::process::exit(exit::EXIT_OK),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(exit::code_for(&e));
        }
    }
}

fn run(args: Cli) -> kanban_core::Result<()> {
    let mut ws = match args.db {
        Some(path) => Workspace::open(&path)?,
        None => Workspace::open_default()?,
    };
    let out = output::Out { json: args.json };
    match args.cmd {
        Cmd::Project(c) => cmd::project::run(c, &mut ws, &out),
        Cmd::Issue(c) => cmd::issue::run(c, &mut ws, &out),
        Cmd::Label(c) => cmd::label::run(c, &mut ws, &out),
        Cmd::Status(c) => cmd::status::run(c, &ws, &out),
        Cmd::Search(c) => cmd::search::run(c, &ws, &out),
        Cmd::Export(c) => cmd::export::run(c, &ws, &out),
        Cmd::Import(c) => cmd::import::run(c, &mut ws, &out),
        Cmd::Undo => cmd::undo::run_undo(&mut ws, &out),
        Cmd::Redo => cmd::undo::run_redo(&mut ws, &out),
        Cmd::Batch(c) => cmd::batch::run(c, args.db.as_deref(), &out),
    }
}
```

Stub `crates/kanban-cli/src/cmd/mod.rs`:

```rust
pub mod batch;
pub mod export;
pub mod import;
pub mod issue;
pub mod label;
pub mod project;
pub mod search;
pub mod status;
pub mod undo;
```

For each stub module, create a minimal placeholder so the binary compiles:

```rust
// project.rs
use clap::Args;
use kanban_core::{Workspace, Result};
use crate::output::Out;

#[derive(Debug, Args)]
pub struct ProjectCmd {
    #[command(subcommand)]
    pub sub: ProjectSub,
}

#[derive(Debug, clap::Subcommand)]
pub enum ProjectSub {
    List,
    Create { name: String, #[arg(long)] prefix: String, #[arg(long)] description: Option<String> },
    Show { id_or_prefix: String },
    Update { id_or_prefix: String, #[arg(long)] name: Option<String> },
    Archive { id_or_prefix: String },
    Delete { id_or_prefix: String, #[arg(long)] yes: bool },
}

pub fn run(_cmd: ProjectCmd, _ws: &mut Workspace, _out: &Out) -> Result<()> {
    Err(kanban_core::Error::InvalidSnapshot("project subcommand not yet implemented".into()))
}
```

(Repeat skeleton for `issue.rs`, `label.rs`, `status.rs`, `search.rs`, `export.rs`, `import.rs`, `undo.rs`, `batch.rs` — each defines its `Args`/`Cmd` struct and a `run` that returns the same `not yet implemented` error. Subsequent tasks implement them one at a time.)

- [ ] **Step 5: Help-output snapshot test**

Create `crates/kanban-cli/tests/help.rs`:

```rust
use assert_cmd::Command;

#[test]
fn help_lists_all_subcommands() {
    let out = Command::cargo_bin("kanban").unwrap().arg("--help").assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    for sub in ["project", "issue", "label", "status", "search", "export", "import", "undo", "redo", "batch"] {
        assert!(stdout.contains(sub), "missing '{sub}' in --help: {stdout}");
    }
}
```

- [ ] **Step 6: Run + commit**

```bash
cargo build -p kanban-cli
cargo test -p kanban-cli --test help
git add crates/kanban-cli/
git commit -m "feat(cli): add clap scaffold, global flags, error→exit mapping, command stubs"
```

---

### Task 33: `kanban project` subcommands

**Files:**
- Modify: `crates/kanban-cli/src/cmd/project.rs`
- Create: `crates/kanban-cli/tests/project_cmds.rs`
- Create: `crates/kanban-cli/tests/snapshots/` (insta will populate)

- [ ] **Step 1: Snapshot tests (failing)**

Create `crates/kanban-cli/tests/project_cmds.rs`:

```rust
use assert_cmd::Command;
use std::path::PathBuf;

fn isolated_db() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.db");
    (dir, path)
}

fn cli(db: &PathBuf) -> Command {
    let mut c = Command::cargo_bin("kanban").unwrap();
    c.env("KANBAN_DB", db);
    c
}

#[test]
fn project_create_then_list() {
    let (_d, db) = isolated_db();
    cli(&db).args(["project", "create", "Auth", "--prefix", "AUTH"]).assert().success();
    let out = cli(&db).args(["project", "list"]).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!("project_list_one_project", stdout, @r#"
    AUTH  Auth  active
    "#);
}

#[test]
fn project_create_emits_json_when_requested() {
    let (_d, db) = isolated_db();
    let out = cli(&db).args(["--json", "project", "create", "Auth", "--prefix", "AUTH"]).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["prefix"], "AUTH");
    assert_eq!(v["name"], "Auth");
}

#[test]
fn project_create_invalid_prefix_exits_validation() {
    let (_d, db) = isolated_db();
    let assert = cli(&db).args(["project", "create", "Bad", "--prefix", "lower"]).assert().failure();
    assert.code(3); // EXIT_VALIDATION
}

#[test]
fn project_show_unknown_prefix_exits_not_found() {
    let (_d, db) = isolated_db();
    let assert = cli(&db).args(["project", "show", "NOPE"]).assert().failure();
    assert.code(2); // EXIT_NOT_FOUND
}

#[test]
fn project_archive_changes_status() {
    let (_d, db) = isolated_db();
    cli(&db).args(["project", "create", "X", "--prefix", "ARC"]).assert().success();
    cli(&db).args(["project", "archive", "ARC"]).assert().success();
    let out = cli(&db).args(["--json", "project", "show", "ARC"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_str(std::str::from_utf8(&out.stdout).unwrap()).unwrap();
    assert_eq!(v["status"], "archived");
}
```

- [ ] **Step 2: Implement subcommands**

Replace `crates/kanban-cli/src/cmd/project.rs`:

```rust
use crate::output::Out;
use clap::{Args, Subcommand};
use kanban_core::operation::{ArchiveProject, CreateProject, DeleteProject, Operation, ProjectPatch, UpdateProject};
use kanban_core::{Result, Workspace, new_id};

#[derive(Debug, Args)]
pub struct ProjectCmd {
    #[command(subcommand)]
    pub sub: ProjectSub,
}

#[derive(Debug, Subcommand)]
pub enum ProjectSub {
    List,
    Create { name: String, #[arg(long)] prefix: String, #[arg(long)] description: Option<String>, #[arg(long)] icon: Option<String> },
    Show { id_or_prefix: String },
    Update { id_or_prefix: String, #[arg(long)] name: Option<String>, #[arg(long)] description: Option<String> },
    Archive { id_or_prefix: String },
    Delete { id_or_prefix: String, #[arg(long)] yes: bool },
}

pub fn run(cmd: ProjectCmd, ws: &mut Workspace, out: &Out) -> Result<()> {
    match cmd.sub {
        ProjectSub::List => list(ws, out),
        ProjectSub::Create { name, prefix, description, icon } => create(ws, out, name, prefix, description, icon),
        ProjectSub::Show { id_or_prefix } => show(ws, out, &id_or_prefix),
        ProjectSub::Update { id_or_prefix, name, description } => update(ws, out, &id_or_prefix, name, description),
        ProjectSub::Archive { id_or_prefix } => archive(ws, out, &id_or_prefix),
        ProjectSub::Delete { id_or_prefix, yes } => delete(ws, out, &id_or_prefix, yes),
    }
}

fn resolve(ws: &Workspace, id_or_prefix: &str) -> Result<kanban_core::Project> {
    if let Ok(uuid) = uuid::Uuid::parse_str(id_or_prefix) {
        return ws.query_project_by_id(uuid);
    }
    let projects = ws.query_projects()?;
    projects.into_iter()
        .find(|p| p.prefix == id_or_prefix)
        .ok_or(kanban_core::Error::NotFound { kind: kanban_core::EntityKind::Project, id: id_or_prefix.to_string() })
}

fn list(ws: &Workspace, out: &Out) -> Result<()> {
    let ps = ws.query_projects()?;
    if out.json {
        out.print_json(&ps).ok();
    } else {
        for p in &ps {
            println!("{}  {}  {}", p.prefix, p.name, p.status.as_str());
        }
    }
    Ok(())
}

fn create(ws: &mut Workspace, out: &Out, name: String, prefix: String, description: Option<String>, icon: Option<String>) -> Result<()> {
    let id = new_id();
    ws.apply(Operation::CreateProject(CreateProject {
        id, name: name.clone(), prefix: prefix.clone(), description: description.clone(), icon: icon.clone(),
    }))?;
    let p = ws.query_project_by_id(id)?;
    if out.json { out.print_json(&p).ok(); } else { println!("created {} ({})", p.prefix, p.name); }
    Ok(())
}

fn show(ws: &Workspace, out: &Out, id_or_prefix: &str) -> Result<()> {
    let p = resolve(ws, id_or_prefix)?;
    if out.json { out.print_json(&p).ok(); } else {
        println!("{}  {}  {}", p.prefix, p.name, p.status.as_str());
        if let Some(d) = &p.description { println!("description: {d}"); }
    }
    Ok(())
}

fn update(ws: &mut Workspace, out: &Out, id_or_prefix: &str, name: Option<String>, description: Option<String>) -> Result<()> {
    let p = resolve(ws, id_or_prefix)?;
    ws.apply(Operation::UpdateProject(UpdateProject {
        id: p.id,
        patch: ProjectPatch {
            name,
            description: description.map(Some),
            ..Default::default()
        },
    }))?;
    let p = ws.query_project_by_id(p.id)?;
    if out.json { out.print_json(&p).ok(); } else { println!("updated {}", p.prefix); }
    Ok(())
}

fn archive(ws: &mut Workspace, out: &Out, id_or_prefix: &str) -> Result<()> {
    let p = resolve(ws, id_or_prefix)?;
    ws.apply(Operation::ArchiveProject(ArchiveProject { id: p.id }))?;
    if !out.json { println!("archived {}", p.prefix); }
    Ok(())
}

fn delete(ws: &mut Workspace, out: &Out, id_or_prefix: &str, yes: bool) -> Result<()> {
    let p = resolve(ws, id_or_prefix)?;
    if !yes {
        return Err(kanban_core::Error::Validation(kanban_core::ValidationError {
            field: "confirm".into(),
            reason: "pass --yes to confirm deletion".into(),
        }));
    }
    ws.apply(Operation::DeleteProject(DeleteProject { id: p.id }))?;
    if !out.json { println!("deleted {}", p.prefix); }
    Ok(())
}
```

- [ ] **Step 3: Run snapshot tests, accept**

Run: `cargo test -p kanban-cli --test project_cmds`
Expected: PASS (or first run requests `cargo insta accept` — accept and rerun).

```bash
cargo insta accept --workspace-root crates/kanban-cli
cargo test -p kanban-cli --test project_cmds
```

- [ ] **Step 4: Commit**

```bash
git add crates/kanban-cli/
git commit -m "feat(cli): implement kanban project list/create/show/update/archive/delete"
```

---

### Task 34: `kanban issue` subcommands

**Files:**
- Modify: `crates/kanban-cli/src/cmd/issue.rs`
- Create: `crates/kanban-cli/tests/issue_cmds.rs`

- [ ] **Step 1: Snapshot/integration tests (failing)**

Cover: `issue create`, `issue list` with filters/sort, `issue show`, `issue update --field=value`, `issue move --status=...`, `issue reorder --before/--after`, `issue delete --yes`, `issue history`. One happy path + one error path each.

```rust
// excerpt — full file follows the same pattern as project_cmds.rs
#[test]
fn issue_create_and_list() {
    let (_d, db) = isolated_db();
    cli(&db).args(["project","create","P","--prefix","PRJ"]).assert().success();
    cli(&db).args(["issue","create","--project","PRJ","--title","add login"]).assert().success();
    cli(&db).args(["issue","create","--project","PRJ","--title","fix crash","--priority","high"]).assert().success();
    let out = cli(&db).args(["issue","list","--project","PRJ","--sort","priority"]).output().unwrap();
    insta::assert_snapshot!("issue_list_priority_sort", String::from_utf8(out.stdout).unwrap());
}

#[test]
fn issue_update_title_via_field_arg() {
    let (_d, db) = isolated_db();
    cli(&db).args(["project","create","P","--prefix","PRJ"]).assert().success();
    cli(&db).args(["issue","create","--project","PRJ","--title","old"]).assert().success();
    cli(&db).args(["issue","update","PRJ-1","--title","new"]).assert().success();
    let out = cli(&db).args(["--json","issue","show","PRJ-1"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["title"], "new");
}

#[test]
fn issue_move_changes_status() { /* similar */ }

#[test]
fn issue_history_shows_field_changes() {
    let (_d, db) = isolated_db();
    cli(&db).args(["project","create","P","--prefix","PRJ"]).assert().success();
    cli(&db).args(["issue","create","--project","PRJ","--title","x"]).assert().success();
    cli(&db).args(["issue","update","PRJ-1","--priority","high"]).assert().success();
    let out = cli(&db).args(["issue","history","PRJ-1"]).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!("issue_history_after_priority", stdout);
}
```

- [ ] **Step 2: Implement** — define `IssueCmd` in `cmd/issue.rs`, with `Create`, `List`, `Show`, `Update` (field flags: `--title`, `--description`, `--priority`, `--status`, `--due`), `Move`, `Reorder`, `Delete`, `History` subcommands. Identifier resolution accepts `KAN-42` or a UUID.

- [ ] **Step 3: Run + accept snapshots + commit**

```bash
cargo insta accept --workspace-root crates/kanban-cli
git add crates/kanban-cli/
git commit -m "feat(cli): implement kanban issue create/list/show/update/move/reorder/delete/history"
```

---

### Task 35: `kanban label` subcommands

**Files:**
- Modify: `crates/kanban-cli/src/cmd/label.rs`
- Create: `crates/kanban-cli/tests/label_cmds.rs`

Same pattern as Task 33: `label list --project`, `label create --project --name --color`, `label update`, `label delete --yes`, `label attach <issue> <label>`, `label detach <issue> <label>`. Tests assert: create+list snapshot, attach affects `issue show`, detach removes association.

- [ ] **Steps 1–4** mirror Task 33 with label-specific Operations.

```bash
cargo insta accept --workspace-root crates/kanban-cli
git add crates/kanban-cli/
git commit -m "feat(cli): implement kanban label list/create/update/delete/attach/detach"
```

---

### Task 36: `kanban status`, `search`, `export`, `import`, `undo`, `redo`

**Files:**
- Modify: `crates/kanban-cli/src/cmd/{status,search,export,import,undo}.rs`
- Create: `crates/kanban-cli/tests/{status,search,export_import,undo_cmds}.rs`

Per command:

- **`status list --project PREFIX`** — prints the 7 default statuses with category and color. Snapshot test.
- **`search "QUERY" [--project PREFIX] [filters]`** — calls `Workspace::search`, prints results. Test: matches happy path, no-match returns 0 results with exit 0.
- **`export -o FILE [--with-history]`** — writes JSON. Test: create one project, export to a temp file, parse back, assert it round-trips with `import`.
- **`import FILE [--conflict skip|overwrite|fail]`** — reads JSON, applies `ImportSnapshot`. Test: round-trip with the export above.
- **`undo` / `redo`** — call `Workspace::undo` / `redo`. Tests: create+undo→list shows nothing; undo+redo→list shows what was created; undo on empty exits with code 3 (Validation/Conflict).

- [ ] **Steps 1–4** mirror previous CLI tasks.

```bash
cargo insta accept --workspace-root crates/kanban-cli
git add crates/kanban-cli/
git commit -m "feat(cli): implement status, search, export, import, undo, redo"
```

---

### Task 37: `kanban batch` (NDJSON stdin)

**Files:**
- Modify: `crates/kanban-cli/src/cmd/batch.rs`
- Create: `crates/kanban-cli/tests/batch.rs`

- [ ] **Step 1: Failing tests**

```rust
use assert_cmd::Command;
use std::io::Write;

#[test]
fn batch_runs_create_then_list() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("data.db");

    let input = r#"{"cmd":"project.create","args":{"name":"B","prefix":"BAT"}}
{"cmd":"issue.create","args":{"project":"BAT","title":"first"}}
{"cmd":"issue.list","args":{"project":"BAT"}}
"#;
    let out = Command::cargo_bin("kanban").unwrap()
        .env("KANBAN_DB", &db).args(["batch"])
        .write_stdin(input).output().unwrap();
    assert!(out.status.success());
    let lines: Vec<_> = String::from_utf8(out.stdout).unwrap().lines().map(str::to_string).collect();
    assert_eq!(lines.len(), 3);
    let last: serde_json::Value = serde_json::from_str(&lines[2]).unwrap();
    assert_eq!(last["ok"], true);
    assert_eq!(last["data"][0]["title"], "first");
}

#[test]
fn batch_continues_past_error_unless_fail_fast() {
    // first command errors, subsequent should still execute and produce {ok: false} then {ok: true}
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("data.db");
    let input = r#"{"cmd":"project.show","args":{"id_or_prefix":"NONE"}}
{"cmd":"project.create","args":{"name":"B","prefix":"BAT"}}
"#;
    let out = Command::cargo_bin("kanban").unwrap()
        .env("KANBAN_DB", &db).args(["batch"]).write_stdin(input).output().unwrap();
    assert!(out.status.success());
    let lines: Vec<_> = String::from_utf8(out.stdout).unwrap().lines().map(str::to_string).collect();
    assert_eq!(lines.len(), 2);
    let l0: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
    let l1: serde_json::Value = serde_json::from_str(&lines[1]).unwrap();
    assert_eq!(l0["ok"], false);
    assert_eq!(l1["ok"], true);
}

#[test]
fn batch_fail_fast_aborts_on_error() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("data.db");
    let input = r#"{"cmd":"project.show","args":{"id_or_prefix":"NONE"}}
{"cmd":"project.create","args":{"name":"B","prefix":"BAT"}}
"#;
    let out = Command::cargo_bin("kanban").unwrap()
        .env("KANBAN_DB", &db).args(["batch","--fail-fast"]).write_stdin(input).output().unwrap();
    assert!(!out.status.success());
}
```

- [ ] **Step 2: Implement**

In `crates/kanban-cli/src/cmd/batch.rs`:

```rust
use clap::Args;
use kanban_core::{Result, Workspace};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write, stdin, stdout};
use std::path::Path;

#[derive(Debug, Args)]
pub struct BatchArgs {
    #[arg(long)]
    pub fail_fast: bool,
}

#[derive(Debug, Deserialize)]
struct Line {
    cmd: String,
    args: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct OkLine<'a> {
    ok: bool,
    cmd: &'a str,
    data: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct ErrLine<'a> {
    ok: bool,
    cmd: &'a str,
    error: String,
}

pub fn run(args: BatchArgs, db: Option<&Path>, _out: &crate::output::Out) -> Result<()> {
    let mut ws = match db {
        Some(p) => Workspace::open(p)?,
        None => Workspace::open_default()?,
    };
    let stdout = stdout();
    let mut stdout = stdout.lock();
    let stdin = stdin();
    for line in stdin.lock().lines() {
        let line = line.map_err(kanban_core::Error::Io)?;
        let parsed: std::result::Result<Line, _> = serde_json::from_str(&line);
        let result = match parsed {
            Ok(l) => dispatch(&mut ws, &l),
            Err(e) => Err(kanban_core::Error::InvalidSnapshot(format!("invalid ndjson: {e}"))),
        };
        match result {
            Ok((cmd, data)) => {
                let s = serde_json::to_string(&OkLine { ok: true, cmd: &cmd, data }).unwrap();
                writeln!(stdout, "{s}").map_err(kanban_core::Error::Io)?;
            }
            Err(e) => {
                let s = serde_json::to_string(&ErrLine { ok: false, cmd: "", error: e.to_string() }).unwrap();
                writeln!(stdout, "{s}").map_err(kanban_core::Error::Io)?;
                if args.fail_fast { return Err(e); }
            }
        }
    }
    Ok(())
}

fn dispatch(ws: &mut Workspace, l: &Line) -> Result<(String, serde_json::Value)> {
    match l.cmd.as_str() {
        "project.create" => {
            #[derive(Deserialize)] struct A { name: String, prefix: String, #[serde(default)] description: Option<String>, #[serde(default)] icon: Option<String> }
            let a: A = serde_json::from_value(l.args.clone()).map_err(kanban_core::Error::Serde)?;
            let id = kanban_core::new_id();
            ws.apply(kanban_core::Operation::CreateProject(kanban_core::operation::CreateProject {
                id, name: a.name, prefix: a.prefix, description: a.description, icon: a.icon,
            }))?;
            let p = ws.query_project_by_id(id)?;
            Ok((l.cmd.clone(), serde_json::to_value(p).unwrap()))
        }
        "project.show" => {
            #[derive(Deserialize)] struct A { id_or_prefix: String }
            let a: A = serde_json::from_value(l.args.clone()).map_err(kanban_core::Error::Serde)?;
            let p = if let Ok(u) = uuid::Uuid::parse_str(&a.id_or_prefix) {
                ws.query_project_by_id(u)?
            } else {
                ws.query_projects()?.into_iter().find(|p| p.prefix == a.id_or_prefix)
                    .ok_or(kanban_core::Error::NotFound { kind: kanban_core::EntityKind::Project, id: a.id_or_prefix })?
            };
            Ok((l.cmd.clone(), serde_json::to_value(p).unwrap()))
        }
        "issue.create" => {
            #[derive(Deserialize)] struct A { project: String, title: String, #[serde(default)] description: Option<String>, #[serde(default)] priority: Option<String> }
            let a: A = serde_json::from_value(l.args.clone()).map_err(kanban_core::Error::Serde)?;
            let p = ws.query_projects()?.into_iter().find(|p| p.prefix == a.project)
                .ok_or(kanban_core::Error::NotFound { kind: kanban_core::EntityKind::Project, id: a.project })?;
            let sid = ws.query_statuses_for_project(p.id)?[0].id;
            let priority: kanban_core::types::Priority = a.priority.as_deref().unwrap_or("none").parse()?;
            let iid = kanban_core::new_id();
            ws.apply(kanban_core::Operation::CreateIssue(kanban_core::operation::CreateIssue {
                id: iid, project_id: p.id, title: a.title, description: a.description,
                status_id: sid, priority, due_date: None, label_ids: vec![],
            }))?;
            let issue = ws.query_issue_by_id(iid)?;
            Ok((l.cmd.clone(), serde_json::to_value(issue).unwrap()))
        }
        "issue.list" => {
            #[derive(Deserialize)] struct A { project: String }
            let a: A = serde_json::from_value(l.args.clone()).map_err(kanban_core::Error::Serde)?;
            let p = ws.query_projects()?.into_iter().find(|p| p.prefix == a.project)
                .ok_or(kanban_core::Error::NotFound { kind: kanban_core::EntityKind::Project, id: a.project })?;
            let issues = ws.query_issues(kanban_core::query::IssueFilter::for_project(p.id))?;
            Ok((l.cmd.clone(), serde_json::to_value(issues).unwrap()))
        }
        other => Err(kanban_core::Error::InvalidSnapshot(format!("unsupported batch cmd: {other}"))),
    }
}
```

(Batch supports a deliberate subset of the full CLI surface for v1: project create/show/update/delete and issue create/list/update/move. Add more commands by following the same pattern. Document the subset in the README.)

- [ ] **Step 3: Run + commit**

```bash
cargo test -p kanban-cli --test batch
git add crates/kanban-cli/
git commit -m "feat(cli): implement kanban batch with NDJSON stdin and per-line results"
```

---

## Phase 13 — Finalization

### Task 38: End-to-end smoke test

**Files:**
- Create: `crates/kanban-cli/tests/smoke_e2e.rs`

- [ ] **Step 1: Test**

```rust
use assert_cmd::Command;
use std::path::PathBuf;

fn isolated_db() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.db");
    (dir, path)
}

#[test]
fn full_v1_smoke_path() {
    let (_d, db) = isolated_db();
    let env = ("KANBAN_DB", db.to_string_lossy().to_string());
    let run = |args: &[&str]| {
        Command::cargo_bin("kanban").unwrap()
            .env(env.0, &env.1).args(args).assert().success()
    };

    run(&["project", "create", "Smoke", "--prefix", "SMK"]);
    run(&["label", "create", "--project", "SMK", "--name", "feat", "--color", "#3b82f6"]);
    run(&["issue", "create", "--project", "SMK", "--title", "first"]);
    run(&["issue", "create", "--project", "SMK", "--title", "second"]);
    run(&["issue", "create", "--project", "SMK", "--title", "third"]);
    run(&["label", "attach", "SMK-1", "feat"]);

    let out = Command::cargo_bin("kanban").unwrap()
        .env(env.0, &env.1).args(["--json","issue","list","--project","SMK"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 3);

    let dir = tempfile::tempdir().unwrap();
    let snap = dir.path().join("snap.json");
    run(&["export", "-o", snap.to_str().unwrap()]);
    assert!(snap.exists());

    // Undo all the way back, then assert empty.
    for _ in 0..6 { run(&["undo"]); }
    let out = Command::cargo_bin("kanban").unwrap()
        .env(env.0, &env.1).args(["--json","project","list"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 0);
}
```

- [ ] **Step 2: Run + commit**

```bash
cargo test -p kanban-cli --test smoke_e2e
git add crates/kanban-cli/tests/smoke_e2e.rs
git commit -m "test(cli): full v1 smoke path covering create→export→undo"
```

---

### Task 39: Cold-start performance smoke

**Files:**
- Create: `crates/kanban-cli/tests/perf.rs`

- [ ] **Step 1: Test**

```rust
use assert_cmd::Command;
use std::path::PathBuf;
use std::time::Instant;

#[test]
fn cold_start_invocation_under_50ms_p95() {
    let dir = tempfile::tempdir().unwrap();
    let db: PathBuf = dir.path().join("data.db");
    Command::cargo_bin("kanban").unwrap()
        .env("KANBAN_DB", &db).args(["project","create","X","--prefix","PRF"])
        .assert().success();

    let mut times = Vec::new();
    for _ in 0..40 {
        let start = Instant::now();
        Command::cargo_bin("kanban").unwrap()
            .env("KANBAN_DB", &db).args(["project","list"]).assert().success();
        times.push(start.elapsed().as_millis() as u64);
    }
    times.sort_unstable();
    let p95 = times[(times.len() * 95) / 100];
    assert!(p95 < 50, "cold-start p95 {p95}ms exceeds 50ms target; full sample: {times:?}");
}
```

- [ ] **Step 2: Run + commit (mark `#[ignore]` if it flakes on slow CI; never delete)**

If the test is too sensitive to CI noise, gate it with `#[cfg_attr(not(feature = "perf"), ignore)]` and document running it locally in DEVELOPMENT.md.

```bash
cargo test -p kanban-cli --test perf -- --ignored
git add crates/kanban-cli/tests/perf.rs
git commit -m "test(cli): cold-start p95 < 50ms smoke"
```

---

### Task 40: GitHub Actions CI

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Workflow file**

```yaml
name: CI

on:
  push:
    branches: [dev, main]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt

      - uses: Swatinem/rust-cache@v2

      - name: Format
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --workspace --all-targets -- -D warnings

      - name: Test
        run: cargo test --workspace --all-targets
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: run fmt, clippy, and full test suite on push and PR"
```

---

### Task 41: README and DEVELOPMENT docs

**Files:**
- Create: `README.md`
- Create: `DEVELOPMENT.md`

- [ ] **Step 1: README.md**

```markdown
# Kanban

Local-first project management. Two crates: `kanban-core` (Rust library) and `kanban-cli` (`kanban` binary). v1 covers projects, issues, labels, default statuses, search, sort, undo/redo, and JSON import/export. GUI, MCP, and AI-agent orchestration ship in later specs.

See `docs/superpowers/specs/2026-05-03-kanban-v2-core-cli-design.md` for the v1 design.

## Quick start

```sh
cargo install --path crates/kanban-cli
kanban project create "Auth Service" --prefix AUTH
kanban issue create --project AUTH --title "Add OAuth login" --priority high
kanban issue list --project AUTH --sort priority
kanban undo
```

By default the workspace lives at `~/.kanban/data.db`. Override with `KANBAN_DB=/path/to.db`.

## Development

See `DEVELOPMENT.md` for build, test, and contribution conventions.
```

- [ ] **Step 2: DEVELOPMENT.md**

```markdown
# Development

## Build

```sh
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
```

## TDD discipline

Every `Operation` variant lands as a failing test first. Every CLI subcommand has a happy-path snapshot and at least one error-path snapshot. The CI gate runs the full test suite — no `#[ignore]` lands in `main` without a comment explaining why.

## Migrations

SQL files in `crates/kanban-core/migrations/NNNN_name.sql` are embedded with `include_str!` and applied in numeric order on `Workspace::open`. **`0001_init.sql` is frozen after v1 ships** — schema changes go in `0002_*.sql` and beyond. Never edit a released migration.

## Snapshot tests

CLI tests use [`insta`](https://insta.rs). After a change to expected output:

```sh
cargo insta review --workspace-root crates/kanban-cli
```

Approve or reject each diff hunk. Snapshots live in `crates/kanban-cli/tests/snapshots/`.

## Performance

`crates/kanban-cli/tests/perf.rs` checks cold-start <50ms p95. It's `#[ignore]` by default; run locally with `cargo test --workspace -- --ignored`.
```

- [ ] **Step 3: Commit**

```bash
git add README.md DEVELOPMENT.md
git commit -m "docs: add README and DEVELOPMENT.md for v1"
```

---

## Definition of Done — v1 ships when

- [ ] `cargo test --workspace` passes (unit, integration, property, CLI snapshot, smoke).
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --all -- --check` clean.
- [ ] Every `Operation` variant has a paired apply-and-undo test.
- [ ] Every CLI subcommand has happy-path and error-path snapshot tests.
- [ ] `kanban export` round-trips through `kanban import --conflict fail` losslessly (covered by `snapshot_roundtrip.rs`).
- [ ] Cold-start CLI invocation completes in <50ms p95 on an empty DB (covered by `perf.rs`).
- [ ] One end-to-end smoke (`smoke_e2e.rs`) covers create → label → search → export → undo back to empty.

If any of those is red, v1 is not done. Open follow-up tasks rather than weakening the bar.



