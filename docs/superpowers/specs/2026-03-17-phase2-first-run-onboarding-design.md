# Phase 2: First-Run Onboarding — Auto-Setup with Sensible Defaults

## Context

Phase 1 gave us a single `kanban` binary that works out of the box with SQLite. On first run, the database is created and the schema is applied, but it's empty — no members, no projects. Phase 2 adds minimal auto-setup so the app feels ready to use immediately.

## Decision

Auto-create a default member on first run. No default project — the user creates their first project via the GUI or CLI. This keeps the setup zero-friction while avoiding assumptions about what the user wants to build.

## Design

### Detection

After schema creation in `init_db()`, check if the database is fresh:

```sql
SELECT COUNT(*) FROM members
```

If the count is 0, the database is new and needs seeding.

### What Gets Created

A single default member:

| Field | Value |
|-------|-------|
| name | "You" |
| display_name | "You" |
| email | NULL |
| avatar_color | "#6366f1" |

### Implementation

A `seed_defaults()` async function in `src/db/mod.rs`, called at the end of `init_db()` for both SQLite and Postgres backends. The function is idempotent — it only inserts if no members exist.

```rust
async fn seed_defaults(pool: &AnyPool) -> Result<(), Box<dyn std::error::Error>> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM members")
        .fetch_one(pool).await?;
    if count.0 == 0 {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        sqlx::query(
            "INSERT INTO members (name, display_name, avatar_color, created_at) VALUES ($1, $2, $3, $4)"
        )
        .bind("You").bind("You").bind("#6366f1").bind(&now)
        .execute(pool).await?;
    }
    Ok(())
}
```

### GUI Behavior

No changes. The existing empty-state UI handles the case where no projects exist. User clicks "New Project" to get started.

### CLI Behavior

No changes. If the user runs a command that requires a project before creating one, the existing error response is sufficient.

### Files Changed

| File | Change |
|------|--------|
| `src-tauri/src/db/mod.rs` | Add `seed_defaults()`, call it at end of `init_db()` |

### Risks

None. This is a trivial, idempotent insert guarded by a count check.
