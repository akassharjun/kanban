# Next-Gen AI Agent Kanban: Design Specification

## Vision

An autonomous task orchestration engine that serves as the shared brain between AI agents. The system decomposes work, routes tasks to capable agents, enforces coordination invariants, and provides full observability. Agents are workers that pull structured task contracts, execute them, and report results. Humans observe via a real-time dashboard and intervene only when the system escalates.

## Core Principles

1. **Tasks are Contracts, Not Tickets.** Every task is a machine-readable contract with a defined objective, inputs, constraints, and success criteria. An agent never needs to "interpret" a task.
2. **The Board Thinks, Agents Execute.** The system owns decomposition, routing, dependency resolution, and prioritization. When an agent asks "what should I do?", the answer is pre-computed.
3. **Everything is Observable.** Every agent action is logged as a structured execution trace. Every task carries its full history. When something fails, you can replay the exact sequence of events.

## Deployment Model

Local-first (SQLite, Tauri desktop app), designed for future network accessibility. Multiple agents on the same machine coordinate through the shared database. The desktop app is the human observation dashboard.

## Target Scale

10+ concurrent agents, scaling to many. The system handles contention, load balancing, and resource management as first-class concerns.

## Primary Users

- **Agents (primary):** Claude Code and Codex agents interacting via CLI and MCP. They cannot communicate with each other directly. The board is the only shared state.
- **Humans (observers):** Monitor agent activity, intervene on escalations, create high-level objectives.

## Agent Interaction Loop

```
Agent connects -> registers capabilities -> polls for work ->
receives pre-filtered task contract with full context ->
claims task (atomic) -> executes -> logs structured trace ->
reports result + confidence score -> system validates ->
system unlocks downstream tasks -> agent polls again
```

## System Invariants

- **No double-claims:** Once an agent claims a task, it is locked.
- **Valid state transitions only:** Tasks follow a state machine, no skipping steps.
- **Dependency resolution:** A task cannot be claimed until all blockers are resolved.
- **Capability matching:** Agents only see tasks they are qualified for.
- **Timeout recovery:** If a claimed task goes stale, the system reclaims and re-queues it.

---

## 1. Task Contract Data Model

Every task is a structured contract:

```yaml
task:
  id: KAN-42
  type: implementation | research | testing | review | decomposition

  # What to do
  objective: "Implement SEACR validation endpoint"
  description: "Full markdown description with details"

  # What you need
  context:
    files:
      - src/validation/mod.rs
      - src/models/seacr.rs
    related_tasks:
      - KAN-40
      - KAN-41
    docs:
      - "SEACR spec v2.1 - section 4.3"
    prior_attempts:
      - agent: codex-1
        attempt_number: 1
        result: failed
        reason: "Missing schema migration, FK constraint error"

  # Boundaries
  constraints:
    - "Must pass existing test suite"
    - "Latency < 200ms on p95"
    - "No new dependencies"

  # How to prove you're done
  success_criteria:
    - tests_pass: true
    - endpoint_responding: true
    - lint_clean: true

  # Routing
  requirements:
    skills: [rust, api, sql]
    estimated_complexity: medium

  # Execution state
  # task_state is the agent-facing state, kept in task_contracts table.
  # It maps to existing issues.status_id as follows:
  #   queued/blocked    -> status category "unstarted"
  #   claimed/executing -> status category "started"
  #   validating        -> status category "started" (custom status "Validating" auto-created)
  #   completed         -> status category "completed"
  #   cancelled         -> status category "discarded"
  # The system keeps both in sync: changing task_state updates status_id and vice versa.
  task_state: queued | claimed | executing | validating | completed | blocked | cancelled
  claimed_by: null
  claimed_at: null
  timeout_minutes: 30

  # Results (filled by agent)
  result:
    status: null
    confidence: null
    output_summary: null
    artifacts:
      - type: branch
        ref: feature/kan-42-seacr-validation
      - type: pr
        ref: "#182"

  # Relationships
  # depends_on/blocks use the existing issue_relations table with
  # relation_type = 'blocks' / 'blocked_by'. The agent layer reads
  # these relations to determine dependency resolution.
  parent_id: KAN-40
  depends_on: [KAN-41]
  blocks: [KAN-43, KAN-44]

  # Metadata
  priority: urgent | high | medium | low
  created_by: human | agent:claude-code-1
  created_at: 2026-03-15T10:00:00Z
  updated_at: 2026-03-15T10:00:00Z
```

### Design Decisions

- **Context packs are first-class.** The `context` block is the primary way the system transfers knowledge between agents. When Agent A fails and Agent B picks up the task, B sees A's execution log and failure reason.
- **Success criteria are machine-evaluable where possible.** `tests_pass: true` means the system can run `cargo test` and verify.
- **Prior attempts accumulate.** Every failed attempt stays on the contract. The third agent to attempt a task has the full failure history of the first two.
- **Constraints are natural language but structured.** Agents read them as instructions. The system validates via success criteria.
- **Timeout recovery is built in.** If `claimed_at + timeout_minutes` passes without completion, the system reclaims the task. Effective timeout is `min(task.timeout_minutes, agent.limits.max_task_timeout)`.
- **Dependencies use existing issue_relations.** The `depends_on` / `blocks` fields map to the existing `issue_relations` table with `relation_type = 'blocks'` / `'blocked_by'`. No new dependency table is needed.

### Task State Machine

```
                    +----------+
                    |  queued  |<------------------+
                    +----+-----+                   |
                         | agent claims            | timeout / reclaim / fail
                    +----v-----+                   |
                    | claimed  |------------------>+
                    +----+-----+                   |
                         | agent starts            |
                    +----v-----+                   |
                    |executing |------------------>+
                    +----+-----+                   |
                         | agent reports done       |
                    +----v------+                  |
                    |validating |---- fail ------->+
                    +----+------+
                         | criteria met OR
                         | human approves
                    +----v-----+
                    |completed |
                    +----------+
```

Additional transitions: any state -> `cancelled` (human), any state -> `blocked` (dependency detected), `blocked` -> `queued` (dependency resolved).

Note: there is no terminal `failed` state. Failed tasks are always re-queued (back to `queued`) with failure context appended. After `max_attempts` failures, the task is moved to `blocked` with a `needs-human` label and an escalation notification. Humans can unblock it manually or cancel it.

### State-to-Status Mapping

The `task_state` field in `task_contracts` is the agent-facing state machine. It maps to the existing `issues.status_id` via status categories:

| Task State | Status Category | Notes |
|------------|----------------|-------|
| `queued` | unstarted | Maps to project's "Todo" status |
| `claimed` | started | Maps to "In Progress" |
| `executing` | started | Maps to "In Progress" |
| `validating` | started | Auto-created "Validating" status |
| `completed` | completed | Maps to "Done" |
| `blocked` | blocked | Maps to "Blocked" |
| `cancelled` | discarded | Maps to "Discarded" |

The system keeps both in sync bidirectionally. Changing `task_state` updates `status_id`. Changing `status_id` from the human board updates `task_state`. The task state machine is authoritative for agent operations; `status_id` is authoritative for human board display.

### Human Intervention Commands

For tasks in `validating` state (low confidence), humans can:

```bash
kanban task approve <IDENTIFIER>     # validating -> completed
kanban task reject <IDENTIFIER>      # validating -> queued (re-queue for another attempt)
```

---

## 2. Agent Registry and Routing

### Agent Registration

```yaml
agent:
  id: claude-code-1
  name: "Claude Code"
  type: claude-code | codex | custom

  capabilities:
    skills: [rust, typescript, react, sql, testing, architecture]
    task_types: [implementation, research, review, decomposition]
    max_complexity: large

  limits:
    max_concurrent_tasks: 3
    max_task_timeout: 60

  status: idle | busy | offline
  current_tasks: []
  registered_at: 2026-03-15T10:00:00Z
  last_heartbeat: 2026-03-15T10:05:00Z

  stats:
    tasks_completed: 0
    tasks_failed: 0
    avg_confidence: null
    avg_completion_time: null
```

### Heartbeat and Liveness

Agents send a heartbeat every N seconds (configurable, default 60). If an agent misses 3 consecutive heartbeats:

1. Status moves to `offline`.
2. All `claimed` and `executing` tasks are reclaimed and moved back to `queued`.
3. Each reclaimed task gets a log entry: "Agent went offline, task reclaimed."

### Routing Algorithm

When an agent calls `next_task`, the system runs this filter chain:

```
All queued tasks
  |
  +- Filter: task.depends_on all resolved?
  |    (remove blocked tasks)
  |
  +- Filter: task.requirements.skills is subset of agent.capabilities.skills?
  |    (remove tasks agent cannot do)
  |
  +- Filter: task.requirements.estimated_complexity <= agent.capabilities.max_complexity?
  |    (remove tasks too complex for agent)
  |
  +- Filter: agent.current_tasks.count < agent.limits.max_concurrent_tasks?
  |    (respect agent capacity)
  |
  +- Sort: priority (urgent > high > medium > low)
  |
  +- Sort: dependency depth (tasks that unblock the most downstream work first)
  |
  +- Return top task with full contract + context pack
```

### Atomic Claiming (Concurrency Control)

SQLite serializes writes, so concurrent `next_task` calls from multiple agents are safe when implemented correctly. The claiming operation uses a single atomic SQL statement:

```sql
UPDATE task_contracts
SET claimed_by = ?agent_id, claimed_at = datetime('now'), task_state = 'claimed'
WHERE issue_id = ?candidate_id AND claimed_by IS NULL
RETURNING *;
```

If the `RETURNING` clause returns no rows, the task was already claimed by another agent between the routing query and the claim attempt. The system retries with the next candidate from the routing pipeline.

The full `next_task` flow:
1. Run the routing filter/sort query to produce a ranked candidate list.
2. Attempt atomic claiming on the top candidate.
3. If claim succeeds, return the full task contract.
4. If claim fails (already taken), try the next candidate.
5. If all candidates exhausted, return null (no work available).

All write operations use `BEGIN IMMEDIATE` transactions to prevent write starvation under high concurrency.

### Timeout Calculation

The effective timeout for a claimed task is: `min(task.timeout_minutes, agent.limits.max_task_timeout)`. The system uses this value when checking for stale claims.

### Design Decisions

- **Agents self-declare capabilities.** The system trusts the registration. Stats can inform routing later.
- **`next_task` is the primary interface.** It produces a candidate list from routing, then attempts atomic claiming. No race window.
- **Capacity is agent-declared.** Claude Code might handle 3 concurrent tasks (via subagents), Codex might handle 1.
- **Dependency depth sort** uses the count of all transitive downstream tasks that are currently blocked. Tasks that unblock the most work get priority.

---

## 3. Task Decomposition Engine

### How Decomposition Works

Decomposition is itself a task type. When a large task enters the system:

1. System detects it needs decomposition (rules-based).
2. System creates a decomposition task.
3. An agent claims the decomposition task.
4. Agent analyzes the codebase and creates sub-tasks via CLI/MCP.
5. Agent completes the decomposition task.
6. System validates: all sub-tasks have contracts, dependency graph is a DAG, at least one sub-task is unblocked.
7. Sub-tasks enter the queue, routing takes over.

### Auto-Decomposition Rules

```yaml
decomposition_rules:
  - condition: complexity == "large" AND has_no_children
    action: create_decomposition_task

  - condition: success_criteria is empty
    action: create_decomposition_task

  - condition: created_by == "human" AND type != "decomposition" AND (complexity == "large" OR success_criteria is empty)
    action: create_decomposition_task

  - condition: agent_requests_decomposition
    action: unclaim_current, create_decomposition_task
```

### Recursive Decomposition

An agent working on a decomposition task might decide a sub-task is too large. It creates that sub-task with `complexity: large`, and the system auto-generates another decomposition task. This recurses until all leaf tasks are atomic.

### Decomposition Validation

When an agent completes a decomposition task, the system validates:

1. All sub-tasks have complete contracts (objective, success criteria, skill requirements).
2. Dependency graph is a DAG (no circular dependencies).
3. At least one sub-task is immediately unblocked (work can begin).

### Discovered Work Pattern

During execution, agents can create new tasks:

```bash
kanban task create --parent KAN-50 --title "Add missing unit tests" \
  --depends-on KAN-52 --skills rust,testing --complexity small
```

The new task enters the queue with full lineage. The system updates the dependency graph.

---

## 4. Execution Logs and Observability

### Execution Log Structure

Every task accumulates a structured timeline:

```yaml
execution_log:
  task_id: KAN-52
  agent_id: claude-code-1

  entries:
    - timestamp: 2026-03-15T10:00:01Z
      type: claim
      message: "Task claimed"

    - timestamp: 2026-03-15T10:00:03Z
      type: reasoning
      message: "Need to understand current embedding implementation"

    - timestamp: 2026-03-15T10:00:15Z
      type: file_read
      message: "Read src/embeddings/pipeline.rs"
      metadata:
        file: src/embeddings/pipeline.rs
        lines: 1-240

    - timestamp: 2026-03-15T10:01:30Z
      type: file_edit
      message: "Added vector index initialization"
      metadata:
        file: src/embeddings/pipeline.rs
        diff: "+15 -3 lines"

    - timestamp: 2026-03-15T10:02:00Z
      type: command
      message: "cargo test"
      metadata:
        exit_code: 1
        output_summary: "2 tests failed"

    - timestamp: 2026-03-15T10:04:30Z
      type: result
      message: "Task failed - blocked on missing schema"
      metadata:
        status: failed
        confidence: 0.0
        reason: "Dependency KAN-51 did not create required table"
```

### Log Entry Types

| Type | Purpose |
|------|---------|
| `claim` | Agent took the task |
| `reasoning` | Agent thinking or decision |
| `file_read` | Agent read a file |
| `file_edit` | Agent modified a file |
| `command` | Agent ran a shell command |
| `discovery` | Agent found unexpected work |
| `error` | Something went wrong |
| `result` | Final outcome |
| `checkpoint` | Progress marker |

### Agent Replay

The killer feature. Given any task, a human sees the exact execution sequence:

```bash
kanban task replay KAN-52
```

```
KAN-52: Create vector search index
Agent: claude-code-1 | Duration: 4m 29s | Result: FAILED (confidence: 0.0)

[10:00:01] CLAIM    Task claimed
[10:00:03] THINK    "Need to understand current embedding implementation"
[10:00:15] READ     src/embeddings/pipeline.rs (240 lines)
[10:01:30] EDIT     src/embeddings/pipeline.rs (+15 -3)
[10:02:00] RUN      cargo test -> FAILED (2 failures)
[10:03:45] THINK    "Tests fail because schema migration hasn't run"
[10:04:00] DISCOVER Created KAN-56: "Add vector_embeddings migration"
[10:04:30] RESULT   Failed - blocked on missing schema
```

### Observability Dashboard (Tauri Desktop)

**Live Feed:** Real-time stream of all agent activity across all tasks.

**Agent Status Panel:** Each registered agent with current status, active tasks, and lifetime stats (completed, failed, avg confidence, avg time).

**Task Pipeline View:** Dependency graph visualized with color-coded statuses (green = completed, red = failed, yellow = executing, gray = blocked).

**Per-Task Detail:** Click any task to see full execution log, all prior attempts, the contract, and the context pack.

### System Metrics

```yaml
system_metrics:
  agent:
    tasks_completed: 47
    tasks_failed: 3
    success_rate: 0.94
    avg_confidence: 0.87
    avg_completion_time_minutes: 12.3
    skills_success_rate:
      rust: 0.96
      typescript: 0.91

  task_type:
    implementation:
      avg_time: 15.2m
      avg_attempts: 1.1
    decomposition:
      avg_time: 5.4m
      avg_subtasks_created: 4.2

  throughput:
    tasks_completed_24h: 23
    tasks_in_progress: 4
    tasks_queued: 12
    tasks_blocked: 3
    agents_online: 2
```

### Prior Attempts Flow

When a task fails and gets re-queued, the system automatically appends the failure to the task contract's `context.prior_attempts`. The next agent sees exactly what went wrong.

---

## 5. Failure Recovery and Confidence System

### Confidence Scores

Every task completion includes a confidence score:

```yaml
confidence_thresholds:
  auto_accept: 0.85       # >= 0.85: completed, downstream unblocked
  human_review: 0.50      # 0.50 - 0.84: moves to validating, human notified
  auto_reject: 0.50       # < 0.50: treated as failure, re-queued
```

Thresholds are project-configurable.

### Five Failure Modes

**1. Explicit failure.** Agent reports failure with reason. Task re-queued with failure context.

**2. Timeout.** Agent went silent. System detects via `claimed_at + timeout_minutes`. Task reclaimed.

**3. Low confidence completion.** Agent says "done" but is unsure. System creates a review task for a different agent or human.

**4. Repeated failures.** Same task failed multiple times:
- 2nd failure: increase priority, add context about prior failures.
- 3rd failure: escalate to human, mark as blocked, add `needs-human` label.

**5. Cascading failure.** Completed task later discovered to be wrong. System invalidates it with defined scope:
- All downstream tasks in `queued`, `claimed`, or `blocked` state are moved to `blocked`.
- Downstream tasks currently `executing` are left to finish but get a warning log entry; on completion they receive an automatic review task.
- Downstream tasks already `completed` get a review task created to verify their work is still valid.
- Artifact rollback (branches, PRs) is out of scope for the system; agents or humans handle this manually.
- The full cascade is logged with lineage for debugging.

### Validation Pipeline

For tasks with automatable success criteria:

```yaml
success_criteria:
  - check: tests_pass
    command: "cargo test"
    expect: exit_code == 0

  - check: lint_clean
    command: "cargo clippy -- -D warnings"
    expect: exit_code == 0
```

System runs these checks on completion. All pass = completed. Any fail = failed, re-queued with validation output.

### Notifications

| Event | Severity | Trigger |
|-------|----------|---------|
| `escalation` | high | Task hits max attempts |
| `low_confidence` | medium | Confidence in review range |
| `agent_offline` | medium | Agent goes offline with active tasks |
| `cascade_failure` | high | Completed task invalidated |
| `task_completed` | low | Normal completion (optional) |

---

## 6. CLI and MCP Interface

### CLI Commands

```bash
# Agent Lifecycle
kanban agent register --name <NAME> --skills <COMMA_LIST> \
  --task-types <COMMA_LIST> --max-concurrent <N> --max-complexity <small|medium|large>
kanban agent heartbeat
kanban agent deregister
kanban agent list
kanban agent stats <AGENT_ID>

# Task Work Loop
kanban task next
kanban task next --skills rust,sql
kanban task start <IDENTIFIER>
kanban task complete <IDENTIFIER> --confidence <0.0-1.0> --summary "..." \
  --artifacts '<JSON>'
kanban task fail <IDENTIFIER> --reason "..."
kanban task unclaim <IDENTIFIER>
kanban task invalidate <IDENTIFIER> --reason "..."
kanban task approve <IDENTIFIER>           # human approves validating task -> completed
kanban task reject <IDENTIFIER>            # human rejects validating task -> re-queued

# Execution Logging
kanban task log <IDENTIFIER> --type <TYPE> --message "..." [--meta '<JSON>']

# Task Management
kanban task create --project <ID> --title "..." --type <TYPE> \
  --objective "..." --skills <COMMA_LIST> --complexity <small|medium|large> \
  --priority <PRIORITY> \
  [--parent <IDENTIFIER>] [--depends-on <COMMA_LIST>] \
  [--context-files <COMMA_LIST>] [--constraints '<JSON>'] \
  [--success-criteria '<JSON>']
kanban task update <IDENTIFIER> [--title] [--priority] [--complexity] [--skills]
kanban task get <IDENTIFIER>
kanban task list --project <ID> [--status] [--type] [--agent] [--available]
kanban task search --project <ID> "query"
kanban task children <IDENTIFIER>
kanban task graph <IDENTIFIER>

# Decomposition
kanban task decompose <IDENTIFIER>

# Replay and Observability
kanban task replay <IDENTIFIER>
kanban task attempts <IDENTIFIER>
kanban metrics --project <ID>
kanban metrics --agent <AGENT_ID>

# Existing commands (unchanged)
kanban project list | create | update | delete
kanban member list | add | delete
kanban label list | create | delete
kanban comment list | add | delete
kanban notifications list | clear
kanban export | import

# Global flag
--json    # all output as JSON
```

### MCP Tools

```yaml
# Agent Lifecycle
- register_agent: { name, skills[], task_types[], max_concurrent, max_complexity } -> { agent_id }
- agent_heartbeat: { agent_id } -> { status, current_tasks[] }
- deregister_agent: { agent_id } -> { success }

# Task Work Loop
- next_task: { agent_id, skills_override[]? } -> { task_contract | null }
- start_task: { agent_id, identifier } -> { success }
- complete_task: { agent_id, identifier, confidence, summary, artifacts? } -> { accepted }
- fail_task: { agent_id, identifier, reason } -> { requeued, attempt_number }
- unclaim_task: { agent_id, identifier } -> { success }
- invalidate_task: { identifier, reason } -> { tasks_blocked[] }
- approve_task: { identifier } -> { success }
- reject_task: { identifier } -> { requeued }

# Execution Logging
- log_task_activity: { identifier, type, message, metadata? } -> { log_entry_id }

# Task Management
- create_task: { project_id, title, type, objective, skills[], complexity, priority, ... } -> { identifier }
- get_task: { identifier } -> { task_contract }
- list_tasks: { project_id, status?, type?, agent_id?, available_only? } -> { tasks[] }
- search_tasks: { project_id, query } -> { tasks[] }
- task_graph: { identifier } -> { nodes[], edges[] }

# Replay and Observability
- task_replay: { identifier } -> { timeline[] }
- task_attempts: { identifier } -> { attempts[] }
- agent_stats: { agent_id } -> { stats }
- system_metrics: { project_id } -> { metrics }
```

### JSON Output Contract

All CLI commands with `--json` return:

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

Error codes: `TASK_NOT_FOUND`, `TASK_ALREADY_CLAIMED`, `TASK_WRONG_STATE`, `AGENT_NOT_REGISTERED`, `AGENT_CAPACITY_FULL`, `DEPENDENCY_UNRESOLVED`, `VALIDATION_FAILED`.

### Agent Bootstrap Protocol

Encoded in CLAUDE.md or system prompt:

```markdown
On session start:
1. Register: kanban agent register --name "claude-code-$(hostname)" --skills rust,typescript --max-concurrent 3
2. Pull work: kanban task next --json
3. If task received:
   a. Read the full contract
   b. kanban task start <ID>
   c. Log reasoning as you work
   d. Execute, logging key actions
   e. Verify success criteria
   f. kanban task complete <ID> --confidence <SCORE> --summary "..."
   g. Go to step 2
4. If no task: report idle, wait or exit

On failure: kanban task fail <ID> --reason "..." then go to step 2
On discovering new work: kanban task create --parent <CURRENT> --title "..."
```

---

## 7. Database Schema (New Tables)

Added to the existing schema. All existing tables remain unchanged.

```sql
-- Agent registry
-- Agent ID is a UUID v4 generated by the system at registration time.
-- Agent name must be unique (used for human display and bootstrap protocol).
CREATE TABLE agents (
    id TEXT PRIMARY KEY,  -- UUID v4, system-generated
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

-- Agent stats (system-maintained)
CREATE TABLE agent_stats (
    agent_id TEXT PRIMARY KEY REFERENCES agents(id) ON DELETE CASCADE,
    tasks_completed INTEGER NOT NULL DEFAULT 0,
    tasks_failed INTEGER NOT NULL DEFAULT 0,
    total_confidence REAL NOT NULL DEFAULT 0.0,
    total_completion_time_seconds INTEGER NOT NULL DEFAULT 0,
    skills_breakdown JSON NOT NULL DEFAULT '{}'  -- Phase 3+: per-skill success rates
);

-- Task contracts (extends existing issues table via 1:1 relationship)
-- Dependencies use the existing issue_relations table (relation_type = 'blocks'/'blocked_by').
-- Prior attempts are stored in the context JSON field and auto-appended on failure.
-- Execution logs are looked up via (issue_id, attempt_number) composite key, not a single log ID.
CREATE TABLE task_contracts (
    issue_id INTEGER PRIMARY KEY REFERENCES issues(id) ON DELETE CASCADE,
    type TEXT NOT NULL DEFAULT 'implementation',
    task_state TEXT NOT NULL DEFAULT 'queued',  -- queued|claimed|executing|validating|completed|blocked|cancelled
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

CREATE INDEX idx_task_contracts_state ON task_contracts(task_state);
CREATE INDEX idx_task_contracts_claimed_by ON task_contracts(claimed_by);

-- Execution logs
CREATE TABLE execution_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL,
    attempt_number INTEGER NOT NULL DEFAULT 1,
    entry_type TEXT NOT NULL,
    message TEXT NOT NULL,
    metadata JSON,
    timestamp DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX idx_execution_logs_issue ON execution_logs(issue_id);
CREATE INDEX idx_execution_logs_agent ON execution_logs(agent_id);

-- Project agent configuration
CREATE TABLE project_agent_config (
    project_id INTEGER PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
    auto_accept_threshold REAL NOT NULL DEFAULT 0.85,
    human_review_threshold REAL NOT NULL DEFAULT 0.50,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    heartbeat_interval_seconds INTEGER NOT NULL DEFAULT 60,
    missed_heartbeats_before_offline INTEGER NOT NULL DEFAULT 3
);
```

---

## 8. Architecture Diagram

```
+-----------------------------------------------------------+
|                    Human Layer                            |
|  +-------------+  +-------------+  +------------------+  |
|  | Board View  |  | Agent Ops   |  | Dependency       |  |
|  | (existing)  |  | Dashboard   |  | Graph View       |  |
|  +------+------+  +------+------+  +--------+---------+  |
|         +----------------+------------------+             |
|                          | Tauri Commands                 |
+-----------------------------------------------------------+
|                    System Layer                           |
|  +-------------+  +-------------+  +------------------+  |
|  | Routing     |  | State       |  | Decomposition    |  |
|  | Engine      |  | Machine     |  | Trigger          |  |
|  +-------------+  +-------------+  +------------------+  |
|  +-------------+  +-------------+  +------------------+  |
|  | Confidence  |  | Timeout     |  | Validation       |  |
|  | Gating      |  | Recovery    |  | Pipeline         |  |
|  +-------------+  +-------------+  +------------------+  |
|  +-------------+  +-------------+  +------------------+  |
|  | Escalation  |  | Dependency  |  | Metrics          |  |
|  | Rules       |  | Resolution  |  | Collector        |  |
|  +------+------+  +------+------+  +--------+---------+  |
|         +----------------+------------------+             |
|                          | SQLite                         |
+-----------------------------------------------------------+
|                    Agent Layer                            |
|  +------------------+  +------------------+               |
|  | CLI Interface    |  | MCP Interface    |               |
|  +--------+---------+  +--------+---------+               |
|           +--------------------+                          |
+-----------------------------------------------------------+
|                    Agent Consumers                        |
|  +------------+  +------------+  +------------+           |
|  | Claude Code|  | Codex      |  | Custom     |           |
|  | Agents     |  | Agents     |  | Agents     |           |
|  +------------+  +------------+  +------------+           |
+-----------------------------------------------------------+
```

---

## 9. Incremental Delivery Phases

### Phase 1: Solo Agent Autonomy

- Task contracts table (extends issues with contract fields)
- Agent registration (register, heartbeat, deregister)
- `next_task` with basic routing (skills + dependencies)
- Atomic claiming with state machine
- Execution logging (CLI + storage)
- CLI commands for the full work loop
- Goal: one agent can autonomously pull, execute, complete, pull again

### Phase 2: Multi-Agent Coordination

- Concurrent claiming (no races via atomic transactions)
- Timeout recovery (background thread)
- Dependency graph resolution (auto-unlock downstream)
- Decomposition trigger (auto-create decomposition tasks)
- Agent capacity management
- Goal: multiple agents working the same backlog in parallel

### Phase 3: Observability and Failure Recovery

- Confidence gating with auto-review task creation
- Escalation rules (max attempts to human)
- Cascading invalidation
- Agent Replay (CLI + desktop dashboard)
- Agent Operations dashboard (live feed, stats, dependency graph view)
- Metrics collection and display
- Validation pipeline (auto-run success criteria commands)
- Goal: full visibility and autonomous error handling

### Phase 4: Network Ready (future)

- HTTP API layer wrapping the same command handlers
- Authentication (API keys per agent)
- Remote agent support
- Multi-machine coordination

---

## 10. What Changes vs. What Stays

### Unchanged

- Existing issues, projects, members, labels, statuses, comments, custom fields
- Existing board/list/tree views
- Existing CLI and MCP commands (backward compatible)
- SQLite + Tauri + React architecture

### New Additions

- 5 new database tables (agents, agent_stats, task_contracts, execution_logs, project_agent_config)
- ~17 new CLI commands (agent + task work loop + approve/reject)
- ~14 new MCP tools
- 3 new frontend views (Agent Ops dashboard, dependency graph, replay viewer)
- 3 background system threads (heartbeat monitor, timeout recovery, decomposition trigger) — these run in the Tauri app process; the app must be running for agent coordination. As a fallback, the CLI performs lazy timeout checks on each `next_task` call, so agents can still operate without the desktop app running (just without proactive timeout recovery).
- Routing engine, state machine, confidence gating logic in Rust backend
