use sqlx::AnyPool;

/// When a task completes, find all downstream tasks that were blocked and can
/// now be unblocked (i.e. all their blockers are completed). Atomically
/// transition them from `blocked` to `queued`, sync the issue status to an
/// `unstarted` category, and log the unblock event.
///
/// Returns the list of issue IDs that were newly unblocked.
pub async fn resolve_downstream(
    pool: &AnyPool,
    completed_issue_id: i64,
) -> Result<Vec<i64>, sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();

    // Atomically find and update all blocked downstream tasks whose blockers
    // are now all completed. Using a single UPDATE avoids races when two
    // blockers complete simultaneously.
    let unblocked_ids: Vec<(i64,)> = sqlx::query_as(
        r#"
        UPDATE task_contracts
        SET task_state = 'queued'
        WHERE task_state = 'blocked'
          AND issue_id IN (
            SELECT ir.target_issue_id
            FROM issue_relations ir
            WHERE ir.source_issue_id = $1
              AND ir.relation_type = 'blocks'
          )
          AND NOT EXISTS (
            SELECT 1
            FROM issue_relations ir2
            JOIN task_contracts btc ON btc.issue_id = ir2.source_issue_id
            WHERE ir2.target_issue_id = task_contracts.issue_id
              AND ir2.relation_type = 'blocks'
              AND btc.task_state NOT IN ('completed')
          )
        RETURNING issue_id
        "#,
    )
    .bind(completed_issue_id)
    .fetch_all(pool)
    .await?;

    let ids: Vec<i64> = unblocked_ids.into_iter().map(|(id,)| id).collect();

    // For each unblocked task, sync the issue status and log the event.
    for &issue_id in &ids {
        // Sync issue status to the first 'unstarted' category status for its project.
        sqlx::query(
            "UPDATE issues SET status_id = (
                SELECT s.id FROM statuses s
                WHERE s.project_id = issues.project_id AND s.category = 'unstarted'
                ORDER BY s.position ASC LIMIT 1
             ), updated_at = $1
             WHERE id = $2",
        )
        .bind(&now)
        .bind(issue_id)
        .execute(pool)
        .await?;

        // Insert execution_log entry for the unblock event.
        sqlx::query(
            "INSERT INTO execution_logs (issue_id, agent_id, attempt_number, entry_type, message, timestamp) \
             VALUES ($1, NULL, (SELECT attempt_count FROM task_contracts WHERE issue_id = $2), 'unblocked', 'Dependencies resolved, task unblocked', $3)",
        )
        .bind(issue_id)
        .bind(issue_id)
        .bind(&now)
        .execute(pool)
        .await?;
    }

    Ok(ids)
}
