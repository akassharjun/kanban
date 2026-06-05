CREATE TABLE workspace_settings (
  key        TEXT PRIMARY KEY NOT NULL,
  value      TEXT NOT NULL,
  updated_at TEXT NOT NULL
) STRICT;

INSERT INTO workspace_settings (key, value, updated_at)
  VALUES ('theme', 'system', datetime('now'));
