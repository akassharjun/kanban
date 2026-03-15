use sqlx::PgPool;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeResult {
    pub invalidated_issue_id: i64,
    pub tasks_blocked: Vec<i64>,       // downstream moved to blocked
    pub tasks_warned: Vec<i64>,        // downstream in executing state, got warning log
    pub review_tasks_created: Vec<i64>, // downstream completed, got review tasks
}

/// Invalidate a completed task and handle all downstream effects.
/// - Requeues the invalidated task
/// - Blocks downstream queued/claimed/blocked tasks (recursively)
/// - Warns executing downstream tasks
/// - Creates review tasks for completed downstream tasks
pub async fn invalidate_task(pool: &PgPool, issue_id: i64, reason: &str) -> Result<CascadeResult, sqlx::Error> {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%SZ").to_string();

    // Get the issue for project context
    let issue = sqlx::query_as::<_, crate::models::Issue>(
        "SELECT * FROM issues WHERE id = $1"
    ).bind(issue_id).fetch_one(pool).await?;

    let contract = sqlx::query_as::<_, crate::models::TaskContract>(
        "SELECT * FROM task_contracts WHERE issue_id = $1"
    ).bind(issue_id).fetch_one(pool).await?;

    // Requeue the invalidated task
    let new_attempt = contract.attempt_count + 1;
    let mut context: serde_json::Value = contract.context.clone();
    let entry = serde_json::json!({
        "agent": contract.claimed_by,
        "attempt_number": new_attempt,
        "result": "invalidated",
        "reason": reason
    });
    if let Some(arr) = context.get_mut("prior_attempts").and_then(|v| v.as_array_mut()) {
        arr.push(entry);
    } else {
        context["prior_attempts"] = serde_json::json!([entry]);
    }

    sqlx::query(
        "UPDATE task_contracts SET task_state = 'queued', claimed_by = NULL, claimed_at = NULL, attempt_count = $1, context = $2 WHERE issue_id = $3"
    ).bind(new_attempt).bind(context.to_string()).bind(issue_id).execute(pool).await?;

    // Sync status to unstarted
    let unstarted_sid: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM statuses WHERE project_id = $1 AND category = 'unstarted' ORDER BY position LIMIT 1"
    ).bind(issue.project_id).fetch_optional(pool).await?;
    if let Some(sid) = unstarted_sid {
        sqlx::query("UPDATE issues SET status_id = $1, updated_at = $2 WHERE id = $3")
            .bind(sid).bind(&now).bind(issue_id).execute(pool).await?;
    }

    // Log invalidation
    sqlx::query(
        "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, 'system', $2, 'error', $3, $4)"
    ).bind(issue_id).bind(new_attempt).bind(format!("Invalidated: {}", reason)).bind(&now).execute(pool).await?;

    // Find ALL transitive downstream tasks using recursive query
    // Walk issue_relations where relation_type = 'blocks' transitively
    let all_downstream: Vec<i64> = sqlx::query_scalar(
        "WITH RECURSIVE downstream(id) AS (
            SELECT target_issue_id FROM issue_relations WHERE source_issue_id = $1 AND relation_type = 'blocks'
            UNION
            SELECT ir.target_issue_id FROM issue_relations ir JOIN downstream d ON ir.source_issue_id = d.id WHERE ir.relation_type = 'blocks'
        )
        SELECT id FROM downstream"
    ).bind(issue_id).fetch_all(pool).await?;

    let mut tasks_blocked = Vec::new();
    let mut tasks_warned = Vec::new();
    let mut review_tasks_created = Vec::new();

    for downstream_id in &all_downstream {
        let ds_contract = sqlx::query_as::<_, crate::models::TaskContract>(
            "SELECT * FROM task_contracts WHERE issue_id = $1"
        ).bind(downstream_id).fetch_optional(pool).await?;

        let Some(ds_contract) = ds_contract else { continue };

        match ds_contract.task_state.as_str() {
            "queued" | "claimed" | "blocked" => {
                // Block these tasks
                sqlx::query("UPDATE task_contracts SET task_state = 'blocked', claimed_by = NULL, claimed_at = NULL WHERE issue_id = $1")
                    .bind(downstream_id).execute(pool).await?;

                let blocked_sid: Option<i64> = sqlx::query_scalar(
                    "SELECT id FROM statuses WHERE project_id = $1 AND category = 'blocked' ORDER BY position LIMIT 1"
                ).bind(issue.project_id).fetch_optional(pool).await?;
                if let Some(sid) = blocked_sid {
                    sqlx::query("UPDATE issues SET status_id = $1, updated_at = $2 WHERE id = $3")
                        .bind(sid).bind(&now).bind(downstream_id).execute(pool).await?;
                }

                sqlx::query(
                    "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, 'system', 1, 'error', 'Blocked due to upstream invalidation', $2)"
                ).bind(downstream_id).bind(&now).execute(pool).await?;

                tasks_blocked.push(*downstream_id);
            }
            "executing" => {
                // Warn but don't interrupt
                sqlx::query(
                    "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) VALUES ($1, 'system', $2, 'error', 'WARNING: Upstream task invalidated, review work on completion', $3)"
                ).bind(downstream_id).bind(ds_contract.attempt_count + 1).bind(&now).execute(pool).await?;

                tasks_warned.push(*downstream_id);
            }
            "completed" => {
                // Create a review task for completed downstream
                let ds_issue = sqlx::query_as::<_, crate::models::Issue>(
                    "SELECT * FROM issues WHERE id = $1"
                ).bind(downstream_id).fetch_one(pool).await?;

                let (counter, prefix): (i64, String) = sqlx::query_as(
                    "UPDATE projects SET issue_counter = issue_counter + 1 WHERE id = $1 RETURNING issue_counter, prefix"
                ).bind(issue.project_id).fetch_one(pool).await?;
                let review_identifier = format!("{}-{}", prefix, counter);
                let review_title = format!("Review: {} (upstream invalidated)", ds_issue.identifier);

                let review_id: i64 = sqlx::query_scalar(
                    "INSERT INTO issues (project_id, identifier, title, description, status_id, priority, position, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, 0.0, $7, $8) RETURNING id"
                ).bind(issue.project_id).bind(&review_identifier).bind(&review_title)
                .bind(format!("Upstream task was invalidated. Verify {} is still valid.", ds_issue.identifier))
                .bind(unstarted_sid.unwrap_or(ds_issue.status_id)).bind(&ds_issue.priority)
                .bind(&now).bind(&now)
                .fetch_one(pool).await?;

                sqlx::query(
                    "INSERT INTO task_contracts (issue_id, type, task_state, objective, required_skills, estimated_complexity, timeout_minutes) VALUES ($1, 'review', 'queued', $2, '[\"review\"]', 'small', 30)"
                ).bind(review_id)
                .bind(format!("Verify that {} is still valid after upstream invalidation", ds_issue.identifier))
                .execute(pool).await?;

                review_tasks_created.push(review_id);
            }
            _ => {} // cancelled tasks are ignored
        }
    }

    // Create cascade notification
    sqlx::query(
        "INSERT INTO notifications (type, issue_id, message, read, created_at) VALUES ('cascade_failure', $1, $2, false, $3)"
    ).bind(issue_id)
    .bind(format!("Task invalidated: {}. {} tasks blocked, {} warned, {} review tasks created.", reason, tasks_blocked.len(), tasks_warned.len(), review_tasks_created.len()))
    .bind(&now)
    .execute(pool).await?;

    Ok(CascadeResult {
        invalidated_issue_id: issue_id,
        tasks_blocked,
        tasks_warned,
        review_tasks_created,
    })
}
