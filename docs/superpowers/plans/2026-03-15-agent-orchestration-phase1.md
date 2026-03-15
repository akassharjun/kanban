# Agent Orchestration Phase 1: Solo Agent Autonomy

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable a single AI agent to autonomously register, pull structured task contracts, execute work, log execution traces, and report results — all via CLI and MCP.

**Architecture:** Extend the existing Kanban system with 5 new database tables (agents, agent_stats, task_contracts, execution_logs, project_agent_config), new Rust models, a routing engine, task state machine, and new CLI/MCP commands. The existing issues table stays unchanged; task_contracts extends it via a 1:1 relationship.

**Tech Stack:** Rust, SQLite (sqlx), clap (CLI), JSON-RPC 2.0 (MCP), uuid crate (agent IDs)

**Spec:** `docs/superpowers/specs/2026-03-15-next-gen-ai-agent-kanban-design.md`

---

## File Structure

### New Files

| File | Responsibility |
|------|---------------|
| `src-tauri/migrations/20260315000000_agent_orchestration.sql` | Schema for 5 new tables |
| `src-tauri/src/models/agent.rs` | Agent, AgentStats, TaskContract, ExecutionLog, ProjectAgentConfig structs |
| `src-tauri/src/orchestration/mod.rs` | Module root |
| `src-tauri/src/orchestration/state_machine.rs` | TaskState enum, valid transitions |
| `src-tauri/src/orchestration/routing.rs` | Routing engine: filter, sort, atomic claim |
| `src-tauri/src/commands/agents.rs` | Tauri commands for agent CRUD |
| `src-tauri/src/commands/task_contracts.rs` | Tauri commands for task contract operations |
| `src-tauri/src/commands/execution_logs.rs` | Tauri commands for execution log operations |

### Modified Files

| File | Changes |
|------|---------|
| `src-tauri/Cargo.toml` | Add `uuid` dependency |
| `src-tauri/src/lib.rs` | Add `orchestration` module, register new commands |
| `src-tauri/src/models/mod.rs` | Re-export agent module |
| `src-tauri/src/commands/mod.rs` | Add new command modules |
| `src-tauri/src/bin/cli.rs` | Add `Agent` and `Task` subcommands |
| `src-tauri/src/bin/mcp.rs` | Add new MCP tools |

---

## Chunk 1: Database Migration & Models

### Task 1: Database Migration

**Files:**
- Create: `src-tauri/migrations/20260315000000_agent_orchestration.sql`

- [ ] **Step 1: Write the migration SQL**

```sql
-- Agent registry
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    type TEXT,
    skills JSON NOT NULL DEFAULT '[]',
    task_types JSON NOT NULL DEFAULT '[]',
    max_concurrent INTEGER NOT NULL DEFAULT 1,
    max_complexity TEXT NOT NULL DEFAULT 'large',
    status TEXT NOT NULL DEFAULT 'idle',
    registered_at DATETIME NOT NULL DEFAULT (datetime('now')),
    last_heartbeat DATETIME NOT NULL DEFAULT (datetime('now'))
);

-- Agent stats
CREATE TABLE IF NOT EXISTS agent_stats (
    agent_id TEXT PRIMARY KEY REFERENCES agents(id) ON DELETE CASCADE,
    tasks_completed INTEGER NOT NULL DEFAULT 0,
    tasks_failed INTEGER NOT NULL DEFAULT 0,
    total_confidence REAL NOT NULL DEFAULT 0.0,
    total_completion_time_seconds INTEGER NOT NULL DEFAULT 0,
    skills_breakdown JSON NOT NULL DEFAULT '{}'
);

-- Task contracts (extends issues 1:1)
CREATE TABLE IF NOT EXISTS task_contracts (
    issue_id INTEGER PRIMARY KEY REFERENCES issues(id) ON DELETE CASCADE,
    type TEXT NOT NULL DEFAULT 'implementation',
    task_state TEXT NOT NULL DEFAULT 'queued',
    objective TEXT NOT NULL DEFAULT '',
    context JSON NOT NULL DEFAULT '{}',
    constraints JSON NOT NULL DEFAULT '[]',
    success_criteria JSON NOT NULL DEFAULT '[]',
    required_skills JSON NOT NULL DEFAULT '[]',
    estimated_complexity TEXT DEFAULT 'medium',
    claimed_by TEXT REFERENCES agents(id) ON DELETE SET NULL,
    claimed_at DATETIME,
    timeout_minutes INTEGER NOT NULL DEFAULT 30,
    result JSON,
    attempt_count INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_task_contracts_state ON task_contracts(task_state);
CREATE INDEX IF NOT EXISTS idx_task_contracts_claimed_by ON task_contracts(claimed_by);

-- Execution logs
CREATE TABLE IF NOT EXISTS execution_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL,
    attempt_number INTEGER NOT NULL DEFAULT 1,
    entry_type TEXT NOT NULL,
    message TEXT NOT NULL,
    metadata JSON,
    timestamp DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_execution_logs_issue ON execution_logs(issue_id);
CREATE INDEX IF NOT EXISTS idx_execution_logs_agent ON execution_logs(agent_id);

-- Project agent configuration
CREATE TABLE IF NOT EXISTS project_agent_config (
    project_id INTEGER PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
    auto_accept_threshold REAL NOT NULL DEFAULT 0.85,
    human_review_threshold REAL NOT NULL DEFAULT 0.50,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    heartbeat_interval_seconds INTEGER NOT NULL DEFAULT 60,
    missed_heartbeats_before_offline INTEGER NOT NULL DEFAULT 3
);
```

- [ ] **Step 2: Verify migration compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles successfully (sqlx will pick up migration at runtime)

- [ ] **Step 3: Commit**

```bash
git add src-tauri/migrations/20260315000000_agent_orchestration.sql
git commit -m "feat: add agent orchestration migration (5 new tables)"
```

---

### Task 2: Add uuid dependency

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add uuid crate**

Add to `[dependencies]`:
```toml
uuid = { version = "1", features = ["v4"] }
```

- [ ] **Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles successfully

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "feat: add uuid dependency for agent ID generation"
```

---

### Task 3: Rust Models

**Files:**
- Create: `src-tauri/src/models/agent.rs`
- Modify: `src-tauri/src/models/mod.rs`

- [ ] **Step 1: Create agent models file**

```rust
// src-tauri/src/models/agent.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub r#type: Option<String>,
    pub skills: String,        // JSON array
    pub task_types: String,    // JSON array
    pub max_concurrent: i64,
    pub max_complexity: String,
    pub status: String,
    pub registered_at: String,
    pub last_heartbeat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AgentStats {
    pub agent_id: String,
    pub tasks_completed: i64,
    pub tasks_failed: i64,
    pub total_confidence: f64,
    pub total_completion_time_seconds: i64,
    pub skills_breakdown: String, // JSON
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TaskContract {
    pub issue_id: i64,
    pub r#type: String,
    pub task_state: String,
    pub objective: String,
    pub context: String,          // JSON
    pub constraints: String,      // JSON array
    pub success_criteria: String, // JSON array
    pub required_skills: String,  // JSON array
    pub estimated_complexity: Option<String>,
    pub claimed_by: Option<String>,
    pub claimed_at: Option<String>,
    pub timeout_minutes: i64,
    pub result: Option<String>,   // JSON
    pub attempt_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ExecutionLog {
    pub id: i64,
    pub issue_id: i64,
    pub agent_id: String,
    pub attempt_number: i64,
    pub entry_type: String,
    pub message: String,
    pub metadata: Option<String>, // JSON
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProjectAgentConfig {
    pub project_id: i64,
    pub auto_accept_threshold: f64,
    pub human_review_threshold: f64,
    pub max_attempts: i64,
    pub heartbeat_interval_seconds: i64,
    pub missed_heartbeats_before_offline: i64,
}
```

- [ ] **Step 2: Re-export from models/mod.rs**

Add to the top of `src-tauri/src/models/mod.rs`:
```rust
pub mod agent;
pub use agent::*;
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/models/agent.rs src-tauri/src/models/mod.rs
git commit -m "feat: add agent orchestration models"
```

---

## Chunk 2: State Machine & Routing Engine

### Task 4: Task State Machine

**Files:**
- Create: `src-tauri/src/orchestration/mod.rs`
- Create: `src-tauri/src/orchestration/state_machine.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create orchestration module root**

```rust
// src-tauri/src/orchestration/mod.rs
pub mod state_machine;
pub mod routing;
```

- [ ] **Step 2: Create state machine**

```rust
// src-tauri/src/orchestration/state_machine.rs
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Queued,
    Claimed,
    Executing,
    Validating,
    Completed,
    Blocked,
    Cancelled,
}

impl TaskState {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskState::Queued => "queued",
            TaskState::Claimed => "claimed",
            TaskState::Executing => "executing",
            TaskState::Validating => "validating",
            TaskState::Completed => "completed",
            TaskState::Blocked => "blocked",
            TaskState::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "queued" => Ok(TaskState::Queued),
            "claimed" => Ok(TaskState::Claimed),
            "executing" => Ok(TaskState::Executing),
            "validating" => Ok(TaskState::Validating),
            "completed" => Ok(TaskState::Completed),
            "blocked" => Ok(TaskState::Blocked),
            "cancelled" => Ok(TaskState::Cancelled),
            _ => Err(format!("Invalid task state: {}", s)),
        }
    }

    /// Returns valid transitions from the current state.
    pub fn valid_transitions(&self) -> &'static [TaskState] {
        match self {
            TaskState::Queued => &[TaskState::Claimed, TaskState::Blocked, TaskState::Cancelled],
            TaskState::Claimed => &[TaskState::Executing, TaskState::Queued, TaskState::Blocked, TaskState::Cancelled],
            TaskState::Executing => &[TaskState::Validating, TaskState::Queued, TaskState::Blocked, TaskState::Cancelled],
            TaskState::Validating => &[TaskState::Completed, TaskState::Queued, TaskState::Blocked, TaskState::Cancelled],
            TaskState::Completed => &[TaskState::Queued, TaskState::Cancelled], // re-queue via invalidate
            TaskState::Blocked => &[TaskState::Queued, TaskState::Cancelled],
            TaskState::Cancelled => &[], // terminal
        }
    }

    pub fn can_transition_to(&self, target: TaskState) -> bool {
        self.valid_transitions().contains(&target)
    }
}

impl fmt::Display for TaskState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Maps a TaskState to the appropriate status category for syncing with issues.status_id.
pub fn task_state_to_status_category(state: TaskState) -> &'static str {
    match state {
        TaskState::Queued => "unstarted",
        TaskState::Claimed | TaskState::Executing | TaskState::Validating => "started",
        TaskState::Completed => "completed",
        TaskState::Blocked => "blocked",
        TaskState::Cancelled => "discarded",
    }
}
```

- [ ] **Step 3: Add orchestration module to lib.rs**

Add to `src-tauri/src/lib.rs` after `pub mod models;`:
```rust
pub mod orchestration;
```

- [ ] **Step 4: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles successfully

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/orchestration/ src-tauri/src/lib.rs
git commit -m "feat: add task state machine with valid transitions"
```

---

### Task 5: Routing Engine

**Files:**
- Create: `src-tauri/src/orchestration/routing.rs`

- [ ] **Step 1: Create routing engine**

```rust
// src-tauri/src/orchestration/routing.rs
use sqlx::SqlitePool;
use crate::models::{TaskContract, Issue};

/// Full task contract joined with issue data, returned by next_task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FullTaskContract {
    // Issue fields
    pub identifier: String,
    pub title: String,
    pub description: Option<String>,
    pub priority: String,
    pub parent_id: Option<i64>,
    // Contract fields
    pub issue_id: i64,
    pub r#type: String,
    pub task_state: String,
    pub objective: String,
    pub context: serde_json::Value,
    pub constraints: serde_json::Value,
    pub success_criteria: serde_json::Value,
    pub required_skills: serde_json::Value,
    pub estimated_complexity: Option<String>,
    pub timeout_minutes: i64,
    pub attempt_count: i64,
}

/// Complexity ordering for comparison.
fn complexity_rank(c: &str) -> i32 {
    match c {
        "small" => 1,
        "medium" => 2,
        "large" => 3,
        _ => 2,
    }
}

/// Find the next available task for an agent and atomically claim it.
/// Returns None if no suitable task is available.
pub async fn next_task(
    pool: &SqlitePool,
    agent_id: &str,
    agent_skills: &[String],
    agent_max_complexity: &str,
    agent_max_concurrent: i64,
) -> Result<Option<FullTaskContract>, sqlx::Error> {
    // Check agent capacity
    let current_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM task_contracts WHERE claimed_by = ? AND task_state IN ('claimed', 'executing')"
    ).bind(agent_id).fetch_one(pool).await?;

    if current_count >= agent_max_concurrent {
        return Ok(None);
    }

    // Get candidate tasks: queued, all dependencies resolved
    // A task is unblocked if it has no 'blocks'/'blocked_by' relations pointing to incomplete issues
    let candidates: Vec<(i64, String, String, Option<String>, String)> = sqlx::query_as(
        "SELECT tc.issue_id, i.priority, tc.required_skills, tc.estimated_complexity, i.identifier
         FROM task_contracts tc
         JOIN issues i ON tc.issue_id = i.id
         WHERE tc.task_state = 'queued'
           AND NOT EXISTS (
             SELECT 1 FROM issue_relations ir
             JOIN issues dep ON ir.source_issue_id = dep.id
             JOIN task_contracts dtc ON dtc.issue_id = dep.id
             WHERE ir.target_issue_id = tc.issue_id
               AND ir.relation_type = 'blocks'
               AND dtc.task_state NOT IN ('completed')
           )
         ORDER BY
           CASE i.priority
             WHEN 'urgent' THEN 0
             WHEN 'high' THEN 1
             WHEN 'medium' THEN 2
             WHEN 'low' THEN 3
             ELSE 4
           END,
           i.created_at ASC"
    ).fetch_all(pool).await?;

    let max_rank = complexity_rank(agent_max_complexity);

    for (issue_id, _priority, required_skills_json, complexity, _identifier) in &candidates {
        // Check complexity
        let task_complexity = complexity.as_deref().unwrap_or("medium");
        if complexity_rank(task_complexity) > max_rank {
            continue;
        }

        // Check skills match
        let required: Vec<String> = serde_json::from_str(required_skills_json).unwrap_or_default();
        if !required.is_empty() {
            let all_matched = required.iter().all(|r| agent_skills.contains(r));
            if !all_matched {
                continue;
            }
        }

        // Attempt atomic claim
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let rows_affected = sqlx::query(
            "UPDATE task_contracts SET claimed_by = ?, claimed_at = ?, task_state = 'claimed'
             WHERE issue_id = ? AND claimed_by IS NULL AND task_state = 'queued'"
        )
        .bind(agent_id)
        .bind(&now)
        .bind(issue_id)
        .execute(pool)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            // Already claimed by another agent, try next
            continue;
        }

        // Sync issues.status_id to a 'started' category status
        let started_status_id: Option<i64> = sqlx::query_scalar(
            "SELECT s.id FROM statuses s
             JOIN issues i ON s.project_id = i.project_id
             WHERE i.id = ? AND s.category = 'started'
             ORDER BY s.position LIMIT 1"
        ).bind(issue_id).fetch_optional(pool).await?;

        if let Some(sid) = started_status_id {
            sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?")
                .bind(sid).bind(&now).bind(issue_id).execute(pool).await?;
        }

        // Log claim in execution_logs
        let attempt = sqlx::query_scalar::<_, i64>(
            "SELECT attempt_count FROM task_contracts WHERE issue_id = ?"
        ).bind(issue_id).fetch_one(pool).await?;

        sqlx::query(
            "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp)
             VALUES (?, ?, ?, 'claim', 'Task claimed', ?)"
        ).bind(issue_id).bind(agent_id).bind(attempt + 1).bind(&now).execute(pool).await?;

        // Build full contract response
        let contract = build_full_contract(pool, *issue_id).await?;
        return Ok(contract);
    }

    Ok(None)
}

/// Build a FullTaskContract from issue_id.
pub async fn build_full_contract(pool: &SqlitePool, issue_id: i64) -> Result<Option<FullTaskContract>, sqlx::Error> {
    let row: Option<(
        String, String, Option<String>, String, Option<i64>, // issue fields
        i64, String, String, String, String, String, String, String, Option<String>, i64, i64 // contract fields
    )> = sqlx::query_as(
        "SELECT i.identifier, i.title, i.description, i.priority, i.parent_id,
                tc.issue_id, tc.type, tc.task_state, tc.objective, tc.context,
                tc.constraints, tc.success_criteria, tc.required_skills,
                tc.estimated_complexity, tc.timeout_minutes, tc.attempt_count
         FROM task_contracts tc
         JOIN issues i ON tc.issue_id = i.id
         WHERE tc.issue_id = ?"
    ).bind(issue_id).fetch_optional(pool).await?;

    let Some((identifier, title, description, priority, parent_id,
              issue_id, r#type, task_state, objective, context_str,
              constraints_str, criteria_str, skills_str,
              complexity, timeout, attempts)) = row else {
        return Ok(None);
    };

    Ok(Some(FullTaskContract {
        identifier,
        title,
        description,
        priority,
        parent_id,
        issue_id,
        r#type,
        task_state,
        objective,
        context: serde_json::from_str(&context_str).unwrap_or(serde_json::json!({})),
        constraints: serde_json::from_str(&constraints_str).unwrap_or(serde_json::json!([])),
        success_criteria: serde_json::from_str(&criteria_str).unwrap_or(serde_json::json!([])),
        required_skills: serde_json::from_str(&skills_str).unwrap_or(serde_json::json!([])),
        estimated_complexity: complexity,
        timeout_minutes: timeout,
        attempt_count: attempts,
    }))
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles successfully

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/orchestration/routing.rs
git commit -m "feat: add routing engine with atomic task claiming"
```

---

## Chunk 3: Agent Commands

### Task 6: Agent Tauri Commands

**Files:**
- Create: `src-tauri/src/commands/agents.rs`
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: Create agent commands**

```rust
// src-tauri/src/commands/agents.rs
use crate::models::{Agent, AgentStats};
use crate::state::AppState;
use tauri::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterAgentInput {
    pub name: String,
    pub r#type: Option<String>,
    pub skills: Vec<String>,
    pub task_types: Vec<String>,
    pub max_concurrent: Option<i64>,
    pub max_complexity: Option<String>,
}

#[tauri::command]
pub fn register_agent(state: State<AppState>, input: RegisterAgentInput) -> Result<Agent, String> {
    state.rt.block_on(async {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let skills_json = serde_json::to_string(&input.skills).unwrap();
        let types_json = serde_json::to_string(&input.task_types).unwrap();
        let max_concurrent = input.max_concurrent.unwrap_or(1);
        let max_complexity = input.max_complexity.unwrap_or_else(|| "large".to_string());

        sqlx::query(
            "INSERT INTO agents (id, name, type, skills, task_types, max_concurrent, max_complexity, status, registered_at, last_heartbeat)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'idle', ?, ?)"
        )
        .bind(&id).bind(&input.name).bind(&input.r#type)
        .bind(&skills_json).bind(&types_json)
        .bind(max_concurrent).bind(&max_complexity)
        .bind(&now).bind(&now)
        .execute(&state.pool).await.map_err(|e| e.to_string())?;

        // Create stats row
        sqlx::query("INSERT INTO agent_stats (agent_id) VALUES (?)")
            .bind(&id).execute(&state.pool).await.map_err(|e| e.to_string())?;

        let agent = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = ?")
            .bind(&id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        Ok(agent)
    })
}

#[tauri::command]
pub fn agent_heartbeat(state: State<AppState>, agent_id: String) -> Result<Agent, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        sqlx::query("UPDATE agents SET last_heartbeat = ?, status = 'idle' WHERE id = ?")
            .bind(&now).bind(&agent_id)
            .execute(&state.pool).await.map_err(|e| e.to_string())?;

        // Update status based on active tasks
        let active: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM task_contracts WHERE claimed_by = ? AND task_state IN ('claimed', 'executing')"
        ).bind(&agent_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let new_status = if active > 0 { "busy" } else { "idle" };
        sqlx::query("UPDATE agents SET status = ? WHERE id = ?")
            .bind(new_status).bind(&agent_id)
            .execute(&state.pool).await.map_err(|e| e.to_string())?;

        sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = ?")
            .bind(&agent_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn deregister_agent(state: State<AppState>, agent_id: String) -> Result<(), String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        // Reclaim all active tasks before deleting
        let active_tasks: Vec<i64> = sqlx::query_scalar(
            "SELECT issue_id FROM task_contracts WHERE claimed_by = ? AND task_state IN ('claimed', 'executing')"
        ).bind(&agent_id).fetch_all(&state.pool).await.map_err(|e| e.to_string())?;

        for issue_id in &active_tasks {
            sqlx::query(
                "UPDATE task_contracts SET claimed_by = NULL, claimed_at = NULL, task_state = 'queued' WHERE issue_id = ?"
            ).bind(issue_id).execute(&state.pool).await.map_err(|e| e.to_string())?;

            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp)
                 VALUES (?, ?, (SELECT attempt_count FROM task_contracts WHERE issue_id = ?), 'error', 'Agent deregistered, task reclaimed', ?)"
            ).bind(issue_id).bind(&agent_id).bind(issue_id).bind(&now)
            .execute(&state.pool).await.map_err(|e| e.to_string())?;
        }

        sqlx::query("DELETE FROM agents WHERE id = ?")
            .bind(&agent_id).execute(&state.pool).await.map_err(|e| e.to_string())?;

        Ok(())
    })
}

#[tauri::command]
pub fn list_agents(state: State<AppState>) -> Result<Vec<Agent>, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, Agent>("SELECT * FROM agents ORDER BY registered_at DESC")
            .fetch_all(&state.pool).await.map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn get_agent_stats(state: State<AppState>, agent_id: String) -> Result<AgentStats, String> {
    state.rt.block_on(async {
        sqlx::query_as::<_, AgentStats>("SELECT * FROM agent_stats WHERE agent_id = ?")
            .bind(&agent_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())
    })
}
```

- [ ] **Step 2: Add to commands/mod.rs**

Add to `src-tauri/src/commands/mod.rs`:
```rust
pub mod agents;
pub mod task_contracts;
pub mod execution_logs;
```

- [ ] **Step 3: Register commands in lib.rs**

Add to the `invoke_handler` in `src-tauri/src/lib.rs`:
```rust
// Agents
commands::agents::register_agent,
commands::agents::agent_heartbeat,
commands::agents::deregister_agent,
commands::agents::list_agents,
commands::agents::get_agent_stats,
```

- [ ] **Step 4: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles successfully

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/agents.rs src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add agent registration commands"
```

---

## Chunk 4: Task Contract Commands

### Task 7: Task Contract Commands

**Files:**
- Create: `src-tauri/src/commands/task_contracts.rs`

- [ ] **Step 1: Create task contract commands**

```rust
// src-tauri/src/commands/task_contracts.rs
use crate::models::TaskContract;
use crate::orchestration::routing::{self, FullTaskContract};
use crate::orchestration::state_machine::TaskState;
use crate::state::AppState;
use tauri::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateTaskContractInput {
    pub project_id: i64,
    pub title: String,
    pub r#type: Option<String>,
    pub objective: String,
    pub description: Option<String>,
    pub status_id: i64,
    pub priority: Option<String>,
    pub assignee_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub skills: Option<Vec<String>>,
    pub complexity: Option<String>,
    pub constraints: Option<Vec<String>>,
    pub success_criteria: Option<serde_json::Value>,
    pub context_files: Option<Vec<String>>,
    pub timeout_minutes: Option<i64>,
    pub depends_on: Option<Vec<String>>, // identifiers
}

#[tauri::command]
pub fn create_task_contract(state: State<AppState>, input: CreateTaskContractInput) -> Result<FullTaskContract, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let priority = input.priority.unwrap_or_else(|| "medium".to_string());
        let task_type = input.r#type.unwrap_or_else(|| "implementation".to_string());

        let mut tx = state.pool.begin().await.map_err(|e| e.to_string())?;

        // Create the issue first
        let (counter, prefix): (i64, String) = sqlx::query_as(
            "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = ? RETURNING issue_counter, prefix"
        ).bind(input.project_id).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
        let identifier = format!("{}-{}", prefix, counter);

        let max_pos: Option<f64> = sqlx::query_scalar(
            "SELECT MAX(position) FROM issues WHERE project_id = ? AND status_id = ?"
        ).bind(input.project_id).bind(input.status_id)
        .fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
        let position = max_pos.unwrap_or(-1.0) + 1.0;

        let result = sqlx::query(
            "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(input.project_id).bind(&identifier).bind(&input.title)
        .bind(&input.description).bind(input.status_id).bind(&priority)
        .bind(input.assignee_id).bind(input.parent_id).bind(position)
        .bind(&now).bind(&now)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

        let issue_id = result.last_insert_rowid();

        // Build context JSON
        let context = serde_json::json!({
            "files": input.context_files.unwrap_or_default(),
            "related_tasks": [],
            "prior_attempts": []
        });

        let skills_json = serde_json::to_string(&input.skills.unwrap_or_default()).unwrap();
        let constraints_json = serde_json::to_string(&input.constraints.unwrap_or_default()).unwrap();
        let criteria_json = serde_json::to_string(&input.success_criteria.unwrap_or(serde_json::json!([]))).unwrap();

        // Create the task contract
        sqlx::query(
            "INSERT INTO task_contracts (issue_id, type, task_state, objective, context, constraints, success_criteria, required_skills, estimated_complexity, timeout_minutes)
             VALUES (?, ?, 'queued', ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(issue_id).bind(&task_type).bind(&input.objective)
        .bind(context.to_string()).bind(&constraints_json).bind(&criteria_json)
        .bind(&skills_json).bind(input.complexity.as_deref().unwrap_or("medium"))
        .bind(input.timeout_minutes.unwrap_or(30))
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;

        // Create dependency relations if specified
        if let Some(deps) = &input.depends_on {
            for dep_identifier in deps {
                let dep_issue_id: Option<i64> = sqlx::query_scalar(
                    "SELECT id FROM issues WHERE identifier = ?"
                ).bind(dep_identifier).fetch_optional(&mut *tx).await.map_err(|e| e.to_string())?;

                if let Some(dep_id) = dep_issue_id {
                    sqlx::query(
                        "INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES (?, ?, 'blocks')"
                    ).bind(dep_id).bind(issue_id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
                }
            }
        }

        tx.commit().await.map_err(|e| e.to_string())?;

        routing::build_full_contract(&state.pool, issue_id).await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Failed to build contract".to_string())
    })
}

#[tauri::command]
pub fn get_task_contract(state: State<AppState>, identifier: String) -> Result<FullTaskContract, String> {
    state.rt.block_on(async {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
            .bind(&identifier).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        routing::build_full_contract(&state.pool, issue_id).await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "No task contract found for this issue".to_string())
    })
}

#[tauri::command]
pub fn next_task(state: State<AppState>, agent_id: String, skills_override: Option<Vec<String>>) -> Result<Option<FullTaskContract>, String> {
    state.rt.block_on(async {
        let agent = sqlx::query_as::<_, crate::models::Agent>("SELECT * FROM agents WHERE id = ?")
            .bind(&agent_id).fetch_one(&state.pool).await
            .map_err(|_| "AGENT_NOT_REGISTERED".to_string())?;

        let skills: Vec<String> = if let Some(ov) = skills_override {
            ov
        } else {
            serde_json::from_str(&agent.skills).unwrap_or_default()
        };

        routing::next_task(
            &state.pool,
            &agent_id,
            &skills,
            &agent.max_complexity,
            agent.max_concurrent,
        ).await.map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn start_task(state: State<AppState>, agent_id: String, identifier: String) -> Result<(), String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
            .bind(&identifier).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?")
            .bind(issue_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let current = TaskState::from_str(&contract.task_state)?;
        if !current.can_transition_to(TaskState::Executing) {
            return Err(format!("TASK_WRONG_STATE: cannot transition from {} to executing", contract.task_state));
        }
        if contract.claimed_by.as_deref() != Some(&agent_id) {
            return Err("TASK_NOT_CLAIMED_BY_AGENT".to_string());
        }

        sqlx::query("UPDATE task_contracts SET task_state = 'executing' WHERE issue_id = ?")
            .bind(issue_id).execute(&state.pool).await.map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp)
             VALUES (?, ?, ?, 'checkpoint', 'Execution started', ?)"
        ).bind(issue_id).bind(&agent_id).bind(contract.attempt_count + 1).bind(&now)
        .execute(&state.pool).await.map_err(|e| e.to_string())?;

        Ok(())
    })
}

#[derive(Deserialize)]
pub struct CompleteTaskInput {
    pub identifier: String,
    pub agent_id: String,
    pub confidence: f64,
    pub summary: String,
    pub artifacts: Option<serde_json::Value>,
}

#[tauri::command]
pub fn complete_task(state: State<AppState>, input: CompleteTaskInput) -> Result<serde_json::Value, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
            .bind(&input.identifier).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?")
            .bind(issue_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        if contract.claimed_by.as_deref() != Some(&input.agent_id) {
            return Err("TASK_NOT_CLAIMED_BY_AGENT".to_string());
        }

        let current = TaskState::from_str(&contract.task_state)?;
        if !current.can_transition_to(TaskState::Validating) && !current.can_transition_to(TaskState::Completed) {
            return Err(format!("TASK_WRONG_STATE: cannot complete from {}", contract.task_state));
        }

        // Get project config for thresholds
        let issue = sqlx::query_as::<_, crate::models::Issue>("SELECT * FROM issues WHERE id = ?")
            .bind(issue_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let config = sqlx::query_as::<_, crate::models::ProjectAgentConfig>(
            "SELECT * FROM project_agent_config WHERE project_id = ?"
        ).bind(issue.project_id).fetch_optional(&state.pool).await.map_err(|e| e.to_string())?;

        let auto_accept = config.as_ref().map(|c| c.auto_accept_threshold).unwrap_or(0.85);
        let human_review = config.as_ref().map(|c| c.human_review_threshold).unwrap_or(0.50);

        let result_json = serde_json::json!({
            "status": "completed",
            "confidence": input.confidence,
            "summary": input.summary,
            "artifacts": input.artifacts
        });

        let (new_state, accepted) = if input.confidence >= auto_accept {
            ("completed", true)
        } else if input.confidence >= human_review {
            ("validating", false)
        } else {
            // Auto-reject: treat as failure
            ("queued", false)
        };

        // Update contract
        sqlx::query(
            "UPDATE task_contracts SET task_state = ?, result = ?, claimed_by = CASE WHEN ? = 'queued' THEN NULL ELSE claimed_by END, claimed_at = CASE WHEN ? = 'queued' THEN NULL ELSE claimed_at END, attempt_count = CASE WHEN ? = 'queued' THEN attempt_count + 1 ELSE attempt_count END WHERE issue_id = ?"
        ).bind(new_state).bind(result_json.to_string())
        .bind(new_state).bind(new_state).bind(new_state)
        .bind(issue_id)
        .execute(&state.pool).await.map_err(|e| e.to_string())?;

        // Sync issue status
        let status_category = crate::orchestration::state_machine::task_state_to_status_category(
            TaskState::from_str(new_state)?
        );
        let target_status: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM statuses WHERE project_id = ? AND category = ? ORDER BY position LIMIT 1"
        ).bind(issue.project_id).bind(status_category).fetch_optional(&state.pool).await.map_err(|e| e.to_string())?;

        if let Some(sid) = target_status {
            sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?")
                .bind(sid).bind(&now).bind(issue_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        }

        // Log result
        sqlx::query(
            "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, metadata, timestamp)
             VALUES (?, ?, ?, 'result', ?, ?, ?)"
        ).bind(issue_id).bind(&input.agent_id).bind(contract.attempt_count + 1)
        .bind(&input.summary).bind(result_json.to_string()).bind(&now)
        .execute(&state.pool).await.map_err(|e| e.to_string())?;

        // Update agent stats
        if accepted {
            sqlx::query(
                "UPDATE agent_stats SET tasks_completed = tasks_completed + 1, total_confidence = total_confidence + ? WHERE agent_id = ?"
            ).bind(input.confidence).bind(&input.agent_id)
            .execute(&state.pool).await.map_err(|e| e.to_string())?;
        }

        Ok(serde_json::json!({ "accepted": accepted, "new_state": new_state }))
    })
}

#[tauri::command]
pub fn fail_task(state: State<AppState>, agent_id: String, identifier: String, reason: String) -> Result<serde_json::Value, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
            .bind(&identifier).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?")
            .bind(issue_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let new_attempt = contract.attempt_count + 1;

        // Append to prior_attempts in context
        let mut context: serde_json::Value = serde_json::from_str(&contract.context).unwrap_or(serde_json::json!({}));
        let attempt_entry = serde_json::json!({
            "agent": agent_id,
            "attempt_number": new_attempt,
            "result": "failed",
            "reason": reason
        });
        if let Some(arr) = context.get_mut("prior_attempts") {
            if let Some(a) = arr.as_array_mut() {
                a.push(attempt_entry);
            }
        } else {
            context["prior_attempts"] = serde_json::json!([attempt_entry]);
        }

        // Re-queue
        sqlx::query(
            "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL, attempt_count = ?, context = ? WHERE issue_id = ?"
        ).bind(new_attempt).bind(context.to_string()).bind(issue_id)
        .execute(&state.pool).await.map_err(|e| e.to_string())?;

        // Log failure
        sqlx::query(
            "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp)
             VALUES (?, ?, ?, 'result', ?, ?)"
        ).bind(issue_id).bind(&agent_id).bind(new_attempt).bind(&reason).bind(&now)
        .execute(&state.pool).await.map_err(|e| e.to_string())?;

        // Update agent stats
        sqlx::query("UPDATE agent_stats SET tasks_failed = tasks_failed + 1 WHERE agent_id = ?")
            .bind(&agent_id).execute(&state.pool).await.map_err(|e| e.to_string())?;

        // Sync issue status back to unstarted
        let issue = sqlx::query_as::<_, crate::models::Issue>("SELECT * FROM issues WHERE id = ?")
            .bind(issue_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;
        let unstarted_status: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM statuses WHERE project_id = ? AND category = 'unstarted' ORDER BY position LIMIT 1"
        ).bind(issue.project_id).fetch_optional(&state.pool).await.map_err(|e| e.to_string())?;
        if let Some(sid) = unstarted_status {
            sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?")
                .bind(sid).bind(&now).bind(issue_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        }

        Ok(serde_json::json!({ "requeued": true, "attempt_number": new_attempt }))
    })
}

#[tauri::command]
pub fn unclaim_task(state: State<AppState>, agent_id: String, identifier: String) -> Result<(), String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
            .bind(&identifier).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?")
            .bind(issue_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        if contract.claimed_by.as_deref() != Some(&agent_id) {
            return Err("TASK_NOT_CLAIMED_BY_AGENT".to_string());
        }

        sqlx::query(
            "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL WHERE issue_id = ?"
        ).bind(issue_id).execute(&state.pool).await.map_err(|e| e.to_string())?;

        sqlx::query(
            "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp)
             VALUES (?, ?, ?, 'checkpoint', 'Task unclaimed voluntarily', ?)"
        ).bind(issue_id).bind(&agent_id).bind(contract.attempt_count + 1).bind(&now)
        .execute(&state.pool).await.map_err(|e| e.to_string())?;

        Ok(())
    })
}
```

- [ ] **Step 1b: Add approve_task and reject_task to task_contracts.rs**

Append to the same file:

```rust
#[tauri::command]
pub fn approve_task(state: State<AppState>, identifier: String) -> Result<(), String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
            .bind(&identifier).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;
        let issue = sqlx::query_as::<_, crate::models::Issue>("SELECT * FROM issues WHERE id = ?")
            .bind(issue_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;
        let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?")
            .bind(issue_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;
        if contract.task_state != "validating" { return Err("TASK_WRONG_STATE: not in validating state".into()); }

        sqlx::query("UPDATE task_contracts SET task_state = 'completed' WHERE issue_id = ?")
            .bind(issue_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        let completed_sid: Option<i64> = sqlx::query_scalar("SELECT id FROM statuses WHERE project_id = ? AND category = 'completed' ORDER BY position LIMIT 1")
            .bind(issue.project_id).fetch_optional(&state.pool).await.map_err(|e| e.to_string())?;
        if let Some(sid) = completed_sid {
            sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?")
                .bind(sid).bind(&now).bind(issue_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        }
        if let Some(agent_id) = &contract.claimed_by {
            sqlx::query("UPDATE agent_stats SET tasks_completed = tasks_completed + 1 WHERE agent_id = ?")
                .bind(agent_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    })
}

#[tauri::command]
pub fn reject_task(state: State<AppState>, identifier: String) -> Result<(), String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
            .bind(&identifier).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;
        let issue = sqlx::query_as::<_, crate::models::Issue>("SELECT * FROM issues WHERE id = ?")
            .bind(issue_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;
        let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?")
            .bind(issue_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;
        if contract.task_state != "validating" { return Err("TASK_WRONG_STATE: not in validating state".into()); }

        sqlx::query("UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL, attempt_count = attempt_count + 1 WHERE issue_id = ?")
            .bind(issue_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        let unstarted_sid: Option<i64> = sqlx::query_scalar("SELECT id FROM statuses WHERE project_id = ? AND category = 'unstarted' ORDER BY position LIMIT 1")
            .bind(issue.project_id).fetch_optional(&state.pool).await.map_err(|e| e.to_string())?;
        if let Some(sid) = unstarted_sid {
            sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?")
                .bind(sid).bind(&now).bind(issue_id).execute(&state.pool).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    })
}
```

- [ ] **Step 2: Register task contract commands in lib.rs**

Add to the `invoke_handler` in `src-tauri/src/lib.rs`:
```rust
// Task Contracts
commands::task_contracts::create_task_contract,
commands::task_contracts::get_task_contract,
commands::task_contracts::next_task,
commands::task_contracts::start_task,
commands::task_contracts::complete_task,
commands::task_contracts::fail_task,
commands::task_contracts::unclaim_task,
commands::task_contracts::approve_task,
commands::task_contracts::reject_task,
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/task_contracts.rs src-tauri/src/lib.rs
git commit -m "feat: add task contract commands (create, next, start, complete, fail, unclaim)"
```

---

## Chunk 5: Execution Log Commands

### Task 8: Execution Log Commands

**Files:**
- Create: `src-tauri/src/commands/execution_logs.rs`

- [ ] **Step 1: Create execution log commands**

```rust
// src-tauri/src/commands/execution_logs.rs
use crate::models::ExecutionLog;
use crate::state::AppState;
use tauri::State;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LogEntryInput {
    pub identifier: String,
    pub agent_id: String,
    pub entry_type: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

#[tauri::command]
pub fn log_task_activity(state: State<AppState>, input: LogEntryInput) -> Result<i64, String> {
    state.rt.block_on(async {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
            .bind(&input.identifier).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let attempt: i64 = sqlx::query_scalar(
            "SELECT COALESCE(attempt_count, 0) + 1 FROM task_contracts WHERE issue_id = ?"
        ).bind(issue_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let meta_str = input.metadata.map(|m| m.to_string());

        let result = sqlx::query(
            "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, metadata, timestamp)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(issue_id).bind(&input.agent_id).bind(attempt)
        .bind(&input.entry_type).bind(&input.message).bind(&meta_str).bind(&now)
        .execute(&state.pool).await.map_err(|e| e.to_string())?;

        Ok(result.last_insert_rowid())
    })
}

#[tauri::command]
pub fn task_replay(state: State<AppState>, identifier: String) -> Result<Vec<ExecutionLog>, String> {
    state.rt.block_on(async {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
            .bind(&identifier).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        sqlx::query_as::<_, ExecutionLog>(
            "SELECT * FROM execution_logs WHERE issue_id = ? ORDER BY timestamp ASC"
        ).bind(issue_id).fetch_all(&state.pool).await.map_err(|e| e.to_string())
    })
}

#[tauri::command]
pub fn task_attempts(state: State<AppState>, identifier: String) -> Result<serde_json::Value, String> {
    state.rt.block_on(async {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?")
            .bind(&identifier).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let contract = sqlx::query_as::<_, crate::models::TaskContract>(
            "SELECT * FROM task_contracts WHERE issue_id = ?"
        ).bind(issue_id).fetch_one(&state.pool).await.map_err(|e| e.to_string())?;

        let context: serde_json::Value = serde_json::from_str(&contract.context).unwrap_or(serde_json::json!({}));
        let prior = context.get("prior_attempts").cloned().unwrap_or(serde_json::json!([]));

        Ok(serde_json::json!({
            "identifier": identifier,
            "total_attempts": contract.attempt_count,
            "prior_attempts": prior
        }))
    })
}
```

- [ ] **Step 2: Register in lib.rs**

Add to the `invoke_handler`:
```rust
// Execution Logs
commands::execution_logs::log_task_activity,
commands::execution_logs::task_replay,
commands::execution_logs::task_attempts,
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles successfully

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/execution_logs.rs src-tauri/src/lib.rs
git commit -m "feat: add execution log commands (log, replay, attempts)"
```

---

## Chunk 6: CLI Extensions

### Task 9: CLI Agent & Task Commands

**Files:**
- Modify: `src-tauri/src/bin/cli.rs`

- [ ] **Step 1: Add Agent subcommand enum**

Add to the `Commands` enum in cli.rs:
```rust
/// Manage AI agents
Agent {
    #[command(subcommand)]
    action: AgentAction,
},
/// Manage task contracts
Task {
    #[command(subcommand)]
    action: TaskAction,
},
/// View system metrics
Metrics {
    #[arg(long)]
    project: Option<i64>,
    #[arg(long)]
    agent: Option<String>,
},
```

- [ ] **Step 2: Add AgentAction enum**

```rust
#[derive(Subcommand)]
enum AgentAction {
    /// Register a new agent
    Register {
        #[arg(long)]
        name: String,
        #[arg(long, value_delimiter = ',')]
        skills: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        task_types: Option<Vec<String>>,
        #[arg(long, default_value = "1")]
        max_concurrent: i64,
        #[arg(long, default_value = "large")]
        max_complexity: String,
    },
    /// Send heartbeat
    Heartbeat {
        #[arg(long)]
        id: String,
    },
    /// Deregister an agent
    Deregister {
        #[arg(long)]
        id: String,
    },
    /// List all agents
    List,
    /// Get agent stats
    Stats { id: String },
}
```

- [ ] **Step 3: Add TaskAction enum**

```rust
#[derive(Subcommand)]
enum TaskAction {
    /// Get next available task (atomic claim)
    Next {
        #[arg(long)]
        agent: String,
        #[arg(long, value_delimiter = ',')]
        skills: Option<Vec<String>>,
    },
    /// Mark task as executing
    Start {
        identifier: String,
        #[arg(long)]
        agent: String,
    },
    /// Complete a task
    Complete {
        identifier: String,
        #[arg(long)]
        agent: String,
        #[arg(long)]
        confidence: f64,
        #[arg(long)]
        summary: String,
        #[arg(long)]
        artifacts: Option<String>,
    },
    /// Fail a task
    Fail {
        identifier: String,
        #[arg(long)]
        agent: String,
        #[arg(long)]
        reason: String,
    },
    /// Voluntarily unclaim a task
    Unclaim {
        identifier: String,
        #[arg(long)]
        agent: String,
    },
    /// Log execution activity
    Log {
        identifier: String,
        #[arg(long)]
        agent: String,
        #[arg(long, name = "type")]
        entry_type: String,
        #[arg(long)]
        message: String,
        #[arg(long)]
        meta: Option<String>,
    },
    /// Create a task contract
    Create {
        #[arg(long)]
        project: i64,
        #[arg(long)]
        title: String,
        #[arg(long)]
        objective: String,
        #[arg(long)]
        status: i64,
        #[arg(long, name = "type")]
        task_type: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long, value_delimiter = ',')]
        skills: Option<Vec<String>>,
        #[arg(long)]
        complexity: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long, value_delimiter = ',')]
        depends_on: Option<Vec<String>>,
        #[arg(long, value_delimiter = ',')]
        context_files: Option<Vec<String>>,
        #[arg(long)]
        constraints: Option<String>,
        #[arg(long)]
        success_criteria: Option<String>,
        #[arg(long)]
        assignee: Option<i64>,
        #[arg(long)]
        timeout: Option<i64>,
    },
    /// Get full task contract
    Get { identifier: String },
    /// Show execution replay
    Replay { identifier: String },
    /// Show prior attempts
    Attempts { identifier: String },
    /// List task contracts
    List {
        #[arg(long)]
        project: i64,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        available: bool,
    },
    /// Show task children
    Children { identifier: String },
    /// Approve a validating task (human)
    Approve { identifier: String },
    /// Reject a validating task (human)
    Reject { identifier: String },
    /// Invalidate a completed task
    Invalidate {
        identifier: String,
        #[arg(long)]
        reason: String,
    },
    /// Search task contracts
    Search {
        #[arg(long)]
        project: i64,
        query: String,
    },
    /// Update a task contract
    Update {
        identifier: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        complexity: Option<String>,
        #[arg(long, value_delimiter = ',')]
        skills: Option<Vec<String>>,
    },
}
```

- [ ] **Step 4: Add handler for Agent commands**

Add a match arm in the `main()` function:
```rust
Commands::Agent { action } => match action {
    AgentAction::Register { name, skills, task_types, max_concurrent, max_complexity } => {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let skills_json = serde_json::to_string(&skills).unwrap();
        let types = task_types.unwrap_or_else(|| vec!["implementation".into(), "research".into(), "review".into(), "decomposition".into()]);
        let types_json = serde_json::to_string(&types).unwrap();

        sqlx::query("INSERT INTO agents (id, name, skills, task_types, max_concurrent, max_complexity, status, registered_at, last_heartbeat) VALUES (?, ?, ?, ?, ?, ?, 'idle', ?, ?)")
            .bind(&id).bind(&name).bind(&skills_json).bind(&types_json)
            .bind(max_concurrent).bind(&max_complexity).bind(&now).bind(&now)
            .execute(&pool).await?;

        sqlx::query("INSERT INTO agent_stats (agent_id) VALUES (?)").bind(&id).execute(&pool).await?;

        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": {"agent_id": id, "name": name}}));
        } else {
            println!("Agent registered: {} ({})", name, id);
        }
    }
    AgentAction::Heartbeat { id } => {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        sqlx::query("UPDATE agents SET last_heartbeat = ? WHERE id = ?").bind(&now).bind(&id).execute(&pool).await?;

        let active: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task_contracts WHERE claimed_by = ? AND task_state IN ('claimed', 'executing')").bind(&id).fetch_one(&pool).await?;
        let status = if active > 0 { "busy" } else { "idle" };
        sqlx::query("UPDATE agents SET status = ? WHERE id = ?").bind(status).bind(&id).execute(&pool).await?;

        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": {"status": status, "active_tasks": active}}));
        } else {
            println!("Heartbeat OK (status: {}, active tasks: {})", status, active);
        }
    }
    AgentAction::Deregister { id } => {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        // Reclaim active tasks
        let active: Vec<i64> = sqlx::query_scalar("SELECT issue_id FROM task_contracts WHERE claimed_by = ? AND task_state IN ('claimed', 'executing')").bind(&id).fetch_all(&pool).await?;
        for issue_id in &active {
            sqlx::query("UPDATE task_contracts SET claimed_by = NULL, claimed_at = NULL, task_state = 'queued' WHERE issue_id = ?").bind(issue_id).execute(&pool).await?;
            sqlx::query("INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, ?, (SELECT attempt_count FROM task_contracts WHERE issue_id = ?), 'error', 'Agent deregistered', ?)").bind(issue_id).bind(&id).bind(issue_id).bind(&now).execute(&pool).await?;
        }
        sqlx::query("DELETE FROM agents WHERE id = ?").bind(&id).execute(&pool).await?;
        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": {"reclaimed_tasks": active.len()}}));
        } else {
            println!("Agent deregistered ({} tasks reclaimed)", active.len());
        }
    }
    AgentAction::List => {
        let agents = sqlx::query_as::<_, Agent>("SELECT * FROM agents ORDER BY registered_at DESC").fetch_all(&pool).await?;
        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": agents}));
        } else {
            if agents.is_empty() {
                println!("No agents registered.");
            } else {
                for a in &agents {
                    println!("{} | {} | {} | skills: {}", a.id, a.name, a.status, a.skills);
                }
            }
        }
    }
    AgentAction::Stats { id } => {
        let stats = sqlx::query_as::<_, AgentStats>("SELECT * FROM agent_stats WHERE agent_id = ?").bind(&id).fetch_one(&pool).await?;
        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": stats}));
        } else {
            let avg_conf = if stats.tasks_completed > 0 { stats.total_confidence / stats.tasks_completed as f64 } else { 0.0 };
            println!("Agent: {}", id);
            println!("Completed: {} | Failed: {} | Avg Confidence: {:.2}", stats.tasks_completed, stats.tasks_failed, avg_conf);
        }
    }
},
```

- [ ] **Step 5: Add handler for Task commands**

This is a large match block. Add the handler for the `Task` subcommand. Key commands:

```rust
Commands::Task { action } => match action {
    TaskAction::Next { agent, skills } => {
        let agent_row = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = ?").bind(&agent).fetch_one(&pool).await?;
        let agent_skills: Vec<String> = if let Some(s) = skills { s } else { serde_json::from_str(&agent_row.skills).unwrap_or_default() };

        let result = kanban_lib::orchestration::routing::next_task(&pool, &agent, &agent_skills, &agent_row.max_complexity, agent_row.max_concurrent).await?;
        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": result}));
        } else if let Some(task) = &result {
            println!("Claimed: {} - {}", task.identifier, task.title);
            println!("Type: {} | Complexity: {}", task.r#type, task.estimated_complexity.as_deref().unwrap_or("medium"));
            println!("Objective: {}", task.objective);
        } else {
            println!("No tasks available.");
        }
    }
    TaskAction::Start { identifier, agent } => {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?").bind(&identifier).fetch_one(&pool).await?;
        let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?").bind(issue_id).fetch_one(&pool).await?;
        if contract.claimed_by.as_deref() != Some(&agent) {
            eprintln!("Error: task not claimed by this agent");
            std::process::exit(1);
        }
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        sqlx::query("UPDATE task_contracts SET task_state = 'executing' WHERE issue_id = ?").bind(issue_id).execute(&pool).await?;
        sqlx::query("INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, ?, ?, 'checkpoint', 'Execution started', ?)").bind(issue_id).bind(&agent).bind(contract.attempt_count + 1).bind(&now).execute(&pool).await?;
        if cli.json {
            println!("{}", serde_json::json!({"success": true}));
        } else {
            println!("Task {} started", identifier);
        }
    }
    TaskAction::Complete { identifier, agent, confidence, summary, artifacts } => {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?").bind(&identifier).fetch_one(&pool).await?;
        let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?").bind(issue_id).fetch_one(&pool).await?;
        let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?").bind(issue_id).fetch_one(&pool).await?;
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

        let config = sqlx::query_as::<_, ProjectAgentConfig>("SELECT * FROM project_agent_config WHERE project_id = ?").bind(issue.project_id).fetch_optional(&pool).await?;
        let auto_accept = config.as_ref().map(|c| c.auto_accept_threshold).unwrap_or(0.85);
        let human_review = config.as_ref().map(|c| c.human_review_threshold).unwrap_or(0.50);

        let artifacts_val: serde_json::Value = artifacts.as_deref().map(|a| serde_json::from_str(a).unwrap_or(serde_json::json!(null))).unwrap_or(serde_json::json!(null));
        let result_json = serde_json::json!({"status": "completed", "confidence": confidence, "summary": summary, "artifacts": artifacts_val});

        let (new_state, accepted) = if confidence >= auto_accept { ("completed", true) } else if confidence >= human_review { ("validating", false) } else { ("queued", false) };

        sqlx::query("UPDATE task_contracts SET task_state = ?, result = ?, claimed_by = CASE WHEN ? = 'queued' THEN NULL ELSE claimed_by END, claimed_at = CASE WHEN ? = 'queued' THEN NULL ELSE claimed_at END, attempt_count = CASE WHEN ? = 'queued' THEN attempt_count + 1 ELSE attempt_count END WHERE issue_id = ?")
            .bind(new_state).bind(result_json.to_string()).bind(new_state).bind(new_state).bind(new_state).bind(issue_id).execute(&pool).await?;

        // Sync status
        let category = kanban_lib::orchestration::state_machine::task_state_to_status_category(kanban_lib::orchestration::state_machine::TaskState::from_str(new_state).unwrap());
        if let Some(sid) = sqlx::query_scalar::<_, i64>("SELECT id FROM statuses WHERE project_id = ? AND category = ? ORDER BY position LIMIT 1").bind(issue.project_id).bind(category).fetch_optional(&pool).await? {
            sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?").bind(sid).bind(&now).bind(issue_id).execute(&pool).await?;
        }

        sqlx::query("INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, metadata, timestamp) VALUES (?, ?, ?, 'result', ?, ?, ?)").bind(issue_id).bind(&agent).bind(contract.attempt_count + 1).bind(&summary).bind(result_json.to_string()).bind(&now).execute(&pool).await?;

        if accepted {
            sqlx::query("UPDATE agent_stats SET tasks_completed = tasks_completed + 1, total_confidence = total_confidence + ? WHERE agent_id = ?").bind(confidence).bind(&agent).execute(&pool).await?;
        }

        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": {"accepted": accepted, "new_state": new_state}}));
        } else {
            println!("Task {} -> {} (confidence: {:.2}, accepted: {})", identifier, new_state, confidence, accepted);
        }
    }
    TaskAction::Fail { identifier, agent, reason } => {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?").bind(&identifier).fetch_one(&pool).await?;
        let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?").bind(issue_id).fetch_one(&pool).await?;
        let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?").bind(issue_id).fetch_one(&pool).await?;
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let new_attempt = contract.attempt_count + 1;

        let mut context: serde_json::Value = serde_json::from_str(&contract.context).unwrap_or(serde_json::json!({}));
        let entry = serde_json::json!({"agent": agent, "attempt_number": new_attempt, "result": "failed", "reason": reason});
        if let Some(arr) = context.get_mut("prior_attempts").and_then(|v| v.as_array_mut()) { arr.push(entry); } else { context["prior_attempts"] = serde_json::json!([entry]); }

        sqlx::query("UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL, attempt_count = ?, context = ? WHERE issue_id = ?").bind(new_attempt).bind(context.to_string()).bind(issue_id).execute(&pool).await?;
        sqlx::query("INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, ?, ?, 'result', ?, ?)").bind(issue_id).bind(&agent).bind(new_attempt).bind(&reason).bind(&now).execute(&pool).await?;
        sqlx::query("UPDATE agent_stats SET tasks_failed = tasks_failed + 1 WHERE agent_id = ?").bind(&agent).execute(&pool).await?;

        // Sync issue status back to unstarted
        if let Some(sid) = sqlx::query_scalar::<_, i64>("SELECT id FROM statuses WHERE project_id = ? AND category = 'unstarted' ORDER BY position LIMIT 1").bind(issue.project_id).fetch_optional(&pool).await? {
            sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?").bind(sid).bind(&now).bind(issue_id).execute(&pool).await?;
        }

        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": {"requeued": true, "attempt_number": new_attempt}}));
        } else {
            println!("Task {} failed (attempt {}), re-queued", identifier, new_attempt);
        }
    }
    TaskAction::Unclaim { identifier, agent } => {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?").bind(&identifier).fetch_one(&pool).await?;
        let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?").bind(issue_id).fetch_one(&pool).await?;
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        sqlx::query("UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL WHERE issue_id = ?").bind(issue_id).execute(&pool).await?;
        // Sync issue status back to unstarted
        if let Some(sid) = sqlx::query_scalar::<_, i64>("SELECT id FROM statuses WHERE project_id = ? AND category = 'unstarted' ORDER BY position LIMIT 1").bind(issue.project_id).fetch_optional(&pool).await? {
            sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?").bind(sid).bind(&now).bind(issue_id).execute(&pool).await?;
        }
        sqlx::query("INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, ?, (SELECT attempt_count + 1 FROM task_contracts WHERE issue_id = ?), 'checkpoint', 'Task unclaimed', ?)").bind(issue_id).bind(&agent).bind(issue_id).bind(&now).execute(&pool).await?;
        if cli.json { println!("{}", serde_json::json!({"success": true})); } else { println!("Task {} unclaimed", identifier); }
    }
    TaskAction::Log { identifier, agent, entry_type, message, meta } => {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?").bind(&identifier).fetch_one(&pool).await?;
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let attempt: i64 = sqlx::query_scalar("SELECT COALESCE(attempt_count, 0) + 1 FROM task_contracts WHERE issue_id = ?").bind(issue_id).fetch_one(&pool).await?;
        sqlx::query("INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, metadata, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?)").bind(issue_id).bind(&agent).bind(attempt).bind(&entry_type).bind(&message).bind(&meta).bind(&now).execute(&pool).await?;
        if cli.json { println!("{}", serde_json::json!({"success": true})); } else { println!("Logged: [{}] {}", entry_type, message); }
    }
    TaskAction::Create { project, title, objective, status, task_type, priority, skills, complexity, description, parent, depends_on, context_files, constraints, success_criteria, assignee, timeout } => {
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let prio = priority.unwrap_or_else(|| "medium".into());
        let ttype = task_type.unwrap_or_else(|| "implementation".into());

        let mut tx = pool.begin().await?;
        let (counter, prefix): (i64, String) = sqlx::query_as("UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = ? RETURNING issue_counter, prefix").bind(project).fetch_one(&mut *tx).await?;
        let identifier = format!("{}-{}", prefix, counter);
        let max_pos: Option<f64> = sqlx::query_scalar("SELECT MAX(position) FROM issues WHERE project_id = ? AND status_id = ?").bind(project).bind(status).fetch_one(&mut *tx).await?;
        let position = max_pos.unwrap_or(-1.0) + 1.0;

        // Resolve parent identifier to id
        let parent_id: Option<i64> = if let Some(ref p) = parent {
            sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?").bind(p).fetch_optional(&mut *tx).await?
        } else { None };

        let result = sqlx::query("INSERT INTO issues (project_id, identifier, title, description, status_id, priority, assignee_id, parent_id, position, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(project).bind(&identifier).bind(&title).bind(&description).bind(status).bind(&prio).bind(assignee).bind(parent_id).bind(position).bind(&now).bind(&now)
            .execute(&mut *tx).await?;
        let issue_id = result.last_insert_rowid();

        let ctx = serde_json::json!({"files": context_files.unwrap_or_default(), "related_tasks": [], "prior_attempts": []});
        let skills_json = serde_json::to_string(&skills.unwrap_or_default()).unwrap();
        let constraints_val: serde_json::Value = constraints.as_deref().map(|c| serde_json::from_str(c).unwrap_or(serde_json::json!([]))).unwrap_or(serde_json::json!([]));
        let criteria_val: serde_json::Value = success_criteria.as_deref().map(|c| serde_json::from_str(c).unwrap_or(serde_json::json!([]))).unwrap_or(serde_json::json!([]));

        sqlx::query("INSERT INTO task_contracts (issue_id, type, task_state, objective, context, constraints, success_criteria, required_skills, estimated_complexity, timeout_minutes) VALUES (?, ?, 'queued', ?, ?, ?, ?, ?, ?, ?)")
            .bind(issue_id).bind(&ttype).bind(&objective).bind(ctx.to_string()).bind(constraints_val.to_string()).bind(criteria_val.to_string()).bind(&skills_json).bind(complexity.as_deref().unwrap_or("medium")).bind(timeout.unwrap_or(30))
            .execute(&mut *tx).await?;

        if let Some(deps) = &depends_on {
            for dep in deps {
                if let Some(dep_id) = sqlx::query_scalar::<_, i64>("SELECT id FROM issues WHERE identifier = ?").bind(dep).fetch_optional(&mut *tx).await? {
                    sqlx::query("INSERT INTO issue_relations (source_issue_id, target_issue_id, relation_type) VALUES (?, ?, 'blocks')").bind(dep_id).bind(issue_id).execute(&mut *tx).await?;
                }
            }
        }

        tx.commit().await?;

        if cli.json {
            let contract = kanban_lib::orchestration::routing::build_full_contract(&pool, issue_id).await?;
            println!("{}", serde_json::json!({"success": true, "data": contract}));
        } else {
            println!("Created: {} - {}", identifier, title);
        }
    }
    TaskAction::Get { identifier } => {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?").bind(&identifier).fetch_one(&pool).await?;
        let contract = kanban_lib::orchestration::routing::build_full_contract(&pool, issue_id).await?;
        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": contract}));
        } else if let Some(c) = &contract {
            println!("{} - {}", c.identifier, c.title);
            println!("State: {} | Type: {} | Complexity: {}", c.task_state, c.r#type, c.estimated_complexity.as_deref().unwrap_or("medium"));
            println!("Objective: {}", c.objective);
            println!("Attempts: {}", c.attempt_count);
        } else {
            println!("No task contract found for {}", identifier);
        }
    }
    TaskAction::Replay { identifier } => {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?").bind(&identifier).fetch_one(&pool).await?;
        let logs = sqlx::query_as::<_, ExecutionLog>("SELECT * FROM execution_logs WHERE issue_id = ? ORDER BY timestamp ASC").bind(issue_id).fetch_all(&pool).await?;
        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": logs}));
        } else {
            let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?").bind(issue_id).fetch_one(&pool).await?;
            println!("{}: {}", identifier, issue.title);
            println!("---");
            for log in &logs {
                let type_label = match log.entry_type.as_str() {
                    "claim" => "CLAIM",
                    "reasoning" => "THINK",
                    "file_read" => "READ",
                    "file_edit" => "EDIT",
                    "command" => "RUN",
                    "discovery" => "DISCOVER",
                    "error" => "ERROR",
                    "result" => "RESULT",
                    "checkpoint" => "CHECK",
                    _ => &log.entry_type,
                };
                println!("[{}] {:<8} {}", &log.timestamp[11..19], type_label, log.message);
            }
        }
    }
    TaskAction::Attempts { identifier } => {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?").bind(&identifier).fetch_one(&pool).await?;
        let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?").bind(issue_id).fetch_one(&pool).await?;
        let context: serde_json::Value = serde_json::from_str(&contract.context).unwrap_or(serde_json::json!({}));
        let prior = context.get("prior_attempts").cloned().unwrap_or(serde_json::json!([]));
        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": {"total_attempts": contract.attempt_count, "prior_attempts": prior}}));
        } else {
            println!("{}: {} attempts", identifier, contract.attempt_count);
            if let Some(arr) = prior.as_array() {
                for a in arr {
                    println!("  #{} by {} - {}: {}", a["attempt_number"], a["agent"], a["result"], a["reason"]);
                }
            }
        }
    }
    TaskAction::List { project, status, agent, available } => {
        let mut qb: sqlx::QueryBuilder<sqlx::Sqlite> = sqlx::QueryBuilder::new(
            "SELECT tc.* FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = "
        );
        qb.push_bind(project);
        if available { qb.push(" AND tc.task_state = 'queued'"); }
        if let Some(ref s) = status { qb.push(" AND tc.task_state = "); qb.push_bind(s.clone()); }
        if let Some(ref a) = agent { qb.push(" AND tc.claimed_by = "); qb.push_bind(a.clone()); }
        qb.push(" ORDER BY i.created_at");

        let contracts = qb.build_query_as::<TaskContract>().fetch_all(&pool).await?;

        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": contracts}));
        } else {
            for c in &contracts {
                let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?").bind(c.issue_id).fetch_one(&pool).await?;
                println!("{} | {} | {} | {}", issue.identifier, c.task_state, c.r#type, issue.title);
            }
        }
    }
    TaskAction::Children { identifier } => {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?").bind(&identifier).fetch_one(&pool).await?;
        let children = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE parent_id = ? ORDER BY position").bind(issue_id).fetch_all(&pool).await?;
        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": children}));
        } else {
            for c in &children {
                let tc = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?").bind(c.id).fetch_optional(&pool).await?;
                let state = tc.map(|t| t.task_state).unwrap_or_else(|| "no-contract".into());
                println!("  {} | {} | {}", c.identifier, state, c.title);
            }
        }
    }
},
    TaskAction::Approve { identifier } => {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?").bind(&identifier).fetch_one(&pool).await?;
        let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?").bind(issue_id).fetch_one(&pool).await?;
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?").bind(issue_id).fetch_one(&pool).await?;
        if contract.task_state != "validating" { eprintln!("Error: task is not in validating state"); std::process::exit(1); }
        sqlx::query("UPDATE task_contracts SET task_state = 'completed' WHERE issue_id = ?").bind(issue_id).execute(&pool).await?;
        if let Some(sid) = sqlx::query_scalar::<_, i64>("SELECT id FROM statuses WHERE project_id = ? AND category = 'completed' ORDER BY position LIMIT 1").bind(issue.project_id).fetch_optional(&pool).await? {
            sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?").bind(sid).bind(&now).bind(issue_id).execute(&pool).await?;
        }
        sqlx::query("INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, 'human', ?, 'result', 'Task approved by human', ?)").bind(issue_id).bind(contract.attempt_count + 1).bind(&now).execute(&pool).await?;
        if let Some(agent_id) = &contract.claimed_by {
            sqlx::query("UPDATE agent_stats SET tasks_completed = tasks_completed + 1 WHERE agent_id = ?").bind(agent_id).execute(&pool).await?;
        }
        if cli.json { println!("{}", serde_json::json!({"success": true})); } else { println!("Task {} approved", identifier); }
    }
    TaskAction::Reject { identifier } => {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?").bind(&identifier).fetch_one(&pool).await?;
        let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?").bind(issue_id).fetch_one(&pool).await?;
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        let contract = sqlx::query_as::<_, TaskContract>("SELECT * FROM task_contracts WHERE issue_id = ?").bind(issue_id).fetch_one(&pool).await?;
        if contract.task_state != "validating" { eprintln!("Error: task is not in validating state"); std::process::exit(1); }
        let new_attempt = contract.attempt_count + 1;
        sqlx::query("UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL, attempt_count = ? WHERE issue_id = ?").bind(new_attempt).bind(issue_id).execute(&pool).await?;
        if let Some(sid) = sqlx::query_scalar::<_, i64>("SELECT id FROM statuses WHERE project_id = ? AND category = 'unstarted' ORDER BY position LIMIT 1").bind(issue.project_id).fetch_optional(&pool).await? {
            sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?").bind(sid).bind(&now).bind(issue_id).execute(&pool).await?;
        }
        sqlx::query("INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, 'human', ?, 'result', 'Task rejected by human', ?)").bind(issue_id).bind(new_attempt).bind(&now).execute(&pool).await?;
        if cli.json { println!("{}", serde_json::json!({"success": true, "data": {"requeued": true}})); } else { println!("Task {} rejected, re-queued", identifier); }
    }
    TaskAction::Invalidate { identifier, reason } => {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?").bind(&identifier).fetch_one(&pool).await?;
        let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?").bind(issue_id).fetch_one(&pool).await?;
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        // Re-queue the task
        sqlx::query("UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL, attempt_count = attempt_count + 1 WHERE issue_id = ?").bind(issue_id).execute(&pool).await?;
        // Block downstream tasks
        let downstream: Vec<i64> = sqlx::query_scalar("SELECT target_issue_id FROM issue_relations WHERE source_issue_id = ? AND relation_type = 'blocks'").bind(issue_id).fetch_all(&pool).await?;
        for dep_id in &downstream {
            sqlx::query("UPDATE task_contracts SET task_state = 'blocked' WHERE issue_id = ? AND task_state IN ('queued', 'claimed', 'blocked')").bind(dep_id).execute(&pool).await?;
        }
        if let Some(sid) = sqlx::query_scalar::<_, i64>("SELECT id FROM statuses WHERE project_id = ? AND category = 'unstarted' ORDER BY position LIMIT 1").bind(issue.project_id).fetch_optional(&pool).await? {
            sqlx::query("UPDATE issues SET status_id = ?, updated_at = ? WHERE id = ?").bind(sid).bind(&now).bind(issue_id).execute(&pool).await?;
        }
        sqlx::query("INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES (?, 'system', 1, 'error', ?, ?)").bind(issue_id).bind(format!("Invalidated: {}", reason)).bind(&now).execute(&pool).await?;
        if cli.json { println!("{}", serde_json::json!({"success": true, "data": {"tasks_blocked": downstream}})); } else { println!("Task {} invalidated, {} downstream tasks blocked", identifier, downstream.len()); }
    }
    TaskAction::Search { project, query } => {
        let pattern = format!("%{}%", query);
        let results = sqlx::query_as::<_, TaskContract>("SELECT tc.* FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = ? AND (i.title LIKE ? OR tc.objective LIKE ? OR i.identifier LIKE ?) ORDER BY i.updated_at DESC")
            .bind(project).bind(&pattern).bind(&pattern).bind(&pattern).fetch_all(&pool).await?;
        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": results}));
        } else {
            for c in &results {
                let issue = sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = ?").bind(c.issue_id).fetch_one(&pool).await?;
                println!("{} | {} | {}", issue.identifier, c.task_state, issue.title);
            }
        }
    }
    TaskAction::Update { identifier, title, priority, complexity, skills } => {
        let issue_id: i64 = sqlx::query_scalar("SELECT id FROM issues WHERE identifier = ?").bind(&identifier).fetch_one(&pool).await?;
        let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
        if let Some(ref t) = title { sqlx::query("UPDATE issues SET title = ?, updated_at = ? WHERE id = ?").bind(t).bind(&now).bind(issue_id).execute(&pool).await?; }
        if let Some(ref p) = priority { sqlx::query("UPDATE issues SET priority = ?, updated_at = ? WHERE id = ?").bind(p).bind(&now).bind(issue_id).execute(&pool).await?; }
        if let Some(ref c) = complexity { sqlx::query("UPDATE task_contracts SET estimated_complexity = ? WHERE issue_id = ?").bind(c).bind(issue_id).execute(&pool).await?; }
        if let Some(ref s) = skills { let j = serde_json::to_string(s).unwrap(); sqlx::query("UPDATE task_contracts SET required_skills = ? WHERE issue_id = ?").bind(&j).bind(issue_id).execute(&pool).await?; }
        if cli.json { println!("{}", serde_json::json!({"success": true})); } else { println!("Task {} updated", identifier); }
    }
},
Commands::Metrics { project, agent: agent_id } => {
    if let Some(aid) = agent_id {
        let stats = sqlx::query_as::<_, AgentStats>("SELECT * FROM agent_stats WHERE agent_id = ?").bind(&aid).fetch_one(&pool).await?;
        if cli.json { println!("{}", serde_json::json!({"success": true, "data": stats})); }
        else {
            let avg = if stats.tasks_completed > 0 { stats.total_confidence / stats.tasks_completed as f64 } else { 0.0 };
            println!("Agent {} | Completed: {} | Failed: {} | Avg Confidence: {:.2}", aid, stats.tasks_completed, stats.tasks_failed, avg);
        }
    } else if let Some(pid) = project {
        let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = ?").bind(pid).fetch_one(&pool).await?;
        let completed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = ? AND tc.task_state = 'completed'").bind(pid).fetch_one(&pool).await?;
        let queued: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = ? AND tc.task_state = 'queued'").bind(pid).fetch_one(&pool).await?;
        let in_progress: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = ? AND tc.task_state IN ('claimed', 'executing')").bind(pid).fetch_one(&pool).await?;
        let agents_online: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agents WHERE status != 'offline'").fetch_one(&pool).await?;
        if cli.json {
            println!("{}", serde_json::json!({"success": true, "data": {"total": total, "completed": completed, "queued": queued, "in_progress": in_progress, "agents_online": agents_online}}));
        } else {
            println!("Project {} | Total: {} | Completed: {} | Queued: {} | In Progress: {} | Agents: {}", pid, total, completed, queued, in_progress, agents_online);
        }
    } else {
        eprintln!("Specify --project or --agent");
        std::process::exit(1);
    }
},
```

- [ ] **Step 6: Add required imports to cli.rs**

Add at the top of cli.rs:
```rust
use kanban_lib::models::{AgentStats, TaskContract, ExecutionLog, ProjectAgentConfig};
```

- [ ] **Step 7: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles (may need minor fixes — iterate until clean)

- [ ] **Step 8: Test basic CLI flow**

```bash
cd src-tauri && cargo build --release --bin kanban-cli
./target/release/kanban-cli agent register --name "test-agent" --skills rust,typescript --json
./target/release/kanban-cli agent list --json
```

Expected: agent registered and listed successfully

- [ ] **Step 9: Commit**

```bash
git add src-tauri/src/bin/cli.rs
git commit -m "feat: add agent and task CLI commands"
```

---

## Chunk 7: MCP Extensions

### Task 10: MCP Agent & Task Tools

**Files:**
- Modify: `src-tauri/src/bin/mcp.rs`

- [ ] **Step 1: Add new tool definitions to `tools_list()`**

Add these tool definitions to the `tools_list()` function in mcp.rs:

```rust
// Agent tools
tool_def("register_agent", "Register a new AI agent",
    json!({"name": prop("string", "Agent name"), "skills": {"type": "array", "items": {"type": "string"}, "description": "Agent skills"}, "task_types": {"type": "array", "items": {"type": "string"}, "description": "Task types agent can handle"}, "max_concurrent": prop("number", "Max concurrent tasks"), "max_complexity": prop("string", "Max complexity: small, medium, large")}),
    vec!["name", "skills"]),
tool_def("agent_heartbeat", "Send agent heartbeat", json!({"agent_id": prop("string", "Agent ID")}), vec!["agent_id"]),
tool_def("deregister_agent", "Deregister an agent", json!({"agent_id": prop("string", "Agent ID")}), vec!["agent_id"]),
// Task tools
tool_def("next_task", "Get next available task and claim it atomically",
    json!({"agent_id": prop("string", "Agent ID"), "skills_override": {"type": "array", "items": {"type": "string"}, "description": "Override agent skills for this query"}}),
    vec!["agent_id"]),
tool_def("start_task", "Mark claimed task as executing", json!({"agent_id": prop("string", "Agent ID"), "identifier": prop("string", "Task identifier")}), vec!["agent_id", "identifier"]),
tool_def("complete_task", "Complete a task with results",
    json!({"agent_id": prop("string", "Agent ID"), "identifier": prop("string", "Task identifier"), "confidence": prop("number", "Confidence score 0.0-1.0"), "summary": prop("string", "Completion summary"), "artifacts": {"type": "object", "description": "Artifacts produced"}}),
    vec!["agent_id", "identifier", "confidence", "summary"]),
tool_def("fail_task", "Report task failure",
    json!({"agent_id": prop("string", "Agent ID"), "identifier": prop("string", "Task identifier"), "reason": prop("string", "Failure reason")}),
    vec!["agent_id", "identifier", "reason"]),
tool_def("unclaim_task", "Voluntarily release a task",
    json!({"agent_id": prop("string", "Agent ID"), "identifier": prop("string", "Task identifier")}),
    vec!["agent_id", "identifier"]),
tool_def("log_task_activity", "Log execution activity for a task",
    json!({"identifier": prop("string", "Task identifier"), "agent_id": prop("string", "Agent ID"), "entry_type": prop("string", "Log type: reasoning, file_read, file_edit, command, discovery, error, checkpoint"), "message": prop("string", "Log message"), "metadata": {"type": "object", "description": "Optional metadata"}}),
    vec!["identifier", "agent_id", "entry_type", "message"]),
tool_def("create_task", "Create a new task contract",
    json!({"project_id": prop("number", "Project ID"), "title": prop("string", "Task title"), "objective": prop("string", "Task objective"), "status_id": prop("number", "Status ID"), "type": prop("string", "Task type"), "priority": prop("string", "Priority"), "skills": {"type": "array", "items": {"type": "string"}}, "complexity": prop("string", "Complexity"), "description": prop("string", "Description"), "depends_on": {"type": "array", "items": {"type": "string"}, "description": "Identifiers this task depends on"}, "context_files": {"type": "array", "items": {"type": "string"}}, "parent_identifier": prop("string", "Parent task identifier"), "timeout_minutes": prop("number", "Timeout in minutes")}),
    vec!["project_id", "title", "objective", "status_id"]),
tool_def("get_task", "Get full task contract by identifier", json!({"identifier": prop("string", "Task identifier")}), vec!["identifier"]),
tool_def("task_replay", "Get execution replay for a task", json!({"identifier": prop("string", "Task identifier")}), vec!["identifier"]),
tool_def("agent_stats", "Get agent performance stats", json!({"agent_id": prop("string", "Agent ID")}), vec!["agent_id"]),
tool_def("approve_task", "Human approves a validating task", json!({"identifier": prop("string", "Task identifier")}), vec!["identifier"]),
tool_def("reject_task", "Human rejects a validating task", json!({"identifier": prop("string", "Task identifier")}), vec!["identifier"]),
```

- [ ] **Step 2: Add tool call handlers**

Add match arms in the `handle_tool_call` function for each new tool. Each handler follows the same pattern as existing tools: extract params, run SQL, return JSON result. The implementation mirrors the CLI handlers but returns JSON-RPC responses.

Key handler pattern (example for `register_agent`):
```rust
"register_agent" => {
    let name = params["name"].as_str().unwrap_or("");
    let skills: Vec<String> = params["skills"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default();
    let task_types: Vec<String> = params["task_types"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_else(|| vec!["implementation".into()]);
    let max_concurrent = params["max_concurrent"].as_i64().unwrap_or(1);
    let max_complexity = params["max_complexity"].as_str().unwrap_or("large");

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
    sqlx::query("INSERT INTO agents (id, name, skills, task_types, max_concurrent, max_complexity, status, registered_at, last_heartbeat) VALUES (?, ?, ?, ?, ?, ?, 'idle', ?, ?)")
        .bind(&id).bind(name).bind(serde_json::to_string(&skills).unwrap()).bind(serde_json::to_string(&task_types).unwrap())
        .bind(max_concurrent).bind(max_complexity).bind(&now).bind(&now)
        .execute(&pool).await?;
    sqlx::query("INSERT INTO agent_stats (agent_id) VALUES (?)").bind(&id).execute(&pool).await?;
    Ok(json!({"agent_id": id, "name": name}))
}
"next_task" => {
    let agent_id = params["agent_id"].as_str().unwrap_or("");
    let agent = sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = ?").bind(agent_id).fetch_one(&pool).await?;
    let skills: Vec<String> = params["skills_override"].as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_else(|| serde_json::from_str(&agent.skills).unwrap_or_default());
    let result = kanban_lib::orchestration::routing::next_task(&pool, agent_id, &skills, &agent.max_complexity, agent.max_concurrent).await?;
    Ok(json!(result))
}
```

Implement ALL remaining tools (`agent_heartbeat`, `deregister_agent`, `start_task`, `complete_task`, `fail_task`, `unclaim_task`, `log_task_activity`, `create_task`, `get_task`, `task_replay`, `agent_stats`, `approve_task`, `reject_task`) following this same pattern. Each maps directly to the CLI handler logic — extract params from JSON, execute SQL, return JSON result. Ensure every MCP tool has a complete handler, not just a tool definition.

- [ ] **Step 3: Add required imports**

```rust
use kanban_lib::models::{AgentStats, TaskContract, ExecutionLog, ProjectAgentConfig};
use kanban_lib::orchestration;
```

- [ ] **Step 4: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles successfully

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/bin/mcp.rs
git commit -m "feat: add agent and task MCP tools"
```

---

## Chunk 8: Integration Test & Build Verification

### Task 11: End-to-End Verification

- [ ] **Step 1: Build all binaries**

Run: `cd src-tauri && cargo build --release`
Expected: all 3 binaries compile (kanban, kanban-cli, kanban-mcp)

- [ ] **Step 2: Run full agent work loop test**

```bash
CLI=./src-tauri/target/release/kanban-cli

# Register agent
$CLI agent register --name "test-agent-1" --skills rust,typescript --max-concurrent 2 --json

# Capture agent ID from output
AGENT_ID=$(echo '...' | jq -r '.data.agent_id')

# Create a task contract
$CLI task create --project 2 --title "Test task" --objective "Verify the work loop" --status 9 --skills rust --complexity small --json

# Get next task (should claim it)
$CLI task next --agent $AGENT_ID --json

# Start it
$CLI task start KAN-XX --agent $AGENT_ID --json

# Log some activity
$CLI task log KAN-XX --agent $AGENT_ID --type reasoning --message "Testing the log system"

# Complete it
$CLI task complete KAN-XX --agent $AGENT_ID --confidence 0.95 --summary "Test passed" --json

# Verify replay
$CLI task replay KAN-XX --json

# Check metrics
$CLI metrics --agent $AGENT_ID --json

# Deregister
$CLI agent deregister --id $AGENT_ID --json
```

Expected: full loop completes without errors, replay shows all logged events

- [ ] **Step 3: Commit final verification**

```bash
git commit --allow-empty -m "feat: phase 1 agent orchestration complete - solo agent autonomy"
```

---

## Summary

| Chunk | Tasks | Description |
|-------|-------|-------------|
| 1 | 1-3 | Migration, uuid dep, Rust models |
| 2 | 4-5 | State machine, routing engine |
| 3 | 6 | Agent Tauri commands |
| 4 | 7 | Task contract Tauri commands |
| 5 | 8 | Execution log Tauri commands |
| 6 | 9 | CLI extensions (agent + task + approve/reject/invalidate/search/update) |
| 7 | 10 | MCP extensions |
| 8 | 11 | Integration test & build verification |
