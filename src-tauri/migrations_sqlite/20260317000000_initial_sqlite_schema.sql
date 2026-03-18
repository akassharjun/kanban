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
    actor_id INTEGER REFERENCES members(id),
    actor_type TEXT DEFAULT 'user',
    timestamp TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_activity_issue ON activity_log(issue_id);
CREATE INDEX IF NOT EXISTS idx_activity_log_actor ON activity_log(actor_id);
CREATE INDEX IF NOT EXISTS idx_activity_log_timestamp ON activity_log(timestamp);

-- Mentions
CREATE TABLE IF NOT EXISTS mentions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    comment_id INTEGER REFERENCES comments(id) ON DELETE CASCADE,
    member_id INTEGER NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    source TEXT NOT NULL CHECK(source IN ('description', 'comment')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_mentions_member ON mentions(member_id);
CREATE INDEX IF NOT EXISTS idx_mentions_issue ON mentions(issue_id);

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

-- Issue-File Links (code heat map)
CREATE TABLE IF NOT EXISTS issue_file_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    link_type TEXT NOT NULL DEFAULT 'related' CHECK(link_type IN ('related', 'cause', 'fix')),
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_issue_file_links_path ON issue_file_links(file_path);
CREATE INDEX IF NOT EXISTS idx_issue_file_links_issue ON issue_file_links(issue_id);

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

-- Saved Views (per-project custom filter/sort/view combos)
CREATE TABLE IF NOT EXISTS saved_views (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    filters TEXT NOT NULL DEFAULT '{}',
    sort_by TEXT,
    sort_direction TEXT DEFAULT 'asc',
    view_mode TEXT DEFAULT 'board',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- GitHub integration config (per-project)
CREATE TABLE IF NOT EXISTS github_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL UNIQUE REFERENCES projects(id) ON DELETE CASCADE,
    repo_owner TEXT NOT NULL,
    repo_name TEXT NOT NULL,
    access_token TEXT,
    branch_pattern TEXT NOT NULL DEFAULT '{{prefix}}-{{number}}/{{slug}}',
    auto_link_prs INTEGER NOT NULL DEFAULT 1,
    auto_transition_on_merge INTEGER NOT NULL DEFAULT 1,
    merge_target_status_id INTEGER REFERENCES statuses(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_saved_views_project ON saved_views(project_id);

-- Starred Issues (per-member favorites)
CREATE TABLE IF NOT EXISTS starred_issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    member_id INTEGER NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(issue_id, member_id)
);

CREATE INDEX IF NOT EXISTS idx_starred_issues_member ON starred_issues(member_id);

-- Recently Viewed Issues (per-member history)
CREATE TABLE IF NOT EXISTS recently_viewed (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    member_id INTEGER NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    viewed_at TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(issue_id, member_id)
);

CREATE INDEX IF NOT EXISTS idx_recently_viewed_member ON recently_viewed(member_id);

-- GitHub events log
CREATE TABLE IF NOT EXISTS github_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL CHECK(event_type IN ('pr_opened', 'pr_merged', 'pr_closed', 'pr_review', 'check_run', 'push')),
    issue_id INTEGER REFERENCES issues(id) ON DELETE SET NULL,
    payload TEXT NOT NULL DEFAULT '{}',
    processed INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_github_events_project ON github_events(project_id);
CREATE INDEX IF NOT EXISTS idx_github_events_issue ON github_events(issue_id);

-- Project agent configuration
CREATE TABLE IF NOT EXISTS project_agent_config (
    project_id INTEGER PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
    auto_accept_threshold REAL NOT NULL DEFAULT 0.85,
    human_review_threshold REAL NOT NULL DEFAULT 0.50,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    heartbeat_interval_seconds INTEGER NOT NULL DEFAULT 60,
    missed_heartbeats_before_offline INTEGER NOT NULL DEFAULT 3
);

-- Epics (project-scoped grouping)
CREATE TABLE IF NOT EXISTS epics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    color TEXT NOT NULL DEFAULT '#6366f1',
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'closed')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_epics_project ON epics(project_id);

-- Milestones (project-scoped time targets)
CREATE TABLE IF NOT EXISTS milestones (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    due_date TEXT,
    status TEXT NOT NULL DEFAULT 'open' CHECK(status IN ('open', 'closed')),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_milestones_project ON milestones(project_id);

-- Git Links (branch/PR/commit linking to issues)
CREATE TABLE IF NOT EXISTS git_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id INTEGER NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    link_type TEXT NOT NULL CHECK(link_type IN ('branch', 'pr', 'commit')),
    url TEXT,
    ref_name TEXT NOT NULL,
    pr_number INTEGER,
    pr_state TEXT,
    pr_merged INTEGER NOT NULL DEFAULT 0,
    ci_status TEXT,
    review_status TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_git_links_issue ON git_links(issue_id);

-- Automation rules
CREATE TABLE IF NOT EXISTS automation_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    trigger_type TEXT NOT NULL CHECK(trigger_type IN (
        'status_change', 'issue_created', 'issue_updated',
        'pr_merged', 'pr_opened', 'task_completed', 'task_failed',
        'agent_assigned', 'label_added', 'priority_changed',
        'comment_added', 'schedule'
    )),
    trigger_config TEXT NOT NULL DEFAULT '{}',
    conditions TEXT NOT NULL DEFAULT '[]',
    actions TEXT NOT NULL DEFAULT '[]',
    execution_count INTEGER NOT NULL DEFAULT 0,
    last_executed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_automation_rules_project ON automation_rules(project_id);
CREATE INDEX IF NOT EXISTS idx_automation_rules_trigger ON automation_rules(trigger_type);

-- Automation execution log
CREATE TABLE IF NOT EXISTS automation_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id INTEGER NOT NULL REFERENCES automation_rules(id) ON DELETE CASCADE,
    issue_id INTEGER REFERENCES issues(id) ON DELETE SET NULL,
    trigger_type TEXT NOT NULL,
    actions_executed TEXT NOT NULL DEFAULT '[]',
    success INTEGER NOT NULL DEFAULT 1,
    error_message TEXT,
    executed_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_automation_log_rule ON automation_log(rule_id);
CREATE INDEX IF NOT EXISTS idx_automation_log_executed ON automation_log(executed_at);
