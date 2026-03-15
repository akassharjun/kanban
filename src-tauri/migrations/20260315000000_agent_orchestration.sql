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

CREATE TABLE IF NOT EXISTS agent_stats (
    agent_id TEXT PRIMARY KEY REFERENCES agents(id) ON DELETE CASCADE,
    tasks_completed INTEGER NOT NULL DEFAULT 0,
    tasks_failed INTEGER NOT NULL DEFAULT 0,
    total_confidence REAL NOT NULL DEFAULT 0.0,
    total_completion_time_seconds INTEGER NOT NULL DEFAULT 0,
    skills_breakdown JSON NOT NULL DEFAULT '{}'
);

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

CREATE TABLE IF NOT EXISTS project_agent_config (
    project_id INTEGER PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
    auto_accept_threshold REAL NOT NULL DEFAULT 0.85,
    human_review_threshold REAL NOT NULL DEFAULT 0.50,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    heartbeat_interval_seconds INTEGER NOT NULL DEFAULT 60,
    missed_heartbeats_before_offline INTEGER NOT NULL DEFAULT 3
);
