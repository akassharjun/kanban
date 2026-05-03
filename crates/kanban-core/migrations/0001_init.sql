-- v1 initial schema. After release this file is FROZEN. Schema changes go in 0002+.

CREATE TABLE IF NOT EXISTS schema_migrations (
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
