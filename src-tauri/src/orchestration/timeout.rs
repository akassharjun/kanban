use sqlx::AnyPool;
use crate::db::DbBackend;
use crate::db::compat;

/// Row type for timed-out task queries.
#[derive(Debug, sqlx::FromRow)]
struct TimedOutTask {
    issue_id: i64,
    claimed_by: Option<String>,
    attempt_count: i64,
    context: String,
}

/// Row type for offline agent queries.
#[derive(Debug, sqlx::FromRow)]
struct OfflineAgentRow {
    id: String,
}

/// Default threshold in seconds for considering an agent offline based on activity.
/// 10 minutes of inactivity = offline.
const DEFAULT_OFFLINE_THRESHOLD_SECONDS: i64 = 600;

/// Update agent's last_activity_at timestamp based on task activity.
/// Also sets the agent back to 'online' if it was previously 'offline'.
pub async fn update_agent_activity(pool: &AnyPool, agent_id: &str) {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();
    let _ = sqlx::query(
        "UPDATE agents SET last_activity_at = $1, status = CASE WHEN status = 'offline' THEN 'online' ELSE status END WHERE id = $2",
    )
    .bind(&now)
    .bind(agent_id)
    .execute(pool)
    .await;
}

/// Reclaim tasks that have timed out (claimed_at + timeout_minutes has elapsed).
pub async fn reclaim_timed_out_tasks(pool: &AnyPool, backend: &DbBackend) -> Result<Vec<i64>, sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

    let timed_out: Vec<TimedOutTask> = sqlx::query_as(
        compat::timed_out_tasks_query(backend),
    )
    .fetch_all(pool)
    .await?;

    let mut reclaimed_ids = Vec::new();
    let jcast = compat::jsonb_cast(backend);

    for task in &timed_out {
        let new_attempt_count = task.attempt_count + 1;
        let agent_id = task.claimed_by.as_deref().unwrap_or("unknown");

        // Parse context JSON, append to prior_attempts
        let mut context: serde_json::Value = serde_json::from_str(&task.context).unwrap_or(serde_json::json!({}));
        let attempt_entry = serde_json::json!({
            "agent": agent_id,
            "attempt_number": new_attempt_count,
            "result": "timeout",
            "reason": "Agent timed out",
        });
        if let Some(obj) = context.as_object_mut() {
            let arr = obj
                .entry("prior_attempts")
                .or_insert(serde_json::json!([]));
            if let Some(a) = arr.as_array_mut() {
                a.push(attempt_entry);
            }
        }
        let context_str =
            serde_json::to_string(&context).unwrap_or_else(|_| "{}".to_string());

        // Check escalation: query project_agent_config for max_attempts
        let max_attempts: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT pac.max_attempts
            FROM project_agent_config pac
            JOIN issues i ON i.project_id = pac.project_id
            WHERE i.id = $1
            "#,
        )
        .bind(task.issue_id)
        .fetch_optional(pool)
        .await?;

        let should_block =
            max_attempts.map_or(false, |max| new_attempt_count >= max);

        let new_state = if should_block { "blocked" } else { "queued" };
        let status_category = if should_block { "blocked" } else { "unstarted" };

        // Update task_contracts: requeue or block, clear claimed_by/claimed_at
        let update_q = format!(
            "UPDATE task_contracts SET task_state = $1, claimed_by = NULL, claimed_at = NULL, attempt_count = $2, context = $3{} WHERE issue_id = $4",
            jcast
        );
        sqlx::query(&update_q)
        .bind(new_state)
        .bind(new_attempt_count)
        .bind(&context_str)
        .bind(task.issue_id)
        .execute(pool)
        .await?;

        // Sync issues.status_id
        sqlx::query(
            "UPDATE issues SET status_id = (
                SELECT s.id FROM statuses s
                WHERE s.project_id = issues.project_id AND s.category = $1
                ORDER BY s.position ASC LIMIT 1
             ), updated_at = $2
             WHERE id = $3",
        )
        .bind(status_category)
        .bind(&now)
        .bind(task.issue_id)
        .execute(pool)
        .await?;

        // Insert execution_log entry
        sqlx::query(
            "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'timeout', 'Task reclaimed due to timeout', $4)",
        )
        .bind(task.issue_id)
        .bind(agent_id)
        .bind(new_attempt_count)
        .bind(&now)
        .execute(pool)
        .await?;

        reclaimed_ids.push(task.issue_id);
    }

    Ok(reclaimed_ids)
}

/// Reclaim tasks from agents that have gone offline (missed heartbeats).
pub async fn reclaim_offline_agents(pool: &AnyPool, backend: &DbBackend) -> Result<Vec<String>, sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

    let offline_agents: Vec<OfflineAgentRow> = sqlx::query_as(
        compat::offline_agents_query(backend),
    )
    .bind(DEFAULT_OFFLINE_THRESHOLD_SECONDS)
    .fetch_all(pool)
    .await?;

    let mut offline_ids = Vec::new();
    let jcast = compat::jsonb_cast(backend);

    for agent in &offline_agents {
        // Set agent status to offline
        sqlx::query("UPDATE agents SET status = 'offline' WHERE id = $1")
            .bind(&agent.id)
            .execute(pool)
            .await?;

        // Find all their claimed/executing tasks
        let tasks: Vec<TimedOutTask> = sqlx::query_as(
            r#"
            SELECT tc.issue_id, tc.claimed_by, tc.attempt_count, tc.context
            FROM task_contracts tc
            WHERE tc.claimed_by = $1
              AND tc.task_state IN ('claimed', 'executing')
            "#,
        )
        .bind(&agent.id)
        .fetch_all(pool)
        .await?;

        for task in &tasks {
            let new_attempt_count = task.attempt_count + 1;

            // Parse context JSON, append to prior_attempts
            let mut context: serde_json::Value = serde_json::from_str(&task.context).unwrap_or(serde_json::json!({}));
            let attempt_entry = serde_json::json!({
                "agent": agent.id,
                "attempt_number": new_attempt_count,
                "result": "timeout",
                "reason": "Agent timed out",
            });
            if let Some(obj) = context.as_object_mut() {
                let arr = obj
                    .entry("prior_attempts")
                    .or_insert(serde_json::json!([]));
                if let Some(a) = arr.as_array_mut() {
                    a.push(attempt_entry);
                }
            }
            let context_str =
                serde_json::to_string(&context).unwrap_or_else(|_| "{}".to_string());

            // Check escalation
            let max_attempts: Option<i64> = sqlx::query_scalar(
                r#"
                SELECT pac.max_attempts
                FROM project_agent_config pac
                JOIN issues i ON i.project_id = pac.project_id
                WHERE i.id = $1
                "#,
            )
            .bind(task.issue_id)
            .fetch_optional(pool)
            .await?;

            let should_block =
                max_attempts.map_or(false, |max| new_attempt_count >= max);

            let new_state = if should_block { "blocked" } else { "queued" };
            let status_category = if should_block { "blocked" } else { "unstarted" };

            // Update task_contracts
            let update_q = format!(
                "UPDATE task_contracts SET task_state = $1, claimed_by = NULL, claimed_at = NULL, attempt_count = $2, context = $3{} WHERE issue_id = $4",
                jcast
            );
            sqlx::query(&update_q)
            .bind(new_state)
            .bind(new_attempt_count)
            .bind(&context_str)
            .bind(task.issue_id)
            .execute(pool)
            .await?;

            // Sync issues.status_id
            sqlx::query(
                "UPDATE issues SET status_id = (
                    SELECT s.id FROM statuses s
                    WHERE s.project_id = issues.project_id AND s.category = $1
                    ORDER BY s.position ASC LIMIT 1
                 ), updated_at = $2
                 WHERE id = $3",
            )
            .bind(status_category)
            .bind(&now)
            .bind(task.issue_id)
            .execute(pool)
            .await?;

            // Insert execution_log entry
            sqlx::query(
                "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, $2, $3, 'timeout', 'Task reclaimed due to agent going offline', $4)",
            )
            .bind(task.issue_id)
            .bind(&agent.id)
            .bind(new_attempt_count)
            .bind(&now)
            .execute(pool)
            .await?;
        }

        offline_ids.push(agent.id.clone());
    }

    Ok(offline_ids)
}
