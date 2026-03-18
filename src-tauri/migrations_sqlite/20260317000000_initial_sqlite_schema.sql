-- Kanban Schema for SQLite
-- This migration creates all tables if they don't exist (fresh DB)
-- and adds any missing columns for existing databases.

-- Projects
CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'paused', 'completed', 'archived')),
    prefix TEXT NOT NULL UNIQUE,
    issue_counter INTEGER NOT NULL DEFAULT 0,
    deleted_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    path TEXT
);

-- Statuses (per-project workflow)
CREATE TABLE IF NOT EXISTS statuses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    category TEXT NOT NULL CHECK (category IN ('unstarted', 'started', 'blocked', 'completed', 'discarded')),
    color TEXT,
    icon TEXT,
    position INTEGER NOT NULL DEFAULT 0,
    UNIQUE(project_id, name)
);

-- Members (workspace-level)
CREATE TABLE IF NOT EXISTS members (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    display_name TEXT,
    email TEXT,
    avatar_color TEXT NOT NULL DEFAULT '#6366f1',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Issues
CREATE TABLE IF NOT EXISTS issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    identifier TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT,
    status_id INTEGER NOT NULL REFERENCES statuses(id),
    priority TEXT NOT NULL DEFAULT 'none' CHECK (priority IN ('none', 'urgent', 'high', 'medium', 'low')),
    assignee_id INTEGER REFERENCES members(id) ON DELETE SET NULL,
    parent_id INTEGER REFERENCES issues(id) ON DELETE SET NULL,
    position REAL NOT NULL DEFAULT 0,
    estimate REAL,
    due_date TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_issues_project ON issues(project_id);
CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status_id);
CREATE INDEX IF NOT EXISTS idx_issues_assignee ON issues(assignee_id);
CREATE INDEX IF NOT EXISTS idx_issues_parent ON issues(parent_id);
CREATE INDEX IF NOT EXISTS idx_issues_identifier ON issues(identifier);

-- Issue Templates
CREATE TABLE IF NOT EXISTS issue_templates (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description_template TEXT,
    default_status_id INTEGER REFERENCES statuses(id) ON DELETE SET NULL,
    default_priority TEXT NOT NULL DEFAULT 'none' CHECK (default_priority IN ('none', 'urgent', 'high', 'medium', 'low')),
    default_label_ids TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Labels (project-scoped)
CREATE TABLE IF NOT EXISTS labels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    UNIQUE(project_id, name)
);

-- Issue-Label junction
CREATE TABLE IF NOT EXISTS issue_labels (
    issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    label_id INTEGER NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    PRIMARY KEY (issue_id, label_id)
);

-- Issue Relations
CREATE TABLE IF NOT EXISTS issue_relations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    target_issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL CHECK (relation_type IN ('related', 'blocks', 'blocked_by', 'duplicate')),
    UNIQUE(source_issue_id, target_issue_id, relation_type)
);

-- Activity Log
CREATE TABLE IF NOT EXISTS activity_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    field_changed TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_activity_issue ON activity_log(issue_id);

-- Undo Log
CREATE TABLE IF NOT EXISTS undo_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_type TEXT NOT NULL CHECK (operation_type IN ('create', 'update', 'delete')),
    entity_type TEXT NOT NULL CHECK (entity_type IN ('issue', 'label', 'project', 'member', 'relation', 'status')),
    entity_id INTEGER NOT NULL,
    snapshot_before TEXT,
    snapshot_after TEXT,
    undone INTEGER NOT NULL DEFAULT 0,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Notifications
CREATE TABLE IF NOT EXISTS notifications (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    type TEXT NOT NULL,
    issue_id INTEGER REFERENCES issues(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    read INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Hooks
CREATE TABLE IF NOT EXISTS hooks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    command TEXT NOT NULL
);

-- Comments
CREATE TABLE IF NOT EXISTS comments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    member_id INTEGER REFERENCES members(id) ON DELETE SET NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_comments_issue ON comments(issue_id);

-- Custom Fields
CREATE TABLE IF NOT EXISTS custom_fields (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    field_type TEXT NOT NULL DEFAULT 'text',
    options TEXT,
    position INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS issue_custom_field_values (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    field_id INTEGER NOT NULL REFERENCES custom_fields(id) ON DELETE CASCADE,
    value TEXT,
    UNIQUE(issue_id, field_id)
);

-- Agent registry
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    agent_type TEXT,
    skills TEXT NOT NULL DEFAULT '[]',
    task_types TEXT NOT NULL DEFAULT '[]',
    max_concurrent INTEGER NOT NULL DEFAULT 1,
    max_complexity TEXT NOT NULL DEFAULT 'large',
    member_id INTEGER REFERENCES members(id) ON DELETE SET NULL,
    status TEXT NOT NULL DEFAULT 'idle',
    registered_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_heartbeat TEXT NOT NULL DEFAULT (datetime('now')),
    last_activity_at TEXT,
    worktree_path TEXT
);

-- Agent stats
CREATE TABLE IF NOT EXISTS agent_stats (
    agent_id TEXT PRIMARY KEY REFERENCES agents(id) ON DELETE CASCADE,
    tasks_completed INTEGER NOT NULL DEFAULT 0,
    tasks_failed INTEGER NOT NULL DEFAULT 0,
    total_confidence REAL NOT NULL DEFAULT 0.0,
    total_completion_time_seconds INTEGER NOT NULL DEFAULT 0,
    skills_breakdown TEXT NOT NULL DEFAULT '{}'
);

-- Task contracts (extends issues 1:1)
CREATE TABLE IF NOT EXISTS task_contracts (
    issue_id INTEGER PRIMARY KEY REFERENCES issues(id) ON DELETE CASCADE,
    type TEXT NOT NULL DEFAULT 'implementation',
    task_state TEXT NOT NULL DEFAULT 'queued',
    objective TEXT NOT NULL DEFAULT '',
    context TEXT NOT NULL DEFAULT '{}',
    constraints TEXT NOT NULL DEFAULT '[]',
    success_criteria TEXT NOT NULL DEFAULT '[]',
    required_skills TEXT NOT NULL DEFAULT '[]',
    estimated_complexity TEXT DEFAULT 'medium',
    claimed_by TEXT REFERENCES agents(id) ON DELETE SET NULL,
    claimed_at TEXT,
    timeout_minutes INTEGER NOT NULL DEFAULT 30,
    result TEXT,
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
    metadata TEXT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
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

-- Cost tracking per task
CREATE TABLE IF NOT EXISTS task_costs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_identifier TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    cost_type TEXT NOT NULL CHECK(cost_type IN ('compute_time', 'api_tokens', 'custom')),
    amount REAL NOT NULL,
    unit TEXT NOT NULL,
    description TEXT,
    recorded_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_task_costs_task ON task_costs(task_identifier);
CREATE INDEX IF NOT EXISTS idx_task_costs_agent ON task_costs(agent_id);

-- Cost budgets per project
CREATE TABLE IF NOT EXISTS cost_budgets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    budget_type TEXT NOT NULL CHECK(budget_type IN ('daily', 'weekly', 'monthly', 'per_task', 'total')),
    amount REAL NOT NULL,
    unit TEXT NOT NULL DEFAULT 'dollars',
    spent REAL NOT NULL DEFAULT 0,
    period_start TEXT,
    period_end TEXT,
    alert_threshold REAL DEFAULT 0.8,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- SLA policies
CREATE TABLE IF NOT EXISTS sla_policies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    target_type TEXT NOT NULL CHECK(target_type IN ('response_time', 'resolution_time', 'task_timeout')),
    priority_filter TEXT,
    warning_minutes INTEGER NOT NULL,
    breach_minutes INTEGER NOT NULL,
    escalation_action TEXT NOT NULL DEFAULT '{}',
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- SLA events
CREATE TABLE IF NOT EXISTS sla_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sla_policy_id INTEGER NOT NULL REFERENCES sla_policies(id) ON DELETE CASCADE,
    issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL CHECK(event_type IN ('warning', 'breach', 'escalated', 'resolved')),
    message TEXT NOT NULL,
    metadata TEXT DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_sla_events_issue ON sla_events(issue_id);
