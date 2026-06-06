CREATE TABLE members (
  id         TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  name       TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE (project_id, name)
) STRICT;
CREATE INDEX idx_members_project ON members(project_id);

ALTER TABLE issues ADD COLUMN assignee_id TEXT REFERENCES members(id) ON DELETE SET NULL;
