use super::DbBackend;

/// Returns `"::jsonb"` for Postgres, `""` for SQLite.
pub fn jsonb_cast(backend: &DbBackend) -> &'static str {
    match backend {
        DbBackend::Postgres => "::jsonb",
        DbBackend::Sqlite => "",
    }
}

/// Returns `"::bigint"` for Postgres, `""` for SQLite.
pub fn bigint_cast(backend: &DbBackend) -> &'static str {
    match backend {
        DbBackend::Postgres => "::bigint",
        DbBackend::Sqlite => "",
    }
}

/// Query to find timed-out tasks (claimed_at + timeout_minutes elapsed).
pub fn timed_out_tasks_query(backend: &DbBackend) -> &'static str {
    match backend {
        DbBackend::Postgres => {
            r#"SELECT tc.issue_id, tc.claimed_by, tc.attempt_count, tc.context
               FROM task_contracts tc
               WHERE tc.task_state IN ('claimed', 'executing')
                 AND tc.claimed_at IS NOT NULL
                 AND tc.claimed_at::timestamptz + (tc.timeout_minutes * interval '1 minute') < NOW()"#
        }
        DbBackend::Sqlite => {
            r#"SELECT tc.issue_id, tc.claimed_by, tc.attempt_count, tc.context
               FROM task_contracts tc
               WHERE tc.task_state IN ('claimed', 'executing')
                 AND tc.claimed_at IS NOT NULL
                 AND datetime(tc.claimed_at, '+' || tc.timeout_minutes || ' minutes') < datetime('now')"#
        }
    }
}

/// Query to find offline agents based on activity threshold.
pub fn offline_agents_query(backend: &DbBackend) -> &'static str {
    match backend {
        DbBackend::Postgres => {
            r#"SELECT a.id
               FROM agents a
               WHERE a.status != 'offline'
                 AND a.last_activity_at IS NOT NULL
                 AND a.last_activity_at::timestamptz + ($1 * interval '1 second') < NOW()"#
        }
        DbBackend::Sqlite => {
            r#"SELECT a.id
               FROM agents a
               WHERE a.status != 'offline'
                 AND a.last_activity_at IS NOT NULL
                 AND datetime(a.last_activity_at, '+' || $1 || ' seconds') < datetime('now')"#
        }
    }
}

/// AVG confidence expression for completed task results.
pub fn avg_confidence_query(backend: &DbBackend) -> &'static str {
    match backend {
        DbBackend::Postgres => {
            "SELECT AVG((tc.result::jsonb->>'confidence')::float) FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = $1 AND tc.task_state = 'completed' AND tc.result IS NOT NULL"
        }
        DbBackend::Sqlite => {
            "SELECT AVG(CAST(json_extract(tc.result, '$.confidence') AS REAL)) FROM task_contracts tc JOIN issues i ON tc.issue_id = i.id WHERE i.project_id = $1 AND tc.task_state = 'completed' AND tc.result IS NOT NULL"
        }
    }
}

/// Count tasks completed in last 24 hours.
pub fn tasks_completed_24h_query(backend: &DbBackend) -> &'static str {
    match backend {
        DbBackend::Postgres => {
            "SELECT COUNT(*)::bigint FROM execution_logs el JOIN issues i ON el.issue_id = i.id WHERE i.project_id = $1 AND el.entry_type IN ('result', 'complete') AND el.timestamp::timestamptz > NOW() - interval '24 hours'"
        }
        DbBackend::Sqlite => {
            "SELECT COUNT(*) FROM execution_logs el JOIN issues i ON el.issue_id = i.id WHERE i.project_id = $1 AND el.entry_type IN ('result', 'complete') AND el.timestamp > datetime('now', '-24 hours')"
        }
    }
}

/// COUNT(*)::bigint or COUNT(*) depending on backend.
pub fn count_query(backend: &DbBackend, base: &str) -> String {
    match backend {
        DbBackend::Postgres => base.replace("COUNT(*)", "COUNT(*)::bigint"),
        DbBackend::Sqlite => base.to_string(),
    }
}

/// COALESCE(SUM(...), 0)::bigint or plain version.
pub fn sum_bigint_query(backend: &DbBackend, base: &str) -> String {
    if *backend == DbBackend::Postgres {
        // Already has ::bigint in the base string — return as-is
        base.to_string()
    } else {
        base.replace("::bigint", "")
    }
}
