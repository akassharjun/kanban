-- Kanban Schema for Postgres
-- Consolidated from 4 SQLite migrations into single Postgres schema

-- Projects
CREATE TABLE IF NOT EXISTS projects (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'paused', 'completed', 'archived')),
    prefix TEXT NOT NULL UNIQUE,
    issue_counter BIGINT NOT NULL DEFAULT 0,
    deleted_at TIMESTAMPTZ,
    created_at TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"'),
    updated_at TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"')
);

-- Statuses (per-project workflow)
CREATE TABLE IF NOT EXISTS statuses (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    category TEXT NOT NULL CHECK (category IN ('unstarted', 'started', 'blocked', 'completed', 'discarded')),
    color TEXT,
    icon TEXT,
    position BIGINT NOT NULL DEFAULT 0,
    UNIQUE(project_id, name)
);

-- Members (workspace-level)
CREATE TABLE IF NOT EXISTS members (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    display_name TEXT,
    email TEXT,
    avatar_color TEXT NOT NULL DEFAULT '#6366f1',
    created_at TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"')
);

-- Issues
CREATE TABLE IF NOT EXISTS issues (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    identifier TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT,
    status_id BIGINT NOT NULL REFERENCES statuses(id),
    priority TEXT NOT NULL DEFAULT 'none' CHECK (priority IN ('none', 'urgent', 'high', 'medium', 'low')),
    assignee_id BIGINT REFERENCES members(id) ON DELETE SET NULL,
    parent_id BIGINT REFERENCES issues(id) ON DELETE SET NULL,
    position DOUBLE PRECISION NOT NULL DEFAULT 0,
    estimate DOUBLE PRECISION,
    due_date TEXT,
    created_at TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"'),
    updated_at TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"')
);

CREATE INDEX IF NOT EXISTS idx_issues_project ON issues(project_id);
CREATE INDEX IF NOT EXISTS idx_issues_status ON issues(status_id);
CREATE INDEX IF NOT EXISTS idx_issues_assignee ON issues(assignee_id);
CREATE INDEX IF NOT EXISTS idx_issues_parent ON issues(parent_id);
CREATE INDEX IF NOT EXISTS idx_issues_identifier ON issues(identifier);

-- Issue Templates
CREATE TABLE IF NOT EXISTS issue_templates (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description_template TEXT,
    default_status_id BIGINT REFERENCES statuses(id) ON DELETE SET NULL,
    default_priority TEXT NOT NULL DEFAULT 'none' CHECK (default_priority IN ('none', 'urgent', 'high', 'medium', 'low')),
    default_label_ids TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"'),
    updated_at TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"')
);

-- Labels (project-scoped)
CREATE TABLE IF NOT EXISTS labels (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    UNIQUE(project_id, name)
);

-- Issue-Label junction
CREATE TABLE IF NOT EXISTS issue_labels (
    issue_id BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    label_id BIGINT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    PRIMARY KEY (issue_id, label_id)
);

-- Issue Relations
CREATE TABLE IF NOT EXISTS issue_relations (
    id BIGSERIAL PRIMARY KEY,
    source_issue_id BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    target_issue_id BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL CHECK (relation_type IN ('related', 'blocks', 'blocked_by', 'duplicate')),
    UNIQUE(source_issue_id, target_issue_id, relation_type)
);

-- Activity Log
CREATE TABLE IF NOT EXISTS activity_log (
    id BIGSERIAL PRIMARY KEY,
    issue_id BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    field_changed TEXT NOT NULL,
    old_value TEXT,
    new_value TEXT,
    timestamp TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"')
);

CREATE INDEX IF NOT EXISTS idx_activity_issue ON activity_log(issue_id);

-- Undo Log
CREATE TABLE IF NOT EXISTS undo_log (
    id BIGSERIAL PRIMARY KEY,
    operation_type TEXT NOT NULL CHECK (operation_type IN ('create', 'update', 'delete')),
    entity_type TEXT NOT NULL CHECK (entity_type IN ('issue', 'label', 'project', 'member', 'relation', 'status')),
    entity_id BIGINT NOT NULL,
    snapshot_before TEXT,
    snapshot_after TEXT,
    undone BOOLEAN NOT NULL DEFAULT FALSE,
    timestamp TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"')
);

-- Notifications
CREATE TABLE IF NOT EXISTS notifications (
    id BIGSERIAL PRIMARY KEY,
    type TEXT NOT NULL,
    issue_id BIGINT REFERENCES issues(id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    read BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"')
);

-- Hooks
CREATE TABLE IF NOT EXISTS hooks (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    command TEXT NOT NULL
);

-- Comments
CREATE TABLE IF NOT EXISTS comments (
    id BIGSERIAL PRIMARY KEY,
    issue_id BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    member_id BIGINT REFERENCES members(id) ON DELETE SET NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"'),
    updated_at TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"')
);

CREATE INDEX IF NOT EXISTS idx_comments_issue ON comments(issue_id);

-- Custom Fields
CREATE TABLE IF NOT EXISTS custom_fields (
    id BIGSERIAL PRIMARY KEY,
    project_id BIGINT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    field_type TEXT NOT NULL DEFAULT 'text',
    options TEXT,
    position BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS issue_custom_field_values (
    id BIGSERIAL PRIMARY KEY,
    issue_id BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    field_id BIGINT NOT NULL REFERENCES custom_fields(id) ON DELETE CASCADE,
    value TEXT,
    UNIQUE(issue_id, field_id)
);

-- Agent registry
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    agent_type TEXT,
    skills JSONB NOT NULL DEFAULT '[]',
    task_types JSONB NOT NULL DEFAULT '[]',
    max_concurrent BIGINT NOT NULL DEFAULT 1,
    max_complexity TEXT NOT NULL DEFAULT 'large',
    status TEXT NOT NULL DEFAULT 'idle',
    registered_at TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"'),
    last_heartbeat TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"')
);

-- Agent stats
CREATE TABLE IF NOT EXISTS agent_stats (
    agent_id TEXT PRIMARY KEY REFERENCES agents(id) ON DELETE CASCADE,
    tasks_completed BIGINT NOT NULL DEFAULT 0,
    tasks_failed BIGINT NOT NULL DEFAULT 0,
    total_confidence DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    total_completion_time_seconds BIGINT NOT NULL DEFAULT 0,
    skills_breakdown JSONB NOT NULL DEFAULT '{}'
);

-- Task contracts (extends issues 1:1)
CREATE TABLE IF NOT EXISTS task_contracts (
    issue_id BIGINT PRIMARY KEY REFERENCES issues(id) ON DELETE CASCADE,
    type TEXT NOT NULL DEFAULT 'implementation',
    task_state TEXT NOT NULL DEFAULT 'queued',
    objective TEXT NOT NULL DEFAULT '',
    context JSONB NOT NULL DEFAULT '{}',
    constraints JSONB NOT NULL DEFAULT '[]',
    success_criteria JSONB NOT NULL DEFAULT '[]',
    required_skills JSONB NOT NULL DEFAULT '[]',
    estimated_complexity TEXT DEFAULT 'medium',
    claimed_by TEXT REFERENCES agents(id) ON DELETE SET NULL,
    claimed_at TEXT,
    timeout_minutes BIGINT NOT NULL DEFAULT 30,
    result JSONB,
    attempt_count BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_task_contracts_state ON task_contracts(task_state);
CREATE INDEX IF NOT EXISTS idx_task_contracts_claimed_by ON task_contracts(claimed_by);

-- Execution logs
CREATE TABLE IF NOT EXISTS execution_logs (
    id BIGSERIAL PRIMARY KEY,
    issue_id BIGINT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL,
    attempt_number BIGINT NOT NULL DEFAULT 1,
    entry_type TEXT NOT NULL,
    message TEXT NOT NULL,
    metadata JSONB,
    timestamp TEXT NOT NULL DEFAULT to_char(NOW(), 'YYYY-MM-DD HH24:MI:SS"Z"')
);

CREATE INDEX IF NOT EXISTS idx_execution_logs_issue ON execution_logs(issue_id);
CREATE INDEX IF NOT EXISTS idx_execution_logs_agent ON execution_logs(agent_id);

-- Project agent configuration
CREATE TABLE IF NOT EXISTS project_agent_config (
    project_id BIGINT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
    auto_accept_threshold DOUBLE PRECISION NOT NULL DEFAULT 0.85,
    human_review_threshold DOUBLE PRECISION NOT NULL DEFAULT 0.50,
    max_attempts BIGINT NOT NULL DEFAULT 3,
    heartbeat_interval_seconds BIGINT NOT NULL DEFAULT 60,
    missed_heartbeats_before_offline BIGINT NOT NULL DEFAULT 3
);
