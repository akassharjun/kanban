ALTER TABLE agents ADD COLUMN IF NOT EXISTS last_activity_at TEXT;
UPDATE agents SET last_activity_at = last_heartbeat WHERE last_activity_at IS NULL;
