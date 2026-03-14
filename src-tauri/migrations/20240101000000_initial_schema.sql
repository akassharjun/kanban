-- Projects
CREATE TABLE IF NOT EXISTS projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'paused', 'completed', 'archived')),
    prefix TEXT NOT NULL UNIQUE,
    issue_counter INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
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
